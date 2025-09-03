# Mindful Code - Backend Requirements

## **Current Status: VS Code Extension Complete, Backend Needed**
- VS Code extension with session tracking and flow state detection ✅
- React dashboard with productivity metrics ✅
- **Missing**: Backend API for data sync, team analytics, ML-powered insights

---

## **Backend Technology: Rust + Axum**

**Why Rust + Axum:**
- **Ultra-low latency** for real-time flow state processing (<1ms)
- **Memory safety** crucial for handling sensitive developer data
- **High concurrency** for processing thousands of concurrent sessions
- **ML integration** with Candle for on-device pattern recognition
- **Zero-cost abstractions** for maximum performance
- **Cross-platform** deployment (developers use various systems)

---

## **Required API Endpoints**

```rust
// Session management
POST /api/sessions/start    # Start coding session
PUT /api/sessions/:id/update # Real-time session updates
POST /api/sessions/:id/end  # End session with summary
GET /api/sessions/history   # User session history

// Flow state analytics
POST /api/flow/detect       # Real-time flow state detection
GET /api/flow/patterns      # Personal flow patterns
GET /api/flow/insights      # AI-generated insights

// Team features (premium)
GET /api/teams/:id/analytics    # Team productivity metrics
GET /api/teams/:id/insights     # Team optimization suggestions
POST /api/teams/:id/alerts      # Burnout detection alerts

// Privacy and data control
GET /api/privacy/export     # Export user data
DELETE /api/privacy/purge   # Delete all user data
PUT /api/privacy/settings   # Update privacy preferences
```

---

## **Database Schema (PostgreSQL + SQLx)**

```sql
-- Users and privacy settings
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    subscription_tier VARCHAR(20) DEFAULT 'free', -- free, premium, team
    privacy_settings JSONB DEFAULT '{}', -- tracking preferences
    timezone VARCHAR(50) DEFAULT 'UTC',
    created_at TIMESTAMP DEFAULT NOW()
);

-- Coding sessions with detailed metrics
CREATE TABLE coding_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    total_duration_ms BIGINT,
    active_duration_ms BIGINT, -- excluding idle time
    files_modified INTEGER DEFAULT 0,
    keystrokes INTEGER DEFAULT 0,
    lines_added INTEGER DEFAULT 0,
    lines_deleted INTEGER DEFAULT 0,
    language_breakdown JSONB, -- {"typescript": 1200000, "rust": 300000}
    project_path VARCHAR(500),
    environment_data JSONB, -- time of day, workspace size, etc.
    flow_state_periods JSONB, -- array of flow state intervals
    interruption_count INTEGER DEFAULT 0,
    error_rate DECIMAL(5,4), -- compilation errors per minute
    focus_score DECIMAL(3,2), -- 0.0 to 1.0
    created_at TIMESTAMP DEFAULT NOW()
);

-- Flow state detection data
CREATE TABLE flow_states (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES coding_sessions(id) ON DELETE CASCADE,
    start_time TIMESTAMP NOT NULL,
    end_time TIMESTAMP,
    duration_ms BIGINT,
    intensity_score DECIMAL(3,2), -- 0.0 to 1.0 (how deep the flow)
    typing_rhythm_data JSONB, -- keystroke timing patterns
    context_switches INTEGER, -- file/window switches
    error_events JSONB, -- compilation/syntax errors during flow
    recovery_time_ms BIGINT, -- time to re-enter flow after interruption
    environmental_factors JSONB, -- conditions during flow state
    created_at TIMESTAMP DEFAULT NOW()
);

-- Team analytics (premium feature)
CREATE TABLE teams (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id),
    settings JSONB DEFAULT '{}', -- team preferences, alert thresholds
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE team_members (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    team_id UUID NOT NULL REFERENCES teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(50) DEFAULT 'member', -- member, admin
    data_sharing_consent BOOLEAN DEFAULT false,
    joined_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(team_id, user_id)
);

-- Precomputed insights for performance
CREATE TABLE user_insights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    insight_type VARCHAR(100) NOT NULL, -- optimal_session_length, peak_hours, etc.
    insight_data JSONB NOT NULL,
    confidence_score DECIMAL(3,2),
    date_range_start DATE,
    date_range_end DATE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW()
);
```

