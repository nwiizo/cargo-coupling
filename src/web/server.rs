//! Web server for coupling visualization
//!
//! Provides an HTTP server using Axum to serve the visualization UI
//! and JSON API endpoints.

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::analyze_history;
use crate::balance::score::IssueThresholds;
use crate::cli_output::{JsonHistory, history_report_to_json};
use crate::config::CompiledConfig;
use crate::metrics::project::ProjectMetrics;
use crate::workspace::WorkspaceInfo;

use super::routes;

pub const DEFAULT_HISTORY_MAX_POINTS: usize = 30;

/// Shared application state
pub struct AppState {
    pub metrics: ProjectMetrics,
    pub thresholds: IssueThresholds,
    pub api_endpoint: Option<String>,
    pub history: JsonHistory,
    pub analysis_path: PathBuf,
    pub source_root: PathBuf,
    pub analysis_config: CompiledConfig,
    pub git_months: usize,
    pub no_git: bool,
}

/// Configuration for the web server
pub struct ServerConfig {
    pub port: u16,
    pub open_browser: bool,
    pub api_endpoint: Option<String>,
    pub analysis_path: PathBuf,
    pub analysis_config: CompiledConfig,
    pub git_months: usize,
    pub history_max_points: usize,
    pub no_git: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            open_browser: true,
            api_endpoint: None,
            analysis_path: PathBuf::from("./src"),
            analysis_config: CompiledConfig::empty(),
            git_months: 6,
            history_max_points: DEFAULT_HISTORY_MAX_POINTS,
            no_git: false,
        }
    }
}

/// Start the web server and serve the visualization
pub async fn start_server(
    metrics: ProjectMetrics,
    thresholds: IssueThresholds,
    config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let history = load_history(&config, &thresholds);
    let source_root = analysis_source_root(&config.analysis_path);

    let state = Arc::new(AppState {
        metrics,
        thresholds,
        api_endpoint: config.api_endpoint.clone(),
        history,
        analysis_path: config.analysis_path.clone(),
        source_root,
        analysis_config: config.analysis_config.clone(),
        git_months: config.git_months,
        no_git: config.no_git,
    });

    let app = Router::new()
        .merge(routes::api_routes())
        .merge(routes::static_routes())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], config.port));
    let listener = TcpListener::bind(addr).await?;

    let url = format!("http://localhost:{}", config.port);
    eprintln!("Starting web server at {}", url);

    if config.open_browser {
        eprintln!("Opening browser...");
        if let Err(e) = open::that(&url) {
            eprintln!("Warning: Could not open browser: {}", e);
            eprintln!("Please open {} manually", url);
        }
    }

    eprintln!("Press Ctrl+C to stop the server");

    axum::serve(listener, app).await?;

    Ok(())
}

fn analysis_source_root(analysis_path: &std::path::Path) -> PathBuf {
    let root = WorkspaceInfo::from_path(analysis_path)
        .map(|workspace| workspace.root)
        .unwrap_or_else(|_| analysis_path.to_path_buf());
    root.canonicalize().unwrap_or(root)
}

fn load_history(config: &ServerConfig, thresholds: &IssueThresholds) -> JsonHistory {
    if config.no_git {
        return JsonHistory {
            months: config.git_months,
            points: Vec::new(),
            skipped: Vec::new(),
        };
    }

    match analyze_history(
        &config.analysis_path,
        &config.analysis_config,
        thresholds,
        config.git_months,
        config.history_max_points,
    ) {
        Ok(report) => history_report_to_json(&report),
        Err(e) => {
            eprintln!("Warning: History analysis failed: {}", e);
            JsonHistory {
                months: config.git_months,
                points: Vec::new(),
                skipped: Vec::new(),
            }
        }
    }
}
