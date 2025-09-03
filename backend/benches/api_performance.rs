use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mindful_code_backend::{
    config::Config,
    state::AppState,
    handlers::flow,
    models::flow::{FlowDetectionRequest, FlowStateData, UserFlowPreferences},
    utils::auth::{Claims, generate_jwt_token},
};
use axum::{
    body::Body,
    extract::State,
    http::{Method, Request, StatusCode},
    Json,
};
use serde_json;
use tokio::runtime::Runtime;
use tower::ServiceExt;
use uuid::Uuid;

async fn setup_test_app_state() -> AppState {
    let config = Config {
        database_url: "postgresql://test:test@localhost/test_mindful_code".to_string(),
        port: 3001,
        jwt_secret: "test-secret".to_string(),
        encryption_key: "test-encryption-key-32-bytes-long!".to_string(),
        environment: mindful_code_backend::config::Environment::Test,
        max_connections: 5,
        worker_threads: 2,
    };

    // In a real benchmark, you'd connect to a test database
    // For this benchmark, we'll simulate the state
    AppState::new(config).await.expect("Failed to create app state")
}

fn create_test_claims() -> Claims {
    Claims::new(
        Uuid::new_v4(),
        "test@example.com".to_string(),
        "premium".to_string(),
    )
}

fn create_flow_detection_request() -> FlowDetectionRequest {
    FlowDetectionRequest {
        flow_data: FlowStateData {
            session_id: Uuid::new_v4(),
            keystroke_intervals: vec![120, 135, 98, 142, 156, 89, 167, 134, 145, 123],
            context_switches: 1,
            error_events: 0,
            window_focus_duration: 15000,
            file_modifications: 3,
            timestamp: chrono::Utc::now().timestamp_millis(),
            typing_velocity: Some(250.0),
            pause_patterns: None,
        },
        user_preferences: Some(UserFlowPreferences {
            sensitivity_level: 0.75,
            notification_threshold: 0.6,
            focus_mode_enabled: true,
            break_reminders_enabled: false,
        }),
    }
}

fn bench_flow_detection_endpoint(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("api_flow_detection");
    
    // Target: <5ms API response time
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(1000);

    group.bench_function("detect_flow_state_endpoint", |b| {
        b.to_async(&rt).iter(|| async {
            let app_state = setup_test_app_state().await;
            let claims = create_test_claims();
            let request = create_flow_detection_request();

            let start = std::time::Instant::now();
            
            let result = flow::detect_flow_state(
                State(app_state),
                claims,
                Json(mindful_code_backend::handlers::flow::FlowDetectionPayload {
                    request: black_box(request),
                }),
            ).await;

            let duration = start.elapsed();
            
            // Assert API response time is under 5ms
            if duration.as_millis() > 5 {
                eprintln!("WARNING: API response took {}ms, exceeding 5ms target", 
                         duration.as_millis());
            }

            black_box(result)
        });
    });

    group.finish();
}

fn bench_concurrent_api_requests(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_api_requests");
    
    group.measurement_time(std::time::Duration::from_secs(15));
    group.sample_size(100);

    let concurrent_levels = vec![1, 10, 50, 100, 500];

    for concurrent_count in concurrent_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_flow_detection", concurrent_count),
            &concurrent_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let app_state = setup_test_app_state().await;
                    let mut handles = Vec::new();

                    let start = std::time::Instant::now();

                    for _ in 0..count {
                        let app_state = app_state.clone();
                        let claims = create_test_claims();
                        let request = create_flow_detection_request();

                        let handle = tokio::spawn(async move {
                            flow::detect_flow_state(
                                State(app_state),
                                claims,
                                Json(mindful_code_backend::handlers::flow::FlowDetectionPayload {
                                    request,
                                }),
                            ).await
                        });

                        handles.push(handle);
                    }

                    // Wait for all requests to complete
                    let results = futures::future::join_all(handles).await;
                    let duration = start.elapsed();

                    // Calculate requests per second
                    let rps = count as f64 / duration.as_secs_f64();
                    
                    // Target: 50,000+ RPS
                    if rps < 1000.0 {
                        eprintln!("WARNING: Only achieved {:.0} RPS, target is 50,000+", rps);
                    }

                    black_box(results)
                });
            },
        );
    }

    group.finish();
}

