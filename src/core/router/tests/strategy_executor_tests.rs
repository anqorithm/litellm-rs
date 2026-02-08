//! Strategy executor tests

use crate::core::router::strategy::executor::StrategyExecutor;
use crate::core::router::strategy::types::{ProviderUsage, RoutingStrategy};
use crate::core::types::context::RequestContext;

#[test]
fn test_provider_usage_percentage() {
    let mut usage = ProviderUsage::default();

    // No limits set = 0%
    assert_eq!(usage.usage_percentage(), 0.0);

    // Set limits and usage
    usage.tpm = 5000;
    usage.tpm_limit = Some(10000);
    usage.rpm = 50;
    usage.rpm_limit = Some(100);

    // Should be 50% (both at 50%)
    assert!((usage.usage_percentage() - 0.5).abs() < 0.001);

    // TPM at 80%, RPM at 50% -> should return 80%
    usage.tpm = 8000;
    assert!((usage.usage_percentage() - 0.8).abs() < 0.001);
}

#[tokio::test]
async fn test_usage_based_routing() {
    let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
        .await
        .unwrap();

    // Set up usage data
    executor
        .update_usage(
            "provider_a",
            ProviderUsage {
                tpm: 8000,
                rpm: 80,
                active_requests: 5,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
            },
        )
        .await
        .unwrap();

    executor
        .update_usage(
            "provider_b",
            ProviderUsage {
                tpm: 2000,
                rpm: 20,
                active_requests: 2,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
            },
        )
        .await
        .unwrap();

    let providers = vec!["provider_a".to_string(), "provider_b".to_string()];
    let context = RequestContext::default();

    // Should select provider_b (20% usage vs 80% usage)
    let selected = executor
        .select_provider(&providers, "gpt-4", &context)
        .await
        .unwrap();
    assert_eq!(selected, "provider_b");
}

#[tokio::test]
async fn test_least_busy_routing() {
    let executor = StrategyExecutor::new(RoutingStrategy::LeastBusy)
        .await
        .unwrap();

    // Set up usage data
    executor
        .update_usage(
            "provider_a",
            ProviderUsage {
                active_requests: 10,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    executor
        .update_usage(
            "provider_b",
            ProviderUsage {
                active_requests: 3,
                ..Default::default()
            },
        )
        .await
        .unwrap();

    let providers = vec!["provider_a".to_string(), "provider_b".to_string()];
    let context = RequestContext::default();

    // Should select provider_b (3 active vs 10 active)
    let selected = executor
        .select_provider(&providers, "gpt-4", &context)
        .await
        .unwrap();
    assert_eq!(selected, "provider_b");
}

#[tokio::test]
async fn test_active_request_tracking() {
    let executor = StrategyExecutor::new(RoutingStrategy::LeastBusy)
        .await
        .unwrap();

    executor
        .increment_active_requests("provider_a")
        .await
        .unwrap();
    executor
        .increment_active_requests("provider_a")
        .await
        .unwrap();
    executor
        .increment_active_requests("provider_a")
        .await
        .unwrap();

    let usage = executor.get_usage("provider_a").await.unwrap();
    assert_eq!(usage.active_requests, 3);

    executor
        .decrement_active_requests("provider_a")
        .await
        .unwrap();
    let usage = executor.get_usage("provider_a").await.unwrap();
    assert_eq!(usage.active_requests, 2);
}

#[tokio::test]
async fn test_token_usage_recording() {
    let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
        .await
        .unwrap();

    executor
        .record_token_usage("provider_a", 1000)
        .await
        .unwrap();
    executor
        .record_token_usage("provider_a", 500)
        .await
        .unwrap();

    let usage = executor.get_usage("provider_a").await.unwrap();
    assert_eq!(usage.tpm, 1500);
    assert_eq!(usage.rpm, 2);
}

#[tokio::test]
async fn test_usage_counter_reset() {
    let executor = StrategyExecutor::new(RoutingStrategy::UsageBased)
        .await
        .unwrap();

    executor
        .record_token_usage("provider_a", 1000)
        .await
        .unwrap();
    executor
        .increment_active_requests("provider_a")
        .await
        .unwrap();

    executor.reset_usage_counters().await.unwrap();

    let usage = executor.get_usage("provider_a").await.unwrap();
    assert_eq!(usage.tpm, 0);
    assert_eq!(usage.rpm, 0);
    // Active requests should NOT be reset
    assert_eq!(usage.active_requests, 1);
}
