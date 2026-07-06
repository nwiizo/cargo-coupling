//! HTTP routes for the web visualization
//!
//! Provides API endpoints for graph data and static file serving.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Query, State},
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
    routing::get,
};
use rust_embed::RustEmbed;

use serde::{Deserialize, Serialize};

use crate::cli_output::JsonHistory;
use crate::history::analyze_ref;
use crate::manifest::{ManifestContext, build_manifest};
use crate::report::{TextReportOptions, generate_report_with_options};

use super::graph;
use super::server::AppState;

/// Embedded static assets
#[derive(RustEmbed)]
#[folder = "web-assets/"]
struct Assets;

/// Frontend configuration
#[derive(Serialize)]
struct FrontendConfig {
    api_endpoint: Option<String>,
}

/// Query parameters for graph request
#[derive(Deserialize)]
struct GraphQuery {
    #[serde(rename = "ref")]
    git_ref: Option<String>,
}

/// Query parameters for source code request
#[derive(Deserialize)]
struct SourceQuery {
    path: String,
    line: Option<usize>,
    context: Option<usize>,
    #[serde(rename = "ref")]
    git_ref: Option<String>,
}

/// Source code response
#[derive(Serialize)]
struct SourceResponse {
    file_path: String,
    file_name: String,
    start_line: usize,
    end_line: usize,
    highlight_line: Option<usize>,
    lines: Vec<SourceLine>,
    total_lines: usize,
}

/// A single line of source code
#[derive(Serialize)]
struct SourceLine {
    number: usize,
    content: String,
    is_highlight: bool,
}

/// Query parameters for module items request
#[derive(Deserialize)]
struct ModuleQuery {
    name: String,
}

/// Create API routes
pub fn api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/graph", get(get_graph))
        .route("/api/report", get(get_report))
        .route("/api/history", get(get_history))
        .route("/api/config", get(get_config))
        .route("/api/health", get(health_check))
        .route("/api/source", get(get_source))
        .route("/api/module", get(get_module))
}

/// Create static file routes
pub fn static_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index_html))
        // Note: axum 0.8+ requires `{*path}` syntax for wildcard routes (was `*path` in 0.7)
        .route("/{*path}", get(static_handler))
}

/// GET /api/graph - Returns the complete coupling graph.
///
/// With `?ref=<git-ref>`, analyzes that revision in a disposable worktree so
/// the timeline can lazy-load graph snapshots without bloating `/api/history`.
async fn get_graph(
    State(state): State<Arc<AppState>>,
    Query(query): Query<GraphQuery>,
) -> impl IntoResponse {
    if let Some(git_ref) = query.git_ref.filter(|value| !value.trim().is_empty()) {
        match analyze_ref(
            &state.analysis_path,
            &state.analysis_config,
            &state.thresholds,
            git_ref.trim(),
            state.git_months,
            !state.no_git,
        ) {
            Ok(analysis) => {
                let graph = graph::project_to_graph(&analysis.metrics, &state.thresholds);
                Json(graph).into_response()
            }
            Err(e) => (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        }
    } else {
        let graph = graph::project_to_graph(&state.metrics, &state.thresholds);
        Json(graph).into_response()
    }
}

/// GET /api/report - Returns the current Markdown analysis report.
async fn get_report(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let manifest = build_manifest(&ManifestContext {
        git_used: !state.no_git
            && (!state.metrics.file_changes.is_empty()
                || !state.metrics.temporal_couplings.is_empty()),
        tests_excluded: state.analysis_config.exclude_tests,
        parse_failures: state.metrics.parse_failures,
        skipped_crates: state.metrics.skipped_crates.clone(),
    });
    let mut output = Vec::new();

    match generate_report_with_options(
        &state.metrics,
        &state.thresholds,
        &manifest,
        TextReportOptions::default(),
        &mut output,
    ) {
        Ok(()) => Response::builder()
            .status(StatusCode::OK)
            .header(header::CONTENT_TYPE, "text/markdown; charset=utf-8")
            .body(axum::body::Body::from(output))
            .unwrap(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/history - Returns precomputed coupling health timeline
async fn get_history(State(state): State<Arc<AppState>>) -> Json<JsonHistory> {
    Json(state.history.clone())
}

/// GET /api/config - Returns frontend configuration
async fn get_config(State(state): State<Arc<AppState>>) -> Json<FrontendConfig> {
    Json(FrontendConfig {
        api_endpoint: state.api_endpoint.clone(),
    })
}

/// GET /api/health - Health check endpoint
async fn health_check() -> &'static str {
    "ok"
}

/// GET /api/source - Returns source code snippet
async fn get_source(Query(query): Query<SourceQuery>) -> impl IntoResponse {
    let path = PathBuf::from(&query.path);

    // Security: only allow reading .rs files
    if path.extension().and_then(|e| e.to_str()) != Some("rs") {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Only .rs files are allowed"})),
        )
            .into_response();
    }

    // Read the file from the current worktree or from a git revision.
    let content = match read_source_content(&path, query.git_ref.as_deref()) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": format!("File not found: {}", e)})),
            )
                .into_response();
        }
    };

    let all_lines: Vec<&str> = content.lines().collect();
    let total_lines = all_lines.len();

    // Calculate range to show
    let context = query.context.unwrap_or(10);
    let highlight_line = query.line.filter(|&l| l > 0);

    let (start_line, end_line) = if let Some(line) = highlight_line {
        let start = line.saturating_sub(context).max(1);
        let end = (line + context).min(total_lines);
        (start, end)
    } else {
        // Show first 30 lines if no specific line requested
        (1, total_lines.min(30))
    };

    // Build lines array
    let lines: Vec<SourceLine> = all_lines
        .iter()
        .enumerate()
        .filter(|(i, _)| {
            let line_num = i + 1;
            line_num >= start_line && line_num <= end_line
        })
        .map(|(i, content)| {
            let line_num = i + 1;
            SourceLine {
                number: line_num,
                content: (*content).to_string(),
                is_highlight: highlight_line == Some(line_num),
            }
        })
        .collect();

    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    Json(SourceResponse {
        file_path: query.path,
        file_name,
        start_line,
        end_line,
        highlight_line,
        lines,
        total_lines,
    })
    .into_response()
}

