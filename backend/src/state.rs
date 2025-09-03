use crate::{config::Config, services::flow::FlowDetectionEngine};
use anyhow::Result;
use dashmap::DashMap;
use parking_lot::RwLock;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub config: Config,
    pub flow_engines: Arc<DashMap<Uuid, Arc<RwLock<FlowDetectionEngine>>>>,
    pub active_sessions: Arc<DashMap<Uuid, SessionInfo>>,
    pub websocket_connections: Arc<DashMap<Uuid, tokio::sync::mpsc::UnboundedSender<String>>>,
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub user_id: Uuid,
    pub session_id: Uuid,
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub last_activity: chrono::DateTime<chrono::Utc>,
    pub is_active: bool,
}

impl AppState {
    pub async fn new(config: Config) -> Result<Self> {
        // Connect to PostgreSQL with optimized pool settings
        let db = PgPoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(5)
            .acquire_timeout(std::time::Duration::from_secs(3))
            .idle_timeout(std::time::Duration::from_secs(600))
            .max_lifetime(std::time::Duration::from_secs(1800))
            .connect(&config.database_url)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;

        // Run database migrations
        sqlx::migrate!("./migrations")
            .run(&db)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to run migrations: {}", e))?;

        tracing::info!("✅ Database connected and migrations applied");

        Ok(Self {
            db,
            config,
            flow_engines: Arc::new(DashMap::new()),
            active_sessions: Arc::new(DashMap::new()),
            websocket_connections: Arc::new(DashMap::new()),
        })
    }

    pub fn get_or_create_flow_engine(&self, user_id: Uuid) -> Arc<RwLock<FlowDetectionEngine>> {
        self.flow_engines
            .entry(user_id)
            .or_insert_with(|| Arc::new(RwLock::new(FlowDetectionEngine::new())))
            .clone()
    }

    pub fn add_websocket_connection(
        &self,
        user_id: Uuid,
        sender: tokio::sync::mpsc::UnboundedSender<String>,
    ) {
        self.websocket_connections.insert(user_id, sender);
        tracing::info!("WebSocket connection added for user {}", user_id);
    }

    pub fn remove_websocket_connection(&self, user_id: Uuid) {
        self.websocket_connections.remove(&user_id);
        tracing::info!("WebSocket connection removed for user {}", user_id);
    }

    pub async fn broadcast_to_user(&self, user_id: Uuid, message: String) {
        if let Some((_, sender)) = self.websocket_connections.get(&user_id) {
            if let Err(e) = sender.send(message) {
                tracing::warn!("Failed to send WebSocket message to user {}: {}", user_id, e);
                // Remove the stale connection
                self.websocket_connections.remove(&user_id);
            }
        }
    }

    pub fn update_session_activity(&self, session_id: Uuid) {
        if let Some(mut session) = self.active_sessions.get_mut(&session_id) {
            session.last_activity = chrono::Utc::now();
        }
    }

    pub fn add_active_session(&self, session_info: SessionInfo) {
        self.active_sessions
            .insert(session_info.session_id, session_info);
    }

    pub fn remove_active_session(&self, session_id: Uuid) {
        self.active_sessions.remove(&session_id);
    }

    pub fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.len()
    }

    pub fn cleanup_idle_sessions(&self, idle_timeout_minutes: i64) {
        let cutoff_time = chrono::Utc::now() - chrono::Duration::minutes(idle_timeout_minutes);
        let mut to_remove = Vec::new();

        for entry in self.active_sessions.iter() {
            if entry.value().last_activity < cutoff_time {
                to_remove.push(*entry.key());
            }
        }

        for session_id in to_remove {
            self.active_sessions.remove(&session_id);
            tracing::info!("Cleaned up idle session: {}", session_id);
        }
    }
}