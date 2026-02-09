use super::*;
use crate::core::router::strategy::types::ProviderUsage;
use std::collections::HashMap;

fn create_routing_data() -> RwLock<RoutingData> {
    RwLock::new(RoutingData::default())
}

fn create_providers() -> Vec<String> {
    vec![
        "openai".to_string(),
        "anthropic".to_string(),
        "azure".to_string(),
    ]
}

// ==================== Round Robin Tests ====================

#[test]
fn test_round_robin_single_provider() {
    let providers = vec!["openai".to_string()];
    let counter = AtomicUsize::new(0);

    let result = SelectionMethods::select_round_robin(&providers, &counter);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "openai");
}

#[test]
fn test_round_robin_multiple_providers() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);

    // First call should select first provider
    let r1 = SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(r1, "openai");

    // Second call should select second provider
    let r2 = SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(r2, "anthropic");

    // Third call should select third provider
    let r3 = SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(r3, "azure");

    // Fourth call should wrap around
    let r4 = SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(r4, "openai");
}

#[test]
fn test_round_robin_counter_increment() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);

    SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 1);

    SelectionMethods::select_round_robin(&providers, &counter).unwrap();
    assert_eq!(counter.load(Ordering::Relaxed), 2);
}

// ==================== Least Latency Tests ====================

#[test]
fn test_least_latency_selects_lowest() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.latencies.insert("openai".to_string(), 100.0);
        data.latencies.insert("anthropic".to_string(), 50.0);
        data.latencies.insert("azure".to_string(), 150.0);
    }

    let result = SelectionMethods::select_least_latency(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_least_latency_missing_data_uses_max() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.latencies.insert("openai".to_string(), 100.0);
        // anthropic and azure have no latency data
    }

    let result = SelectionMethods::select_least_latency(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "openai");
}

#[test]
fn test_least_latency_all_equal() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.latencies.insert("openai".to_string(), 100.0);
        data.latencies.insert("anthropic".to_string(), 100.0);
        data.latencies.insert("azure".to_string(), 100.0);
    }

    let result = SelectionMethods::select_least_latency(&providers, &routing_data);
    assert!(result.is_ok());
    // Should return first provider when all equal
    assert_eq!(result.unwrap(), "openai");
}

// ==================== Least Cost Tests ====================

#[test]
fn test_least_cost_selects_cheapest() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.costs.insert("openai:gpt-4".to_string(), 0.03);
        data.costs.insert("anthropic:gpt-4".to_string(), 0.015);
        data.costs.insert("azure:gpt-4".to_string(), 0.025);
    }

    let result = SelectionMethods::select_least_cost(&providers, "gpt-4", &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_least_cost_missing_data_uses_max() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.costs.insert("openai:gpt-4".to_string(), 0.03);
        // Others have no cost data
    }

    let result = SelectionMethods::select_least_cost(&providers, "gpt-4", &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "openai");
}

// ==================== Random Selection Tests ====================

#[test]
fn test_random_returns_valid_provider() {
    let providers = create_providers();

    for _ in 0..100 {
        let result = SelectionMethods::select_random(&providers);
        assert!(result.is_ok());
        let selected = result.unwrap();
        assert!(providers.contains(&selected));
    }
}

#[test]
fn test_random_single_provider() {
    let providers = vec!["openai".to_string()];

    let result = SelectionMethods::select_random(&providers);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "openai");
}

// ==================== Weighted Selection Tests ====================

#[test]
fn test_weighted_respects_weights() {
    let providers = create_providers();
    let routing_data = create_routing_data();
    let counter = AtomicUsize::new(0);

    {
        let mut data = routing_data.write();
        data.weights.insert("openai".to_string(), 10.0);
        data.weights.insert("anthropic".to_string(), 1.0);
        data.weights.insert("azure".to_string(), 1.0);
    }

    // Run many selections and check distribution
    let mut counts: HashMap<String, usize> = HashMap::new();
    for _ in 0..1000 {
        let result = SelectionMethods::select_weighted(&providers, &routing_data, &counter);
        assert!(result.is_ok());
        *counts.entry(result.unwrap()).or_insert(0) += 1;
    }

    // openai should be selected significantly more often
    let openai_count = counts.get("openai").copied().unwrap_or(0);
    let anthropic_count = counts.get("anthropic").copied().unwrap_or(0);
    assert!(
        openai_count > anthropic_count * 3,
        "openai: {}, anthropic: {}",
        openai_count,
        anthropic_count
    );
}

