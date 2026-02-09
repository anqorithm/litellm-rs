use super::*;

// ==================== CostCalculator Creation Tests ====================

#[test]
fn test_cost_calculator_new() {
    let calculator = CostCalculator::new();
    // Should have default providers
    let providers = calculator.get_providers();
    assert!(providers.contains(&"openai".to_string()));
    assert!(providers.contains(&"anthropic".to_string()));
}

#[test]
fn test_cost_calculator_default() {
    let calculator = CostCalculator::default();
    let providers = calculator.get_providers();
    assert!(!providers.is_empty());
}

// ==================== Cost Calculation Tests ====================

#[test]
fn test_cost_calculation() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    let cost = calculator
        .calculate_cost("openai", "gpt-3.5-turbo", &token_usage, 1, None, None)
        .unwrap();

    assert!(cost.total_cost > 0.0);
    assert_eq!(cost.currency, "USD");
}

#[test]
fn test_cost_calculation_gpt4() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    let cost = calculator
        .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
        .unwrap();

    // GPT-4 input: 0.00003 * 1000 = 0.03
    // GPT-4 output: 0.00006 * 500 = 0.03
    assert!((cost.input_cost - 0.03).abs() < 1e-10);
    assert!((cost.output_cost - 0.03).abs() < 1e-10);
    assert!((cost.total_cost - 0.06).abs() < 1e-10);
}

#[test]
fn test_cost_calculation_gpt4_turbo() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    let cost = calculator
        .calculate_cost("openai", "gpt-4-turbo", &token_usage, 1, None, None)
        .unwrap();

    // GPT-4-turbo input: 0.00001 * 1000 = 0.01
    // GPT-4-turbo output: 0.00003 * 500 = 0.015
    assert!((cost.input_cost - 0.01).abs() < 1e-10);
    assert!((cost.output_cost - 0.015).abs() < 1e-10);
    assert!((cost.total_cost - 0.025).abs() < 1e-10);
}

#[test]
fn test_cost_calculation_anthropic() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    let cost = calculator
        .calculate_cost("anthropic", "claude-3-opus", &token_usage, 1, None, None)
        .unwrap();

    // Claude-3-opus input: 0.000015 * 1000 = 0.015
    // Claude-3-opus output: 0.000075 * 500 = 0.0375
    assert!((cost.input_cost - 0.015).abs() < 1e-10);
    assert!((cost.output_cost - 0.0375).abs() < 1e-10);
    assert!((cost.total_cost - 0.0525).abs() < 1e-10);
}

#[test]
fn test_cost_calculation_zero_tokens() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(0, 0);

    let cost = calculator
        .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
        .unwrap();

    assert_eq!(cost.input_cost, 0.0);
    assert_eq!(cost.output_cost, 0.0);
    assert_eq!(cost.total_cost, 0.0);
}

#[test]
fn test_cost_calculation_large_tokens() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(100000, 50000);

    let cost = calculator
        .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
        .unwrap();

    // GPT-4 input: 0.00003 * 100000 = 3.0
    // GPT-4 output: 0.00006 * 50000 = 3.0
    assert!((cost.input_cost - 3.0).abs() < 1e-10);
    assert!((cost.output_cost - 3.0).abs() < 1e-10);
    assert!((cost.total_cost - 6.0).abs() < 1e-10);
}

// ==================== Default Model Fallback Tests ====================

#[test]
fn test_cost_calculation_default_model() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    // Use an unknown model that should fall back to default
    let cost = calculator
        .calculate_cost("openai", "unknown-model", &token_usage, 1, None, None)
        .unwrap();

    // Default openai model: input 0.00001, output 0.00002
    assert!((cost.input_cost - 0.01).abs() < 1e-10);
    assert!((cost.output_cost - 0.01).abs() < 1e-10);
    assert!((cost.total_cost - 0.02).abs() < 1e-10);
}

// ==================== Error Cases ====================

