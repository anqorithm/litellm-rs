use super::*;

fn create_usage(prompt_tokens: u32, completion_tokens: u32) -> UsageTokens {
    UsageTokens::new(prompt_tokens, completion_tokens)
}

fn assert_approx_eq(actual: f64, expected: f64) {
    assert!(
        (actual - expected).abs() < 1e-10,
        "expected {expected}, got {actual}"
    );
}

#[test]
fn test_generic_cost_per_token_basic() {
    let usage = create_usage(1000, 500);
    let result = generic_cost_per_token("openai/gpt-4o-mini", &usage, "openai");

    assert!(result.is_ok());
    let breakdown = result.unwrap();
    assert_eq!(breakdown.model, "openai/gpt-4o-mini");
    assert_eq!(breakdown.provider, "openai");
    assert_eq!(breakdown.usage.prompt_tokens, 1000);
    assert_eq!(breakdown.usage.completion_tokens, 500);
    assert_approx_eq(breakdown.input_cost, 0.00015);
    assert_approx_eq(breakdown.output_cost, 0.0003);
    assert_approx_eq(breakdown.total_cost, 0.00045);
}

#[test]
fn test_generic_cost_per_token_with_cache() {
    let mut usage = create_usage(2000, 1000);
    usage.cached_tokens = Some(500);

    let result = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(result.is_ok());
    let breakdown = result.unwrap();

    let expected_input = (1500.0 / 1000.0) * 0.0025;
    assert_approx_eq(breakdown.input_cost, expected_input);
}

#[test]
fn test_generic_cost_per_token_with_reasoning() {
    let mut usage = create_usage(1000, 500);
    usage.reasoning_tokens = Some(200);

    let result = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(result.is_ok());

    // Current canonical pricing file has no non-zero reasoning surcharge for this model.
    assert_eq!(result.unwrap().reasoning_cost, 0.0);
}

#[test]
fn test_generic_cost_per_token_requires_qualified_model() {
    let usage = create_usage(1000, 500);
    let result = generic_cost_per_token("unknown-model", &usage, "openai");

    match result.unwrap_err() {
        CostError::ModelNotSupported { model, provider } => {
            assert_eq!(model, "unknown-model");
            assert_eq!(provider, "openai");
        }
        _ => panic!("Expected ModelNotSupported error"),
    }
}

#[test]
fn test_generic_cost_per_token_unsupported_provider() {
    let usage = create_usage(1000, 500);
    let result = generic_cost_per_token("openai/gpt-4o", &usage, "unknown-provider");

    match result.unwrap_err() {
        CostError::ProviderNotSupported { provider } => {
            assert_eq!(provider, "unknown-provider");
        }
        _ => panic!("Expected ProviderNotSupported error"),
    }
}

#[test]
fn test_generic_cost_per_token_provider_mismatch() {
    let usage = create_usage(1000, 500);
    let result = generic_cost_per_token("openai/gpt-4o", &usage, "anthropic");

    match result.unwrap_err() {
        CostError::ModelNotSupported { model, provider } => {
            assert_eq!(model, "openai/gpt-4o");
            assert_eq!(provider, "anthropic");
        }
        _ => panic!("Expected ModelNotSupported error"),
    }
}

#[test]
fn test_get_openai_pricing_gpt4o_mini() {
    let pricing = get_model_pricing("openai/gpt-4o-mini", "openai");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.00015);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.0006);
    assert_eq!(pricing.currency, "USD");
}

#[test]
fn test_get_openai_pricing_gpt4o() {
    let pricing = get_model_pricing("openai/gpt-4o", "openai");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0025);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.01);
}

#[test]
fn test_get_openai_pricing_gpt4_turbo() {
    let pricing = get_model_pricing("openai/gpt-4-turbo", "openai");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.01);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.03);
}

#[test]
fn test_get_openai_pricing_gpt35_turbo() {
    let pricing = get_model_pricing("openai/gpt-3.5-turbo", "openai");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0005);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.0015);
}

#[test]
fn test_get_anthropic_pricing_claude35_sonnet() {
    let pricing = get_model_pricing("anthropic/claude-3.5-sonnet", "anthropic");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.006);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.03);
}

