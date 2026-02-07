//! Unit tests for Infinity provider

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use crate::core::types::ProviderCapability;

    #[tokio::test]
    async fn test_provider_creation() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };

        let provider = InfinityProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_with_api_base() {
        let provider = InfinityProvider::with_api_base("http://localhost:8080").await;
        assert!(provider.is_ok());
    }

    #[test]
    fn test_provider_name() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = InfinityProvider::with_api_base("http://localhost:8080")
                .await
                .unwrap();
            assert_eq!(provider.name(), "infinity");
        });
    }

    #[test]
    fn test_provider_capabilities() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = InfinityProvider::with_api_base("http://localhost:8080")
                .await
                .unwrap();
            let capabilities = provider.capabilities();

            assert!(capabilities.contains(&ProviderCapability::Embeddings));
            // Infinity doesn't support chat completion
            assert!(!capabilities.contains(&ProviderCapability::ChatCompletion));
        });
    }

    #[test]
    fn test_model_info() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = InfinityProvider::with_api_base("http://localhost:8080")
                .await
                .unwrap();
            let models = provider.models();

            assert!(!models.is_empty());
            assert!(models[0].id.contains("infinity"));
        });
    }

    #[test]
    fn test_supported_openai_params() {
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let provider = InfinityProvider::with_api_base("http://localhost:8080")
                .await
                .unwrap();

            let params = provider.get_supported_openai_params("any-model");
            assert!(params.contains(&"encoding_format"));
            assert!(params.contains(&"modality"));
            assert!(params.contains(&"dimensions"));
        });
    }

    #[tokio::test]
    async fn test_cost_calculation() {
        let provider = InfinityProvider::with_api_base("http://localhost:8080")
            .await
            .unwrap();

        // Infinity is self-hosted, cost should be 0
        let cost = provider.calculate_cost("any-model", 1000, 1000).await;
        assert!(cost.is_ok());
        assert_eq!(cost.unwrap(), 0.0);
    }

    // Note: Error mapping tests removed - InfinityError is now a type alias to ProviderError

    #[test]
    fn test_config_api_base() {
        let config = InfinityConfig::default();
        assert!(config.get_api_base().is_none());

        let custom_config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            custom_config.get_api_base(),
            Some("http://localhost:8080".to_string())
        );
    }

    #[test]
    fn test_config_embeddings_url() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_embeddings_url(),
            Some("http://localhost:8080/embeddings".to_string())
        );
    }

    #[test]
    fn test_config_rerank_url() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_rerank_url(),
            Some("http://localhost:8080/rerank".to_string())
        );
    }

    #[tokio::test]
    async fn test_chat_completion_not_supported() {
        use crate::core::types::{ChatMessage, ChatRequest, MessageContent, MessageRole};

        let provider = InfinityProvider::with_api_base("http://localhost:8080")
            .await
            .unwrap();

        let request = ChatRequest {
            model: "test-model".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            }],
            ..Default::default()
        };

        let result = provider
            .chat_completion(
                request,
                crate::core::types::RequestContext::default(),
            )
            .await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_map_openai_params() {
        let provider = InfinityProvider::with_api_base("http://localhost:8080")
            .await
            .unwrap();

        let mut params = std::collections::HashMap::new();
        params.insert(
            "dimensions".to_string(),
            serde_json::Value::Number(serde_json::Number::from(512)),
        );

        let result = provider.map_openai_params(params, "model").await.unwrap();
        assert!(result.contains_key("output_dimension"));
        assert!(!result.contains_key("dimensions"));
    }

    #[test]
    fn test_rerank_request_serialization() {
        use super::super::provider::InfinityRerankRequest;

        let request = InfinityRerankRequest {
            query: "What is the capital of France?".to_string(),
            documents: vec![
                "Paris is the capital of France.".to_string(),
                "London is the capital of England.".to_string(),
            ],
            model: "rerank-model".to_string(),
            top_n: Some(2),
            return_documents: Some(true),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert_eq!(json["query"], "What is the capital of France?");
        assert_eq!(json["documents"].as_array().unwrap().len(), 2);
        assert_eq!(json["model"], "rerank-model");
        assert_eq!(json["top_n"], 2);
        assert_eq!(json["return_documents"], true);
    }
}
