use super::*;

// ==================== ProviderMetrics Tests ====================

#[test]
fn test_provider_metrics_default() {
    let metrics = ProviderMetrics::default();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.total_response_time, Duration::from_secs(0));
    assert!(metrics.min_response_time.is_none());
    assert!(metrics.max_response_time.is_none());
    assert!(metrics.last_request.is_none());
    assert!(metrics.error_counts.is_empty());
}

#[test]
fn test_provider_metrics_clone() {
    let mut error_counts = HashMap::new();
    error_counts.insert("timeout".to_string(), 3);
    let metrics = ProviderMetrics {
        total_requests: 100,
        successful_requests: 95,
        failed_requests: 5,
        total_response_time: Duration::from_millis(5000),
        error_counts,
        ..Default::default()
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.total_requests, 100);
    assert_eq!(cloned.successful_requests, 95);
    assert_eq!(cloned.failed_requests, 5);
    assert_eq!(cloned.error_counts.get("timeout"), Some(&3));
}

#[test]
fn test_provider_metrics_debug() {
    let metrics = ProviderMetrics::default();
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("ProviderMetrics"));
    assert!(debug.contains("total_requests"));
}

// ==================== ModelMetrics Tests ====================

#[test]
fn test_model_metrics_default() {
    let metrics = ModelMetrics::default();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.total_response_time, Duration::from_secs(0));
    assert!(metrics.providers_used.is_empty());
}

#[test]
fn test_model_metrics_clone() {
    let mut providers_used = HashMap::new();
    providers_used.insert("openai".to_string(), 30);
    providers_used.insert("anthropic".to_string(), 20);
    let metrics = ModelMetrics {
        total_requests: 50,
        successful_requests: 48,
        failed_requests: 2,
        providers_used,
        ..Default::default()
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.total_requests, 50);
    assert_eq!(cloned.providers_used.len(), 2);
    assert_eq!(cloned.providers_used.get("openai"), Some(&30));
}

#[test]
fn test_model_metrics_debug() {
    let metrics = ModelMetrics::default();
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("ModelMetrics"));
    assert!(debug.contains("providers_used"));
}

// ==================== OverallMetrics Tests ====================

#[test]
fn test_overall_metrics_default() {
    let metrics = OverallMetrics::default();
    assert_eq!(metrics.total_requests, 0);
    assert_eq!(metrics.successful_requests, 0);
    assert_eq!(metrics.failed_requests, 0);
    assert_eq!(metrics.total_response_time, Duration::from_secs(0));
    assert_eq!(metrics.requests_per_second, 0.0);
    assert_eq!(metrics.avg_response_time, Duration::from_secs(0));
}

#[test]
fn test_overall_metrics_clone() {
    let metrics = OverallMetrics {
        total_requests: 1000,
        successful_requests: 950,
        failed_requests: 50,
        requests_per_second: 10.5,
        ..Default::default()
    };

    let cloned = metrics.clone();
    assert_eq!(cloned.total_requests, 1000);
    assert_eq!(cloned.requests_per_second, 10.5);
}

#[test]
fn test_overall_metrics_debug() {
    let metrics = OverallMetrics::default();
    let debug = format!("{:?}", metrics);
    assert!(debug.contains("OverallMetrics"));
    assert!(debug.contains("requests_per_second"));
}

// ==================== RouterMetrics Tests ====================

#[tokio::test]
async fn test_router_metrics_new() {
    let metrics = RouterMetrics::new().await;
    assert!(metrics.is_ok());
}

#[tokio::test]
async fn test_record_request_success() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;

    let snapshot = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot.overall_metrics.total_requests, 1);
    assert_eq!(snapshot.overall_metrics.successful_requests, 1);
    assert_eq!(snapshot.overall_metrics.failed_requests, 0);
}

#[tokio::test]
async fn test_record_request_failure() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(50), false)
        .await;

    let snapshot = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot.overall_metrics.total_requests, 1);
    assert_eq!(snapshot.overall_metrics.successful_requests, 0);
    assert_eq!(snapshot.overall_metrics.failed_requests, 1);
}

