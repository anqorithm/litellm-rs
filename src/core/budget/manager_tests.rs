use super::{BudgetConfig, BudgetManager, BudgetManagerConfig, BudgetScope, ResetPeriod};
use crate::utils::error::gateway_error::GatewayError;

#[tokio::test]
async fn test_manager_creation() {
    let manager = BudgetManager::new();
    assert_eq!(manager.budget_count(), 0);
}

#[tokio::test]
async fn test_create_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0);
    let budget = manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    assert_eq!(budget.name, "Test Budget");
    assert_eq!(budget.max_budget, 100.0);
    assert_eq!(budget.soft_limit, 80.0); // Default 80%
    assert_eq!(manager.budget_count(), 1);
}

#[tokio::test]
async fn test_create_budget_with_custom_soft_limit() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0).with_soft_limit(90.0);
    let budget = manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    assert_eq!(budget.soft_limit, 90.0);
}

#[tokio::test]
async fn test_create_budget_validation() {
    let manager = BudgetManager::new();

    // Test negative budget
    let config = BudgetConfig::new("Test", -10.0);
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());

    let config = BudgetConfig::new("Test", f64::NAN);
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());

    let config = BudgetConfig::new("Test", f64::INFINITY);
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());

    let config = BudgetConfig::new("Test", 100.0).with_soft_limit(f64::NAN);
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());

    // Test empty name
    let config = BudgetConfig::new("", 100.0);
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_create_duplicate_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0);
    manager
        .create_budget(BudgetScope::Global, config.clone())
        .await
        .unwrap();

    // Second create should fail
    let result = manager.create_budget(BudgetScope::Global, config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Original", 100.0);
    manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    let update_config = BudgetConfig::new("Updated", 200.0);
    let updated = manager
        .update_budget(&BudgetScope::Global, update_config)
        .await
        .unwrap();

    assert_eq!(updated.name, "Updated");
    assert_eq!(updated.max_budget, 200.0);
}

#[tokio::test]
async fn test_update_nonexistent_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test", 100.0);
    let result = manager.update_budget(&BudgetScope::Global, config).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_update_budget_validation_rejects_non_finite_values() {
    let manager = BudgetManager::new();
    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Original", 100.0))
        .await
        .unwrap();

    let result = manager
        .update_budget(&BudgetScope::Global, BudgetConfig::new("Updated", f64::NAN))
        .await;
    assert!(result.is_err());

    let result = manager
        .update_budget(
            &BudgetScope::Global,
            BudgetConfig::new("Updated", 200.0).with_soft_limit(f64::INFINITY),
        )
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_delete_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0);
    manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    assert_eq!(manager.budget_count(), 1);

    manager.delete_budget(&BudgetScope::Global).await.unwrap();
    assert_eq!(manager.budget_count(), 0);
}

#[tokio::test]
async fn test_delete_nonexistent_budget() {
    let manager = BudgetManager::new();

    let result = manager.delete_budget(&BudgetScope::Global).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_get_budget() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0);
    let created = manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    let retrieved = manager.get_budget(&BudgetScope::Global).unwrap();
    assert_eq!(retrieved.id, created.id);
}

#[tokio::test]
async fn test_get_budget_by_id() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test Budget", 100.0);
    let created = manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    let retrieved = manager.get_budget_by_id(&created.id).unwrap();
    assert_eq!(retrieved.name, "Test Budget");
}

#[tokio::test]
async fn test_list_budgets() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();
    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 50.0),
        )
        .await
        .unwrap();

    let budgets = manager.list_budgets();
    assert_eq!(budgets.len(), 2);
}

#[tokio::test]
async fn test_list_budgets_filtered() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();
    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 50.0),
        )
        .await
        .unwrap();
    manager
        .create_budget(
            BudgetScope::User("user-2".to_string()),
            BudgetConfig::new("User 2", 50.0),
        )
        .await
        .unwrap();

    let user_budgets = manager.list_budgets_filtered(Some("user"), None);
    assert_eq!(user_budgets.len(), 2);

    let global_budgets = manager.list_budgets_filtered(Some("global"), None);
    assert_eq!(global_budgets.len(), 1);
}

#[tokio::test]
async fn test_record_spend() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    let result = manager
        .record_spend(&BudgetScope::Global, 25.0)
        .await
        .unwrap();

    assert_eq!(result.current_spend, 25.0);
    assert_eq!(manager.get_current_spend(&BudgetScope::Global), 25.0);
}

#[tokio::test]
async fn test_check_spend() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    // Record some spend first
    manager.record_spend(&BudgetScope::Global, 90.0).await;

    let result_ok = manager.check_spend(&BudgetScope::Global, 10.0).await;
    assert!(result_ok.allowed);

    let result_exceed = manager.check_spend(&BudgetScope::Global, 11.0).await;
    assert!(!result_exceed.allowed);
}

#[tokio::test]
async fn test_check_spend_disabled_blocking() {
    let config = BudgetManagerConfig {
        block_on_exceeded: false,
        ..Default::default()
    };
    let manager = BudgetManager::with_config(config);

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    manager.record_spend(&BudgetScope::Global, 100.0).await;

    // Should still be allowed even though exceeded
    let result = manager.check_spend(&BudgetScope::Global, 10.0).await;
    assert!(result.allowed);
}