#[test]
fn test_unknown_provider() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(100, 50);

    let result = calculator.calculate_cost(
        "unknown_provider",
        "unknown_model",
        &token_usage,
        1,
        None,
        None,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unknown provider"));
}

#[test]
fn test_unknown_model_no_default() {
    // Create a provider without default config
    let mut calculator = CostCalculator::new();
    let config = ProviderCostConfig {
        provider: "test_provider".to_string(),
        models: HashMap::new(),
        default: None,
    };
    calculator.add_provider_config(config);

    let token_usage = TokenUsage::new(100, 50);
    let result = calculator.calculate_cost(
        "test_provider",
        "unknown_model",
        &token_usage,
        1,
        None,
        None,
    );

    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.to_string().contains("Unknown model"));
}

// ==================== Cost Rates Tests ====================

#[test]
fn test_cost_rates() {
    let calculator = CostCalculator::new();
    let rates = calculator.get_cost_rates("openai", "gpt-4").unwrap();

    assert!(rates.input_cost_per_token > 0.0);
    assert!(rates.output_cost_per_token > 0.0);
}

#[test]
fn test_cost_rates_unknown_provider() {
    let calculator = CostCalculator::new();
    let result = calculator.get_cost_rates("unknown_provider", "model");

    assert!(result.is_err());
}

#[test]
fn test_cost_rates_unknown_model_no_default() {
    let mut calculator = CostCalculator::new();
    let config = ProviderCostConfig {
        provider: "test_provider".to_string(),
        models: HashMap::new(),
        default: None,
    };
    calculator.add_provider_config(config);

    let result = calculator.get_cost_rates("test_provider", "unknown_model");
    assert!(result.is_err());
}

#[test]
fn test_cost_rates_default_fallback() {
    let calculator = CostCalculator::new();
    let rates = calculator
        .get_cost_rates("openai", "unknown-model")
        .unwrap();

    // Should fall back to default rates
    assert_eq!(rates.input_cost_per_token, 0.00001);
    assert_eq!(rates.output_cost_per_token, 0.00002);
}

// ==================== Provider/Model Management Tests ====================

#[test]
fn test_get_providers() {
    let calculator = CostCalculator::new();
    let providers = calculator.get_providers();

    assert!(providers.len() >= 2);
    assert!(providers.contains(&"openai".to_string()));
    assert!(providers.contains(&"anthropic".to_string()));
}

#[test]
fn test_get_models() {
    let calculator = CostCalculator::new();

    let openai_models = calculator.get_models("openai");
    assert!(openai_models.contains(&"gpt-4".to_string()));
    assert!(openai_models.contains(&"gpt-4-turbo".to_string()));
    assert!(openai_models.contains(&"gpt-3.5-turbo".to_string()));

    let anthropic_models = calculator.get_models("anthropic");
    assert!(anthropic_models.contains(&"claude-3-opus".to_string()));
    assert!(anthropic_models.contains(&"claude-3-sonnet".to_string()));
}

#[test]
fn test_get_models_unknown_provider() {
    let calculator = CostCalculator::new();
    let models = calculator.get_models("unknown_provider");
    assert!(models.is_empty());
}

#[test]
fn test_add_provider_config() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "test-model".to_string(),
        ModelCostConfig {
            model: "test-model".to_string(),
            input_cost_per_token: 0.001,
            output_cost_per_token: 0.002,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "EUR".to_string(),
            billing_model: BillingModel::PerToken,
        },
    );

    let config = ProviderCostConfig {
        provider: "test_provider".to_string(),
        models,
        default: None,
    };

    calculator.add_provider_config(config);

    let providers = calculator.get_providers();
    assert!(providers.contains(&"test_provider".to_string()));

    let test_models = calculator.get_models("test_provider");
    assert!(test_models.contains(&"test-model".to_string()));
}

// ==================== BillingModel Tests ====================

