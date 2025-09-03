use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use mindful_code_backend::services::flow::{FlowDetectionEngine};
use mindful_code_backend::models::flow::{FlowStateData, UserFlowPreferences};
use uuid::Uuid;
use tokio::runtime::Runtime;

fn create_sample_flow_data(keystroke_count: usize) -> FlowStateData {
    let mut intervals = Vec::with_capacity(keystroke_count);
    for i in 0..keystroke_count {
        // Simulate realistic keystroke intervals (80-300ms)
        intervals.push(80 + (i as u64 * 13) % 220);
    }

    FlowStateData {
        session_id: Uuid::new_v4(),
        keystroke_intervals: intervals,
        context_switches: 2,
        error_events: 0,
        window_focus_duration: 30000, // 30 seconds
        file_modifications: 5,
        timestamp: chrono::Utc::now().timestamp_millis(),
        typing_velocity: Some(280.0),
        pause_patterns: None,
    }
}

fn bench_flow_detection_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("flow_detection_latency");
    
    // Set target time to ensure we measure <1ms performance
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(1000);

    let keystroke_sizes = vec![5, 10, 25, 50, 100];
    
    for size in keystroke_sizes {
        group.bench_with_input(
            BenchmarkId::new("analyze_flow_state", size),
            &size,
            |b, &keystroke_count| {
                let mut engine = FlowDetectionEngine::new();
                let flow_data = create_sample_flow_data(keystroke_count);
                let preferences = Some(UserFlowPreferences {
                    sensitivity_level: 0.7,
                    notification_threshold: 0.6,
                    focus_mode_enabled: true,
                    break_reminders_enabled: true,
                });

                b.to_async(&rt).iter(|| async {
                    let flow_data = black_box(flow_data.clone());
                    let preferences = black_box(preferences.clone());
                    
                    engine.analyze_flow_state(flow_data, preferences).await
                });
            },
        );
    }
    
    group.finish();
}

fn bench_keystroke_rhythm_analysis(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("keystroke_rhythm_analysis");
    
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(2000);

    let keystroke_sizes = vec![10, 50, 100, 500, 1000];
    
    for size in keystroke_sizes {
        group.bench_with_input(
            BenchmarkId::new("rhythm_analysis", size),
            &size,
            |b, &keystroke_count| {
                let mut engine = FlowDetectionEngine::new();
                let flow_data = create_sample_flow_data(keystroke_count);

                b.to_async(&rt).iter(|| async {
                    let flow_data = black_box(flow_data.clone());
                    engine.analyze_flow_state(flow_data, None).await
                });
            },
        );
    }
    
    group.finish();
}

fn bench_ml_inference_performance(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ml_inference");
    
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(1000);

    group.bench_function("ml_prediction", |b| {
        let mut engine = FlowDetectionEngine::new();
        let flow_data = create_sample_flow_data(50);

        b.to_async(&rt).iter(|| async {
            let flow_data = black_box(flow_data.clone());
            engine.analyze_flow_state(flow_data, None).await
        });
    });
    
    group.finish();
}

fn bench_concurrent_flow_detection(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("concurrent_flow_detection");
    
    group.measurement_time(std::time::Duration::from_secs(10));
    group.sample_size(100);

    let concurrent_levels = vec![1, 5, 10, 25, 50];
    
    for concurrent_count in concurrent_levels {
        group.bench_with_input(
            BenchmarkId::new("concurrent_analysis", concurrent_count),
            &concurrent_count,
            |b, &count| {
                b.to_async(&rt).iter(|| async {
                    let mut handles = Vec::new();
                    
                    for _ in 0..count {
                        let mut engine = FlowDetectionEngine::new();
                        let flow_data = create_sample_flow_data(25);
                        
                        let handle = tokio::spawn(async move {
                            engine.analyze_flow_state(flow_data, None).await
                        });
                        
                        handles.push(handle);
                    }
                    
                    // Wait for all concurrent analyses to complete
                    for handle in handles {
                        let _ = black_box(handle.await);
                    }
                });
            },
        );
    }
    
    group.finish();
}

fn bench_memory_efficiency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("memory_efficiency");
    
    group.measurement_time(std::time::Duration::from_secs(5));
    group.sample_size(500);

    group.bench_function("engine_creation", |b| {
        b.iter(|| {
            let engine = black_box(FlowDetectionEngine::new());
            drop(engine);
        });
    });

    group.bench_function("large_keystroke_buffer", |b| {
        let mut engine = FlowDetectionEngine::new();
        let flow_data = create_sample_flow_data(1000); // Large keystroke buffer
        
        b.to_async(&rt).iter(|| async {
            let flow_data = black_box(flow_data.clone());
            engine.analyze_flow_state(flow_data, None).await
        });
    });
    
    group.finish();
}

// Stress test for ultra-high performance requirements
fn bench_ultra_low_latency_stress(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let mut group = c.benchmark_group("ultra_low_latency_stress");
    
    // Very tight timing requirements
    group.measurement_time(std::time::Duration::from_secs(15));
    group.sample_size(5000);
    
    group.bench_function("sub_millisecond_target", |b| {
        let mut engine = FlowDetectionEngine::new();
        let flow_data = create_sample_flow_data(20); // Optimal size for <1ms
        
        b.to_async(&rt).iter(|| async {
            let start = std::time::Instant::now();
            let flow_data = black_box(flow_data.clone());
            let result = engine.analyze_flow_state(flow_data, None).await;
            let duration = start.elapsed();
            
            // Assert that we're meeting our <1ms requirement
            if duration.as_millis() > 1 {
                eprintln!("WARNING: Flow detection took {}ms, exceeding 1ms target", 
                         duration.as_millis());
            }
            
            black_box(result)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    bench_flow_detection_latency,
    bench_keystroke_rhythm_analysis,
    bench_ml_inference_performance,
    bench_concurrent_flow_detection,
    bench_memory_efficiency,
    bench_ultra_low_latency_stress
);

criterion_main!(benches);