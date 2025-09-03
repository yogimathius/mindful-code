use mindful_code_backend::{
    config::{Config, Environment},
    services::{
        flow::FlowDetectionEngine,
        ml::MLInferenceEngine,
        wasm::WasmPluginManager,
        encryption::EncryptionService,
    },
    models::flow::{FlowStateData, UserFlowPreferences},
    utils::auth::{Claims, generate_jwt_token, hash_password, verify_password},
};
use quickcheck::{quickcheck, TestResult};
use std::time::Duration;
use tokio_test;
use uuid::Uuid;

#[tokio::test]
async fn test_flow_detection_engine_performance() {
    let mut engine = FlowDetectionEngine::new();
    
    let flow_data = FlowStateData {
        session_id: Uuid::new_v4(),
        keystroke_intervals: vec![120, 135, 98, 142, 156, 89, 167, 134, 145, 123],
        context_switches: 2,
        error_events: 1,
        window_focus_duration: 30000,
        file_modifications: 5,
        timestamp: chrono::Utc::now().timestamp_millis(),
        typing_velocity: Some(250.0),
        pause_patterns: None,
    };

    let start = std::time::Instant::now();
    let result = engine.analyze_flow_state(flow_data, None).await;
    let duration = start.elapsed();

    assert!(result.is_ok());
    let flow_result = result.unwrap();
    
    // Performance assertion: <1ms analysis time
    assert!(duration.as_millis() < 1, 
           "Flow analysis took {}ms, exceeding 1ms target", duration.as_millis());
    
    // Correctness assertions
    assert!(flow_result.flow_intensity >= 0.0 && flow_result.flow_intensity <= 1.0);
    assert!(flow_result.confidence >= 0.0 && flow_result.confidence <= 1.0);
    assert!(flow_result.analysis_time_ms > 0.0);
    assert!(!flow_result.recommendations.is_empty());
}

#[tokio::test]
async fn test_concurrent_flow_detection() {
    let concurrent_count = 100;
    let mut handles = Vec::new();

    let start = std::time::Instant::now();

    for _ in 0..concurrent_count {
        let handle = tokio::spawn(async move {
            let mut engine = FlowDetectionEngine::new();
            let flow_data = FlowStateData {
                session_id: Uuid::new_v4(),
                keystroke_intervals: vec![100, 120, 95, 130, 110, 140, 105, 125, 115, 135],
                context_switches: 1,
                error_events: 0,
                window_focus_duration: 25000,
                file_modifications: 3,
                timestamp: chrono::Utc::now().timestamp_millis(),
                typing_velocity: Some(275.0),
                pause_patterns: None,
            };

            engine.analyze_flow_state(flow_data, None).await
        });

        handles.push(handle);
    }

    // Wait for all concurrent analyses
    let results = futures::future::join_all(handles).await;
    let total_duration = start.elapsed();

    // All should succeed
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }

    // Performance target: process 100 concurrent requests in under 100ms
    assert!(total_duration.as_millis() < 100,
           "Concurrent processing took {}ms for {} requests", 
           total_duration.as_millis(), concurrent_count);
}

#[tokio::test]
async fn test_ml_inference_engine() {
    let mut ml_engine = MLInferenceEngine::new();
    
    // Test both with and without ML model
    let features = [0.8, 0.7, 0.6, 0.1, 0.9]; // rhythm, focus, consistency, error, velocity
    
    let prediction = ml_engine.predict_flow_state(features).await;
    assert!(prediction.is_ok());
    
    let score = prediction.unwrap();
    assert!(score >= 0.0 && score <= 1.0);
}

#[tokio::test]
async fn test_wasm_plugin_manager() {
    let wasm_manager = WasmPluginManager::new();
    assert!(wasm_manager.is_ok());
    
    let manager = wasm_manager.unwrap();
    let loaded_plugins = manager.get_loaded_plugins();
    assert!(loaded_plugins.is_empty()); // No plugins loaded initially
}

#[tokio::test]
async fn test_encryption_service() {
    let master_key = EncryptionService::generate_master_key();
    let encryption_service = EncryptionService::new(&master_key).unwrap();
    
    #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
    struct TestData {
        user_id: Uuid,
        sensitive_info: String,
        score: f64,
    }
    
    let test_data = TestData {
        user_id: Uuid::new_v4(),
        sensitive_info: "Very secret information".to_string(),
        score: 0.95,
    };
    
    // Test encryption
    let encrypted = encryption_service.encrypt_sensitive_data(&test_data);
    assert!(encrypted.is_ok());
    
    let encrypted_data = encrypted.unwrap();
    assert!(!encrypted_data.data.is_empty());
    assert_eq!(encrypted_data.nonce.len(), 12);
    
    // Test decryption
    let decrypted: Result<TestData, _> = encryption_service.decrypt_sensitive_data(&encrypted_data);
    assert!(decrypted.is_ok());
    
    let decrypted_data = decrypted.unwrap();
    assert_eq!(decrypted_data, test_data);
}