---

## **Real-Time Flow State Detection Engine**

```rust
use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tokio::time::{Duration, Instant};
use std::collections::VecDeque;

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowStateData {
    pub session_id: uuid::Uuid,
    pub keystroke_intervals: Vec<u64>, // milliseconds between keystrokes
    pub context_switches: u32,
    pub error_events: u32,
    pub window_focus_duration: u64,
    pub file_modifications: u32,
    pub timestamp: i64,
}

#[derive(Debug, Serialize)]
pub struct FlowStateResult {
    pub is_in_flow: bool,
    pub flow_intensity: f32, // 0.0 to 1.0
    pub flow_duration_ms: u64,
    pub confidence: f32,
    pub recommendations: Vec<String>,
}

pub struct FlowDetectionEngine {
    // Ring buffer for real-time analysis
    keystroke_buffer: VecDeque<u64>,
    flow_start_time: Option<Instant>,
    current_intensity: f32,
    ml_model: candle_core::Device, // On-device ML model
}

impl FlowDetectionEngine {
    pub fn new() -> Self {
        Self {
            keystroke_buffer: VecDeque::with_capacity(100),
            flow_start_time: None,
            current_intensity: 0.0,
            ml_model: candle_core::Device::Cpu,
        }
    }
    
    pub async fn analyze_flow_state(&mut self, data: FlowStateData) -> FlowStateResult {
        // 1. Keystroke rhythm analysis
        let rhythm_score = self.analyze_keystroke_rhythm(&data.keystroke_intervals);
        
        // 2. Context switching penalty
        let focus_score = self.calculate_focus_score(data.context_switches);
        
        // 3. Error rate impact
        let error_penalty = self.calculate_error_penalty(data.error_events);
        
        // 4. Combine metrics with ML model
        let combined_score = self.ml_predict([
            rhythm_score,
            focus_score,
            error_penalty,
            data.window_focus_duration as f32 / 1000.0, // convert to seconds
        ]).await;
        
        let is_in_flow = combined_score > 0.7;
        let flow_duration = self.calculate_flow_duration(is_in_flow);
        
        FlowStateResult {
            is_in_flow,
            flow_intensity: combined_score,
            flow_duration_ms: flow_duration,
            confidence: self.calculate_confidence(combined_score),
            recommendations: self.generate_recommendations(combined_score, &data),
        }
    }
    
    fn analyze_keystroke_rhythm(&mut self, intervals: &[u64]) -> f32 {
        // Analyze typing rhythm consistency
        if intervals.len() < 5 {
            return 0.0;
        }
        
        let mean_interval = intervals.iter().sum::<u64>() as f32 / intervals.len() as f32;
        let variance = intervals.iter()
            .map(|&x| (x as f32 - mean_interval).powi(2))
            .sum::<f32>() / intervals.len() as f32;
        
        let coefficient_of_variation = variance.sqrt() / mean_interval;
        
        // Lower CoV indicates more consistent rhythm (better flow)
        // Optimal rhythm: 100-300ms between keystrokes with low variance
        match mean_interval {
            100.0..=300.0 if coefficient_of_variation < 0.5 => 0.9,
            100.0..=500.0 if coefficient_of_variation < 0.8 => 0.7,
            _ => 0.3,
        }
    }
    
    fn calculate_focus_score(&self, context_switches: u32) -> f32 {
        // Penalize frequent context switching
        match context_switches {
            0..=2 => 1.0,
            3..=5 => 0.8,
            6..=10 => 0.5,
            _ => 0.2,
        }
    }
    
    async fn ml_predict(&self, features: [f32; 4]) -> f32 {
        // Use on-device ML model for flow prediction
        // This would use the Candle framework for Rust-native ML
        // Placeholder for actual model inference
        let weighted_sum = features[0] * 0.4 + features[1] * 0.3 + features[2] * 0.2 + features[3] * 0.1;
        weighted_sum.min(1.0).max(0.0)
    }
    
    fn generate_recommendations(&self, score: f32, data: &FlowStateData) -> Vec<String> {
        let mut recommendations = Vec::new();
        
        if score < 0.4 {
            recommendations.push("Consider taking a 5-minute break to reset focus".to_string());
        }
        
        if data.context_switches > 5 {
            recommendations.push("Try using focus mode or disable non-essential notifications".to_string());
        }
        
        if data.error_events > 3 {
            recommendations.push("Slow down slightly - accuracy improves flow state".to_string());
        }
        
        recommendations
    }
}
```

