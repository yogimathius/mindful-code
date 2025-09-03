use crate::{
    error::{AppError, Result},
    models::flow::{FlowMetrics, FlowStateData, FlowStateResult, UserFlowPreferences},
    services::ml::MLInferenceEngine,
};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use tokio::time;
use tracing::{debug, info, warn};

pub struct FlowDetectionEngine {
    keystroke_buffer: VecDeque<u64>,
    flow_start_time: Option<Instant>,
    current_intensity: f32,
    ml_engine: MLInferenceEngine,
    last_analysis: Instant,
    flow_session_count: u32,
    total_flow_time: Duration,
    confidence_history: VecDeque<f32>,
}

impl FlowDetectionEngine {
    pub fn new() -> Self {
        Self {
            keystroke_buffer: VecDeque::with_capacity(100),
            flow_start_time: None,
            current_intensity: 0.0,
            ml_engine: MLInferenceEngine::new(),
            last_analysis: Instant::now(),
            flow_session_count: 0,
            total_flow_time: Duration::new(0, 0),
            confidence_history: VecDeque::with_capacity(50),
        }
    }

    pub async fn analyze_flow_state(
        &mut self,
        data: FlowStateData,
        user_preferences: Option<UserFlowPreferences>,
    ) -> Result<FlowStateResult> {
        let start_time = Instant::now();

        // Update keystroke buffer with ring buffer for memory efficiency
        for interval in &data.keystroke_intervals {
            self.keystroke_buffer.push_back(*interval);
            if self.keystroke_buffer.len() > 100 {
                self.keystroke_buffer.pop_front();
            }
        }

        // Calculate individual metrics
        let rhythm_score = self.analyze_keystroke_rhythm(&data.keystroke_intervals)?;
        let focus_score = self.calculate_focus_score(data.context_switches);
        let consistency_score = self.calculate_consistency_score(&data)?;
        let error_penalty = self.calculate_error_penalty(data.error_events);
        let velocity_score = self.calculate_velocity_score(&data)?;

        // Combine metrics using ML model for optimal weighting
        let combined_score = self
            .ml_engine
            .predict_flow_state([
                rhythm_score,
                focus_score,
                consistency_score,
                1.0 - error_penalty,
                velocity_score,
            ])
            .await?;

        let sensitivity = user_preferences
            .as_ref()
            .map(|p| p.sensitivity_level)
            .unwrap_or(0.7);

        let is_in_flow = combined_score > sensitivity;
        let flow_duration = self.calculate_flow_duration(is_in_flow);
        let confidence = self.calculate_confidence(combined_score, &data);

        // Update flow tracking state
        self.update_flow_tracking(is_in_flow, combined_score);
        self.confidence_history.push_back(confidence);
        if self.confidence_history.len() > 50 {
            self.confidence_history.pop_front();
        }

        let metrics = FlowMetrics {
            rhythm_score,
            focus_score,
            consistency_score,
            error_penalty,
            velocity_score,
        };

        let recommendations = self.generate_recommendations(combined_score, &data, &metrics);
        let analysis_time = start_time.elapsed().as_secs_f32() * 1000.0;

        debug!(
            "Flow analysis completed in {:.3}ms, score: {:.3}, is_in_flow: {}",
            analysis_time, combined_score, is_in_flow
        );

        // Ensure <1ms analysis time for real-time performance
        if analysis_time > 1.0 {
            warn!(
                "Flow analysis took {:.3}ms, exceeding 1ms target",
                analysis_time
            );
        }

        Ok(FlowStateResult {
            is_in_flow,
            flow_intensity: combined_score,
            flow_duration_ms: flow_duration.as_millis() as u64,
            confidence,
            recommendations,
            metrics,
            analysis_time_ms: analysis_time,
        })
    }

