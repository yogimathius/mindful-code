use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Query, State,
    },
    http::StatusCode,
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    error::{AppError, Result},
    state::AppState,
    utils::auth::validate_jwt_token,
};

#[derive(Debug, Deserialize)]
pub struct WebSocketQuery {
    token: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketMessage {
    #[serde(rename = "ping")]
    Ping { timestamp: i64 },
    #[serde(rename = "pong")]
    Pong { timestamp: i64 },
    #[serde(rename = "flow_state_update")]
    FlowStateUpdate {
        session_id: Uuid,
        flow_state: serde_json::Value,
    },
    #[serde(rename = "session_update")]
    SessionUpdate {
        session_id: Uuid,
        status: String,
        metrics: serde_json::Value,
    },
    #[serde(rename = "notification")]
    Notification {
        title: String,
        message: String,
        level: NotificationLevel,
        timestamp: i64,
    },
    #[serde(rename = "team_alert")]
    TeamAlert {
        team_id: Uuid,
        alert_type: String,
        data: serde_json::Value,
    },
    #[serde(rename = "system_message")]
    SystemMessage { message: String },
    #[serde(rename = "error")]
    Error { code: u16, message: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum NotificationLevel {
    Info,
    Warning,
    Error,
    Success,
}

pub async fn websocket_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<WebSocketQuery>,
    State(state): State<AppState>,
) -> Result<Response> {
    // Validate JWT token
    let claims = validate_jwt_token(&params.token, &state.config.jwt_secret)
        .map_err(|e| AppError::Authentication(format!("Invalid WebSocket token: {}", e)))?;

    info!("WebSocket connection established for user {}", claims.user_id);

    Ok(ws.on_upgrade(move |socket| websocket_connection(socket, claims.user_id, state)))
}

async fn websocket_connection(socket: WebSocket, user_id: Uuid, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    
    // Create a channel for sending messages to this WebSocket
    let (tx, mut rx) = mpsc::unbounded_channel::<String>();
    
    // Register this connection
    state.add_websocket_connection(user_id, tx);
    
    // Spawn task to handle outgoing messages
    let sender_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut ping_interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
    let mut last_pong = tokio::time::Instant::now();
    
    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = receiver.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        if let Err(e) = handle_websocket_message(&text, user_id, &state).await {
                            error!("Error handling WebSocket message: {}", e);
                            let error_msg = WebSocketMessage::Error {
                                code: 500,
                                message: "Internal server error".to_string(),
                            };
                            if let Ok(error_json) = serde_json::to_string(&error_msg) {
                                let _ = state.broadcast_to_user(user_id, error_json).await;
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = tokio::time::Instant::now();
                        debug!("Received pong from user {}", user_id);
                    }
                    Some(Ok(Message::Close(_))) => {
                        info!("WebSocket connection closed by user {}", user_id);
                        break;
                    }
                    Some(Err(e)) => {
                        warn!("WebSocket error for user {}: {}", user_id, e);
                        break;
                    }
                    None => {
                        info!("WebSocket stream ended for user {}", user_id);
                        break;
                    }
                    _ => {
                        // Handle other message types (binary, etc.)
                        debug!("Received non-text WebSocket message from user {}", user_id);
                    }
                }
            }
            
            // Send periodic ping
            _ = ping_interval.tick() => {
                let ping_msg = WebSocketMessage::Ping {
                    timestamp: chrono::Utc::now().timestamp_millis(),
                };
                if let Ok(ping_json) = serde_json::to_string(&ping_msg) {
                    let _ = state.broadcast_to_user(user_id, ping_json).await;
                }
                
                // Check if connection is stale
                if last_pong.elapsed() > tokio::time::Duration::from_secs(90) {
                    warn!("WebSocket connection stale for user {}, disconnecting", user_id);
                    break;
                }
            }
        }
    }
    
    // Cleanup
    sender_task.abort();
    state.remove_websocket_connection(user_id);
    info!("WebSocket connection cleaned up for user {}", user_id);
}

async fn handle_websocket_message(
    message: &str,
    user_id: Uuid,
    state: &AppState,
) -> Result<()> {
    let ws_message: WebSocketMessage = serde_json::from_str(message)
        .map_err(|e| AppError::BadRequest(format!("Invalid WebSocket message: {}", e)))?;

    match ws_message {
        WebSocketMessage::Ping { timestamp } => {
            let pong_msg = WebSocketMessage::Pong { timestamp };
            let pong_json = serde_json::to_string(&pong_msg)
                .map_err(|e| AppError::Internal(format!("Failed to serialize pong: {}", e)))?;
            state.broadcast_to_user(user_id, pong_json).await;
        }
        WebSocketMessage::Pong { .. } => {
            debug!("Received pong from user {}", user_id);
        }
        _ => {
            debug!("Received WebSocket message from user {}: {:?}", user_id, ws_message);
        }
    }

    Ok(())
}

