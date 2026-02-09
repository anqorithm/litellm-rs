use super::*;

#[tokio::test]
async fn test_prometheus_integration_creation() {
    let integration = PrometheusIntegration::with_defaults();
    assert_eq!(integration.name(), "prometheus");
    assert!(integration.is_enabled());
}

#[tokio::test]
async fn test_on_llm_start() {
    let integration = PrometheusIntegration::with_defaults();

    let event = LlmStartEvent::new("req-1", "gpt-4").provider("openai");
    integration.on_llm_start(&event).await.unwrap();

    assert_eq!(integration.metrics.active_requests.get(), 1.0);
}

#[tokio::test]
async fn test_on_llm_end() {
    let integration = PrometheusIntegration::with_defaults();

    let start_event = LlmStartEvent::new("req-1", "gpt-4").provider("openai");
    integration.on_llm_start(&start_event).await.unwrap();

    let end_event = LlmEndEvent::new("req-1", "gpt-4")
        .provider("openai")
        .tokens(100, 50)
        .latency(150);
    integration.on_llm_end(&end_event).await.unwrap();

    assert_eq!(integration.metrics.active_requests.get(), 0.0);
}

#[tokio::test]
async fn test_on_llm_error() {
    let integration = PrometheusIntegration::with_defaults();

    let start_event = LlmStartEvent::new("req-1", "gpt-4").provider("openai");
    integration.on_llm_start(&start_event).await.unwrap();

    let error_event = LlmErrorEvent::new("req-1", "gpt-4", "Rate limited").provider("openai");
    integration.on_llm_error(&error_event).await.unwrap();

    assert_eq!(integration.metrics.active_requests.get(), 0.0);
}

#[tokio::test]
async fn test_cache_hit() {
    let integration = PrometheusIntegration::with_defaults();

    let event = CacheHitEvent {
        request_id: "req-1".to_string(),
        cache_key: "key-1".to_string(),
        cache_backend: "redis".to_string(),
        time_saved_ms: Some(100),
        cost_saved_usd: Some(0.01),
        timestamp_ms: 0,
    };
    integration.on_cache_hit(&event).await.unwrap();

    assert_eq!(integration.metrics.cache_hits.get(), 1);
}

#[tokio::test]
async fn test_render_metrics() {
    let integration = PrometheusIntegration::with_defaults();

    let event = LlmStartEvent::new("req-1", "gpt-4").provider("openai");
    integration.on_llm_start(&event).await.unwrap();

    let metrics = integration.render_metrics();
    assert!(metrics.contains("litellm_requests_total"));
    assert!(metrics.contains("litellm_active_requests"));
}

#[tokio::test]
async fn test_disabled_integration() {
    let config = PrometheusConfig {
        enabled: false,
        ..Default::default()
    };
    let integration = PrometheusIntegration::new(config);

    assert!(!integration.is_enabled());
}

#[tokio::test]
async fn test_custom_prefix() {
    let config = PrometheusConfig {
        prefix: "myapp".to_string(),
        ..Default::default()
    };
    let integration = PrometheusIntegration::new(config);

    let event = LlmStartEvent::new("req-1", "gpt-4");
    integration.on_llm_start(&event).await.unwrap();

    let metrics = integration.render_metrics();
    assert!(metrics.contains("myapp_requests_total"));
}
