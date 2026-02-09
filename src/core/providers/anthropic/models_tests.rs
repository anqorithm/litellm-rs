use super::*;

#[test]
fn test_model_registry() {
    let registry = get_anthropic_registry();

    // Test latest flagship model
    let opus_spec = registry.get_model_spec("claude-opus-4-6").unwrap();
    assert_eq!(opus_spec.family, AnthropicModelFamily::ClaudeOpus46);
    assert!(
        opus_spec
            .features
            .contains(&ModelFeature::MultimodalSupport)
    );
    assert!(opus_spec.features.contains(&ModelFeature::ComputerUse));

    // Test pricing
    assert_eq!(opus_spec.pricing.input_price, 5.0);
    assert_eq!(opus_spec.pricing.output_price, 25.0);
}

#[test]
fn test_model_family_detection() {
    assert_eq!(
        AnthropicModelRegistry::from_model_name("claude-opus-4-6"),
        Some(AnthropicModelFamily::ClaudeOpus46)
    );

    assert_eq!(
        AnthropicModelRegistry::from_model_name("claude-3-5-sonnet-20241022"),
        Some(AnthropicModelFamily::Claude35Sonnet)
    );

    assert_eq!(
        AnthropicModelRegistry::from_model_name("claude-3-opus-20240229"),
        Some(AnthropicModelFamily::Claude3Opus)
    );

    assert_eq!(
        AnthropicModelRegistry::from_model_name("unknown-model"),
        None
    );
}

#[test]
fn test_cost_calculation() {
    let cost = CostCalculator::calculate_cost("claude-opus-4-6", 1000, 500);
    assert!(cost.is_some());

    let cost_value = cost.unwrap();
    // Expected: (1000/1M * $5) + (500/1M * $25) = $0.005 + $0.0125 = $0.0175
    assert!((cost_value - 0.0175).abs() < 0.0001);
}

#[test]
fn test_feature_support() {
    let registry = get_anthropic_registry();

    // Claude Opus 4.6 supports computer tools
    assert!(registry.supports_feature("claude-opus-4-6", &ModelFeature::ComputerUse));

    // Claude 2.1 does not support computer tools
    assert!(!registry.supports_feature("claude-2.1", &ModelFeature::ComputerUse));
}
