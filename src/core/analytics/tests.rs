//! Tests for analytics module

#[cfg(test)]
use super::collector::MetricsCollector;
use super::optimizer::CostOptimizer;

#[tokio::test]
async fn test_metrics_collection() {
    let collector = MetricsCollector::new();
    let usage_data = vec![serde_json::json!({
        "total_tokens": 100,
        "cost": 0.01
    })];

    let metrics = collector
        .process_user_data("user123", &usage_data)
        .await
        .unwrap();
    assert_eq!(metrics.request_count, 1);
    assert_eq!(metrics.token_usage.total_tokens, 100);
}

#[test]
fn test_cost_optimizer() {
    let optimizer = CostOptimizer::new();
    assert!(!optimizer.rules().is_empty());
}
