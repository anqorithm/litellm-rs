//! Comprehensive unit tests for NVIDIA NIM provider

use super::*;
use crate::core::traits::ProviderConfig;

// ==================== Config Tests ====================

#[test]
fn test_nvidia_nim_config_default_values() {
    let config = NvidiaNimConfig::default();
    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 3);
    assert!(!config.debug);
}

#[test]
fn test_nvidia_nim_config_custom_values() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-test-key".to_string()),
        api_base: Some("https://custom.nvidia.com/v1".to_string()),
        timeout: 120,
        max_retries: 5,
        debug: true,
    };

    assert_eq!(config.api_key, Some("nvapi-test-key".to_string()));
    assert_eq!(config.api_base, Some("https://custom.nvidia.com/v1".to_string()));
    assert_eq!(config.timeout, 120);
    assert_eq!(config.max_retries, 5);
    assert!(config.debug);
}

#[test]
fn test_nvidia_nim_config_get_api_base_default() {
    let config = NvidiaNimConfig::default();
    assert_eq!(
        config.get_api_base(),
        "https://integrate.api.nvidia.com/v1"
    );
}

#[test]
fn test_nvidia_nim_config_get_api_base_custom() {
    let config = NvidiaNimConfig {
        api_base: Some("https://my-nim.nvidia.com".to_string()),
        ..Default::default()
    };
    assert_eq!(config.get_api_base(), "https://my-nim.nvidia.com");
}

#[test]
fn test_nvidia_nim_config_validation_with_key() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-valid-key".to_string()),
        ..Default::default()
    };
    assert!(config.validate().is_ok());
}

#[test]
fn test_nvidia_nim_config_validation_zero_timeout() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-key".to_string()),
        timeout: 0,
        ..Default::default()
    };
    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Timeout"));
}

#[test]
fn test_nvidia_nim_config_provider_config_trait() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-test".to_string()),
        api_base: Some("https://custom.com".to_string()),
        timeout: 90,
        max_retries: 4,
        ..Default::default()
    };

    assert_eq!(config.api_key(), Some("nvapi-test"));
    assert_eq!(config.api_base(), Some("https://custom.com"));
    assert_eq!(config.timeout(), std::time::Duration::from_secs(90));
    assert_eq!(config.max_retries(), 4);
}

#[test]
fn test_nvidia_nim_config_clone() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-key".to_string()),
        api_base: Some("https://base.com".to_string()),
        timeout: 60,
        max_retries: 3,
        debug: true,
    };

    let cloned = config.clone();
    assert_eq!(cloned.api_key, config.api_key);
    assert_eq!(cloned.api_base, config.api_base);
    assert_eq!(cloned.timeout, config.timeout);
}

#[test]
fn test_nvidia_nim_config_serialization_roundtrip() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-test-key".to_string()),
        api_base: Some("https://test.com".to_string()),
        timeout: 45,
        max_retries: 2,
        debug: true,
    };

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: NvidiaNimConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.api_key, config.api_key);
    assert_eq!(deserialized.api_base, config.api_base);
    assert_eq!(deserialized.timeout, config.timeout);
    assert_eq!(deserialized.max_retries, config.max_retries);
    assert_eq!(deserialized.debug, config.debug);
}

// Note: Error tests removed - NvidiaNimError is now a type alias to ProviderError

// ==================== Model Info Tests ====================

#[test]
fn test_get_available_models() {
    let models = model_info::get_available_models();
    assert!(!models.is_empty());

    // Check for expected models
    assert!(models.contains(&"meta/llama3-70b-instruct"));
    assert!(models.contains(&"meta/llama3-8b-instruct"));
    assert!(models.contains(&"mistralai/mistral-large"));
    assert!(models.contains(&"nvidia/nemotron-4-340b-instruct"));
    assert!(models.contains(&"google/gemma-2-27b-it"));
}

#[test]
fn test_get_model_info_llama3() {
    let info = model_info::get_model_info("meta/llama3-70b-instruct").unwrap();
    assert_eq!(info.display_name, "Llama 3 70B Instruct");
    assert_eq!(info.max_context_length, 8192);
    assert!(info.supports_streaming);
    assert!(info.supports_tools);
    assert!(!info.supports_multimodal);
}

