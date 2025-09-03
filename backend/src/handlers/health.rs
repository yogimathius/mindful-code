use axum::{extract::State, http::StatusCode, Json};
use serde_json::{json, Value};
use std::time::SystemTime;

use crate::{error::Result, state::AppState};

pub async fn health_check(State(state): State<AppState>) -> Result<Json<Value>> {
    // Check database connection
    let db_status = match sqlx::query("SELECT 1").fetch_one(&state.db).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    // Get system metrics
    let active_sessions = state.get_active_sessions_count();
    let flow_engines = state.flow_engines.len();
    let websocket_connections = state.websocket_connections.len();

    let response = json!({
        "status": "ok",
        "timestamp": SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        "version": env!("CARGO_PKG_VERSION"),
        "database": {
            "status": db_status,
            "pool_size": state.db.size(),
            "idle_connections": state.db.num_idle()
        },
        "services": {
            "active_sessions": active_sessions,
            "flow_engines": flow_engines,
            "websocket_connections": websocket_connections
        },
        "environment": state.config.environment
    });

    Ok(Json(response))
}

pub async fn metrics(State(state): State<AppState>) -> Result<(StatusCode, String)> {
    // Prometheus-compatible metrics format
    let active_sessions = state.get_active_sessions_count();
    let flow_engines = state.flow_engines.len();
    let websocket_connections = state.websocket_connections.len();
    let db_pool_size = state.db.size();
    let db_idle_connections = state.db.num_idle();

    let metrics = format!(
        r#"# HELP mindful_code_active_sessions Number of active coding sessions
# TYPE mindful_code_active_sessions gauge
mindful_code_active_sessions {{}} {}

# HELP mindful_code_flow_engines Number of active flow detection engines
# TYPE mindful_code_flow_engines gauge
mindful_code_flow_engines {{}} {}

# HELP mindful_code_websocket_connections Number of active WebSocket connections
# TYPE mindful_code_websocket_connections gauge
mindful_code_websocket_connections {{}} {}

# HELP mindful_code_db_pool_size Database connection pool size
# TYPE mindful_code_db_pool_size gauge
mindful_code_db_pool_size {{}} {}

# HELP mindful_code_db_idle_connections Number of idle database connections
# TYPE mindful_code_db_idle_connections gauge
mindful_code_db_idle_connections {{}} {}
"#,
        active_sessions,
        flow_engines,
        websocket_connections,
        db_pool_size,
        db_idle_connections
    );

    Ok((
        StatusCode::OK,
        metrics,
    ))
}