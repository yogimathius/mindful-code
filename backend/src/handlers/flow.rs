use axum::{extract::State, Json};
use serde::{Deserialize, Serialize};
use tracing::{debug, instrument};
use uuid::Uuid;
use validator::Validate;

use crate::{
    error::{AppError, Result},
    models::flow::{FlowDetectionRequest, FlowStateResult, FlowPattern, FlowInsight, FlowAnalytics},
    state::AppState,
    utils::auth::Claims,
};

#[derive(Debug, Deserialize, Validate)]
pub struct FlowDetectionPayload {
    #[validate(nested)]
    pub request: FlowDetectionRequest,
}

#[instrument(skip(state, claims))]
pub async fn detect_flow_state(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<FlowDetectionPayload>,
) -> Result<Json<FlowStateResult>> {
    // Validate input
    payload.validate().map_err(|e| {
        AppError::Validation(format!("Invalid flow detection request: {}", e))
    })?;

    let user_id = claims.user_id;
    let flow_data = payload.request.flow_data;
    let user_preferences = payload.request.user_preferences;

    // Get or create flow detection engine for this user
    let flow_engine_arc = state.get_or_create_flow_engine(user_id);
    let mut flow_engine = flow_engine_arc.write();

    // Analyze flow state with ultra-low latency
    let flow_result = flow_engine
        .analyze_flow_state(flow_data.clone(), user_preferences)
        .await?;

    // Update session activity
    state.update_session_activity(flow_data.session_id);

    // Store flow state in database (async, non-blocking)
    let db = state.db.clone();
    let session_id = flow_data.session_id;
    let flow_result_clone = flow_result.clone();
    
    tokio::spawn(async move {
        let result = sqlx::query!(
            r#"
            INSERT INTO flow_states (
                session_id, start_time, intensity_score, typing_rhythm_data,
                context_switches, ml_features, confidence_score
            ) VALUES ($1, NOW(), $2, $3, $4, $5, $6)
            "#,
            session_id,
            flow_result_clone.flow_intensity as f64,
            serde_json::to_value(&flow_result_clone.metrics).unwrap_or_default(),
            flow_result_clone.metrics.focus_score as i32,
            serde_json::json!({
                "rhythm_score": flow_result_clone.metrics.rhythm_score,
                "focus_score": flow_result_clone.metrics.focus_score,
                "consistency_score": flow_result_clone.metrics.consistency_score,
                "velocity_score": flow_result_clone.metrics.velocity_score,
                "error_penalty": flow_result_clone.metrics.error_penalty
            }),
            flow_result_clone.confidence as f64,
        ).execute(&db).await;

        if let Err(e) = result {
            tracing::error!("Failed to store flow state: {}", e);
        }
    });

    // Send real-time update via WebSocket
    let websocket_message = serde_json::json!({
        "type": "flow_state_update",
        "session_id": flow_data.session_id,
        "flow_state": &flow_result
    }).to_string();

    state.broadcast_to_user(user_id, websocket_message).await;

    debug!(
        "Flow state detected for user {}: intensity={:.3}, in_flow={}",
        user_id, flow_result.flow_intensity, flow_result.is_in_flow
    );

    Ok(Json(flow_result))
}