#[test]
fn test_get_anthropic_pricing_claude_opus_45() {
    let pricing = get_model_pricing("anthropic/claude-opus-4.5", "anthropic");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.005);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.025);
}

#[test]
fn test_get_anthropic_pricing_claude_sonnet_45() {
    let pricing = get_model_pricing("anthropic/claude-sonnet-4.5", "anthropic");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.003);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.015);
}

#[test]
fn test_get_anthropic_pricing_claude35_haiku() {
    let pricing = get_model_pricing("anthropic/claude-3.5-haiku", "anthropic");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0008);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.004);
}

#[test]
fn test_get_anthropic_pricing_claude3_haiku() {
    let pricing = get_model_pricing("anthropic/claude-3-haiku", "anthropic");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.00025);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.00125);
}

#[test]
fn test_get_gemini_pricing() {
    let pricing = get_model_pricing("google/gemini-2.5-flash", "gemini");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0003);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.0025);
}

#[test]
fn test_get_gemini_wrong_key_fails() {
    let pricing = get_model_pricing("gemini/gemini-2.5-flash", "gemini");
    match pricing.unwrap_err() {
        CostError::MissingPricing { model } => assert_eq!(model, "gemini/gemini-2.5-flash"),
        _ => panic!("Expected MissingPricing error"),
    }
}

#[test]
fn test_get_deepseek_pricing() {
    let pricing = get_model_pricing("deepseek/deepseek-chat", "deepseek");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0003);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.0012);
}

#[test]
fn test_get_moonshot_pricing_kimi_k2() {
    let pricing = get_model_pricing("moonshotai/kimi-k2", "moonshot");
    assert!(pricing.is_ok());
    let pricing = pricing.unwrap();
    assert_approx_eq(pricing.input_cost_per_1k_tokens, 0.0005);
    assert_approx_eq(pricing.output_cost_per_1k_tokens, 0.0024);
}

#[test]
fn test_get_model_pricing_requires_exact_provider_case() {
    let pricing = get_model_pricing("openai/gpt-4o", "OpenAI");
    match pricing.unwrap_err() {
        CostError::ProviderNotSupported { provider } => assert_eq!(provider, "OpenAI"),
        _ => panic!("Expected ProviderNotSupported error"),
    }
}

#[test]
fn test_get_model_pricing_rejects_legacy_vertex_alias() {
    let pricing = get_model_pricing("google/gemini-2.5-flash", "vertex_ai");
    match pricing.unwrap_err() {
        CostError::ProviderNotSupported { provider } => assert_eq!(provider, "vertex_ai"),
        _ => panic!("Expected ProviderNotSupported error"),
    }
}

#[test]
fn test_get_model_pricing_rejects_legacy_azure_provider() {
    let pricing = get_model_pricing("openai/gpt-4o", "azure");
    match pricing.unwrap_err() {
        CostError::ProviderNotSupported { provider } => assert_eq!(provider, "azure"),
        _ => panic!("Expected ProviderNotSupported error"),
    }
}

#[test]
fn test_calculate_input_cost_no_cache() {
    let usage = create_usage(1000, 500);
    let cost = calculate_input_cost(&usage, 1.0);
    assert_eq!(cost, 1.0);
}

#[test]
fn test_calculate_input_cost_with_cache() {
    let mut usage = create_usage(2000, 500);
    usage.cached_tokens = Some(500);
    let cost = calculate_input_cost(&usage, 1.0);
    assert_eq!(cost, 1.5);
}

#[test]
fn test_calculate_input_cost_all_cached() {
    let mut usage = create_usage(1000, 500);
    usage.cached_tokens = Some(1000);
    let cost = calculate_input_cost(&usage, 1.0);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_input_cost_zero_tokens() {
    let usage = create_usage(0, 500);
    let cost = calculate_input_cost(&usage, 1.0);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_output_cost_basic() {
    let usage = create_usage(1000, 500);
    let cost = calculate_output_cost(&usage, 2.0);
    assert_eq!(cost, 1.0);
}

#[test]
fn test_calculate_output_cost_zero() {
    let usage = create_usage(1000, 0);
    let cost = calculate_output_cost(&usage, 2.0);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_cache_cost() {
    let cost = calculate_cache_cost(1000, 0.5, 0.1);
    assert_eq!(cost, 0.1);
}

#[test]
fn test_calculate_cache_cost_zero_tokens() {
    let cost = calculate_cache_cost(0, 0.5, 0.1);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_audio_cost_with_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        input_cost_per_audio_token: Some(0.001),
        ..Default::default()
    };

    let cost = calculate_audio_cost(&pricing, 1000);
    assert_eq!(cost, 1.0);
}

#[test]
fn test_calculate_audio_cost_no_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        ..Default::default()
    };

    let cost = calculate_audio_cost(&pricing, 1000);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_image_cost_with_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        image_cost_per_token: Some(0.002),
        ..Default::default()
    };

    let cost = calculate_image_cost(&pricing, 500);
    assert_eq!(cost, 1.0);
}