    fn analyze_keystroke_rhythm(&self, intervals: &[u64]) -> Result<f32> {
        if intervals.len() < 3 {
            return Ok(0.0);
        }

        let mean_interval = intervals.iter().sum::<u64>() as f32 / intervals.len() as f32;
        let variance = intervals
            .iter()
            .map(|&x| (x as f32 - mean_interval).powi(2))
            .sum::<f32>()
            / intervals.len() as f32;

        let std_dev = variance.sqrt();
        let coefficient_of_variation = if mean_interval > 0.0 {
            std_dev / mean_interval
        } else {
            return Ok(0.0);
        };

        // Optimal keystroke rhythm analysis based on research
        let score = match mean_interval {
            // Optimal flow rhythm: 80-200ms with low variance
            80.0..=200.0 if coefficient_of_variation < 0.3 => 0.95,
            // Good rhythm: 50-300ms with moderate variance
            50.0..=300.0 if coefficient_of_variation < 0.5 => 0.80,
            // Acceptable rhythm: 30-500ms with higher variance
            30.0..=500.0 if coefficient_of_variation < 0.8 => 0.60,
            // Poor rhythm patterns
            _ => 0.20,
        };

        // Bonus for sustained rhythm patterns
        let sustained_bonus = if intervals.len() > 20 {
            let recent_intervals = &intervals[intervals.len() - 20..];
            let recent_cv = self.calculate_coefficient_of_variation(recent_intervals);
            if recent_cv < coefficient_of_variation - 0.1 {
                0.1 // Improving rhythm gets bonus
            } else {
                0.0
            }
        } else {
            0.0
        };

        Ok((score + sustained_bonus).min(1.0))
    }

    fn calculate_focus_score(&self, context_switches: u32) -> f32 {
        // Exponential decay for context switching penalty
        let base_score = (-0.2 * context_switches as f32).exp();

        // Time-based focus bonus
        let time_since_last = self.last_analysis.elapsed().as_secs_f32();
        let time_bonus = if time_since_last > 10.0 {
            // Sustained focus bonus
            (time_since_last / 60.0).min(0.2)
        } else {
            0.0
        };

        (base_score + time_bonus).min(1.0)
    }

    fn calculate_consistency_score(&self, data: &FlowStateData) -> Result<f32> {
        if self.keystroke_buffer.len() < 10 {
            return Ok(0.5); // Neutral score for insufficient data
        }

        // Analyze typing pattern consistency over time
        let recent_intervals: Vec<u64> = self.keystroke_buffer.iter().rev().take(20).copied().collect();
        let all_intervals: Vec<u64> = self.keystroke_buffer.iter().copied().collect();

        let recent_cv = self.calculate_coefficient_of_variation(&recent_intervals);
        let overall_cv = self.calculate_coefficient_of_variation(&all_intervals);

        // Reward improving consistency
        let consistency_score = if recent_cv < overall_cv {
            (1.0 - recent_cv).max(0.0)
        } else {
            (1.0 - overall_cv).max(0.0)
        };

        // Factor in file modification patterns
        let mod_consistency = if data.file_modifications > 0 {
            let mod_rate = data.file_modifications as f32 / data.window_focus_duration as f32 * 1000.0;
            // Optimal modification rate: 0.5-2.0 modifications per second
            match mod_rate {
                0.5..=2.0 => 0.2,
                0.1..=0.5 => 0.1,
                _ => 0.0,
            }
        } else {
            0.0
        };

        Ok((consistency_score + mod_consistency).min(1.0))
    }

    fn calculate_error_penalty(&self, error_events: u32) -> f32 {
        // Logarithmic penalty for errors to avoid harsh punishment
        if error_events == 0 {
            0.0
        } else {
            (error_events as f32).ln() / 10.0
        }
    }

    fn calculate_velocity_score(&self, data: &FlowStateData) -> Result<f32> {
        if let Some(velocity) = data.typing_velocity {
            // Optimal typing velocity: 200-400 characters per minute
            match velocity {
                200.0..=400.0 => Ok(0.9),
                100.0..=200.0 => Ok(0.7),
                400.0..=600.0 => Ok(0.8),
                _ => Ok(0.5),
            }
        } else if !data.keystroke_intervals.is_empty() {
            // Calculate velocity from keystroke intervals
            let avg_interval_ms = data.keystroke_intervals.iter().sum::<u64>() as f32
                / data.keystroke_intervals.len() as f32;
            let chars_per_minute = 60000.0 / avg_interval_ms;

            match chars_per_minute {
                200.0..=400.0 => Ok(0.9),
                100.0..=600.0 => Ok(0.7),
                _ => Ok(0.5),
            }
        } else {
            Ok(0.5)
        }
    }

