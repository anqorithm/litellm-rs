use super::*;
use crate::core::budget::types::ResetPeriod;

fn create_test_budget(id: &str, scope: BudgetScope, max_budget: f64) -> Budget {
    Budget::new(id, format!("Test Budget {}", id), scope, max_budget)
}

#[test]
fn test_tracker_creation() {
    let tracker = BudgetTracker::new();
    assert_eq!(tracker.budget_count(), 0);
}

#[test]
fn test_tracker_with_capacity() {
    let tracker = BudgetTracker::with_capacity(100);
    assert_eq!(tracker.budget_count(), 0);
}

#[test]
fn test_register_and_get_budget() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget.clone());
    assert!(tracker.has_budget(&BudgetScope::Global));
    let retrieved = tracker.get_budget(&BudgetScope::Global).unwrap();
    assert_eq!(retrieved.id, "test-1");
    assert_eq!(retrieved.max_budget, 100.0);
}

#[test]
fn test_unregister_budget() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    assert!(tracker.has_budget(&BudgetScope::Global));
    tracker.unregister_budget(&BudgetScope::Global);
    assert!(!tracker.has_budget(&BudgetScope::Global));
}

#[test]
fn test_record_spend() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    let result = tracker.record_spend(&BudgetScope::Global, 25.0).unwrap();
    assert_eq!(result.current_spend, 25.0);
    assert_eq!(result.remaining, 75.0);
    assert_eq!(result.new_status, BudgetStatus::Ok);
}

#[test]
fn test_record_spend_triggers_warning() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    let result1 = tracker.record_spend(&BudgetScope::Global, 79.0).unwrap();
    assert_eq!(result1.new_status, BudgetStatus::Ok);
    assert!(!result1.should_alert_soft_limit);
    let result2 = tracker.record_spend(&BudgetScope::Global, 1.0).unwrap();
    assert_eq!(result2.new_status, BudgetStatus::Warning);
    assert!(result2.should_alert_soft_limit);
}

#[test]
fn test_record_spend_triggers_exceeded() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    let result = tracker.record_spend(&BudgetScope::Global, 100.0).unwrap();
    assert_eq!(result.new_status, BudgetStatus::Exceeded);
    assert!(result.should_alert_exceeded);
}

#[test]
fn test_record_spend_no_duplicate_alerts() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    let result1 = tracker.record_spend(&BudgetScope::Global, 100.0).unwrap();
    assert!(result1.should_alert_exceeded);
    let result2 = tracker.record_spend(&BudgetScope::Global, 10.0).unwrap();
    assert!(!result2.should_alert_exceeded);
}

#[test]
fn test_check_budget() {
    let tracker = BudgetTracker::new();
    let budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    tracker.register_budget(budget);
    let result = tracker.check_budget(&BudgetScope::Global);
    assert!(result.allowed);
    assert_eq!(result.status, BudgetStatus::Ok);
    assert_eq!(result.max_budget, 100.0);
}

#[test]
fn test_check_budget_no_budget() {
    let tracker = BudgetTracker::new();
    let result = tracker.check_budget(&BudgetScope::User("unknown".to_string()));
    assert!(result.allowed);
    assert!(result.max_budget.is_infinite());
}

#[test]
fn test_check_spend() {
    let tracker = BudgetTracker::new();
    let mut budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    budget.current_spend = 90.0;
    tracker.register_budget(budget);
    let result_ok = tracker.check_spend(&BudgetScope::Global, 10.0);
    assert!(result_ok.allowed);
    let result_exceed = tracker.check_spend(&BudgetScope::Global, 11.0);
    assert!(!result_exceed.allowed);
}

#[test]
fn test_get_remaining() {
    let tracker = BudgetTracker::new();
    let mut budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    budget.current_spend = 30.0;
    tracker.register_budget(budget);
    assert_eq!(tracker.get_remaining(&BudgetScope::Global), 70.0);
}

#[test]
fn test_get_remaining_no_budget() {
    let tracker = BudgetTracker::new();
    assert!(tracker.get_remaining(&BudgetScope::Global).is_infinite());
}

#[test]
fn test_get_current_spend() {
    let tracker = BudgetTracker::new();
    let mut budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    budget.current_spend = 45.0;
    tracker.register_budget(budget);
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 45.0);
}

#[test]
fn test_get_all_budgets() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(create_test_budget("b1", BudgetScope::Global, 100.0));
    tracker.register_budget(create_test_budget(
        "b2",
        BudgetScope::User("user-1".to_string()),
        50.0,
    ));
    let budgets = tracker.get_all_budgets();
    assert_eq!(budgets.len(), 2);
}