#[test]
fn test_calculate_image_cost_no_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        ..Default::default()
    };

    let cost = calculate_image_cost(&pricing, 500);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_calculate_reasoning_cost_with_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        reasoning_cost_per_token: Some(0.003),
        ..Default::default()
    };

    let cost = calculate_reasoning_cost(&pricing, 300);
    assert_eq!(cost, 0.9);
}

#[test]
fn test_calculate_reasoning_cost_no_pricing() {
    use chrono::Utc;
    let pricing = ModelPricing {
        model: "test".to_string(),
        input_cost_per_1k_tokens: 0.0,
        output_cost_per_1k_tokens: 0.0,
        currency: "USD".to_string(),
        updated_at: Utc::now(),
        ..Default::default()
    };

    let cost = calculate_reasoning_cost(&pricing, 300);
    assert_eq!(cost, 0.0);
}

#[test]
fn test_estimate_cost_basic() {
    let result = estimate_cost("openai/gpt-4o-mini", "openai", 1000, Some(500));
    assert!(result.is_ok());
    let estimate = result.unwrap();

    let expected_input = (1000.0 / 1000.0) * 0.00015;
    let expected_output = (500.0 / 1000.0) * 0.0006;

    assert_approx_eq(estimate.input_cost, expected_input);
    assert_approx_eq(estimate.estimated_output_cost, expected_output);
    assert_approx_eq(estimate.min_cost, expected_input);
    assert_approx_eq(estimate.max_cost, expected_input + expected_output);
    assert_eq!(estimate.currency, "USD");
}

#[test]
fn test_estimate_cost_no_max_output() {
    let result = estimate_cost("openai/gpt-4o", "openai", 1000, None);
    assert!(result.is_ok());
    let estimate = result.unwrap();

    let expected_output = (100.0 / 1000.0) * 0.01;
    assert_approx_eq(estimate.estimated_output_cost, expected_output);
}

#[test]
fn test_estimate_cost_unsupported_model() {
    let result = estimate_cost("unknown-model", "openai", 1000, Some(500));
    assert!(result.is_err());
}

#[test]
fn test_compare_model_costs_single_model() {
    let models = vec![("openai/gpt-4o-mini".to_string(), "openai".to_string())];
    let comparisons = compare_model_costs(&models, 1000, 500);

    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].model, "openai/gpt-4o-mini");
    assert_eq!(comparisons[0].provider, "openai");
    assert!(comparisons[0].total_cost > 0.0);
    assert!(comparisons[0].cost_per_token > 0.0);
    assert!(comparisons[0].efficiency_score > 0.0);
}

#[test]
fn test_compare_model_costs_multiple_models() {
    let models = vec![
        ("openai/gpt-4o".to_string(), "openai".to_string()),
        ("openai/gpt-4o-mini".to_string(), "openai".to_string()),
        (
            "anthropic/claude-3-haiku".to_string(),
            "anthropic".to_string(),
        ),
    ];
    let comparisons = compare_model_costs(&models, 1000, 500);

    assert_eq!(comparisons.len(), 3);

    for i in 1..comparisons.len() {
        assert!(comparisons[i - 1].total_cost <= comparisons[i].total_cost);
    }

    for comparison in &comparisons {
        let expected_efficiency = 1500.0 / comparison.total_cost;
        assert_approx_eq(comparison.efficiency_score, expected_efficiency);
    }
}

