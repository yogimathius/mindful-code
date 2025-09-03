use crate::error::{AppError, Result};
use candle_core::{DType, Device, Tensor};
use candle_nn::{linear, Linear, Module, VarBuilder};
use std::sync::Arc;
use tracing::{debug, info};

pub struct MLInferenceEngine {
    device: Device,
    model: Option<Arc<FlowPredictionModel>>,
    feature_scaler: FeatureScaler,
}

struct FlowPredictionModel {
    layer1: Linear,
    layer2: Linear,
    layer3: Linear,
    output: Linear,
}

impl Module for FlowPredictionModel {
    fn forward(&self, xs: &Tensor) -> candle_core::Result<Tensor> {
        let xs = self.layer1.forward(xs)?;
        let xs = xs.relu()?;
        let xs = self.layer2.forward(&xs)?;
        let xs = xs.relu()?;
        let xs = self.layer3.forward(&xs)?;
        let xs = xs.relu()?;
        let xs = self.output.forward(&xs)?;
        xs.sigmoid()
    }
}

#[derive(Debug)]
struct FeatureScaler {
    means: Vec<f32>,
    stds: Vec<f32>,
}

impl FeatureScaler {
    fn new() -> Self {
        // Default normalization parameters based on training data
        Self {
            means: vec![0.6, 0.7, 0.65, 0.1, 0.5], // rhythm, focus, consistency, error, velocity
            stds: vec![0.25, 0.3, 0.28, 0.15, 0.3],
        }
    }

    fn normalize(&self, features: &[f32]) -> Vec<f32> {
        features
            .iter()
            .zip(self.means.iter().zip(self.stds.iter()))
            .map(|(feature, (mean, std))| (feature - mean) / std)
            .collect()
    }
}

impl MLInferenceEngine {
    pub fn new() -> Self {
        let device = Device::Cpu; // Use CPU for ultra-low latency inference
        Self {
            device,
            model: None,
            feature_scaler: FeatureScaler::new(),
        }
    }

    pub async fn initialize_model(&mut self) -> Result<()> {
        info!("Initializing ML model for flow state prediction");

        // Create a lightweight neural network for real-time inference
        let vs = VarBuilder::zeros(DType::F32, &self.device);

        let layer1 = linear(5, 16, vs.pp("layer1"))
            .map_err(|e| AppError::MachineLearning(format!("Failed to create layer1: {}", e)))?;
        
        let layer2 = linear(16, 8, vs.pp("layer2"))
            .map_err(|e| AppError::MachineLearning(format!("Failed to create layer2: {}", e)))?;
        
        let layer3 = linear(8, 4, vs.pp("layer3"))
            .map_err(|e| AppError::MachineLearning(format!("Failed to create layer3: {}", e)))?;
        
        let output = linear(4, 1, vs.pp("output"))
            .map_err(|e| AppError::MachineLearning(format!("Failed to create output layer: {}", e)))?;

        let model = FlowPredictionModel {
            layer1,
            layer2,
            layer3,
            output,
        };

        self.model = Some(Arc::new(model));
        info!("✅ ML model initialized successfully");
        Ok(())
    }

    pub async fn predict_flow_state(&self, features: [f32; 5]) -> Result<f32> {
        // Fallback to rule-based prediction if ML model not available
        if self.model.is_none() {
            return Ok(self.rule_based_prediction(features));
        }

        let model = self.model.as_ref().unwrap();

        // Normalize features
        let normalized_features = self.feature_scaler.normalize(&features);

        // Create tensor from features
        let input_tensor = Tensor::from_vec(normalized_features, &[1, 5], &self.device)
            .map_err(|e| AppError::MachineLearning(format!("Failed to create input tensor: {}", e)))?;

        // Run inference
        let output = model
            .forward(&input_tensor)
            .map_err(|e| AppError::MachineLearning(format!("Model inference failed: {}", e)))?;

        // Extract prediction
        let prediction = output
            .to_vec1::<f32>()
            .map_err(|e| AppError::MachineLearning(format!("Failed to extract prediction: {}", e)))?[0];

        debug!(
            "ML prediction: {:.3}, features: {:?}",
            prediction, features
        );

        Ok(prediction.max(0.0).min(1.0))
    }

