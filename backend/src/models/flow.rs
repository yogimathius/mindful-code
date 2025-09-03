use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, Validate)]
pub struct FlowStateData {
    pub session_id: Uuid,
    #[validate(length(min = 1, max = 1000))]
    pub keystroke_intervals: Vec<u64>,
    #[validate(range(min = 0, max = 1000))]
    pub context_switches: u32,
    #[validate(range(min = 0, max = 1000))]
    pub error_events: u32,
    #[validate(range(min = 0))]
    pub window_focus_duration: u64,
    #[validate(range(min = 0, max = 10000))]
    pub file_modifications: u32,
    pub timestamp: i64,
    pub typing_velocity: Option<f32>,
    pub pause_patterns: Option<Vec<u64>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowStateResult {
    pub is_in_flow: bool,
    pub flow_intensity: f32,
    pub flow_duration_ms: u64,
    pub confidence: f32,
    pub recommendations: Vec<String>,
    pub metrics: FlowMetrics,
    pub analysis_time_ms: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowMetrics {
    pub rhythm_score: f32,
    pub focus_score: f32,
    pub consistency_score: f32,
    pub error_penalty: f32,
    pub velocity_score: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowPattern {
    pub user_id: Uuid,
    pub optimal_session_length: u64,
    pub peak_hours: Vec<u8>,
    pub average_flow_intensity: f32,
    pub flow_triggers: Vec<String>,
    pub interruption_tolerance: f32,
    pub best_languages: Vec<String>,
    pub environmental_factors: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlowInsight {
    pub insight_type: String,
    pub title: String,
    pub description: String,
    pub impact_score: f32,
    pub actionable_suggestions: Vec<String>,
    pub confidence: f32,
    pub data_points: u32,
    pub time_range: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowDetectionRequest {
    pub flow_data: FlowStateData,
    pub user_preferences: Option<UserFlowPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserFlowPreferences {
    pub sensitivity_level: f32,
    pub notification_threshold: f32,
    pub focus_mode_enabled: bool,
    pub break_reminders_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FlowAnalytics {
    pub total_flow_time_ms: u64,
    pub average_flow_intensity: f32,
    pub flow_sessions_count: u32,
    pub longest_flow_session_ms: u64,
    pub interruption_rate: f32,
    pub productivity_score: f32,
    pub weekly_trend: f32,
    pub daily_distribution: Vec<DailyFlowData>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DailyFlowData {
    pub date: chrono::NaiveDate,
    pub total_flow_time_ms: u64,
    pub session_count: u32,
    pub average_intensity: f32,
    pub peak_intensity_hour: Option<u8>,
}