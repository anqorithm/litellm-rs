use super::*;

// ==================== PrometheusMetrics Tests ====================

#[test]
fn test_prometheus_metrics_default() {
    let metrics = PrometheusMetrics::default();
    assert!(metrics.request_total.is_empty());
    assert!(metrics.request_duration.is_empty());
    assert!(metrics.error_total.is_empty());
    assert!(metrics.token_usage.is_empty());
    assert!(metrics.cost_total.is_empty());
    assert!(metrics.provider_health.is_empty());
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.cache_misses, 0);
    assert_eq!(metrics.active_connections, 0);
    assert!(metrics.queue_size.is_empty());
}

// ==================== MetricsCollector Creation Tests ====================

#[test]
fn test_metrics_collector_new() {
    let collector = MetricsCollector::new();
    // Should have empty metrics initially
    assert!(collector.datadog_client.is_none());
    assert!(collector.otel_exporter.is_none());
}

#[test]
fn test_metrics_collector_default() {
    let collector = MetricsCollector::default();
    assert!(collector.datadog_client.is_none());
    assert!(collector.otel_exporter.is_none());
}

#[test]
fn test_metrics_collector_with_datadog() {
    let collector =
        MetricsCollector::new().with_datadog("api-key".to_string(), "datadoghq.com".to_string());
    assert!(collector.datadog_client.is_some());
    let client = collector.datadog_client.unwrap();
    assert_eq!(client.api_key, "api-key");
    assert_eq!(client.base_url, "https://api.datadoghq.com");
    assert!(!client.default_tags.is_empty());
}

#[test]
fn test_metrics_collector_with_otel() {
    let mut headers = HashMap::new();
    headers.insert("Authorization".to_string(), "Bearer token".to_string());

    let collector =
        MetricsCollector::new().with_otel("https://otel.example.com".to_string(), headers.clone());
    assert!(collector.otel_exporter.is_some());
    let exporter = collector.otel_exporter.unwrap();
    assert_eq!(exporter.endpoint, "https://otel.example.com");
    assert!(exporter.headers.contains_key("Authorization"));
}

#[test]
fn test_metrics_collector_chained_config() {
    let headers = HashMap::new();
    let collector = MetricsCollector::new()
        .with_datadog("api-key".to_string(), "datadoghq.com".to_string())
        .with_otel("https://otel.example.com".to_string(), headers);

    assert!(collector.datadog_client.is_some());
    assert!(collector.otel_exporter.is_some());
}

// ==================== Record Request Tests ====================

#[tokio::test]
async fn test_record_request_basic() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
}

#[tokio::test]
async fn test_record_request_multiple() {
    let collector = MetricsCollector::new();

    for _ in 0..5 {
        collector
            .record_request(
                "openai",
                "gpt-4",
                Duration::from_millis(100),
                None,
                None,
                true,
            )
            .await;
    }

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 5);
}

#[tokio::test]
async fn test_record_request_with_error() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            false, // Error
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
    assert_eq!(*metrics.error_total.get("openai:gpt-4").unwrap(), 1);
}

#[tokio::test]
async fn test_record_request_updates_all_primary_metrics() {
    let collector = MetricsCollector::new();
    let tokens = TokenUsage {
        prompt_tokens: 42,
        completion_tokens: 24,
        total_tokens: 66,
    };

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            Some(tokens),
            Some(0.12),
            false,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
    assert_eq!(*metrics.error_total.get("openai:gpt-4").unwrap(), 1);
    assert_eq!(*metrics.token_usage.get("openai:gpt-4:prompt").unwrap(), 42);
    assert_eq!(
        *metrics.token_usage.get("openai:gpt-4:completion").unwrap(),
        24
    );
    assert_eq!(*metrics.cost_total.get("openai:gpt-4").unwrap(), 0.12);
}

#[tokio::test]
async fn test_record_request_with_tokens() {
    let collector = MetricsCollector::new();
    let tokens = TokenUsage {
        prompt_tokens: 100,
        completion_tokens: 50,
        total_tokens: 150,
    };

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            Some(tokens),
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(
        *metrics.token_usage.get("openai:gpt-4:prompt").unwrap(),
        100
    );
    assert_eq!(
        *metrics.token_usage.get("openai:gpt-4:completion").unwrap(),
        50
    );
}

#[tokio::test]
async fn test_record_request_with_cost() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            Some(0.05),
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.cost_total.get("openai:gpt-4").unwrap(), 0.05);
}

#[tokio::test]
async fn test_record_request_cost_accumulates() {
    let collector = MetricsCollector::new();

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            Some(0.05),
            true,
        )
        .await;
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            Some(0.03),
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert!((metrics.cost_total.get("openai:gpt-4").unwrap() - 0.08).abs() < 0.0001);
}

#[tokio::test]
async fn test_record_request_different_providers() {
    let collector = MetricsCollector::new();

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            true,
        )
        .await;
    collector
        .record_request(
            "anthropic",
            "claude-3",
            Duration::from_millis(150),
            None,
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
    assert_eq!(*metrics.request_total.get("anthropic:claude-3").unwrap(), 1);
}

// ==================== Cache Metrics Tests ====================

#[tokio::test]
async fn test_record_cache_hit() {
    let collector = MetricsCollector::new();
    collector.record_cache_hit(true).await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(metrics.cache_hits, 1);
    assert_eq!(metrics.cache_misses, 0);
}