#[test]
fn test_compare_model_costs_with_invalid_model() {
    let models = vec![
        ("openai/gpt-4o-mini".to_string(), "openai".to_string()),
        ("invalid-model".to_string(), "openai".to_string()),
        (
            "anthropic/claude-3-haiku".to_string(),
            "anthropic".to_string(),
        ),
    ];
    let comparisons = compare_model_costs(&models, 1000, 500);

    assert_eq!(comparisons.len(), 2);
}

#[test]
fn test_compare_model_costs_empty_list() {
    let models: Vec<(String, String)> = vec![];
    let comparisons = compare_model_costs(&models, 1000, 500);
    assert_eq!(comparisons.len(), 0);
}

#[test]
fn test_compare_model_costs_zero_tokens() {
    let models = vec![("openai/gpt-4o-mini".to_string(), "openai".to_string())];
    let comparisons = compare_model_costs(&models, 0, 0);

    assert_eq!(comparisons.len(), 1);
    assert_eq!(comparisons[0].total_cost, 0.0);
}

#[test]
fn test_generic_cost_per_token_all_features() {
    let mut usage = create_usage(5000, 2000);
    usage.cached_tokens = Some(1000);
    usage.audio_tokens = Some(500);
    usage.image_tokens = Some(300);
    usage.reasoning_tokens = Some(200);

    let result = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(result.is_ok());
    let breakdown = result.unwrap();

    let calculated_total = breakdown.input_cost
        + breakdown.output_cost
        + breakdown.cache_cost
        + breakdown.audio_cost
        + breakdown.image_cost
        + breakdown.reasoning_cost;

    assert!((breakdown.total_cost - calculated_total).abs() < 1e-10);
}

#[test]
fn test_large_token_counts() {
    let usage = create_usage(1_000_000, 500_000);
    let result = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(result.is_ok());
    let breakdown = result.unwrap();
    assert!(breakdown.total_cost > 0.0);
    assert!(breakdown.total_cost < 1_000_000.0);
}

#[test]
fn test_case_insensitive_model_names_are_rejected() {
    let usage = create_usage(1000, 500);
    let result = generic_cost_per_token("openai/GPT-4O-MINI", &usage, "openai");

    match result.unwrap_err() {
        CostError::MissingPricing { model } => assert_eq!(model, "openai/GPT-4O-MINI"),
        _ => panic!("Expected MissingPricing error"),
    }
}

#[test]
fn test_case_insensitive_provider_names_are_rejected() {
    let result = get_model_pricing("openai/gpt-4o", "OPENAI");
    match result.unwrap_err() {
        CostError::ProviderNotSupported { provider } => assert_eq!(provider, "OPENAI"),
        _ => panic!("Expected ProviderNotSupported error"),
    }
}

#[test]
fn test_cached_tokens_exceed_prompt_tokens() {
    let mut usage = create_usage(1000, 500);
    usage.cached_tokens = Some(1500);

    let result = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(result.is_ok());

    let breakdown = result.unwrap();
    assert_eq!(breakdown.input_cost, 0.0);
}

#[test]
fn test_cost_calculation_workflow() {
    let usage = create_usage(2000, 1000);

    let pricing = get_model_pricing("openai/gpt-4o-mini", "openai");
    assert!(pricing.is_ok());

    let breakdown = generic_cost_per_token("openai/gpt-4o-mini", &usage, "openai");
    assert!(breakdown.is_ok());
    let breakdown = breakdown.unwrap();

    assert_eq!(breakdown.model, "openai/gpt-4o-mini");
    assert_eq!(breakdown.provider, "openai");
    assert_eq!(breakdown.currency, "USD");
    assert!(breakdown.total_cost > 0.0);
    assert_eq!(breakdown.usage.total_tokens, 3000);
}

#[test]
fn test_estimate_and_actual_cost_consistency() {
    let input_tokens = 1000;
    let output_tokens = 500;

    let estimate = estimate_cost("openai/gpt-4o", "openai", input_tokens, Some(output_tokens));
    assert!(estimate.is_ok());
    let estimate = estimate.unwrap();

    let usage = create_usage(input_tokens, output_tokens);
    let breakdown = generic_cost_per_token("openai/gpt-4o", &usage, "openai");
    assert!(breakdown.is_ok());
    let breakdown = breakdown.unwrap();

    assert_approx_eq(breakdown.total_cost, estimate.max_cost);
    assert_approx_eq(breakdown.input_cost, estimate.input_cost);
}