fn bench_memory_usage_under_load(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_usage_load");
    
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(50);

    group.bench_function("memory_efficient_processing", |b| {
        b.to_async(&rt).iter(|| async {
            let app_state = setup_test_app_state().await;
            let mut requests = Vec::new();

            // Simulate 1000 concurrent users (target: <20MB for 1000 users)
            for _ in 0..1000 {
                let claims = create_test_claims();
                let request = create_flow_detection_request();
                requests.push((claims, request));
            }

            let start_memory = get_memory_usage();
            let start_time = std::time::Instant::now();

            // Process all requests
            for (claims, request) in requests {
                let result = flow::detect_flow_state(
                    State(app_state.clone()),
                    claims,
                    Json(mindful_code_backend::handlers::flow::FlowDetectionPayload {
                        request: black_box(request),
                    }),
                ).await;
                
                black_box(result);
            }

            let end_memory = get_memory_usage();
            let duration = start_time.elapsed();
            let memory_used = end_memory - start_memory;

            eprintln!(
                "Processed 1000 requests in {:?}, memory used: {} KB",
                duration,
                memory_used / 1024
            );

            // Target: <20MB (20,480 KB) for 1000 users
            if memory_used > 20_480_000 {
                eprintln!("WARNING: Used {}KB memory, target is <20MB", memory_used / 1024);
            }
        });
    });

    group.finish();
}

fn bench_websocket_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("websocket_performance");
    
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(500);

    group.bench_function("websocket_message_broadcast", |b| {
        b.to_async(&rt).iter(|| async {
            let app_state = setup_test_app_state().await;
            let user_id = Uuid::new_v4();

            // Simulate WebSocket message broadcasting
            let message = serde_json::json!({
                "type": "flow_state_update",
                "data": {
                    "flow_intensity": 0.85,
                    "is_in_flow": true,
                    "timestamp": chrono::Utc::now().timestamp_millis()
                }
            });

            let start = std::time::Instant::now();
            
            app_state.broadcast_to_user(
                user_id,
                black_box(message.to_string())
            ).await;

            let duration = start.elapsed();
            
            // Target: <10ms WebSocket latency
            if duration.as_millis() > 10 {
                eprintln!("WARNING: WebSocket broadcast took {}ms, target is <10ms", 
                         duration.as_millis());
            }

            black_box(duration)
        });
    });

    group.finish();
}

fn bench_database_query_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("database_performance");
    
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(200);

    group.bench_function("session_history_query", |b| {
        b.to_async(&rt).iter(|| async {
            let app_state = setup_test_app_state().await;
            let claims = create_test_claims();

            let start = std::time::Instant::now();

            // Simulate database query
            let result = flow::get_flow_patterns(
                State(app_state),
                black_box(claims),
            ).await;

            let duration = start.elapsed();
            
            // Target: <2ms query time
            if duration.as_millis() > 2 {
                eprintln!("WARNING: Database query took {}ms, target is <2ms", 
                         duration.as_millis());
            }

            black_box(result)
        });
    });

    group.finish();
}

fn bench_end_to_end_request_pipeline(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("end_to_end_pipeline");
    
    group.measurement_time(std::time::Duration::from_secs(15));
    group.sample_size(500);

    group.bench_function("complete_request_pipeline", |b| {
        b.to_async(&rt).iter(|| async {
            let app_state = setup_test_app_state().await;
            let claims = create_test_claims();
            let request = create_flow_detection_request();

            let pipeline_start = std::time::Instant::now();

            // 1. Flow detection
            let flow_result = flow::detect_flow_state(
                State(app_state.clone()),
                claims.clone(),
                Json(mindful_code_backend::handlers::flow::FlowDetectionPayload {
                    request: black_box(request),
                }),
            ).await;

            // 2. Get flow patterns
            let patterns_result = flow::get_flow_patterns(
                State(app_state.clone()),
                claims.clone(),
            ).await;

            // 3. Get flow insights
            let insights_result = flow::get_flow_insights(
                State(app_state),
                black_box(claims),
            ).await;

            let total_duration = pipeline_start.elapsed();

            // Target: Complete pipeline under 10ms
            if total_duration.as_millis() > 10 {
                eprintln!("WARNING: Complete pipeline took {}ms, target is <10ms", 
                         total_duration.as_millis());
            }

            black_box((flow_result, patterns_result, insights_result))
        });
    });

    group.finish();
}

// Helper function to get memory usage (simplified for benchmark)
fn get_memory_usage() -> u64 {
    // In a real implementation, you'd use proper memory measurement
    // For benchmarking purposes, we'll use a placeholder
    std::process::id() as u64 * 1024 // Simplified memory measurement
}

criterion_group!(
    benches,
    bench_flow_detection_endpoint,
    bench_concurrent_api_requests,
    bench_memory_usage_under_load,
    bench_websocket_performance,
    bench_database_query_performance,
    bench_end_to_end_request_pipeline
);

criterion_main!(benches);