pub async fn broadcast_flow_update(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
    flow_state: serde_json::Value,
) {
    let message = WebSocketMessage::FlowStateUpdate {
        session_id,
        flow_state,
    };

    if let Ok(json) = serde_json::to_string(&message) {
        state.broadcast_to_user(user_id, json).await;
    }
}

pub async fn broadcast_session_update(
    state: &AppState,
    user_id: Uuid,
    session_id: Uuid,
    status: String,
    metrics: serde_json::Value,
) {
    let message = WebSocketMessage::SessionUpdate {
        session_id,
        status,
        metrics,
    };

    if let Ok(json) = serde_json::to_string(&message) {
        state.broadcast_to_user(user_id, json).await;
    }
}

pub async fn send_notification(
    state: &AppState,
    user_id: Uuid,
    title: String,
    message: String,
    level: NotificationLevel,
) {
    let notification = WebSocketMessage::Notification {
        title,
        message,
        level,
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    if let Ok(json) = serde_json::to_string(&notification) {
        state.broadcast_to_user(user_id, json).await;
    }
}

pub async fn send_team_alert(
    state: &AppState,
    team_id: Uuid,
    alert_type: String,
    data: serde_json::Value,
) -> Result<()> {
    // Get all team members
    let team_members = sqlx::query!(
        "SELECT user_id FROM team_members WHERE team_id = $1",
        team_id
    )
    .fetch_all(&state.db)
    .await?;

    let alert = WebSocketMessage::TeamAlert {
        team_id,
        alert_type: alert_type.clone(),
        data,
    };

    let json = serde_json::to_string(&alert)
        .map_err(|e| AppError::Internal(format!("Failed to serialize team alert: {}", e)))?;

    // Broadcast to all team members
    for member in team_members {
        state.broadcast_to_user(member.user_id, json.clone()).await;
    }

    info!("Team alert '{}' sent to {} members of team {}", 
          alert_type, team_members.len(), team_id);

    Ok(())
}

pub async fn send_system_message(state: &AppState, user_id: Uuid, message: String) {
    let sys_message = WebSocketMessage::SystemMessage { message };

    if let Ok(json) = serde_json::to_string(&sys_message) {
        state.broadcast_to_user(user_id, json).await;
    }
}

// WebSocket metrics and monitoring
pub struct WebSocketMetrics {
    pub active_connections: usize,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub connection_errors: u64,
}

impl WebSocketMetrics {
    pub fn new() -> Self {
        Self {
            active_connections: 0,
            messages_sent: 0,
            messages_received: 0,
            connection_errors: 0,
        }
    }
}

// Real-time performance monitoring
pub async fn monitor_websocket_performance(state: AppState) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(60));
    
    loop {
        interval.tick().await;
        
        let active_connections = state.websocket_connections.len();
        let active_sessions = state.get_active_sessions_count();
        
        debug!("WebSocket metrics: {} active connections, {} active sessions",
               active_connections, active_sessions);
        
        // Cleanup idle sessions
        state.cleanup_idle_sessions(30); // 30 minutes timeout
        
        // Send system health updates to connected clients
        if active_connections > 0 {
            let health_msg = WebSocketMessage::SystemMessage {
                message: format!(
                    "System healthy: {} active connections, {} sessions",
                    active_connections, active_sessions
                ),
            };
            
            if let Ok(json) = serde_json::to_string(&health_msg) {
                // This would broadcast to admin users only
                debug!("System health: {}", json);
            }
        }
    }
}

// Utility functions for testing WebSocket functionality
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_websocket_message_serialization() {
        let ping = WebSocketMessage::Ping { timestamp: 12345 };
        let json = serde_json::to_string(&ping).unwrap();
        assert!(json.contains("ping"));
        assert!(json.contains("12345"));

        let notification = WebSocketMessage::Notification {
            title: "Test".to_string(),
            message: "Test message".to_string(),
            level: NotificationLevel::Info,
            timestamp: 12345,
        };
        let json = serde_json::to_string(&notification).unwrap();
        assert!(json.contains("notification"));
        assert!(json.contains("Test"));
    }

    #[test]
    fn test_websocket_message_deserialization() {
        let json = r#"{"type":"ping","timestamp":12345}"#;
        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        match msg {
            WebSocketMessage::Ping { timestamp } => assert_eq!(timestamp, 12345),
            _ => panic!("Wrong message type"),
        }
    }
}