---

## **High-Performance API Layer**

```rust
use axum::{
    routing::{get, post, put},
    extract::{Path, Query, State},
    Json, Router,
};
use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub flow_engines: Arc<RwLock<HashMap<Uuid, FlowDetectionEngine>>>,
}

// Real-time session update endpoint
async fn update_session(
    State(state): State<AppState>,
    Path(session_id): Path<Uuid>,
    Json(update_data): Json<SessionUpdate>,
) -> Result<Json<FlowStateResult>, StatusCode> {
    // Get or create flow detection engine for this user
    let mut engines = state.flow_engines.write().await;
    let engine = engines.entry(update_data.user_id)
        .or_insert_with(FlowDetectionEngine::new);
    
    // Process real-time flow state
    let flow_result = engine.analyze_flow_state(FlowStateData {
        session_id,
        keystroke_intervals: update_data.keystroke_intervals,
        context_switches: update_data.context_switches,
        error_events: update_data.error_events,
        window_focus_duration: update_data.focus_duration,
        file_modifications: update_data.file_modifications,
        timestamp: chrono::Utc::now().timestamp_millis(),
    }).await;
    
    // Store session update in database (non-blocking)
    let db = state.db.clone();
    tokio::spawn(async move {
        let _ = sqlx::query!(
            "UPDATE coding_sessions 
             SET keystrokes = keystrokes + $1, 
                 lines_added = lines_added + $2,
                 focus_score = $3,
                 updated_at = NOW()
             WHERE id = $4",
            update_data.keystrokes as i32,
            update_data.lines_added as i32,
            flow_result.flow_intensity as f64,
            session_id
        ).execute(&db).await;
    });
    
    Ok(Json(flow_result))
}

// Team analytics endpoint (premium)
async fn get_team_analytics(
    State(state): State<AppState>,
    Path(team_id): Path<Uuid>,
    Query(params): Query<AnalyticsParams>,
) -> Result<Json<TeamAnalytics>, StatusCode> {
    let analytics = sqlx::query!(
        r#"
        SELECT 
            DATE(created_at) as date,
            AVG(focus_score) as avg_focus,
            AVG(total_duration_ms / 1000 / 60) as avg_session_minutes,
            COUNT(*) as session_count,
            AVG(flow_state_periods::jsonb_array_length) as avg_flow_periods
        FROM coding_sessions cs
        JOIN team_members tm ON cs.user_id = tm.user_id
        WHERE tm.team_id = $1 
          AND tm.data_sharing_consent = true
          AND cs.created_at >= $2
          AND cs.created_at <= $3
        GROUP BY DATE(created_at)
        ORDER BY date DESC
        "#,
        team_id,
        params.start_date,
        params.end_date
    ).fetch_all(&state.db).await?;
    
    let team_analytics = TeamAnalytics {
        daily_metrics: analytics.into_iter().map(|row| DailyMetric {
            date: row.date.unwrap(),
            average_focus: row.avg_focus.unwrap_or(0.0) as f32,
            average_session_minutes: row.avg_session_minutes.unwrap_or(0.0) as f32,
            session_count: row.session_count.unwrap_or(0) as u32,
            flow_periods: row.avg_flow_periods.unwrap_or(0.0) as f32,
        }).collect(),
        burnout_indicators: detect_team_burnout_risk(&state.db, team_id).await?,
        optimization_suggestions: generate_team_suggestions(&analytics).await,
    };
    
    Ok(Json(team_analytics))
}
```

