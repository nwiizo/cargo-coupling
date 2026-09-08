//! HTTP access to design jobs. Analysis stays in the shared Rust pipeline.

use super::server::AppState;
use crate::design::{
    assessment::{AssessmentRequest, assess},
    changes,
    source::SourceInventory,
};
use axum::{
    Json, Router,
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use serde::Deserialize;
use std::sync::Arc;

pub(super) fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/design", get(get_design))
        .route("/api/impact", get(get_impact))
}

#[derive(Deserialize)]
struct DesignQuery {
    changed_since: Option<String>,
    depth: Option<usize>,
}

async fn get_design(
    State(state): State<Arc<AppState>>,
    Query(query): Query<DesignQuery>,
) -> Response {
    if query.depth == Some(0) {
        return api_error(StatusCode::BAD_REQUEST, "Depth must be at least 1");
    }
    let Some(reference) = query.changed_since.filter(|s| !s.trim().is_empty()) else {
        return Json(&state.design).into_response();
    };
    let result = tokio::task::spawn_blocking(move || {
        // Refuse to combine startup metrics with later source edits.
        let current = crate::analyzer::analyze_workspace_with_config(&state.analysis_path, &state.analysis_config)
            .map_err(|error| (StatusCode::CONFLICT, error.to_string()))?;
        let inventory = SourceInventory::read(&current, state.analysis_config.exclude_tests);
        let root = changes::git_root(&state.analysis_path).unwrap_or_else(|| state.source_root.clone());
        if changes::source_fingerprint(&inventory, &root) != state.design.provenance.source_fingerprint {
            return Err((StatusCode::CONFLICT, "Source files changed after startup. Restart cargo coupling --web to refresh the snapshot.".to_string()));
        }
        assess(&state.metrics, &state.analysis_config, &state.thresholds, AssessmentRequest {
            path: &state.analysis_path, context_path: state.context_path.as_deref(), changed_since: Some(reference.trim()), baseline: None,
            max_depth: query.depth.or(state.design.provenance.max_depth), git_months: state.git_months, git_used: state.design.provenance.git_used,
        }).map_err(|error| (StatusCode::BAD_REQUEST, error.to_string()))
    }).await;
    match result {
        Ok(Ok(report)) => Json(report).into_response(),
        Ok(Err((status, error))) => api_error(status, &error),
        Err(error) => api_error(StatusCode::INTERNAL_SERVER_ERROR, &error.to_string()),
    }
}

#[derive(Deserialize)]
struct ImpactQuery {
    module: String,
    depth: Option<usize>,
}

async fn get_impact(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ImpactQuery>,
) -> Response {
    if query.depth == Some(0) {
        return api_error(StatusCode::BAD_REQUEST, "Depth must be at least 1");
    }
    match crate::cli_output::analyze_impact_with_depth(
        &state.metrics,
        &query.module,
        query.depth.or(state.design.provenance.max_depth),
    ) {
        Some(impact) => Json(impact).into_response(),
        None => api_error(
            StatusCode::NOT_FOUND,
            "Module not found or ambiguous; use its complete name",
        ),
    }
}

fn api_error(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompiledConfig, IssueThresholds};

    fn state() -> (tempfile::TempDir, Arc<AppState>) {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("a.rs"), "pub fn a() {}\n").unwrap();
        let metrics = crate::analyzer::analyze_project(dir.path()).unwrap();
        let config = CompiledConfig::empty();
        let thresholds = IssueThresholds::default();
        let design = assess(
            &metrics,
            &config,
            &thresholds,
            AssessmentRequest {
                path: dir.path(),
                context_path: None,
                changed_since: None,
                baseline: None,
                max_depth: None,
                git_months: 6,
                git_used: false,
            },
        )
        .unwrap();
        let state = Arc::new(AppState {
            metrics,
            thresholds,
            api_endpoint: None,
            history: crate::cli_output::JsonHistory {
                months: 6,
                points: vec![],
                skipped: vec![],
            },
            analysis_path: dir.path().into(),
            source_root: dir.path().canonicalize().unwrap(),
            analysis_config: config,
            git_months: 6,
            no_git: true,
            design,
            context_path: None,
        });
        (dir, state)
    }

    #[tokio::test]
    async fn comparison_refuses_new_sources_missing_from_startup_metrics() {
        let (dir, state) = state();
        std::fs::write(dir.path().join("new.rs"), "pub fn added() {}\n").unwrap();
        let result = get_design(
            State(state),
            Query(DesignQuery {
                changed_since: Some("HEAD".into()),
                depth: None,
            }),
        )
        .await;
        assert_eq!(result.status(), StatusCode::CONFLICT);
    }

    #[tokio::test]
    async fn web_jobs_distinguish_invalid_depth_missing_origin_and_stale_sources() {
        let (dir, state) = state();
        let result = get_impact(
            State(state.clone()),
            Query(ImpactQuery {
                module: "a".into(),
                depth: Some(0),
            }),
        )
        .await;
        assert_eq!(result.status(), StatusCode::BAD_REQUEST);
        let result = get_impact(
            State(state.clone()),
            Query(ImpactQuery {
                module: "missing".into(),
                depth: None,
            }),
        )
        .await;
        assert_eq!(result.status(), StatusCode::NOT_FOUND);
        let result = get_impact(
            State(state.clone()),
            Query(ImpactQuery {
                module: "a".into(),
                depth: None,
            }),
        )
        .await;
        assert_eq!(result.status(), StatusCode::OK);
        std::fs::write(dir.path().join("a.rs"), "pub fn changed() {}\n").unwrap();
        let result = get_design(
            State(state),
            Query(DesignQuery {
                changed_since: Some("HEAD".into()),
                depth: None,
            }),
        )
        .await;
        assert_eq!(result.status(), StatusCode::CONFLICT);
    }
}
