    use super::*;

    #[test]
    fn test_budget_creation() {
        let budget = Budget::new("budget-1", "Test Budget", BudgetScope::Global, 100.0);

        assert_eq!(budget.id, "budget-1");
        assert_eq!(budget.name, "Test Budget");
        assert_eq!(budget.max_budget, 100.0);
        assert_eq!(budget.soft_limit, 80.0);
        assert_eq!(budget.current_spend, 0.0);
        assert!(budget.enabled);
    }

    #[test]
    fn test_budget_status() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);

        assert_eq!(budget.status(), BudgetStatus::Ok);

        budget.current_spend = 79.0;
        assert_eq!(budget.status(), BudgetStatus::Ok);

        budget.current_spend = 80.0;
        assert_eq!(budget.status(), BudgetStatus::Warning);

        budget.current_spend = 100.0;
        assert_eq!(budget.status(), BudgetStatus::Exceeded);

        budget.current_spend = 150.0;
        assert_eq!(budget.status(), BudgetStatus::Exceeded);
    }

    #[test]
    fn test_budget_remaining() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);

        assert_eq!(budget.remaining(), 100.0);

        budget.current_spend = 30.0;
        assert_eq!(budget.remaining(), 70.0);

        budget.current_spend = 100.0;
        assert_eq!(budget.remaining(), 0.0);

        budget.current_spend = 150.0;
        assert_eq!(budget.remaining(), 0.0);
    }

    #[test]
    fn test_budget_usage_percentage() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);

        assert_eq!(budget.usage_percentage(), 0.0);

        budget.current_spend = 50.0;
        assert!((budget.usage_percentage() - 50.0).abs() < f64::EPSILON);

        budget.current_spend = 100.0;
        assert!((budget.usage_percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_budget_can_spend() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);

        assert!(budget.can_spend(50.0));
        assert!(budget.can_spend(100.0));
        assert!(!budget.can_spend(101.0));

        budget.current_spend = 90.0;
        assert!(budget.can_spend(10.0));
        assert!(!budget.can_spend(11.0));
    }

    #[test]
    fn test_budget_record_spend() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 25.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 50.0);
    }

    #[test]
    fn test_budget_reset() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);
        budget.current_spend = 75.0;

        budget.reset();
        assert_eq!(budget.current_spend, 0.0);
        assert!(budget.last_reset_at.is_some());
    }

    #[test]
    fn test_budget_scope_display() {
        assert_eq!(
            BudgetScope::User("user-1".to_string()).to_string(),
            "user:user-1"
        );
        assert_eq!(
            BudgetScope::Team("team-1".to_string()).to_string(),
            "team:team-1"
        );
        assert_eq!(
            BudgetScope::ApiKey("key-1".to_string()).to_string(),
            "api_key:key-1"
        );
        assert_eq!(
            BudgetScope::Provider("openai".to_string()).to_string(),
            "provider:openai"
        );
        assert_eq!(
            BudgetScope::Model("gpt-4".to_string()).to_string(),
            "model:gpt-4"
        );
        assert_eq!(BudgetScope::Global.to_string(), "global");
    }

    #[test]
    fn test_budget_scope_from_key() {
        assert_eq!(
            BudgetScope::from_key("user:user-1"),
            Some(BudgetScope::User("user-1".to_string()))
        );
        assert_eq!(BudgetScope::from_key("global"), Some(BudgetScope::Global));
        assert_eq!(BudgetScope::from_key("invalid"), None);
    }

    #[test]
    fn test_budget_alert_creation() {
        let budget = Budget::new("budget-1", "Test Budget", BudgetScope::Global, 100.0);
        let alert = BudgetAlert::new(&budget, BudgetAlertType::SoftLimitReached, 80.0);

        assert_eq!(alert.budget_id, "budget-1");
        assert_eq!(alert.alert_type, BudgetAlertType::SoftLimitReached);
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert!(!alert.acknowledged);
    }

    #[test]
    fn test_budget_alert_acknowledge() {
        let budget = Budget::new("budget-1", "Test Budget", BudgetScope::Global, 100.0);
        let mut alert = BudgetAlert::new(&budget, BudgetAlertType::BudgetExceeded, 100.0);

        assert!(!alert.acknowledged);
        alert.acknowledge();
        assert!(alert.acknowledged);
        assert!(alert.acknowledged_at.is_some());
    }

    #[test]
    fn test_budget_check_result() {
        let budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);
        let result = BudgetCheckResult::from_budget(&budget, 10.0);

        assert!(result.allowed);
        assert_eq!(result.status, BudgetStatus::Ok);
        assert_eq!(result.max_budget, 100.0);
    }

    #[test]
    fn test_budget_check_result_no_budget() {
        let result = BudgetCheckResult::no_budget();

        assert!(result.allowed);
        assert_eq!(result.status, BudgetStatus::Ok);
        assert!(result.max_budget.is_infinite());
    }

    #[test]
    fn test_budget_config() {
        let config = BudgetConfig::new("Test Budget", 100.0)
            .with_soft_limit(75.0)
            .with_reset_period(ResetPeriod::Weekly)
            .with_currency(Currency::EUR);

        assert_eq!(config.name, "Test Budget");
        assert_eq!(config.max_budget, 100.0);
        assert_eq!(config.soft_limit, Some(75.0));
        assert_eq!(config.reset_period, Some(ResetPeriod::Weekly));
        assert_eq!(config.currency, Some(Currency::EUR));
    }

    #[test]
    fn test_reset_period_display() {
        assert_eq!(ResetPeriod::Daily.to_string(), "daily");
        assert_eq!(ResetPeriod::Weekly.to_string(), "weekly");
        assert_eq!(ResetPeriod::Monthly.to_string(), "monthly");
        assert_eq!(ResetPeriod::Never.to_string(), "never");
    }

    #[test]
    fn test_currency_display() {
        assert_eq!(Currency::USD.to_string(), "USD");
        assert_eq!(Currency::EUR.to_string(), "EUR");
        assert_eq!(Currency::GBP.to_string(), "GBP");
    }

    #[test]
    fn test_budget_status_display() {
        assert_eq!(BudgetStatus::Ok.to_string(), "ok");
        assert_eq!(BudgetStatus::Warning.to_string(), "warning");
        assert_eq!(BudgetStatus::Exceeded.to_string(), "exceeded");
    }

    #[test]
    fn test_budget_serialization() {
        let budget = Budget::new(
            "test",
            "Test",
            BudgetScope::User("user-1".to_string()),
            100.0,
        );
        let json = serde_json::to_value(&budget).unwrap();

        assert_eq!(json["id"], "test");
        assert_eq!(json["name"], "Test");
        assert_eq!(json["max_budget"], 100.0);
    }

    #[test]
    fn test_budget_scope_serialization() {
        let scope = BudgetScope::User("user-123".to_string());
        let json = serde_json::to_value(&scope).unwrap();

        assert_eq!(json["type"], "User");
        assert_eq!(json["id"], "user-123");
    }

    #[test]
    fn test_disabled_budget_allows_spend() {
        let mut budget = Budget::new("test", "Test", BudgetScope::Global, 100.0);
        budget.enabled = false;
        budget.current_spend = 150.0;

        // Disabled budget should allow any spend
        assert!(budget.can_spend(1000.0));
    }

    // Tests for ProviderBudget
    #[test]
    fn test_provider_budget_creation() {
        let budget = ProviderBudget::new("openai", 1000.0);

        assert_eq!(budget.provider_name, "openai");
        assert_eq!(budget.max_budget, 1000.0);
        assert_eq!(budget.soft_limit, 800.0);
        assert_eq!(budget.current_spend, 0.0);
        assert!(budget.enabled);
    }

    #[test]
    fn test_provider_budget_status() {
        let mut budget = ProviderBudget::new("openai", 100.0);

        assert_eq!(budget.status(), BudgetStatus::Ok);

        budget.current_spend = 79.0;
        assert_eq!(budget.status(), BudgetStatus::Ok);

        budget.current_spend = 80.0;
        assert_eq!(budget.status(), BudgetStatus::Warning);

        budget.current_spend = 100.0;
        assert_eq!(budget.status(), BudgetStatus::Exceeded);
    }

    #[test]
    fn test_provider_budget_can_spend() {
        let mut budget = ProviderBudget::new("openai", 100.0);

        assert!(budget.can_spend(50.0));
        assert!(budget.can_spend(100.0));
        assert!(!budget.can_spend(101.0));

        budget.current_spend = 90.0;
        assert!(budget.can_spend(10.0));
        assert!(!budget.can_spend(11.0));
    }

    #[test]
    fn test_provider_budget_record_spend() {
        let mut budget = ProviderBudget::new("openai", 100.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 25.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 50.0);
    }

    #[test]
    fn test_provider_budget_reset() {
        let mut budget = ProviderBudget::new("openai", 100.0);
        budget.current_spend = 75.0;

        budget.reset();
        assert_eq!(budget.current_spend, 0.0);
        assert!(budget.last_reset_at.is_some());
    }

    #[test]
    fn test_provider_budget_remaining() {
        let mut budget = ProviderBudget::new("openai", 100.0);

        assert_eq!(budget.remaining(), 100.0);

        budget.current_spend = 30.0;
        assert_eq!(budget.remaining(), 70.0);

        budget.current_spend = 150.0;
        assert_eq!(budget.remaining(), 0.0);
    }

    #[test]
    fn test_provider_budget_usage_percentage() {
        let mut budget = ProviderBudget::new("openai", 100.0);

        assert_eq!(budget.usage_percentage(), 0.0);

        budget.current_spend = 50.0;
        assert!((budget.usage_percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_provider_budget_disabled() {
        let mut budget = ProviderBudget::new("openai", 100.0);
        budget.enabled = false;
        budget.current_spend = 150.0;

        assert!(budget.can_spend(1000.0));
    }

    // Tests for ModelBudget
    #[test]
    fn test_model_budget_creation() {
        let budget = ModelBudget::new("gpt-4", 500.0);

        assert_eq!(budget.model_name, "gpt-4");
        assert_eq!(budget.max_budget, 500.0);
        assert_eq!(budget.soft_limit, 400.0);
        assert_eq!(budget.current_spend, 0.0);
        assert!(budget.enabled);
    }

    #[test]
    fn test_model_budget_status() {
        let mut budget = ModelBudget::new("gpt-4", 100.0);

        assert_eq!(budget.status(), BudgetStatus::Ok);

        budget.current_spend = 80.0;
        assert_eq!(budget.status(), BudgetStatus::Warning);

        budget.current_spend = 100.0;
        assert_eq!(budget.status(), BudgetStatus::Exceeded);
    }

    #[test]
    fn test_model_budget_can_spend() {
        let mut budget = ModelBudget::new("gpt-4", 100.0);

        assert!(budget.can_spend(50.0));
        assert!(!budget.can_spend(101.0));

        budget.current_spend = 90.0;
        assert!(budget.can_spend(10.0));
        assert!(!budget.can_spend(11.0));
    }

    #[test]
    fn test_model_budget_record_spend() {
        let mut budget = ModelBudget::new("gpt-4", 100.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 25.0);

        budget.record_spend(25.0);
        assert_eq!(budget.current_spend, 50.0);
    }

    #[test]
    fn test_model_budget_reset() {
        let mut budget = ModelBudget::new("gpt-4", 100.0);
        budget.current_spend = 75.0;

        budget.reset();
        assert_eq!(budget.current_spend, 0.0);
    }

    #[test]
    fn test_model_budget_remaining() {
        let mut budget = ModelBudget::new("gpt-4", 100.0);

        assert_eq!(budget.remaining(), 100.0);

        budget.current_spend = 30.0;
        assert_eq!(budget.remaining(), 70.0);
    }

    #[test]
    fn test_provider_budget_serialization() {
        let budget = ProviderBudget::new("openai", 1000.0);
        let json = serde_json::to_value(&budget).unwrap();

        assert_eq!(json["provider_name"], "openai");
        assert_eq!(json["max_budget"], 1000.0);
    }

    #[test]
    fn test_model_budget_serialization() {
        let budget = ModelBudget::new("gpt-4", 500.0);
        let json = serde_json::to_value(&budget).unwrap();

        assert_eq!(json["model_name"], "gpt-4");
        assert_eq!(json["max_budget"], 500.0);
    }
