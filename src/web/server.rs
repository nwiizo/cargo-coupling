//! Web server for coupling visualization
//!
//! Provides an HTTP server using Axum to serve the visualization UI
//! and JSON API endpoints.

use std::net::SocketAddr;
use std::sync::Arc;

use axum::Router;
use tokio::net::TcpListener;

use crate::balance::IssueThresholds;
use crate::metrics::ProjectMetrics;

use super::routes;

/// Shared application state
pub struct AppState {
    pub metrics: ProjectMetrics,
    pub thresholds: IssueThresholds,
    pub api_endpoint: Option<String>,
}

/// Configuration for the web server
pub struct ServerConfig {
    pub port: u16,
    pub open_browser: bool,
    pub api_endpoint: Option<String>,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3000,
            open_browser: true,
            api_endpoint: None,
        }
    }
}

/// Start the web server and serve the visualization
pub async fn start_server(
    metrics: ProjectMetrics,
    thresholds: IssueThresholds,
    config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(AppState {
        metrics,
        thresholds,
        api_endpoint: config.api_endpoint.clone(),
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
