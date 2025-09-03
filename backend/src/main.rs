use anyhow::Result;
use axum::{
    routing::{get, post, put, delete},
    Router,
};
use std::{env, net::SocketAddr};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod handlers;
mod middleware;
mod models;
mod services;
mod state;
mod utils;

use crate::{
    config::Config,
    handlers::{auth, flow, health, privacy, sessions, teams, websocket},
    middleware::auth::auth_middleware,
    state::AppState,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "mindful_code_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Config::from_env()?;
    info!("Starting Mindful Code Backend API");
    info!("Database URL: {}", config.database_url.chars().take(20).collect::<String>() + "...");

    // Initialize application state
    let app_state = AppState::new(config.clone()).await?;

    // Build our application with routes
    let app = Router::new()
        // Health check (no auth required)
        .route("/health", get(health::health_check))
        .route("/metrics", get(health::metrics))
        
        // Authentication routes (no auth required)
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/auth/refresh", post(auth::refresh_token))
        
        // Session management (requires auth)
        .route("/api/sessions/start", post(sessions::start_session))
        .route("/api/sessions/:id/update", put(sessions::update_session))
        .route("/api/sessions/:id/end", post(sessions::end_session))
        .route("/api/sessions/history", get(sessions::get_session_history))
        
        // Real-time flow state detection (requires auth)
        .route("/api/flow/detect", post(flow::detect_flow_state))
        .route("/api/flow/patterns", get(flow::get_flow_patterns))
        .route("/api/flow/insights", get(flow::get_flow_insights))
        
        // Team features (requires auth)
        .route("/api/teams/:id/analytics", get(teams::get_team_analytics))
        .route("/api/teams/:id/insights", get(teams::get_team_insights))
        .route("/api/teams/:id/alerts", post(teams::create_alert))
        
        // Privacy and data control (requires auth)
        .route("/api/privacy/export", get(privacy::export_user_data))
        .route("/api/privacy/purge", delete(privacy::purge_user_data))
        .route("/api/privacy/settings", put(privacy::update_privacy_settings))
        
        // WebSocket for real-time updates
        .route("/ws", get(websocket::websocket_handler))
        
        // Apply middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(std::time::Duration::from_secs(10)))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                )
        )
        .layer(axum::middleware::from_fn_with_state(
            app_state.clone(),
            auth_middleware,
        ))
        .with_state(app_state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    info!("🚀 Server starting on http://{}", addr);
    info!("📊 Health check available at http://{}/health", addr);
    info!("🔌 WebSocket endpoint at ws://{}/ws", addr);
    
    axum::serve(listener, app.into_make_service())
        .await
        .map_err(|e| {
            warn!("Server error: {}", e);
            anyhow::anyhow!("Server failed to start: {}", e)
        })?;

    Ok(())
}