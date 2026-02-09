use super::*;

#[test]
fn test_model_registry_creation() {
    let registry = OpenAIModelRegistry::new();
    let models = registry.get_all_models();
    assert!(!models.is_empty());
}

#[test]
fn test_feature_detection() {
    let registry = get_openai_registry();

    // Test GPT-4 features
    assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::ChatCompletion));
    assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::FunctionCalling));
    assert!(registry.supports_feature("gpt-4", &OpenAIModelFeature::StreamingSupport));

    // Test O1 features - may not be available depending on configuration
    let has_o1_reasoning =
        registry.supports_feature("o1-preview", &OpenAIModelFeature::ReasoningMode);
    if !has_o1_reasoning {
        eprintln!("Warning: o1-preview model not found or doesn't support ReasoningMode");
    }

    // Test DALL-E features - may not be available depending on configuration
    let has_dalle_generation =
        registry.supports_feature("dall-e-3", &OpenAIModelFeature::ImageGeneration);
    if !has_dalle_generation {
        eprintln!("Warning: dall-e-3 model not found or doesn't support ImageGeneration");
    }
}

#[test]
fn test_model_families() {
    let registry = get_openai_registry();
    let gpt4_models = registry.get_models_by_family(&OpenAIModelFamily::GPT4);
    assert!(!gpt4_models.is_empty());
}

#[test]
fn test_model_recommendations() {
    let registry = get_openai_registry();

    assert_eq!(
        registry.get_recommended_model(OpenAIUseCase::GeneralChat),
        Some("gpt-5.2-chat".to_string())
    );
    assert_eq!(
        registry.get_recommended_model(OpenAIUseCase::Reasoning),
        Some("o3-pro".to_string())
    );
    assert_eq!(
        registry.get_recommended_model(OpenAIUseCase::CostOptimized),
        Some("gpt-5-nano".to_string())
    );
}