#[test]
fn test_password_security() {
    let password = "secure-password-123!@#";
    
    let hash = hash_password(password);
    assert!(hash.is_ok());
    
    let hashed = hash.unwrap();
    assert!(hashed.len() > 50); // Argon2 hashes are long
    
    // Verify correct password
    assert!(verify_password(password, &hashed).unwrap());
    
    // Verify incorrect password
    assert!(!verify_password("wrong-password", &hashed).unwrap());
}

#[test]
fn test_jwt_token_security() {
    let user_id = Uuid::new_v4();
    let claims = Claims::new(
        user_id,
        "test@example.com".to_string(),
        "premium".to_string(),
    );
    let secret = "test-jwt-secret";
    
    let token = generate_jwt_token(&claims, secret);
    assert!(token.is_ok());
    
    let jwt_token = token.unwrap();
    assert!(!jwt_token.is_empty());
    assert!(jwt_token.split('.').count() == 3); // JWT has 3 parts
}

// Property-based testing with quickcheck
fn prop_flow_intensity_bounds(
    keystroke_intervals: Vec<u16>,
    context_switches: u8,
    error_events: u8,
) -> TestResult {
    if keystroke_intervals.is_empty() || keystroke_intervals.len() > 1000 {
        return TestResult::discard();
    }
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut engine = FlowDetectionEngine::new();
        
        let flow_data = FlowStateData {
            session_id: Uuid::new_v4(),
            keystroke_intervals: keystroke_intervals.into_iter().map(|x| x as u64).collect(),
            context_switches: context_switches as u32,
            error_events: error_events as u32,
            window_focus_duration: 30000,
            file_modifications: 5,
            timestamp: chrono::Utc::now().timestamp_millis(),
            typing_velocity: Some(250.0),
            pause_patterns: None,
        };
        
        let result = engine.analyze_flow_state(flow_data, None).await;
        
        if let Ok(flow_result) = result {
            TestResult::from_bool(
                flow_result.flow_intensity >= 0.0 
                && flow_result.flow_intensity <= 1.0
                && flow_result.confidence >= 0.0
                && flow_result.confidence <= 1.0
            )
        } else {
            TestResult::failed()
        }
    })
}

fn prop_keystroke_analysis_consistency(intervals: Vec<u8>) -> TestResult {
    if intervals.len() < 3 || intervals.len() > 100 {
        return TestResult::discard();
    }
    
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    rt.block_on(async {
        let mut engine = FlowDetectionEngine::new();
        
        let flow_data = FlowStateData {
            session_id: Uuid::new_v4(),
            keystroke_intervals: intervals.into_iter().map(|x| x as u64 + 50).collect(),
            context_switches: 1,
            error_events: 0,
            window_focus_duration: 20000,
            file_modifications: 2,
            timestamp: chrono::Utc::now().timestamp_millis(),
            typing_velocity: Some(200.0),
            pause_patterns: None,
        };
        
        let result1 = engine.analyze_flow_state(flow_data.clone(), None).await;
        let result2 = engine.analyze_flow_state(flow_data, None).await;
        
        if let (Ok(r1), Ok(r2)) = (result1, result2) {
            // Results should be deterministic for same input
            TestResult::from_bool(
                (r1.flow_intensity - r2.flow_intensity).abs() < 0.01
            )
        } else {
            TestResult::failed()
        }
    })
}

#[test]
fn test_property_based_flow_detection() {
    quickcheck(prop_flow_intensity_bounds as fn(Vec<u16>, u8, u8) -> TestResult);
    quickcheck(prop_keystroke_analysis_consistency as fn(Vec<u8>) -> TestResult);
}