#[test]
fn test_billing_model_per_request() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "per-request-model".to_string(),
        ModelCostConfig {
            model: "per-request-model".to_string(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
            cost_per_request: Some(0.05),
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::PerRequest,
        },
    );

    calculator.add_provider_config(ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    });

    let token_usage = TokenUsage::new(1000, 500);
    let cost = calculator
        .calculate_cost("test", "per-request-model", &token_usage, 10, None, None)
        .unwrap();

    // 10 requests * 0.05 = 0.50
    assert!((cost.total_cost - 0.50).abs() < 1e-10);
}

#[test]
fn test_billing_model_per_image() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "image-model".to_string(),
        ModelCostConfig {
            model: "image-model".to_string(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
            cost_per_request: None,
            cost_per_image: Some(0.02),
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::PerImage,
        },
    );

    calculator.add_provider_config(ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    });

    let token_usage = TokenUsage::new(0, 0);
    let cost = calculator
        .calculate_cost("test", "image-model", &token_usage, 1, Some(5), None)
        .unwrap();

    // 5 images * 0.02 = 0.10
    assert!((cost.total_cost - 0.10).abs() < 1e-10);
}

#[test]
fn test_billing_model_per_audio_second() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "audio-model".to_string(),
        ModelCostConfig {
            model: "audio-model".to_string(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: Some(0.001),
            currency: "USD".to_string(),
            billing_model: BillingModel::PerAudioSecond,
        },
    );

    calculator.add_provider_config(ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    });

    let token_usage = TokenUsage::new(0, 0);
    let cost = calculator
        .calculate_cost("test", "audio-model", &token_usage, 1, None, Some(60.0))
        .unwrap();

    // 60 seconds * 0.001 = 0.06
    assert!((cost.total_cost - 0.06).abs() < 1e-10);
}

#[test]
fn test_billing_model_flat_rate() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "flat-rate-model".to_string(),
        ModelCostConfig {
            model: "flat-rate-model".to_string(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::FlatRate,
        },
    );

    calculator.add_provider_config(ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    });

    let token_usage = TokenUsage::new(1000, 500);
    let cost = calculator
        .calculate_cost("test", "flat-rate-model", &token_usage, 1, None, None)
        .unwrap();

    // Flat rate means 0 cost at request level
    assert_eq!(cost.total_cost, 0.0);
}

#[test]
fn test_billing_model_free() {
    let mut calculator = CostCalculator::new();

    let mut models = HashMap::new();
    models.insert(
        "free-model".to_string(),
        ModelCostConfig {
            model: "free-model".to_string(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::Free,
        },
    );

    calculator.add_provider_config(ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    });

    let token_usage = TokenUsage::new(1000, 500);
    let cost = calculator
        .calculate_cost("test", "free-model", &token_usage, 1, None, None)
        .unwrap();

    assert_eq!(cost.total_cost, 0.0);
}

// ==================== Utils Tests ====================

#[test]
fn test_utils() {
    let savings = utils::calculate_savings(1.0, 0.8);
    assert!((savings.0 - 0.2).abs() < 1e-10);
    assert!((savings.1 - 20.0).abs() < 1e-10);

    let monthly_cost = utils::estimate_monthly_cost(100, 1000, 500, 0.00001, 0.00002);
    assert!(monthly_cost > 0.0);

    let cost_per_req = utils::cost_per_request(10.0, 100);
    assert_eq!(cost_per_req, 0.1);
}

#[test]
fn test_convert_currency_same() {
    let amount = utils::convert_currency(100.0, "USD", "USD", 1.0);
    assert_eq!(amount, 100.0);
}

#[test]
fn test_convert_currency_different() {
    let amount = utils::convert_currency(100.0, "USD", "EUR", 0.85);
    assert!((amount - 85.0).abs() < 1e-10);
}

#[test]
fn test_calculate_savings_equal() {
    let (savings, percentage) = utils::calculate_savings(1.0, 1.0);
    assert_eq!(savings, 0.0);
    assert_eq!(percentage, 0.0);
}

