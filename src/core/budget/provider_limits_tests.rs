use super::*;

// Provider Budget Manager Tests
#[test]
fn test_provider_budget_manager_creation() {
    let manager = ProviderBudgetManager::new();
    assert_eq!(manager.provider_count(), 0);
    assert!(manager.is_enabled());
}

#[test]
fn test_set_provider_limit() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);

    assert!(manager.has_provider_limit("openai"));
    assert_eq!(manager.provider_count(), 1);
}

#[test]
fn test_remove_provider_limit() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);
    assert!(manager.has_provider_limit("openai"));

    assert!(manager.remove_provider_limit("openai"));
    assert!(!manager.has_provider_limit("openai"));
    assert_eq!(manager.provider_count(), 0);
}

#[test]
fn test_check_provider_budget() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);

    assert_eq!(manager.check_provider_budget("openai"), BudgetStatus::Ok);
    assert_eq!(manager.check_provider_budget("unknown"), BudgetStatus::Ok);
}

#[test]
fn test_can_provider_spend() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);

    assert!(manager.can_provider_spend("openai", 50.0));
    assert!(manager.can_provider_spend("openai", 100.0));
    assert!(!manager.can_provider_spend("openai", 101.0));

    // Unknown provider has no limit
    assert!(manager.can_provider_spend("unknown", 10000.0));
}

#[test]
fn test_record_provider_spend() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);

    let status = manager.record_provider_spend("openai", 50.0);
    assert_eq!(status, Some(BudgetStatus::Ok));

    let status = manager.record_provider_spend("openai", 30.0);
    assert_eq!(status, Some(BudgetStatus::Warning));

    let status = manager.record_provider_spend("openai", 25.0);
    assert_eq!(status, Some(BudgetStatus::Exceeded));
}

#[test]
fn test_get_provider_usage() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);
    manager.record_provider_spend("openai", 30.0);

    let usage = manager.get_provider_usage("openai").unwrap();

    assert_eq!(usage.provider_name, "openai");
    assert_eq!(usage.current_spend, 30.0);
    assert_eq!(usage.max_budget, 100.0);
    assert_eq!(usage.remaining, 70.0);
    assert_eq!(usage.request_count, 1);
}

#[test]
fn test_get_available_providers() {
    let manager = ProviderBudgetManager::new();

    manager.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );
    manager.set_provider_limit(
        "anthropic",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );

    // Exceed openai budget
    manager.record_provider_spend("openai", 150.0);

    let available = manager.get_available_providers();
    assert!(available.contains(&"anthropic".to_string()));
    assert!(!available.contains(&"openai".to_string()));
}

#[test]
fn test_get_exceeded_providers() {
    let manager = ProviderBudgetManager::new();

    manager.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );
    manager.record_provider_spend("openai", 150.0);

    let exceeded = manager.get_exceeded_providers();
    assert_eq!(exceeded.len(), 1);
    assert_eq!(exceeded[0], "openai");
}

#[test]
fn test_reset_provider_budget() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);
    manager.record_provider_spend("openai", 75.0);

    assert!(manager.reset_provider_budget("openai"));

    let usage = manager.get_provider_usage("openai").unwrap();
    assert_eq!(usage.current_spend, 0.0);
    assert_eq!(usage.request_count, 0);
}

#[test]
fn test_disabled_manager_allows_all() {
    let manager = ProviderBudgetManager::new();
    let config = ProviderLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_provider_limit("openai", config);
    manager.record_provider_spend("openai", 150.0);

    // Normally would be exceeded
    assert_eq!(
        manager.check_provider_budget("openai"),
        BudgetStatus::Exceeded
    );

    // Disable manager
    manager.set_enabled(false);

    // Now returns Ok and allows spending
    assert_eq!(manager.check_provider_budget("openai"), BudgetStatus::Ok);
    assert!(manager.can_provider_spend("openai", 1000.0));
}

// Model Budget Manager Tests
#[test]
fn test_model_budget_manager_creation() {
    let manager = ModelBudgetManager::new();
    assert_eq!(manager.model_count(), 0);
    assert!(manager.is_enabled());
}

#[test]
fn test_set_model_limit() {
    let manager = ModelBudgetManager::new();
    let config = ModelLimitConfig::new(500.0, ResetPeriod::Monthly);

    manager.set_model_limit("gpt-4", config);

    assert!(manager.has_model_limit("gpt-4"));
    assert_eq!(manager.model_count(), 1);
}

#[test]
fn test_check_model_budget() {
    let manager = ModelBudgetManager::new();
    let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_model_limit("gpt-4", config);

    assert_eq!(manager.check_model_budget("gpt-4"), BudgetStatus::Ok);
}

#[test]
fn test_record_model_spend() {
    let manager = ModelBudgetManager::new();
    let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_model_limit("gpt-4", config);

    let status = manager.record_model_spend("gpt-4", 50.0);
    assert_eq!(status, Some(BudgetStatus::Ok));

    let status = manager.record_model_spend("gpt-4", 55.0);
    assert_eq!(status, Some(BudgetStatus::Exceeded));
}

#[test]
fn test_get_model_usage() {
    let manager = ModelBudgetManager::new();
    let config = ModelLimitConfig::new(100.0, ResetPeriod::Monthly);

    manager.set_model_limit("gpt-4", config);
    manager.record_model_spend("gpt-4", 25.0);

    let usage = manager.get_model_usage("gpt-4").unwrap();

    assert_eq!(usage.model_name, "gpt-4");
    assert_eq!(usage.current_spend, 25.0);
    assert_eq!(usage.request_count, 1);
}

// Unified Budget Limits Tests
#[test]
fn test_unified_budget_limits() {
    let limits = UnifiedBudgetLimits::new();

    limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(1000.0, ResetPeriod::Monthly),
    );
    limits
        .models
        .set_model_limit("gpt-4", ModelLimitConfig::new(500.0, ResetPeriod::Monthly));

    assert!(limits.can_spend("openai", "gpt-4", 100.0));

    limits.record_spend("openai", "gpt-4", 100.0);

    let provider_usage = limits.providers.get_provider_usage("openai").unwrap();
    let model_usage = limits.models.get_model_usage("gpt-4").unwrap();

    assert_eq!(provider_usage.current_spend, 100.0);
    assert_eq!(model_usage.current_spend, 100.0);
}

#[test]
fn test_filter_available_providers() {
    let limits = UnifiedBudgetLimits::new();

    limits.providers.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );
    limits.providers.set_provider_limit(
        "anthropic",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );
    limits.providers.set_provider_limit(
        "google",
        ProviderLimitConfig::new(100.0, ResetPeriod::Monthly),
    );

    // Exceed openai budget
    limits.providers.record_provider_spend("openai", 150.0);

    let providers = vec![
        "openai".to_string(),
        "anthropic".to_string(),
        "google".to_string(),
    ];
    let available = limits.filter_available_providers(providers);

    assert_eq!(available.len(), 2);
    assert!(!available.contains(&"openai".to_string()));
    assert!(available.contains(&"anthropic".to_string()));
    assert!(available.contains(&"google".to_string()));
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let manager = Arc::new(ProviderBudgetManager::new());
    manager.set_provider_limit(
        "openai",
        ProviderLimitConfig::new(10000.0, ResetPeriod::Monthly),
    );

    let mut handles = vec![];

    for _ in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                manager_clone.record_provider_spend("openai", 1.0);
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let usage = manager.get_provider_usage("openai").unwrap();
    assert_eq!(usage.current_spend, 1000.0);
    assert_eq!(usage.request_count, 1000);
}