#[tokio::test]
async fn test_record_multiple_requests() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(150), true)
        .await;
    metrics
        .record_request("anthropic", "claude-3", Duration::from_millis(200), true)
        .await;
    metrics
        .record_request("openai", "gpt-3.5", Duration::from_millis(50), false)
        .await;

    let snapshot = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot.overall_metrics.total_requests, 4);
    assert_eq!(snapshot.overall_metrics.successful_requests, 3);
    assert_eq!(snapshot.overall_metrics.failed_requests, 1);
}

#[tokio::test]
async fn test_record_request_updates_provider_metrics() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(200), true)
        .await;

    let provider_metrics = metrics.get_provider_metrics("openai").await.unwrap();
    assert!(provider_metrics.is_some());

    let pm = provider_metrics.unwrap();
    assert_eq!(pm.total_requests, 2);
    assert_eq!(pm.successful_requests, 2);
    assert!(pm.min_response_time.is_some());
    assert!(pm.max_response_time.is_some());
}

#[tokio::test]
async fn test_record_request_updates_model_metrics() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics
        .record_request("anthropic", "gpt-4", Duration::from_millis(150), true)
        .await;

    let model_metrics = metrics.get_model_metrics("gpt-4").await.unwrap();
    assert!(model_metrics.is_some());

    let mm = model_metrics.unwrap();
    assert_eq!(mm.total_requests, 2);
    assert_eq!(mm.providers_used.len(), 2);
    assert_eq!(mm.providers_used.get("openai"), Some(&1));
    assert_eq!(mm.providers_used.get("anthropic"), Some(&1));
}

#[tokio::test]
async fn test_record_request_min_max_response_time() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(50), true)
        .await;
    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(200), true)
        .await;

    let provider_metrics = metrics
        .get_provider_metrics("openai")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        provider_metrics.min_response_time,
        Some(Duration::from_millis(50))
    );
    assert_eq!(
        provider_metrics.max_response_time,
        Some(Duration::from_millis(200))
    );
}

#[tokio::test]
async fn test_record_error() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics.record_error("openai", "timeout").await;
    metrics.record_error("openai", "timeout").await;
    metrics.record_error("openai", "rate_limit").await;

    let provider_metrics = metrics
        .get_provider_metrics("openai")
        .await
        .unwrap()
        .unwrap();
    assert_eq!(provider_metrics.error_counts.get("timeout"), Some(&2));
    assert_eq!(provider_metrics.error_counts.get("rate_limit"), Some(&1));
}

#[tokio::test]
async fn test_get_provider_metrics_not_found() {
    let metrics = RouterMetrics::new().await.unwrap();

    let result = metrics.get_provider_metrics("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_model_metrics_not_found() {
    let metrics = RouterMetrics::new().await.unwrap();

    let result = metrics.get_model_metrics("nonexistent").await.unwrap();
    assert!(result.is_none());
}

#[tokio::test]
async fn test_get_top_providers() {
    let metrics = RouterMetrics::new().await.unwrap();

    // Record requests for different providers
    for _ in 0..10 {
        metrics
            .record_request("openai", "gpt-4", Duration::from_millis(100), true)
            .await;
    }
    for _ in 0..5 {
        metrics
            .record_request("anthropic", "claude-3", Duration::from_millis(100), true)
            .await;
    }
    for _ in 0..3 {
        metrics
            .record_request("google", "gemini", Duration::from_millis(100), true)
            .await;
    }

    let top = metrics.get_top_providers(2).await.unwrap();
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].0, "openai");
    assert_eq!(top[0].1, 10);
    assert_eq!(top[1].0, "anthropic");
    assert_eq!(top[1].1, 5);
}

#[tokio::test]
async fn test_get_top_models() {
    let metrics = RouterMetrics::new().await.unwrap();

    for _ in 0..8 {
        metrics
            .record_request("openai", "gpt-4", Duration::from_millis(100), true)
            .await;
    }
    for _ in 0..12 {
        metrics
            .record_request("openai", "gpt-3.5", Duration::from_millis(100), true)
            .await;
    }

    let top = metrics.get_top_models(2).await.unwrap();
    assert_eq!(top.len(), 2);
    assert_eq!(top[0].0, "gpt-3.5");
    assert_eq!(top[0].1, 12);
    assert_eq!(top[1].0, "gpt-4");
    assert_eq!(top[1].1, 8);
}