#[test]
fn test_calculate_savings_reverse() {
    // Order shouldn't matter for absolute savings
    let (savings1, _) = utils::calculate_savings(1.0, 0.5);
    let (savings2, _) = utils::calculate_savings(0.5, 1.0);
    assert_eq!(savings1, savings2);
}

#[test]
fn test_calculate_savings_zero() {
    let (savings, percentage) = utils::calculate_savings(0.0, 0.0);
    assert_eq!(savings, 0.0);
    assert_eq!(percentage, 0.0);
}

#[test]
fn test_estimate_monthly_cost_zero_requests() {
    let cost = utils::estimate_monthly_cost(0, 1000, 500, 0.00001, 0.00002);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_estimate_monthly_cost_calculation() {
    // 100 daily requests, 1000 input tokens, 500 output tokens
    // Daily cost = 100 * (1000 * 0.00001 + 500 * 0.00002) = 100 * (0.01 + 0.01) = 2.0
    // Monthly = 2.0 * 30 = 60.0
    let cost = utils::estimate_monthly_cost(100, 1000, 500, 0.00001, 0.00002);
    assert!((cost - 60.0).abs() < 1e-10);
}

#[test]
fn test_cost_per_request_zero_requests() {
    let cpr = utils::cost_per_request(10.0, 0);
    assert_eq!(cpr, 0.0);
}

#[test]
fn test_cost_per_request_zero_cost() {
    let cpr = utils::cost_per_request(0.0, 100);
    assert_eq!(cpr, 0.0);
}

// ==================== ModelCostConfig Tests ====================

#[test]
fn test_model_cost_config_serialization() {
    let config = ModelCostConfig {
        model: "test-model".to_string(),
        input_cost_per_token: 0.001,
        output_cost_per_token: 0.002,
        cost_per_request: Some(0.05),
        cost_per_image: None,
        cost_per_audio_second: None,
        currency: "USD".to_string(),
        billing_model: BillingModel::PerToken,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ModelCostConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.model, "test-model");
    assert_eq!(deserialized.input_cost_per_token, 0.001);
    assert_eq!(deserialized.cost_per_request, Some(0.05));
}

#[test]
fn test_billing_model_serialization() {
    let models = vec![
        (BillingModel::PerToken, "\"per_token\""),
        (BillingModel::PerRequest, "\"per_request\""),
        (BillingModel::PerImage, "\"per_image\""),
        (BillingModel::PerAudioSecond, "\"per_audio_second\""),
        (BillingModel::FlatRate, "\"flat_rate\""),
        (BillingModel::Free, "\"free\""),
    ];

    for (model, expected) in models {
        let json = serde_json::to_string(&model).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn test_provider_cost_config_serialization() {
    let mut models = HashMap::new();
    models.insert(
        "model1".to_string(),
        ModelCostConfig {
            model: "model1".to_string(),
            input_cost_per_token: 0.001,
            output_cost_per_token: 0.002,
            cost_per_request: None,
            cost_per_image: None,
            cost_per_audio_second: None,
            currency: "USD".to_string(),
            billing_model: BillingModel::PerToken,
        },
    );

    let config = ProviderCostConfig {
        provider: "test".to_string(),
        models,
        default: None,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: ProviderCostConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.provider, "test");
    assert!(deserialized.models.contains_key("model1"));
}

// ==================== CostInfo Tests ====================

#[test]
fn test_cost_info_structure() {
    let calculator = CostCalculator::new();
    let token_usage = TokenUsage::new(1000, 500);

    let cost = calculator
        .calculate_cost("openai", "gpt-4", &token_usage, 1, None, None)
        .unwrap();

    // Verify rates are included in the response
    assert!(cost.rates.input_cost_per_token > 0.0);
    assert!(cost.rates.output_cost_per_token > 0.0);
}