#[tokio::test]
async fn test_reset_budget() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();

    manager.record_spend(&BudgetScope::Global, 50.0).await;
    assert_eq!(manager.get_current_spend(&BudgetScope::Global), 50.0);

    manager.reset_budget(&BudgetScope::Global).await.unwrap();
    assert_eq!(manager.get_current_spend(&BudgetScope::Global), 0.0);
}

#[tokio::test]
async fn test_get_summary() {
    let manager = BudgetManager::new();

    manager
        .create_budget(BudgetScope::Global, BudgetConfig::new("Global", 100.0))
        .await
        .unwrap();
    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 50.0),
        )
        .await
        .unwrap();

    manager.record_spend(&BudgetScope::Global, 85.0).await; // Warning
    manager
        .record_spend(&BudgetScope::User("user-1".to_string()), 10.0)
        .await; // OK

    let summary = manager.get_summary();

    assert_eq!(summary.total_budgets, 2);
    assert_eq!(summary.total_allocated, 150.0);
    assert_eq!(summary.total_spent, 95.0);
    assert_eq!(summary.total_remaining, 55.0);
    assert_eq!(summary.ok_count, 1);
    assert_eq!(summary.warning_count, 1);
    assert_eq!(summary.exceeded_count, 0);
}

#[tokio::test]
async fn test_config_management() {
    let manager = BudgetManager::new();

    let config = manager.get_config().await;
    assert!(config.enabled);

    manager.set_enabled(false).await;
    assert!(!manager.is_enabled().await);

    let new_config = BudgetManagerConfig {
        enabled: true,
        default_soft_limit_percentage: 0.9,
        block_on_exceeded: false,
        auto_reset_enabled: false,
        reset_check_interval_secs: 120,
    };

    manager.update_config(new_config).await;

    let updated_config = manager.get_config().await;
    assert!(updated_config.enabled);
    assert!(!updated_config.block_on_exceeded);
    assert_eq!(updated_config.default_soft_limit_percentage, 0.9);
}

#[tokio::test]
async fn test_get_warning_and_exceeded_budgets() {
    let manager = BudgetManager::new();

    // OK budget
    manager
        .create_budget(
            BudgetScope::User("user-1".to_string()),
            BudgetConfig::new("User 1", 100.0),
        )
        .await
        .unwrap();

    // Warning budget
    manager
        .create_budget(
            BudgetScope::User("user-2".to_string()),
            BudgetConfig::new("User 2", 100.0),
        )
        .await
        .unwrap();
    manager
        .record_spend(&BudgetScope::User("user-2".to_string()), 85.0)
        .await;

    // Exceeded budget
    manager
        .create_budget(
            BudgetScope::User("user-3".to_string()),
            BudgetConfig::new("User 3", 100.0),
        )
        .await
        .unwrap();
    manager
        .record_spend(&BudgetScope::User("user-3".to_string()), 110.0)
        .await;

    let warning_budgets = manager.get_warning_budgets();
    assert_eq!(warning_budgets.len(), 1);
    assert_eq!(warning_budgets[0].name, "User 2");

    let exceeded_budgets = manager.get_exceeded_budgets();
    assert_eq!(exceeded_budgets.len(), 1);
    assert_eq!(exceeded_budgets[0].name, "User 3");
}

#[tokio::test]
async fn test_create_budget_with_reset_period() {
    let manager = BudgetManager::new();

    let config = BudgetConfig::new("Test", 100.0).with_reset_period(ResetPeriod::Weekly);

    let budget = manager
        .create_budget(BudgetScope::Global, config)
        .await
        .unwrap();

    assert_eq!(budget.reset_period, ResetPeriod::Weekly);
}

#[tokio::test]
async fn test_create_budget_concurrent_no_toctou() {
    use std::sync::Arc;

    let manager = Arc::new(BudgetManager::new());
    let concurrency = 20;

    let handles: Vec<_> = (0..concurrency)
        .map(|_| {
            let m = Arc::clone(&manager);
            tokio::spawn(async move {
                m.create_budget(BudgetScope::Global, BudgetConfig::new("Concurrent", 100.0))
                    .await
            })
        })
        .collect();

    let mut ok_count = 0usize;
    let mut conflict_count = 0usize;

    for handle in handles {
        match handle.await {
            Ok(Ok(_)) => ok_count += 1,
            Ok(Err(GatewayError::Conflict(_))) => conflict_count += 1,
            Ok(Err(e)) => panic!("unexpected error: {:?}", e),
            Err(e) => panic!("task panicked: {:?}", e),
        }
    }

    // Exactly one insertion must succeed; the rest must return Conflict
    assert_eq!(ok_count, 1, "exactly one create_budget should succeed");
    assert_eq!(
        conflict_count,
        concurrency - 1,
        "all other concurrent calls should return Conflict"
    );
    assert_eq!(manager.budget_count(), 1);
}