#[tokio::test]
async fn test_get_top_providers_empty() {
    let metrics = RouterMetrics::new().await.unwrap();

    let top = metrics.get_top_providers(5).await.unwrap();
    assert!(top.is_empty());
}

#[tokio::test]
async fn test_reset() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics.record_error("openai", "timeout").await;

    let snapshot_before = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot_before.overall_metrics.total_requests, 1);

    metrics.reset().await.unwrap();

    let snapshot_after = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot_after.overall_metrics.total_requests, 0);
    assert!(snapshot_after.provider_metrics.is_empty());
    assert!(snapshot_after.model_metrics.is_empty());
}

#[tokio::test]
async fn test_get_snapshot() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;

    let snapshot = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot.overall_metrics.total_requests, 1);
    assert!(!snapshot.provider_metrics.is_empty());
    assert!(!snapshot.model_metrics.is_empty());
    assert!(snapshot.uptime > Duration::from_secs(0));
}

#[tokio::test]
async fn test_export_prometheus() {
    let metrics = RouterMetrics::new().await.unwrap();

    metrics
        .record_request("openai", "gpt-4", Duration::from_millis(100), true)
        .await;
    metrics
        .record_request("anthropic", "claude-3", Duration::from_millis(150), false)
        .await;

    let output = metrics.export_prometheus().await.unwrap();

    // Check for overall metrics
    assert!(output.contains("router_requests_total 2"));
    assert!(output.contains("router_requests_successful_total 1"));
    assert!(output.contains("router_requests_failed_total 1"));
    assert!(output.contains("router_response_time_seconds"));

    // Check for provider metrics
    assert!(output.contains("router_provider_requests_total{provider=\"openai\"}"));
    assert!(output.contains("router_provider_requests_total{provider=\"anthropic\"}"));
}

#[tokio::test]
async fn test_export_prometheus_empty() {
    let metrics = RouterMetrics::new().await.unwrap();

    let output = metrics.export_prometheus().await.unwrap();

    assert!(output.contains("router_requests_total 0"));
    assert!(output.contains("router_requests_successful_total 0"));
    assert!(output.contains("router_requests_failed_total 0"));
}

// ==================== RouterMetricsSnapshot Tests ====================

#[test]
fn test_router_metrics_snapshot_debug() {
    let snapshot = RouterMetricsSnapshot {
        provider_metrics: HashMap::new(),
        model_metrics: HashMap::new(),
        overall_metrics: OverallMetrics::default(),
        uptime: Duration::from_secs(100),
        timestamp: Instant::now(),
    };
    let debug = format!("{:?}", snapshot);
    assert!(debug.contains("RouterMetricsSnapshot"));
}

#[test]
fn test_router_metrics_snapshot_clone() {
    let mut provider_metrics = HashMap::new();
    provider_metrics.insert("openai".to_string(), ProviderMetrics::default());

    let snapshot = RouterMetricsSnapshot {
        provider_metrics,
        model_metrics: HashMap::new(),
        overall_metrics: OverallMetrics::default(),
        uptime: Duration::from_secs(100),
        timestamp: Instant::now(),
    };

    let cloned = snapshot.clone();
    assert_eq!(cloned.uptime, Duration::from_secs(100));
    assert!(cloned.provider_metrics.contains_key("openai"));
}

// ==================== Concurrent Access Tests ====================

#[tokio::test]
async fn test_concurrent_requests() {
    let metrics = Arc::new(RouterMetrics::new().await.unwrap());

    let mut handles = vec![];

    for i in 0..10 {
        let m = metrics.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                m.record_request(
                    &format!("provider_{}", i % 3),
                    "model",
                    Duration::from_millis(10),
                    true,
                )
                .await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let snapshot = metrics.get_snapshot().await.unwrap();
    assert_eq!(snapshot.overall_metrics.total_requests, 1000);
}

// ==================== MetricsData Tests ====================

#[test]
fn test_metrics_data_default() {
    let data = MetricsData::default();
    assert!(data.provider.is_empty());
    assert!(data.model.is_empty());
    assert_eq!(data.overall.total_requests, 0);
}

#[test]
fn test_metrics_data_debug() {
    let data = MetricsData::default();
    let debug = format!("{:?}", data);
    assert!(debug.contains("MetricsData"));
}