    fn calculate_coefficient_of_variation(&self, intervals: &[u64]) -> f32 {
        if intervals.len() < 2 {
            return 1.0;
        }

        let mean = intervals.iter().sum::<u64>() as f32 / intervals.len() as f32;
        let variance = intervals
            .iter()
            .map(|&x| (x as f32 - mean).powi(2))
            .sum::<f32>()
            / intervals.len() as f32;

        if mean > 0.0 {
            variance.sqrt() / mean
        } else {
            1.0
        }
    }

    fn calculate_flow_duration(&mut self, is_in_flow: bool) -> Duration {
        match (is_in_flow, self.flow_start_time) {
            (true, None) => {
                // Starting new flow session
                self.flow_start_time = Some(Instant::now());
                Duration::new(0, 0)
            }
            (true, Some(start_time)) => {
                // Continuing flow session
                start_time.elapsed()
            }
            (false, Some(start_time)) => {
                // Ending flow session
                let duration = start_time.elapsed();
                self.total_flow_time += duration;
                self.flow_session_count += 1;
                self.flow_start_time = None;
                duration
            }
            (false, None) => {
                // Not in flow, no active session
                Duration::new(0, 0)
            }
        }
    }

    fn calculate_confidence(&self, score: f32, data: &FlowStateData) -> f32 {
        let mut confidence = score;

        // Adjust confidence based on data quality
        let data_quality = if data.keystroke_intervals.len() >= 10 {
            1.0
        } else if data.keystroke_intervals.len() >= 5 {
            0.8
        } else {
            0.5
        };

        confidence *= data_quality;

        // Adjust based on recent confidence history
        if let Some(avg_recent_confidence) = self.get_average_recent_confidence() {
            let stability_factor = 1.0 - (confidence - avg_recent_confidence).abs() * 0.5;
            confidence *= stability_factor;
        }

        confidence.max(0.0).min(1.0)
    }

    fn get_average_recent_confidence(&self) -> Option<f32> {
        if self.confidence_history.len() < 3 {
            None
        } else {
            let recent: Vec<f32> = self.confidence_history.iter().rev().take(5).copied().collect();
            Some(recent.iter().sum::<f32>() / recent.len() as f32)
        }
    }

    fn update_flow_tracking(&mut self, is_in_flow: bool, intensity: f32) {
        self.current_intensity = intensity;
        self.last_analysis = Instant::now();

        if is_in_flow && self.flow_start_time.is_none() {
            info!("Flow state detected, starting new session");
        } else if !is_in_flow && self.flow_start_time.is_some() {
            info!("Flow state ended");
        }
    }

    fn generate_recommendations(
        &self,
        score: f32,
        data: &FlowStateData,
        metrics: &FlowMetrics,
    ) -> Vec<String> {
        let mut recommendations = Vec::new();

        if score < 0.4 {
            recommendations.push("Consider taking a 2-3 minute break to reset focus".to_string());
        }

        if data.context_switches > 5 {
            recommendations.push(
                "Try enabling focus mode or using a single monitor to reduce distractions"
                    .to_string(),
            );
        }

        if data.error_events > 3 {
            recommendations.push(
                "Slow down slightly - accuracy and flow go hand in hand".to_string(),
            );
        }

        if metrics.rhythm_score < 0.5 {
            recommendations.push(
                "Try to maintain a steady typing rhythm for better flow state".to_string(),
            );
        }

        if data.window_focus_duration < 300000 {
            // Less than 5 minutes
            recommendations.push(
                "Consider working in longer focused blocks for deeper flow states".to_string(),
            );
        }

        if metrics.velocity_score < 0.6 {
            recommendations.push(
                "Your typing pace seems off today - consider warming up or adjusting your setup"
                    .to_string(),
            );
        }

        if recommendations.is_empty() {
            recommendations.push("Great focus! Keep up the excellent work.".to_string());
        }

        recommendations
    }

    pub fn get_session_stats(&self) -> (u32, Duration) {
        (self.flow_session_count, self.total_flow_time)
    }

    pub fn reset_session_stats(&mut self) {
        self.flow_session_count = 0;
        self.total_flow_time = Duration::new(0, 0);
        self.flow_start_time = None;
    }
}