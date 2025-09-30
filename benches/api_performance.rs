use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;

// Mock API request for benchmarking
async fn mock_api_request(size: usize) -> Vec<u8> {
    tokio::time::sleep(Duration::from_micros(10)).await;
    vec![0u8; size]
}

fn api_performance_benchmark(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    
    c.bench_function("small_api_request", |b| {
        b.to_async(&rt).iter(|| async {
            let response = mock_api_request(black_box(1024)).await;
            black_box(response);
        })
    });

    c.bench_function("medium_api_request", |b| {
        b.to_async(&rt).iter(|| async {
            let response = mock_api_request(black_box(10240)).await;
            black_box(response);
        })
    });

    c.bench_function("large_api_request", |b| {
        b.to_async(&rt).iter(|| async {
            let response = mock_api_request(black_box(102400)).await;
            black_box(response);
        })
    });
}

criterion_group!(benches, api_performance_benchmark);
criterion_main!(benches);