use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use std::collections::HashMap;

// Mock flow detection data structures
#[derive(Debug, Clone)]
struct FlowMetric {
    timestamp: u64,
    focus_score: f64,
    productivity_level: u8,
    context_switches: u32,
}

#[derive(Debug, Clone)]
struct FlowSession {
    start_time: u64,
    end_time: u64,
    metrics: Vec<FlowMetric>,
    flow_intensity: f64,
}

// Mock flow detection algorithm
fn detect_flow_state(metrics: &[FlowMetric]) -> bool {
    if metrics.len() < 3 {
        return false;
    }

    let avg_focus = metrics.iter().map(|m| m.focus_score).sum::<f64>() / metrics.len() as f64;
    let context_switches: u32 = metrics.iter().map(|m| m.context_switches).sum();
    
    avg_focus > 0.7 && context_switches < 5
}

// More complex flow analysis
fn analyze_flow_patterns(sessions: &[FlowSession]) -> HashMap<u8, f64> {
    let mut pattern_map = HashMap::new();
    
    for session in sessions {
        if detect_flow_state(&session.metrics) {
            let hour = ((session.start_time / 3600) % 24) as u8;
            *pattern_map.entry(hour).or_insert(0.0) += session.flow_intensity;
        }
    }
    
    pattern_map
}

// Generate mock data for benchmarking
fn generate_mock_metrics(count: usize) -> Vec<FlowMetric> {
    (0..count).map(|i| FlowMetric {
        timestamp: i as u64 * 60, // Every minute
        focus_score: (i as f64 * 0.1) % 1.0,
        productivity_level: ((i * 7) % 10) as u8,
        context_switches: (i % 10) as u32,
    }).collect()
}

fn generate_mock_sessions(count: usize) -> Vec<FlowSession> {
    (0..count).map(|i| {
        let metrics = generate_mock_metrics(20 + (i % 30));
        FlowSession {
            start_time: i as u64 * 3600, // Every hour
            end_time: i as u64 * 3600 + 3600,
            metrics,
            flow_intensity: (i as f64 * 0.05) % 1.0,
        }
    }).collect()
}

fn flow_detection_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("flow_detection");

    // Benchmark simple flow state detection
    for size in [10, 50, 100, 500].iter() {
        let metrics = generate_mock_metrics(*size);
        group.bench_with_input(
            BenchmarkId::new("detect_flow_state", size),
            &metrics,
            |b, metrics| {
                b.iter(|| {
                    detect_flow_state(black_box(metrics))
                });
            },
        );
    }

    // Benchmark flow pattern analysis
    for size in [10, 25, 50, 100].iter() {
        let sessions = generate_mock_sessions(*size);
        group.bench_with_input(
            BenchmarkId::new("analyze_flow_patterns", size),
            &sessions,
            |b, sessions| {
                b.iter(|| {
                    analyze_flow_patterns(black_box(sessions))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, flow_detection_benchmark);
criterion_main!(benches);