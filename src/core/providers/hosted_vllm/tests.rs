//! Tests for Hosted vLLM Provider

use super::*;

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = HostedVLLMConfig::default();
        assert!(config.api_base.is_none());
        assert!(config.api_key.is_none());
        assert!(config.model.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(config.skip_model_validation);
    }

    #[test]
    fn test_config_new() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1");
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
    }

    #[test]
    fn test_config_with_credentials() {
        let config = HostedVLLMConfig::with_credentials(
            "http://localhost:8000/v1",
            Some("test-key".to_string()),
        );
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_config_builder_chain() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1")
            .with_model("meta-llama/Llama-3.1-8B-Instruct")
            .with_api_key("my-api-key")
            .with_timeout(60)
            .with_max_retries(5)
            .with_debug(true)
            .with_skip_model_validation(false)
            .with_header("X-Custom", "value");

        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(
            config.model,
            Some("meta-llama/Llama-3.1-8B-Instruct".to_string())
        );
        assert_eq!(config.api_key, Some("my-api-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 5);
        assert!(config.debug);
        assert!(!config.skip_model_validation);
        assert_eq!(
            config.custom_headers.get("X-Custom"),
            Some(&"value".to_string())
        );
    }
}

#[cfg(test)]
mod model_tests {
    use super::models::*;

    #[test]
    fn test_known_model_lookup() {
        let info = get_model_info("meta-llama/Meta-Llama-3.1-8B-Instruct");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.model_id, "meta-llama/Meta-Llama-3.1-8B-Instruct");
        assert!(info.supports_tools);
        assert_eq!(info.family, "llama");
    }

    #[test]
    fn test_unknown_model_fallback() {
        let info = get_or_create_model_info("unknown-custom-model");
        assert_eq!(info.model_id, "unknown-custom-model");
        assert_eq!(info.family, "custom");
        assert_eq!(info.context_length, 4096);
    }

    #[test]
    fn test_model_families() {
        let llama_models = get_models_by_family("llama");
        assert!(!llama_models.is_empty());

        let mistral_models = get_models_by_family("mistral");
        assert!(!mistral_models.is_empty());

        let qwen_models = get_models_by_family("qwen");
        assert!(!qwen_models.is_empty());
    }

    #[test]
    fn test_tool_capable_models() {
        let tool_models = get_tool_capable_models();
        assert!(!tool_models.is_empty());

        for model_id in tool_models {
            let info = get_model_info(model_id).unwrap();
            assert!(
                info.supports_tools,
                "Model {} should support tools",
                model_id
            );
        }
    }

    #[test]
    fn test_model_info_builder() {
        let info = HostedVLLMModelInfo::new("custom-model", "Custom Model", 8192)
            .with_tools(true)
            .with_vision(true)
            .with_family("custom-family")
            .with_max_output(4096);

        assert_eq!(info.model_id, "custom-model");
        assert_eq!(info.display_name, "Custom Model");
        assert_eq!(info.context_length, 8192);
        assert_eq!(info.max_output_tokens, 4096);
        assert!(info.supports_tools);
        assert!(info.supports_vision);
        assert_eq!(info.family, "custom-family");
    }
}

#[cfg(test)]
mod provider_tests {
    use super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    #[tokio::test]
    async fn test_provider_creation_requires_api_base() {
        let config = HostedVLLMConfig::default();
        let result = HostedVLLMProvider::new(config).await;

        // Should fail validation without API base
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(matches!(
            err,
            crate::core::providers::unified_provider::ProviderError::Configuration { .. }
        ));
    }

    #[tokio::test]
    async fn test_provider_creation_with_valid_config() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1")
            .with_model("meta-llama/Llama-3.1-8B-Instruct");

        let result = HostedVLLMProvider::new(config).await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.name(), "hosted_vllm");
        assert!(!provider.models().is_empty());
    }

    #[tokio::test]
    async fn test_provider_with_api_base_helper() {
        let result = HostedVLLMProvider::with_api_base("http://localhost:8000/v1").await;
        assert!(result.is_ok());

        let provider = result.unwrap();
        assert_eq!(provider.name(), "hosted_vllm");
    }

    #[tokio::test]
    async fn test_provider_with_credentials_helper() {
        let result = HostedVLLMProvider::with_credentials(
            "http://localhost:8000/v1",
            Some("test-key".to_string()),
        )
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_provider_capabilities() {
        let provider = HostedVLLMProvider::with_api_base("http://localhost:8000/v1")
            .await
            .unwrap();

        let capabilities = provider.capabilities();
        assert!(
            capabilities.contains(&crate::core::types::model::ProviderCapability::ChatCompletion)
        );
        assert!(
            capabilities
                .contains(&crate::core::types::model::ProviderCapability::ChatCompletionStream)
        );
        assert!(capabilities.contains(&crate::core::types::model::ProviderCapability::ToolCalling));
    }

    #[tokio::test]
    async fn test_provider_supported_params() {
        let provider = HostedVLLMProvider::with_api_base("http://localhost:8000/v1")
            .await
            .unwrap();

        let params = provider.get_supported_openai_params("any-model");

        // Check standard OpenAI params
        assert!(params.contains(&"messages"));
        assert!(params.contains(&"model"));
        assert!(params.contains(&"temperature"));
        assert!(params.contains(&"stream"));
        assert!(params.contains(&"tools"));

        // Check vLLM-specific params
        assert!(params.contains(&"top_k"));
        assert!(params.contains(&"min_p"));
        assert!(params.contains(&"repetition_penalty"));
    }

    #[tokio::test]
    async fn test_provider_cost_calculation() {
        let provider = HostedVLLMProvider::with_api_base("http://localhost:8000/v1")
            .await
            .unwrap();

        // Self-hosted should always return 0 cost
        let cost = provider
            .calculate_cost("any-model", 1000, 500)
            .await
            .unwrap();
        assert_eq!(cost, 0.0);
    }
}

#[cfg(test)]
mod streaming_tests {
    use super::streaming::*;
    use crate::core::types::responses::{ChatChoice, ChatResponse, FinishReason, Usage};
    use crate::core::types::{ChatMessage, message::MessageContent, message::MessageRole};

    fn create_test_response() -> ChatResponse {
        ChatResponse {
            id: "test-id".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "test-model".to_string(),
            system_fingerprint: Some("fp_test".to_string()),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text(
                        "Hello world this is a test response with enough words to create multiple chunks"
                            .to_string(),
                    )),
                    name: None,
                    tool_calls: None,
                    function_call: None,
                    thinking: None,
                    ..Default::default()
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 15,
                total_tokens: 25,
                ..Default::default()
            }),
        }
    }

    #[tokio::test]
    async fn test_create_fake_stream() {
        use futures::StreamExt;

        let response = create_test_response();
        let stream = create_fake_stream(response).await.unwrap();

        let chunks: Vec<_> = stream.collect().await;
        assert!(!chunks.is_empty());

        // All chunks should be Ok
        for chunk in &chunks {
            assert!(chunk.is_ok());
        }

        // First chunk should have role
        let first = chunks[0].as_ref().unwrap();
        assert_eq!(first.choices[0].delta.role, Some(MessageRole::Assistant));

        // Last chunk should have finish_reason
        let last = chunks.last().unwrap().as_ref().unwrap();
        assert_eq!(last.choices[0].finish_reason, Some(FinishReason::Stop));
    }
}