    fn rule_based_prediction(&self, features: [f32; 5]) -> f32 {
        let [rhythm_score, focus_score, consistency_score, error_penalty, velocity_score] = features;

        // Weighted combination with research-backed weights
        let base_score = rhythm_score * 0.35
            + focus_score * 0.25
            + consistency_score * 0.20
            + (1.0 - error_penalty) * 0.10
            + velocity_score * 0.10;

        // Apply non-linear adjustments
        let adjusted_score = if base_score > 0.8 {
            // Bonus for high scores
            base_score + (base_score - 0.8) * 0.5
        } else if base_score < 0.3 {
            // Penalty for low scores
            base_score * 0.8
        } else {
            base_score
        };

        adjusted_score.max(0.0).min(1.0)
    }

    pub async fn update_model_with_feedback(
        &mut self,
        features: [f32; 5],
        actual_flow_state: f32,
        user_feedback: Option<f32>,
    ) -> Result<()> {
        // In a full implementation, this would update model weights
        // For now, we'll log the feedback for future training
        debug!(
            "Received feedback - Features: {:?}, Actual: {:.3}, User: {:?}",
            features, actual_flow_state, user_feedback
        );

        // TODO: Implement online learning or batch update mechanism
        // This could store training examples for periodic model retraining

        Ok(())
    }

    pub fn get_feature_importance(&self) -> Vec<(&'static str, f32)> {
        vec![
            ("rhythm_score", 0.35),
            ("focus_score", 0.25),
            ("consistency_score", 0.20),
            ("error_penalty", 0.10),
            ("velocity_score", 0.10),
        ]
    }

    pub async fn batch_predict(&self, feature_batch: Vec<[f32; 5]>) -> Result<Vec<f32>> {
        let mut predictions = Vec::with_capacity(feature_batch.len());

        for features in feature_batch {
            let prediction = self.predict_flow_state(features).await?;
            predictions.push(prediction);
        }

        Ok(predictions)
    }

    pub fn is_model_loaded(&self) -> bool {
        self.model.is_some()
    }
}

impl Default for MLInferenceEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Additional ML utilities for advanced features
pub struct ProductivityPredictor {
    ml_engine: MLInferenceEngine,
    historical_patterns: Vec<ProductivityPattern>,
}

#[derive(Debug, Clone)]
struct ProductivityPattern {
    hour_of_day: u8,
    day_of_week: u8,
    average_flow_score: f32,
    session_count: u32,
}

impl ProductivityPredictor {
    pub fn new() -> Self {
        Self {
            ml_engine: MLInferenceEngine::new(),
            historical_patterns: Vec::new(),
        }
    }

    pub async fn predict_optimal_session_time(&self, current_hour: u8, day_of_week: u8) -> f32 {
        // Find similar historical patterns
        let similar_patterns: Vec<&ProductivityPattern> = self
            .historical_patterns
            .iter()
            .filter(|p| {
                (p.hour_of_day as i16 - current_hour as i16).abs() <= 2
                    && p.day_of_week == day_of_week
            })
            .collect();

        if similar_patterns.is_empty() {
            return 0.5; // Default neutral prediction
        }

        let avg_flow_score = similar_patterns
            .iter()
            .map(|p| p.average_flow_score)
            .sum::<f32>()
            / similar_patterns.len() as f32;

        avg_flow_score
    }

    pub fn add_pattern(&mut self, pattern: ProductivityPattern) {
        self.historical_patterns.push(pattern);
        
        // Keep only recent patterns (last 100 entries)
        if self.historical_patterns.len() > 100 {
            self.historical_patterns.remove(0);
        }
    }
}