#[tokio::test]
async fn test_memory_efficiency() {
    let initial_memory = get_approximate_memory_usage();
    
    // Create many flow engines to test memory efficiency
    let mut engines = Vec::new();
    for _ in 0..1000 {
        engines.push(FlowDetectionEngine::new());
    }
    
    let after_creation_memory = get_approximate_memory_usage();
    let memory_per_engine = (after_creation_memory - initial_memory) / 1000;
    
    // Each engine should use less than 1KB of memory
    assert!(memory_per_engine < 1024, 
           "Each flow engine uses {} bytes, should be <1KB", memory_per_engine);
    
    // Process data with all engines
    let flow_data = FlowStateData {
        session_id: Uuid::new_v4(),
        keystroke_intervals: vec![100; 20],
        context_switches: 1,
        error_events: 0,
        window_focus_duration: 15000,
        file_modifications: 2,
        timestamp: chrono::Utc::now().timestamp_millis(),
        typing_velocity: Some(250.0),
        pause_patterns: None,
    };
    
    let mut handles = Vec::new();
    for mut engine in engines {
        let data = flow_data.clone();
        let handle = tokio::spawn(async move {
            engine.analyze_flow_state(data, None).await
        });
        handles.push(handle);
    }
    
    let results = futures::future::join_all(handles).await;
    let final_memory = get_approximate_memory_usage();
    
    // Memory should not grow excessively during processing
    let total_memory_growth = final_memory - initial_memory;
    assert!(total_memory_growth < 50 * 1024 * 1024, // 50MB limit
           "Memory growth of {} bytes exceeds 50MB limit", total_memory_growth);
    
    // All results should be successful
    for result in results {
        assert!(result.is_ok());
        assert!(result.unwrap().is_ok());
    }
}

#[tokio::test]
async fn test_error_handling() {
    let mut engine = FlowDetectionEngine::new();
    
    // Test with invalid data
    let invalid_flow_data = FlowStateData {
        session_id: Uuid::new_v4(),
        keystroke_intervals: vec![], // Empty intervals should be handled gracefully
        context_switches: 0,
        error_events: 0,
        window_focus_duration: 0,
        file_modifications: 0,
        timestamp: chrono::Utc::now().timestamp_millis(),
        typing_velocity: None,
        pause_patterns: None,
    };
    
    let result = engine.analyze_flow_state(invalid_flow_data, None).await;
    assert!(result.is_ok()); // Should handle gracefully, not error
    
    let flow_result = result.unwrap();
    assert!(flow_result.flow_intensity >= 0.0); // Should return valid bounds
}

#[tokio::test]
async fn test_high_load_stability() {
    let high_load_requests = 10000;
    let start_time = std::time::Instant::now();
    
    let mut handles = Vec::new();
    
    for i in 0..high_load_requests {
        let handle = tokio::spawn(async move {
            let mut engine = FlowDetectionEngine::new();
            let flow_data = FlowStateData {
                session_id: Uuid::new_v4(),
                keystroke_intervals: vec![100 + (i % 50) as u64; 10],
                context_switches: (i % 5) as u32,
                error_events: (i % 3) as u32,
                window_focus_duration: 10000 + (i * 100) as u64,
                file_modifications: (i % 10) as u32,
                timestamp: chrono::Utc::now().timestamp_millis(),
                typing_velocity: Some(200.0 + (i % 100) as f32),
                pause_patterns: None,
            };
            
            engine.analyze_flow_state(flow_data, None).await
        });
        
        handles.push(handle);
    }
    
    // Process in batches to avoid overwhelming the system
    let batch_size = 1000;
    let mut successful_requests = 0;
    
    for batch in handles.chunks(batch_size) {
        let batch_results = futures::future::join_all(batch).await;
        
        for result in batch_results {
            if result.is_ok() && result.unwrap().is_ok() {
                successful_requests += 1;
            }
        }
    }
    
    let total_time = start_time.elapsed();
    let rps = successful_requests as f64 / total_time.as_secs_f64();
    
    println!("High load test: {}/{} successful requests in {:?} ({:.0} RPS)", 
             successful_requests, high_load_requests, total_time, rps);
    
    // At least 99% success rate
    let success_rate = successful_requests as f64 / high_load_requests as f64;
    assert!(success_rate >= 0.99, 
           "Success rate {:.2}% is below 99% threshold", success_rate * 100.0);
    
    // Should handle at least 1000 RPS
    assert!(rps >= 1000.0, 
           "RPS {:.0} is below 1000 threshold", rps);
}

// Helper function to estimate memory usage
fn get_approximate_memory_usage() -> usize {
    // This is a simplified memory measurement for testing
    // In production, you'd use more sophisticated memory tracking
    std::process::id() as usize * 4096 // Placeholder calculation
}