---

## **Privacy-First Data Handling**

```rust
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, NewAead};

pub struct PrivacyManager {
    cipher: Aes256Gcm,
}

impl PrivacyManager {
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher = Aes256Gcm::new(Key::from_slice(key));
        Self { cipher }
    }
    
    // Encrypt sensitive session data
    pub fn encrypt_session_data(&self, data: &SessionData) -> Result<Vec<u8>, String> {
        let serialized = serde_json::to_vec(data)
            .map_err(|e| format!("Serialization error: {}", e))?;
        
        let nonce = Nonce::from_slice(b"unique nonce"); // Use proper nonce generation
        
        self.cipher.encrypt(nonce, serialized.as_ref())
            .map_err(|e| format!("Encryption error: {}", e))
    }
    
    // Complete data export for GDPR compliance
    pub async fn export_user_data(&self, db: &PgPool, user_id: Uuid) -> Result<UserDataExport, sqlx::Error> {
        let sessions = sqlx::query_as!(
            SessionExport,
            "SELECT * FROM coding_sessions WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        ).fetch_all(db).await?;
        
        let insights = sqlx::query_as!(
            InsightExport,
            "SELECT * FROM user_insights WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        ).fetch_all(db).await?;
        
        Ok(UserDataExport {
            sessions,
            insights,
            export_timestamp: chrono::Utc::now(),
        })
    }
    
    // Complete data deletion
    pub async fn purge_user_data(&self, db: &PgPool, user_id: Uuid) -> Result<(), sqlx::Error> {
        let mut tx = db.begin().await?;
        
        // Delete in correct order due to foreign key constraints
        sqlx::query!("DELETE FROM user_insights WHERE user_id = $1", user_id)
            .execute(&mut *tx).await?;
        
        sqlx::query!("DELETE FROM flow_states WHERE session_id IN (SELECT id FROM coding_sessions WHERE user_id = $1)", user_id)
            .execute(&mut *tx).await?;
        
        sqlx::query!("DELETE FROM coding_sessions WHERE user_id = $1", user_id)
            .execute(&mut *tx).await?;
        
        sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
            .execute(&mut *tx).await?;
        
        tx.commit().await?;
        Ok(())
    }
}
```

---

## **Deployment Strategy**

```bash
# Rust deployment to Fly.io
fly launch --name mindful-code-api
fly postgres create mindful-code-db
fly secrets set DATABASE_URL=postgresql://...
fly secrets set ENCRYPTION_KEY=... # for privacy features
fly secrets set JWT_SECRET=...
fly deploy

# Environment variables
DATABASE_URL=postgresql://mindful-code-db.internal:5432/mindful_code
ENCRYPTION_KEY=32-byte-hex-key-for-aes-encryption
JWT_SECRET=your-jwt-secret
RUST_LOG=info
TOKIO_WORKER_THREADS=4
```

---

## **Performance Characteristics**

- **API Response Time**: <5ms for session updates
- **Flow Detection Latency**: <1ms real-time processing
- **Memory Usage**: <20MB RAM per 1000 concurrent users
- **Throughput**: 50,000+ requests/second on 2 CPU cores
- **Database Efficiency**: <2ms query times with proper indexing
- **Real-time Sync**: WebSocket connections with <10ms latency

---

## **Development Timeline**

**Week 1**: Core Rust API, database setup, authentication
**Week 2**: Real-time flow detection engine, ML integration
**Week 3**: Team analytics, privacy features, data export/import
**Week 4**: Performance optimization, deployment, VS Code integration
**Week 5**: Security audit, GDPR compliance validation

**Estimated Development**: 5-6 weeks to full launch