#[test]
fn test_weighted_zero_weights_falls_back() {
    let providers = create_providers();
    let routing_data = create_routing_data();
    let counter = AtomicUsize::new(0);

    {
        let mut data = routing_data.write();
        data.weights.insert("openai".to_string(), 0.0);
        data.weights.insert("anthropic".to_string(), 0.0);
        data.weights.insert("azure".to_string(), 0.0);
    }

    // Should fall back to round-robin
    let result = SelectionMethods::select_weighted(&providers, &routing_data, &counter);
    assert!(result.is_ok());
}

#[test]
fn test_weighted_default_weights() {
    let providers = create_providers();
    let routing_data = create_routing_data();
    let counter = AtomicUsize::new(0);

    // No weights set - should use default of 1.0 for each
    let result = SelectionMethods::select_weighted(&providers, &routing_data, &counter);
    assert!(result.is_ok());
    let selected = result.unwrap();
    assert!(providers.contains(&selected));
}

// ==================== Priority Selection Tests ====================

#[test]
fn test_priority_selects_highest() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.priorities.insert("openai".to_string(), 1);
        data.priorities.insert("anthropic".to_string(), 3);
        data.priorities.insert("azure".to_string(), 2);
    }

    let result = SelectionMethods::select_priority(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_priority_missing_data_uses_zero() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.priorities.insert("azure".to_string(), 1);
        // Others have no priority
    }

    let result = SelectionMethods::select_priority(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "azure");
}

#[test]
fn test_priority_all_equal() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    // All have default priority of 0
    let result = SelectionMethods::select_priority(&providers, &routing_data);
    assert!(result.is_ok());
    // Should return first provider when all equal
    assert_eq!(result.unwrap(), "openai");
}

// ==================== A/B Test Selection Tests ====================

#[test]
fn test_ab_test_single_provider() {
    let providers = vec!["openai".to_string()];

    let result = SelectionMethods::select_ab_test(&providers, 0.5);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "openai");
}

#[test]
fn test_ab_test_respects_ratio() {
    let providers = vec!["openai".to_string(), "anthropic".to_string()];

    // Run many selections with 0.8 ratio
    let mut openai_count = 0;
    let mut anthropic_count = 0;

    for _ in 0..1000 {
        let result = SelectionMethods::select_ab_test(&providers, 0.8);
        match result.unwrap().as_str() {
            "openai" => openai_count += 1,
            "anthropic" => anthropic_count += 1,
            _ => panic!("Unexpected provider"),
        }
    }

    // openai should be selected ~80% of the time
    let openai_ratio = openai_count as f64 / (openai_count + anthropic_count) as f64;
    assert!(
        openai_ratio > 0.7 && openai_ratio < 0.9,
        "Expected ~0.8, got {}",
        openai_ratio
    );
}

#[test]
fn test_ab_test_zero_ratio() {
    let providers = vec!["openai".to_string(), "anthropic".to_string()];

    // All selections should go to provider B (anthropic)
    for _ in 0..100 {
        let result = SelectionMethods::select_ab_test(&providers, 0.0);
        assert_eq!(result.unwrap(), "anthropic");
    }
}

#[test]
fn test_ab_test_full_ratio() {
    let providers = vec!["openai".to_string(), "anthropic".to_string()];

    // All selections should go to provider A (openai)
    for _ in 0..100 {
        let result = SelectionMethods::select_ab_test(&providers, 1.0);
        assert_eq!(result.unwrap(), "openai");
    }
}

// ==================== Custom Strategy Tests ====================