#[test]
fn test_get_model_info_mistral() {
    let info = model_info::get_model_info("mistralai/mistral-large").unwrap();
    assert_eq!(info.display_name, "Mistral Large");
    assert_eq!(info.max_context_length, 32768);
    assert!(info.supports_streaming);
    assert!(info.supports_tools);
}

#[test]
fn test_get_model_info_phi3() {
    let info = model_info::get_model_info("microsoft/phi-3-small-128k-instruct").unwrap();
    assert_eq!(info.display_name, "Phi-3 Small 128K Instruct");
    assert_eq!(info.max_context_length, 131072);
    assert!(info.supports_streaming);
    assert!(!info.supports_tools);
}

#[test]
fn test_get_model_info_gemma() {
    let info = model_info::get_model_info("google/gemma-2-9b-it").unwrap();
    assert!(info.supports_streaming);
    assert!(!info.supports_tools);
}

#[test]
fn test_get_model_info_nemotron() {
    let info = model_info::get_model_info("nvidia/nemotron-4-340b-instruct").unwrap();
    assert_eq!(info.display_name, "Nemotron 4 340B Instruct");
    assert!(!info.supports_tools);
}

#[test]
fn test_get_model_info_nemotron_reward() {
    let info = model_info::get_model_info("nvidia/nemotron-4-340b-reward").unwrap();
    assert_eq!(info.display_name, "Nemotron 4 340B Reward");
    assert!(info.supports_streaming);
    assert!(!info.supports_tools);
}

#[test]
fn test_get_model_info_unknown_model() {
    let info = model_info::get_model_info("unknown/model");
    assert!(info.is_some()); // Returns default model info
}

#[test]
fn test_get_supported_params_default() {
    let params = model_info::get_supported_params("meta/llama3-70b-instruct");
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"tools"));
    assert!(params.contains(&"tool_choice"));
    assert!(params.contains(&"response_format"));
}

#[test]
fn test_get_supported_params_gemma() {
    let params = model_info::get_supported_params("google/gemma-2-9b-it");
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"seed"));
    assert!(!params.contains(&"tools"));
}

#[test]
fn test_get_supported_params_nemotron_instruct() {
    let params = model_info::get_supported_params("nvidia/nemotron-4-340b-instruct");
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"max_tokens"));
    assert!(!params.contains(&"tools"));
}

#[test]
fn test_get_supported_params_nemotron_reward() {
    let params = model_info::get_supported_params("nvidia/nemotron-4-340b-reward");
    assert!(params.contains(&"stream"));
    assert_eq!(params.len(), 1);
}

#[test]
fn test_get_supported_params_codegemma() {
    let params = model_info::get_supported_params("google/codegemma-1.1-7b");
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"temperature"));
    assert!(!params.contains(&"seed"));
    assert!(!params.contains(&"tools"));
}

#[test]
fn test_supports_tools_function() {
    // Models that support tools
    assert!(model_info::supports_tools("meta/llama3-70b-instruct"));
    assert!(model_info::supports_tools("meta/llama3-8b-instruct"));
    assert!(model_info::supports_tools("mistralai/mistral-large"));
    assert!(model_info::supports_tools("mistralai/mixtral-8x22b-instruct-v0.1"));

    // Models that don't support tools
    assert!(!model_info::supports_tools("google/gemma-2-9b-it"));
    assert!(!model_info::supports_tools("google/recurrentgemma-2b"));
    assert!(!model_info::supports_tools("nvidia/nemotron-4-340b-instruct"));
    assert!(!model_info::supports_tools("nvidia/nemotron-4-340b-reward"));
    assert!(!model_info::supports_tools("google/codegemma-1.1-7b"));
}

#[test]
fn test_get_models_map() {
    let map = model_info::get_models_map();
    assert!(!map.is_empty());
    assert!(map.contains_key("meta/llama3-70b-instruct"));
    assert!(map.contains_key("mistralai/mistral-large"));

    let model = map.get("meta/llama3-70b-instruct").unwrap();
    assert_eq!(model.display_name, "Llama 3 70B Instruct");
}

// ==================== Provider Tests ====================

#[tokio::test]
async fn test_nvidia_nim_provider_creation() {
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-test-key".to_string()),
        ..Default::default()
    };

    let provider = NvidiaNimProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "nvidia_nim");
}

#[tokio::test]
async fn test_nvidia_nim_provider_with_api_key() {
    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "nvidia_nim");
}