#[tokio::test]
async fn test_record_cache_miss() {
    let collector = MetricsCollector::new();
    collector.record_cache_hit(false).await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(metrics.cache_hits, 0);
    assert_eq!(metrics.cache_misses, 1);
}

#[tokio::test]
async fn test_record_cache_mixed() {
    let collector = MetricsCollector::new();

    for _ in 0..5 {
        collector.record_cache_hit(true).await;
    }
    for _ in 0..3 {
        collector.record_cache_hit(false).await;
    }

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(metrics.cache_hits, 5);
    assert_eq!(metrics.cache_misses, 3);
}

// ==================== Provider Health Tests ====================

#[tokio::test]
async fn test_update_provider_health() {
    let collector = MetricsCollector::new();
    collector.update_provider_health("openai", 0.95).await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.provider_health.get("openai").unwrap(), 0.95);
}

#[tokio::test]
async fn test_update_provider_health_multiple() {
    let collector = MetricsCollector::new();
    collector.update_provider_health("openai", 0.95).await;
    collector.update_provider_health("anthropic", 0.99).await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.provider_health.get("openai").unwrap(), 0.95);
    assert_eq!(*metrics.provider_health.get("anthropic").unwrap(), 0.99);
}

#[tokio::test]
async fn test_update_provider_health_overwrite() {
    let collector = MetricsCollector::new();
    collector.update_provider_health("openai", 0.95).await;
    collector.update_provider_health("openai", 0.80).await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.provider_health.get("openai").unwrap(), 0.80);
}

// ==================== Prometheus Export Tests ====================

#[tokio::test]
async fn test_export_prometheus_empty() {
    let collector = MetricsCollector::new();
    let output = collector.export_prometheus().await;

    // Should contain headers even when empty
    assert!(output.contains("# HELP litellm_requests_total"));
    assert!(output.contains("# TYPE litellm_requests_total counter"));
    assert!(output.contains("litellm_cache_hits_total 0"));
    assert!(output.contains("litellm_cache_misses_total 0"));
}

#[tokio::test]
async fn test_export_prometheus_with_requests() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            true,
        )
        .await;

    let output = collector.export_prometheus().await;

    assert!(output.contains("litellm_requests_total{provider=\"openai\",model=\"gpt-4\"} 1"));
}

#[tokio::test]
async fn test_export_prometheus_with_errors() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            false,
        )
        .await;

    let output = collector.export_prometheus().await;

    assert!(output.contains("litellm_errors_total{provider=\"openai\",model=\"gpt-4\"} 1"));
}

#[tokio::test]
async fn test_export_prometheus_with_cache() {
    let collector = MetricsCollector::new();
    collector.record_cache_hit(true).await;
    collector.record_cache_hit(true).await;
    collector.record_cache_hit(false).await;

    let output = collector.export_prometheus().await;

    assert!(output.contains("litellm_cache_hits_total 2"));
    assert!(output.contains("litellm_cache_misses_total 1"));
}

#[tokio::test]
async fn test_export_prometheus_with_health() {
    let collector = MetricsCollector::new();
    collector.update_provider_health("openai", 0.95).await;

    let output = collector.export_prometheus().await;

    assert!(output.contains("litellm_provider_health{provider=\"openai\"} 0.95"));
}

#[tokio::test]
async fn test_export_prometheus_format() {
    let collector = MetricsCollector::new();
    let output = collector.export_prometheus().await;

    // Check Prometheus format compliance
    for line in output.lines() {
        if !line.is_empty() {
            assert!(
                line.starts_with('#') || line.starts_with("litellm_"),
                "Invalid line format: {}",
                line
            );
        }
    }
}

// ==================== DataDog Integration Tests ====================

#[tokio::test]
async fn test_send_to_datadog_no_client() {
    let collector = MetricsCollector::new();
    let result = collector.send_to_datadog().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_send_to_datadog_with_client() {
    let collector =
        MetricsCollector::new().with_datadog("api-key".to_string(), "datadoghq.com".to_string());

    // This won't actually send (no network call in test)
    // but should not error
    let result = collector.send_to_datadog().await;
    assert!(result.is_ok());
}

// ==================== Duration Recording Tests ====================

#[tokio::test]
async fn test_record_request_duration() {
    let collector = MetricsCollector::new();

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            None,
            true,
        )
        .await;
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(200),
            None,
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    let histogram = metrics.request_duration.get("openai:gpt-4").unwrap();
    assert_eq!(histogram.count(), 2);
}

// ==================== Edge Cases ====================

#[tokio::test]
async fn test_record_request_zero_duration() {
    let collector = MetricsCollector::new();
    collector
        .record_request("openai", "gpt-4", Duration::ZERO, None, None, true)
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
}

#[tokio::test]
async fn test_record_request_very_long_duration() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_secs(3600),
            None,
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.request_total.get("openai:gpt-4").unwrap(), 1);
}

#[tokio::test]
async fn test_record_request_zero_cost() {
    let collector = MetricsCollector::new();
    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            None,
            Some(0.0),
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.cost_total.get("openai:gpt-4").unwrap(), 0.0);
}

#[tokio::test]
async fn test_record_request_zero_tokens() {
    let collector = MetricsCollector::new();
    let tokens = TokenUsage {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
    };

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(100),
            Some(tokens),
            None,
            true,
        )
        .await;

    let metrics = collector.prometheus_metrics.read().await;
    assert_eq!(*metrics.token_usage.get("openai:gpt-4:prompt").unwrap(), 0);
}
