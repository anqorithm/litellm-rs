//! End-to-end benchmarks for litellm-rs
//!
//! These benchmarks measure the full request processing pipeline,
//! simulating real-world usage patterns.

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Instant;

/// Benchmark request parsing overhead
fn bench_request_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("request_parsing");

    // Typical chat completion request
    let simple_request = r#"{
        "model": "gpt-4",
        "messages": [{"role": "user", "content": "Hello!"}]
    }"#;

    // Complex request with tools
    let complex_request = r#"{
        "model": "gpt-4",
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What's the weather in Tokyo?"}
        ],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "get_weather",
                    "description": "Get weather information",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "location": {"type": "string"}
                        }
                    }
                }
            }
        ],
        "temperature": 0.7,
        "max_tokens": 1000
    }"#;

    // Large context request (simulating RAG)
    let large_context: String = format!(
        r#"{{
            "model": "gpt-4",
            "messages": [
                {{"role": "system", "content": "{}"}},
                {{"role": "user", "content": "Summarize this."}}
            ]
        }}"#,
        "x".repeat(10000) // 10KB context
    );

    group.throughput(Throughput::Elements(1));

    group.bench_function("simple_request", |b| {
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(simple_request).unwrap()));
    });

    group.bench_function("complex_request_with_tools", |b| {
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(complex_request).unwrap()));
    });

    group.bench_function("large_context_10kb", |b| {
        b.iter(|| black_box(serde_json::from_str::<serde_json::Value>(&large_context).unwrap()));
    });

    group.finish();
}

/// Benchmark response serialization
fn bench_response_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("response_serialization");

    // Typical response
    let response = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 12,
            "total_tokens": 21
        }
    });

    // Streaming chunk
    let chunk = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion.chunk",
        "created": 1677652288,
        "model": "gpt-4",
        "choices": [{
            "index": 0,
            "delta": {"content": "Hello"},
            "finish_reason": null
        }]
    });

    group.bench_function("serialize_response", |b| {
        b.iter(|| black_box(serde_json::to_string(&response).unwrap()));
    });

    group.bench_function("serialize_chunk", |b| {
        b.iter(|| black_box(serde_json::to_string(&chunk).unwrap()));
    });

    // Measure SSE formatting overhead
    group.bench_function("format_sse_chunk", |b| {
        let json = serde_json::to_string(&chunk).unwrap();
        b.iter(|| black_box(format!("data: {}\n\n", json)));
    });

    group.finish();
}

/// Benchmark header processing (common in proxy scenarios)
fn bench_header_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("header_processing");

    let api_key = "sk-1234567890abcdef1234567890abcdef";

    group.bench_function("validate_api_key_format", |b| {
        b.iter(|| {
            let key = black_box(api_key);
            // Simple validation
            key.starts_with("sk-") && key.len() > 20
        });
    });

    group.bench_function("extract_bearer_token", |b| {
        let auth_header = "Bearer sk-1234567890abcdef";
        b.iter(|| {
            let header = black_box(auth_header);
            header.strip_prefix("Bearer ").unwrap_or("")
        });
    });

    group.finish();
}

/// Benchmark model name routing/matching
fn bench_model_routing(c: &mut Criterion) {
    let mut group = c.benchmark_group("model_routing");

    let models = [
        "gpt-4",
        "gpt-4-turbo",
        "gpt-4-turbo-preview",
        "gpt-3.5-turbo",
        "claude-3-opus",
        "claude-3-sonnet",
        "gemini-pro",
        "gemini-1.5-pro",
    ];

    // HashMap lookup
    let model_map: std::collections::HashMap<&str, usize> =
        models.iter().enumerate().map(|(i, m)| (*m, i)).collect();

    group.bench_function("hashmap_lookup", |b| {
        b.iter(|| black_box(model_map.get("gpt-4-turbo")));
    });

    // Linear search (for small lists)
    group.bench_function("linear_search", |b| {
        b.iter(|| black_box(models.iter().position(|m| *m == "gpt-4-turbo")));
    });

    // Prefix matching
    group.bench_function("prefix_match", |b| {
        let target = "gpt-4";
        b.iter(|| {
            black_box(
                models
                    .iter()
                    .filter(|m| m.starts_with(target))
                    .collect::<Vec<_>>(),
            )
        });
    });

    group.finish();
}

/// Benchmark latency measurement overhead
fn bench_latency_tracking(c: &mut Criterion) {
    let mut group = c.benchmark_group("latency_tracking");

    group.bench_function("instant_now", |b| {
        b.iter(|| black_box(Instant::now()));
    });

    group.bench_function("instant_elapsed", |b| {
        let start = Instant::now();
        b.iter(|| black_box(start.elapsed()));
    });

    group.bench_function("system_time_now", |b| {
        b.iter(|| black_box(std::time::SystemTime::now()));
    });

    group.finish();
}

/// Benchmark token counting approximation
fn bench_token_counting(c: &mut Criterion) {
    let mut group = c.benchmark_group("token_counting");

    let short_text = "Hello, how are you?";
    let medium_text = "The quick brown fox jumps over the lazy dog. ".repeat(10);
    let long_text = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. ".repeat(100);

    // Simple approximation: chars / 4
    group.bench_function("approx_short", |b| {
        b.iter(|| black_box((short_text.len() as f64 / 4.0).ceil() as usize));
    });

    group.bench_function("approx_medium", |b| {
        b.iter(|| black_box((medium_text.len() as f64 / 4.0).ceil() as usize));
    });

    group.bench_function("approx_long", |b| {
        b.iter(|| black_box((long_text.len() as f64 / 4.0).ceil() as usize));
    });

    // Word-based approximation
    group.bench_function("word_count_short", |b| {
        b.iter(|| black_box(short_text.split_whitespace().count()));
    });

    group.bench_function("word_count_long", |b| {
        b.iter(|| black_box(long_text.split_whitespace().count()));
    });

    group.finish();
}

/// Benchmark concurrent request handling simulation
fn bench_concurrent_requests(c: &mut Criterion) {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    let mut group = c.benchmark_group("concurrent_requests");

    // Simulate request counter
    let counter = Arc::new(AtomicU64::new(0));

    group.bench_function("atomic_increment", |b| {
        let counter = counter.clone();
        b.iter(|| black_box(counter.fetch_add(1, Ordering::Relaxed)));
    });

    // Simulate rate limiting check
    group.bench_function("rate_limit_check", |b| {
        let requests = Arc::new(AtomicU64::new(0));
        let limit = 1000u64;
        b.iter(|| {
            let current = requests.fetch_add(1, Ordering::Relaxed);
            black_box(current < limit)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_request_parsing,
    bench_response_serialization,
    bench_header_processing,
    bench_model_routing,
    bench_latency_tracking,
    bench_token_counting,
    bench_concurrent_requests,
);

criterion_main!(benches);