#[tokio::test]
async fn test_nvidia_nim_provider_capabilities() {
    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();
    let capabilities = provider.capabilities();

    assert!(capabilities.contains(&crate::core::types::model::ProviderCapability::ChatCompletion));
    assert!(capabilities.contains(&crate::core::types::model::ProviderCapability::ChatCompletionStream));
    assert!(capabilities.contains(&crate::core::types::model::ProviderCapability::ToolCalling));
    assert!(capabilities.contains(&crate::core::types::model::ProviderCapability::Embeddings));
}

#[tokio::test]
async fn test_nvidia_nim_provider_models() {
    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();
    let models = provider.models();

    assert!(!models.is_empty());

    // Check that models have correct provider
    for model in models {
        assert_eq!(model.provider, "nvidia_nim");
    }
}

#[tokio::test]
async fn test_nvidia_nim_provider_get_supported_params() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    // Default model params
    let params = provider.get_supported_openai_params("meta/llama3-70b-instruct");
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"tools"));

    // Gemma params (limited)
    let params = provider.get_supported_openai_params("google/gemma-2-9b-it");
    assert!(params.contains(&"stream"));
    assert!(!params.contains(&"tools"));
}

#[tokio::test]
async fn test_nvidia_nim_provider_map_openai_params() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use std::collections::HashMap;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    let mut params = HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.7));
    params.insert("max_completion_tokens".to_string(), serde_json::json!(1000));
    params.insert("unsupported_param".to_string(), serde_json::json!("value"));

    let result = provider.map_openai_params(params, "meta/llama3-70b-instruct").await;
    assert!(result.is_ok());

    let mapped = result.unwrap();
    assert!(mapped.contains_key("temperature"));
    assert!(mapped.contains_key("max_tokens")); // max_completion_tokens mapped to max_tokens
    assert!(!mapped.contains_key("max_completion_tokens"));
    assert!(!mapped.contains_key("unsupported_param"));
}

#[tokio::test]
async fn test_nvidia_nim_provider_transform_request() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ChatMessage;
    use crate::core::types::message::MessageRole;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    let request = crate::core::types::ChatRequest {
        model: "meta/llama3-70b-instruct".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(crate::core::types::message::MessageContent::Text("Hello".to_string())),
            name: None,
            tool_calls: None,
            tool_call_id: None,
            ..Default::default()
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    let context = crate::core::types::context::RequestContext::default();
    let result = provider.transform_request(request, context).await;
    assert!(result.is_ok());

    let json = result.unwrap();
    assert!(json.get("model").is_some());
    assert!(json.get("messages").is_some());
}

#[tokio::test]
async fn test_nvidia_nim_provider_transform_response() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    let response_json = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1234567890,
        "model": "meta/llama3-70b-instruct",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you today?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 15,
            "total_tokens": 25
        }
    });

    let response_bytes = serde_json::to_vec(&response_json).unwrap();
    let result = provider.transform_response(&response_bytes, "meta/llama3-70b-instruct", "req-123").await;

    assert!(result.is_ok());
    let response = result.unwrap();
    assert_eq!(response.model, "meta/llama3-70b-instruct");
}

#[tokio::test]
async fn test_nvidia_nim_provider_calculate_cost() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    let result = provider.calculate_cost("meta/llama3-70b-instruct", 1000, 500).await;
    assert!(result.is_ok());

    // Cost should be >= 0
    let cost = result.unwrap();
    assert!(cost >= 0.0);
}

#[tokio::test]
async fn test_nvidia_nim_provider_calculate_cost_unknown_model() {
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    let provider = NvidiaNimProvider::with_api_key("nvapi-test-key").await.unwrap();

    // Unknown models should still work (return default pricing)
    let result = provider.calculate_cost("unknown/model", 1000, 500).await;
    assert!(result.is_ok());
}

#[test]
fn test_nvidia_nim_provider_debug_impl() {
    // Create a minimal config for testing Debug trait
    let config = NvidiaNimConfig {
        api_key: Some("nvapi-test-key".to_string()),
        ..Default::default()
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("NvidiaNimConfig"));
}

#[test]
fn test_nvidia_nim_error_debug_impl() {
    let err = NvidiaNimError::ApiError("test error".to_string());
    let debug_str = format!("{:?}", err);
    assert!(debug_str.contains("ApiError"));
    assert!(debug_str.contains("test error"));
}
