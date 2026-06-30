use super::*;

fn model_info_from_json(value: serde_json::Value) -> crate::core::pricing::LiteLLMModelInfo {
    serde_json::from_value(value).expect("valid LiteLLMModelInfo json")
}

#[test]
fn test_litellm_pricing_errors_when_both_token_costs_missing() {
    // A catalog entry with neither input nor output cost must not bill at $0.
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "mode": "chat"
    }));
    let result = litellm_to_cost_pricing("mystery-model", &info);
    assert!(matches!(
        result,
        Err(CostError::MissingPricing { ref model }) if model == "mystery-model"
    ));
}

#[test]
fn test_litellm_pricing_errors_when_chat_has_single_missing_side() {
    // Chat completions use prompt and completion tokens, so a missing side
    // must not be billed at $0.
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "mode": "chat",
        "input_cost_per_token": 0.000_01
    }));
    let result = litellm_to_cost_pricing("half-priced-chat", &info);
    assert!(matches!(
        result,
        Err(CostError::MissingPricing { ref model }) if model == "half-priced-chat"
    ));
}

#[test]
fn test_litellm_pricing_errors_when_missing_mode_has_single_missing_side() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "input_cost_per_token": 0.000_01
    }));
    let result = litellm_to_cost_pricing("half-priced-missing-mode", &info);
    assert!(matches!(
        result,
        Err(CostError::MissingPricing { ref model }) if model == "half-priced-missing-mode"
    ));
}

#[test]
fn test_litellm_pricing_allows_blank_mode_non_token_pricing() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "replicate",
        "cost_per_second": 0.001
    }));
    let pricing = match litellm_to_cost_pricing("time-priced-missing-mode", &info) {
        Ok(pricing) => pricing,
        Err(err) => panic!("blank-mode non-token pricing should be accepted: {err}"),
    };
    assert_eq!(pricing.cost_per_second, Some(0.001));
}

#[test]
fn test_litellm_pricing_allows_missing_mode_with_non_token_pricing() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "cost_per_second": 0.000_01
    }));
    let pricing = match litellm_to_cost_pricing("time-priced-missing-mode", &info) {
        Ok(pricing) => pricing,
        Err(err) => panic!("non-token pricing should not require token prices: {err}"),
    };
    assert_eq!(pricing.cost_per_second, Some(0.000_01));
}

#[test]
fn test_litellm_pricing_allows_flat_output_image_pricing_without_token_prices() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "bedrock",
        "mode": "image_generation",
        "output_cost_per_image": 0.06
    }));

    let pricing = match litellm_to_cost_pricing("flat-image-model", &info) {
        Ok(pricing) => pricing,
        Err(err) => panic!("flat image pricing should not require token prices: {err}"),
    };

    assert_eq!(pricing.input_cost_per_1k_tokens, 0.0);
    assert_eq!(pricing.output_cost_per_1k_tokens, 0.0);
}

#[test]
fn test_litellm_pricing_maps_flat_output_image_pricing_to_legacy_cost_per_image() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "bedrock",
        "mode": "image_generation",
        "output_cost_per_image": 0.06
    }));

    let pricing = litellm_to_cost_pricing("flat-image-model", &info)
        .expect("flat image pricing should convert");

    assert_eq!(
        pricing
            .cost_per_image
            .as_ref()
            .and_then(|prices| prices.get("base")),
        Some(&0.06)
    );
}

#[test]
fn test_litellm_pricing_rejects_negative_flat_output_image_pricing() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "bedrock",
        "mode": "image_generation",
        "output_cost_per_image": -0.06
    }));

    let result = litellm_to_cost_pricing("negative-flat-image-model", &info);

    assert!(matches!(result, Err(CostError::InvalidUsage { .. })));
}

#[test]
fn test_litellm_pricing_allows_single_missing_side_for_embedding() {
    // Embeddings can have only input-side token pricing.
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "mode": "embedding",
        "input_cost_per_token": 0.000_01
    }));
    let pricing = litellm_to_cost_pricing("half-priced", &info).expect("should be priced");
    assert!(pricing.input_cost_per_1k_tokens > 0.0);
    assert_eq!(pricing.output_cost_per_1k_tokens, 0.0);
}

#[test]
fn test_litellm_pricing_ok_when_both_present() {
    let info = model_info_from_json(serde_json::json!({
        "litellm_provider": "openai",
        "mode": "chat",
        "input_cost_per_token": 0.000_01,
        "output_cost_per_token": 0.000_03
    }));
    let pricing = litellm_to_cost_pricing("full", &info).expect("should be priced");
    assert!(pricing.input_cost_per_1k_tokens > 0.0);
    assert!(pricing.output_cost_per_1k_tokens > 0.0);
}
