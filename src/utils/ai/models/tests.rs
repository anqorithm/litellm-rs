#[cfg(test)]
use super::utils::ModelUtils;

#[test]
fn test_model_capabilities() {
    let caps = ModelUtils::get_model_capabilities("gpt-4");
    assert!(caps.supports_function_calling);
    assert!(caps.supports_parallel_function_calling);

    let caps_35 = ModelUtils::get_model_capabilities("gpt-3.5-turbo");
    assert!(caps_35.supports_function_calling);
    assert!(!caps_35.supports_parallel_function_calling);

    let caps_claude = ModelUtils::get_model_capabilities("claude-3-opus");
    assert!(caps_claude.supports_function_calling);
    assert!(caps_claude.supports_vision);
}

#[test]
fn test_provider_detection() {
    assert_eq!(
        ModelUtils::get_provider_from_model("gpt-4"),
        Some("openai".to_string())
    );
    assert_eq!(
        ModelUtils::get_provider_from_model("claude-3-opus"),
        Some("anthropic".to_string())
    );
    assert_eq!(
        ModelUtils::get_provider_from_model("gemini-pro"),
        Some("google".to_string())
    );
    assert_eq!(ModelUtils::get_provider_from_model("unknown-model"), None);
}

#[test]
fn test_base_model_extraction() {
    assert_eq!(ModelUtils::get_base_model("gpt-4-0314"), "gpt-4");
    assert_eq!(
        ModelUtils::get_base_model("gpt-4-turbo-preview"),
        "gpt-4-turbo"
    );
    assert_eq!(
        ModelUtils::get_base_model("claude-3-opus-20240229"),
        "claude-3-opus"
    );
}

#[test]
fn test_model_validation() {
    assert!(ModelUtils::is_valid_model("gpt-4"));
    assert!(ModelUtils::is_valid_model("claude-3-opus"));
    assert!(ModelUtils::is_valid_model("gemini-pro"));
    assert!(!ModelUtils::is_valid_model("unknown-model-xyz"));
}

#[test]
fn test_model_family() {
    assert_eq!(ModelUtils::get_model_family("gpt-4-turbo"), "gpt");
    assert_eq!(ModelUtils::get_model_family("claude-3-opus"), "claude");
    assert_eq!(ModelUtils::get_model_family("gemini-pro"), "gemini");
}

#[test]
fn test_model_pricing() {
    let pricing = ModelUtils::get_model_pricing("gpt-4");
    assert!(pricing.is_some());
    assert_eq!(pricing.unwrap(), (0.03, 0.06));

    assert!(ModelUtils::get_model_pricing("unknown-model").is_none());
}

#[test]
fn test_compatible_models() {
    let openai_models = ModelUtils::get_compatible_models_for_provider("openai");
    assert!(openai_models.contains(&"gpt-4".to_string()));

    let anthropic_models = ModelUtils::get_compatible_models_for_provider("anthropic");
    assert!(anthropic_models.contains(&"claude-3-opus".to_string()));

    let unknown_models = ModelUtils::get_compatible_models_for_provider("unknown");
    assert!(unknown_models.is_empty());
}

#[test]
fn test_model_type_detection() {
    assert!(ModelUtils::is_chat_model("gpt-4"));
    assert!(ModelUtils::is_chat_model("claude-3-opus"));
    assert!(ModelUtils::is_completion_model("text-davinci-003"));
    assert!(!ModelUtils::is_completion_model("gpt-4"));
}

#[test]
fn test_recommended_temperature() {
    assert_eq!(ModelUtils::get_recommended_temperature("gpt-4"), 0.7);
    assert_eq!(
        ModelUtils::get_recommended_temperature("claude-3-opus"),
        0.9
    );
    assert_eq!(ModelUtils::get_recommended_temperature("gemini-pro"), 0.8);
}