pub async fn get_flow_patterns(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<FlowPattern>> {
    let user_id = claims.user_id;

    // Query flow patterns from the database
    let patterns = sqlx::query!(
        r#"
        SELECT 
            AVG(cs.total_duration_ms) as avg_session_length,
            AVG(fs.intensity_score) as avg_flow_intensity,
            json_agg(DISTINCT cs.language_breakdown) as languages
        FROM coding_sessions cs
        LEFT JOIN flow_states fs ON cs.id = fs.session_id
        WHERE cs.user_id = $1 
          AND cs.end_time IS NOT NULL
          AND cs.created_at >= NOW() - INTERVAL '30 days'
        "#,
        user_id
    ).fetch_optional(&state.db).await?;

    let flow_pattern = if let Some(row) = patterns {
        // Analyze peak hours from session data
        let peak_hours_query = sqlx::query!(
            r#"
            SELECT 
                EXTRACT(HOUR FROM start_time) as hour,
                AVG(focus_score) as avg_focus,
                COUNT(*) as session_count
            FROM coding_sessions
            WHERE user_id = $1 
              AND created_at >= NOW() - INTERVAL '30 days'
            GROUP BY EXTRACT(HOUR FROM start_time)
            ORDER BY avg_focus DESC, session_count DESC
            LIMIT 3
            "#,
            user_id
        ).fetch_all(&state.db).await?;

        let peak_hours: Vec<u8> = peak_hours_query
            .into_iter()
            .filter_map(|row| row.hour.map(|h| h as u8))
            .collect();

        let languages = row.languages
            .and_then(|l| serde_json::from_value::<Vec<serde_json::Value>>(l).ok())
            .unwrap_or_default();

        let best_languages = languages
            .into_iter()
            .filter_map(|lang_data| {
                lang_data.as_object()?.keys().next().cloned()
            })
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .take(5)
            .collect();

        FlowPattern {
            user_id,
            optimal_session_length: row.avg_session_length.unwrap_or(0) as u64,
            peak_hours,
            average_flow_intensity: row.avg_flow_intensity.unwrap_or(0.0) as f32,
            flow_triggers: vec![
                "Consistent typing rhythm".to_string(),
                "Minimal context switching".to_string(),
                "Quiet environment".to_string(),
            ],
            interruption_tolerance: 0.3, // Based on analysis
            best_languages,
            environmental_factors: serde_json::json!({
                "preferred_session_length": row.avg_session_length,
                "optimal_break_frequency": 25
            }),
        }
    } else {
        // Default pattern for new users
        FlowPattern {
            user_id,
            optimal_session_length: 1800000, // 30 minutes default
            peak_hours: vec![9, 14, 16], // Common productive hours
            average_flow_intensity: 0.0,
            flow_triggers: vec!["Getting started with tracking...".to_string()],
            interruption_tolerance: 0.5,
            best_languages: vec!["javascript".to_string(), "typescript".to_string()],
            environmental_factors: serde_json::json!({}),
        }
    };

    Ok(Json(flow_pattern))
}

pub async fn get_flow_insights(
    State(state): State<AppState>,
    claims: Claims,
) -> Result<Json<Vec<FlowInsight>>> {
    let user_id = claims.user_id;

    // Query recent insights
    let insights_data = sqlx::query!(
        r#"
        SELECT 
            insight_type,
            insight_data,
            confidence_score,
            created_at
        FROM user_insights
        WHERE user_id = $1 
          AND is_active = true
          AND created_at >= NOW() - INTERVAL '7 days'
        ORDER BY confidence_score DESC, created_at DESC
        LIMIT 10
        "#,
        user_id
    ).fetch_all(&state.db).await?;

    let mut insights = Vec::new();

    for row in insights_data {
        if let Ok(insight_data) = serde_json::from_value::<serde_json::Value>(row.insight_data) {
            let insight = FlowInsight {
                insight_type: row.insight_type,
                title: insight_data["title"]
                    .as_str()
                    .unwrap_or("Productivity Insight")
                    .to_string(),
                description: insight_data["description"]
                    .as_str()
                    .unwrap_or("No description available")
                    .to_string(),
                impact_score: insight_data["impact_score"]
                    .as_f64()
                    .unwrap_or(0.5) as f32,
                actionable_suggestions: insight_data["suggestions"]
                    .as_array()
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default(),
                confidence: row.confidence_score.unwrap_or(0.0) as f32,
                data_points: insight_data["data_points"]
                    .as_u64()
                    .unwrap_or(0) as u32,
                time_range: "Last 7 days".to_string(),
            };
            insights.push(insight);
        }
    }

    // If no insights exist, generate some basic ones
    if insights.is_empty() {
        let basic_insights = vec![
            FlowInsight {
                insight_type: "getting_started".to_string(),
                title: "Welcome to Flow State Tracking".to_string(),
                description: "Start coding to begin tracking your flow states and productivity patterns.".to_string(),
                impact_score: 0.8,
                actionable_suggestions: vec![
                    "Begin a coding session to start collecting data".to_string(),
                    "Try to maintain consistent typing rhythms".to_string(),
                    "Minimize context switching between files".to_string(),
                ],
                confidence: 0.9,
                data_points: 0,
                time_range: "No data yet".to_string(),
            }
        ];
        insights.extend(basic_insights);
    }

    Ok(Json(insights))
}

#[derive(Debug, Deserialize)]
pub struct FlowAnalyticsQuery {
    pub days: Option<i32>,
}

pub async fn get_flow_analytics(
    State(state): State<AppState>,
    claims: Claims,
    Json(query): Json<FlowAnalyticsQuery>,
) -> Result<Json<FlowAnalytics>> {
    let user_id = claims.user_id;
    let days = query.days.unwrap_or(30).min(365); // Max 365 days

    let analytics_data = sqlx::query!(
        r#"
        SELECT 
            SUM(COALESCE(fs.duration_ms, 0)) as total_flow_time,
            AVG(fs.intensity_score) as avg_intensity,
            COUNT(DISTINCT fs.session_id) as flow_sessions,
            MAX(fs.duration_ms) as longest_flow,
            AVG(cs.interruption_count::float / GREATEST(cs.total_duration_ms::float / 60000, 1)) as interruption_rate,
            AVG(cs.focus_score) as productivity_score
        FROM flow_states fs
        JOIN coding_sessions cs ON fs.session_id = cs.id
        WHERE cs.user_id = $1 
          AND fs.created_at >= NOW() - INTERVAL '%s days'
        "#,
        user_id,
        &format!("{}", days)
    ).fetch_optional(&state.db).await?;

    let daily_data = sqlx::query!(
        r#"
        SELECT 
            DATE(fs.start_time) as date,
            SUM(COALESCE(fs.duration_ms, 0)) as daily_flow_time,
            COUNT(*) as session_count,
            AVG(fs.intensity_score) as avg_intensity
        FROM flow_states fs
        JOIN coding_sessions cs ON fs.session_id = cs.id
        WHERE cs.user_id = $1 
          AND fs.created_at >= NOW() - INTERVAL '%s days'
        GROUP BY DATE(fs.start_time)
        ORDER BY date DESC
        "#,
        user_id,
        &format!("{}", days)
    ).fetch_all(&state.db).await?;

    let daily_distribution = daily_data
        .into_iter()
        .map(|row| crate::models::flow::DailyFlowData {
            date: row.date.unwrap_or_else(|| chrono::Utc::now().date_naive()),
            total_flow_time_ms: row.daily_flow_time.unwrap_or(0) as u64,
            session_count: row.session_count.unwrap_or(0) as u32,
            average_intensity: row.avg_intensity.unwrap_or(0.0) as f32,
            peak_intensity_hour: None, // Could be calculated if needed
        })
        .collect();

    let analytics = if let Some(data) = analytics_data {
        FlowAnalytics {
            total_flow_time_ms: data.total_flow_time.unwrap_or(0) as u64,
            average_flow_intensity: data.avg_intensity.unwrap_or(0.0) as f32,
            flow_sessions_count: data.flow_sessions.unwrap_or(0) as u32,
            longest_flow_session_ms: data.longest_flow.unwrap_or(0) as u64,
            interruption_rate: data.interruption_rate.unwrap_or(0.0) as f32,
            productivity_score: data.productivity_score.unwrap_or(0.0) as f32,
            weekly_trend: 0.0, // Could calculate week-over-week change
            daily_distribution,
        }
    } else {
        FlowAnalytics {
            total_flow_time_ms: 0,
            average_flow_intensity: 0.0,
            flow_sessions_count: 0,
            longest_flow_session_ms: 0,
            interruption_rate: 0.0,
            productivity_score: 0.0,
            weekly_trend: 0.0,
            daily_distribution: vec![],
        }
    };

    Ok(Json(analytics))
}