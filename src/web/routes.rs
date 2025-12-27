//! HTTP routes for the web visualization
//!
//! Provides API endpoints for graph data and static file serving.

use std::fs;
use std::path::PathBuf;
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

use super::graph::{self, GraphData};
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

/// Query parameters for source code request
#[derive(Deserialize)]
struct SourceQuery {
    path: String,
    line: Option<usize>,
    context: Option<usize>,
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
        .route("/api/config", get(get_config))
        .route("/api/health", get(health_check))
        .route("/api/source", get(get_source))
        .route("/api/module", get(get_module))
}

/// Create static file routes
pub fn static_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/", get(index_html))
        .route("/{*path}", get(static_handler))
}

/// GET /api/graph - Returns the complete coupling graph
async fn get_graph(State(state): State<Arc<AppState>>) -> Json<GraphData> {
    let graph = graph::project_to_graph(&state.metrics, &state.thresholds);
    Json(graph)
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

    // Read the file
    let content = match fs::read_to_string(&path) {
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