fn read_source_content(path: &Path, git_ref: Option<&str>) -> Result<String, String> {
    let Some(git_ref) = git_ref.filter(|value| !value.trim().is_empty()) else {
        return fs::read_to_string(path).map_err(|e| e.to_string());
    };

    let repo_root = git_repo_root(path)?;
    let relative = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .strip_prefix(&repo_root)
        .map_err(|_| "source path is outside the git repository".to_string())?
        .to_string_lossy()
        .replace('\\', "/");

    let object = format!("{}:{}", git_ref.trim(), relative);
    let output = Command::new("git")
        .args(["show", &object])
        .current_dir(&repo_root)
        .output()
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn git_repo_root(path: &Path) -> Result<PathBuf, String> {
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .map_err(|e| e.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    let root = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());
    Ok(root.canonicalize().unwrap_or(root))
}

/// GET /api/module - Returns module details including items
async fn get_module(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ModuleQuery>,
) -> impl IntoResponse {
    let graph = graph::project_to_graph(&state.metrics, &state.thresholds);

    // Find the module by name
    if let Some(node) = graph
        .nodes
        .iter()
        .find(|n| n.id == query.name || n.label == query.name)
    {
        Json(serde_json::json!({
            "id": node.id,
            "label": node.label,
            "file_path": node.file_path,
            "items": node.items,
            "metrics": node.metrics,
            "in_cycle": node.in_cycle,
        }))
        .into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": format!("Module '{}' not found", query.name)})),
        )
            .into_response()
    }
}

/// GET / - Serve index.html
async fn index_html() -> impl IntoResponse {
    match Assets::get("index.html") {
        Some(content) => Html(content.data.into_owned()).into_response(),
        None => (StatusCode::NOT_FOUND, "index.html not found").into_response(),
    }
}

/// Static file handler for embedded assets
async fn static_handler(
    axum::extract::Path(path): axum::extract::Path<String>,
) -> impl IntoResponse {
    let path = path.trim_start_matches('/');

    match Assets::get(path) {
        Some(content) => {
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(axum::body::Body::from(content.data.into_owned()))
                .unwrap()
        }
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(axum::body::Body::from(format!("File not found: {}", path)))
            .unwrap(),
    }
}