#[test]
fn test_reset_budget() {
    let tracker = BudgetTracker::new();
    let mut budget = create_test_budget("test-1", BudgetScope::Global, 100.0);
    budget.current_spend = 75.0;
    tracker.register_budget(budget);
    assert!(tracker.reset_budget(&BudgetScope::Global));
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 0.0);
}

#[test]
fn test_reset_budget_not_found() {
    let tracker = BudgetTracker::new();
    assert!(!tracker.reset_budget(&BudgetScope::Global));
}

#[test]
fn test_update_budget() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(create_test_budget("test-1", BudgetScope::Global, 100.0));
    let updated = tracker.update_budget(&BudgetScope::Global, |budget| {
        budget.max_budget = 200.0;
        budget.soft_limit = 160.0;
    });
    assert!(updated);
    let budget = tracker.get_budget(&BudgetScope::Global).unwrap();
    assert_eq!(budget.max_budget, 200.0);
    assert_eq!(budget.soft_limit, 160.0);
}

#[test]
fn test_get_warning_budgets() {
    let tracker = BudgetTracker::new();
    let mut warning_budget = create_test_budget("warn", BudgetScope::Global, 100.0);
    warning_budget.current_spend = 85.0;
    let ok_budget = create_test_budget("ok", BudgetScope::User("user-1".to_string()), 100.0);
    tracker.register_budget(warning_budget);
    tracker.register_budget(ok_budget);
    let warning_budgets = tracker.get_warning_budgets();
    assert_eq!(warning_budgets.len(), 1);
    assert_eq!(warning_budgets[0].id, "warn");
}

#[test]
fn test_get_exceeded_budgets() {
    let tracker = BudgetTracker::new();
    let mut exceeded_budget = create_test_budget("exceeded", BudgetScope::Global, 100.0);
    exceeded_budget.current_spend = 150.0;
    let ok_budget = create_test_budget("ok", BudgetScope::User("user-1".to_string()), 100.0);
    tracker.register_budget(exceeded_budget);
    tracker.register_budget(ok_budget);
    let exceeded_budgets = tracker.get_exceeded_budgets();
    assert_eq!(exceeded_budgets.len(), 1);
    assert_eq!(exceeded_budgets[0].id, "exceeded");
}

#[test]
fn test_get_budgets_by_type() {
    let tracker = BudgetTracker::new();
    tracker.register_budget(create_test_budget("global", BudgetScope::Global, 100.0));
    tracker.register_budget(create_test_budget(
        "user1",
        BudgetScope::User("user-1".to_string()),
        50.0,
    ));
    tracker.register_budget(create_test_budget(
        "user2",
        BudgetScope::User("user-2".to_string()),
        50.0,
    ));
    tracker.register_budget(create_test_budget(
        "team1",
        BudgetScope::Team("team-1".to_string()),
        75.0,
    ));
    let user_budgets = tracker.get_budgets_by_type("user");
    assert_eq!(user_budgets.len(), 2);
    let global_budgets = tracker.get_budgets_by_type("global");
    assert_eq!(global_budgets.len(), 1);
    let team_budgets = tracker.get_budgets_by_type("team");
    assert_eq!(team_budgets.len(), 1);
}

#[test]
fn test_spend_result_helpers() {
    let result = SpendResult {
        budget_id: "test".to_string(),
        scope: BudgetScope::Global,
        previous_status: BudgetStatus::Ok,
        new_status: BudgetStatus::Warning,
        current_spend: 80.0,
        max_budget: 100.0,
        remaining: 20.0,
        should_alert_soft_limit: true,
        should_alert_exceeded: false,
    };
    assert!(result.should_alert());
    assert!(result.status_changed());
}

#[test]
fn test_reset_budgets_by_period() {
    let tracker = BudgetTracker::new();
    let mut budget = create_test_budget("test", BudgetScope::Global, 100.0);
    budget.reset_period = ResetPeriod::Never;
    budget.current_spend = 50.0;
    tracker.register_budget(budget);
    let reset_ids = tracker.reset_budgets();
    assert!(reset_ids.is_empty());
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 50.0);
}

#[test]
fn test_concurrent_access() {
    use std::sync::Arc;
    use std::thread;

    let tracker = Arc::new(BudgetTracker::new());
    tracker.register_budget(create_test_budget("test", BudgetScope::Global, 1000.0));
    let mut handles = vec![];
    for _ in 0..10 {
        let tracker_clone = Arc::clone(&tracker);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                tracker_clone.record_spend(&BudgetScope::Global, 1.0);
            }
        });
        handles.push(handle);
    }
    for handle in handles {
        handle.join().unwrap();
    }
    assert_eq!(tracker.get_current_spend(&BudgetScope::Global), 1000.0);
}