#[test]
fn test_custom_selects_named_provider() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);
    let context = RequestContext::default();

    let r1 = SelectionMethods::select_custom(&providers, "azure", &context, &counter);
    assert!(r1.is_ok());
    assert_eq!(r1.unwrap(), "azure");
}

#[test]
fn test_custom_rule_match_user_id() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);
    let context = RequestContext::default().with_user_id("user-123");

    let logic = "user_id=user-123->anthropic;default->openai";
    let selected = SelectionMethods::select_custom(&providers, logic, &context, &counter)
        .expect("expected provider");
    assert_eq!(selected, "anthropic");
}

#[test]
fn test_custom_default_rule() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);
    let context = RequestContext::default();

    let logic = "user_id=user-123->anthropic;default->openai";
    let selected = SelectionMethods::select_custom(&providers, logic, &context, &counter)
        .expect("expected provider");
    assert_eq!(selected, "openai");
}

#[test]
fn test_custom_invalid_rules_fall_back() {
    let providers = create_providers();
    let counter = AtomicUsize::new(0);
    let context = RequestContext::default();

    let r1 = SelectionMethods::select_custom(&providers, "invalid_rule", &context, &counter);
    assert!(r1.is_ok());
    assert_eq!(r1.unwrap(), "openai");

    let r2 = SelectionMethods::select_custom(&providers, "invalid_rule", &context, &counter);
    assert_eq!(r2.unwrap(), "anthropic");
}

// ==================== Usage-Based Selection Tests ====================

#[test]
fn test_usage_based_selects_lowest_usage() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.usage.insert(
            "openai".to_string(),
            ProviderUsage {
                tpm: 5000,
                rpm: 50,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 5,
            },
        );
        data.usage.insert(
            "anthropic".to_string(),
            ProviderUsage {
                tpm: 2000,
                rpm: 20,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 2,
            },
        );
        data.usage.insert(
            "azure".to_string(),
            ProviderUsage {
                tpm: 8000,
                rpm: 80,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 8,
            },
        );
    }

    let result = SelectionMethods::select_usage_based(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_usage_based_no_data_uses_zero() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.usage.insert(
            "openai".to_string(),
            ProviderUsage {
                tpm: 5000,
                rpm: 50,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 5,
            },
        );
        // Others have no usage data (= 0% usage)
    }

    let result = SelectionMethods::select_usage_based(&providers, &routing_data);
    assert!(result.is_ok());
    // anthropic has no data = 0% usage
    assert_eq!(result.unwrap(), "anthropic");
}

// ==================== Least Busy Selection Tests ====================

#[test]
fn test_least_busy_selects_fewest_requests() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.usage.insert(
            "openai".to_string(),
            ProviderUsage {
                tpm: 0,
                rpm: 0,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 10,
            },
        );
        data.usage.insert(
            "anthropic".to_string(),
            ProviderUsage {
                tpm: 0,
                rpm: 0,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 3,
            },
        );
        data.usage.insert(
            "azure".to_string(),
            ProviderUsage {
                tpm: 0,
                rpm: 0,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 7,
            },
        );
    }

    let result = SelectionMethods::select_least_busy(&providers, &routing_data);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_least_busy_no_data_uses_zero() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    {
        let mut data = routing_data.write();
        data.usage.insert(
            "openai".to_string(),
            ProviderUsage {
                tpm: 0,
                rpm: 0,
                tpm_limit: Some(10000),
                rpm_limit: Some(100),
                active_requests: 5,
            },
        );
        // Others have no usage data (= 0 active requests)
    }

    let result = SelectionMethods::select_least_busy(&providers, &routing_data);
    assert!(result.is_ok());
    // anthropic has no data = 0 active
    assert_eq!(result.unwrap(), "anthropic");
}

#[test]
fn test_least_busy_all_zero() {
    let providers = create_providers();
    let routing_data = create_routing_data();

    // No usage data for any provider
    let result = SelectionMethods::select_least_busy(&providers, &routing_data);
    assert!(result.is_ok());
    // Should return first provider
    assert_eq!(result.unwrap(), "openai");
}
