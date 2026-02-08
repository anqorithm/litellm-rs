//! OCI Generative AI Provider Tests
//!
//! Unit tests for OCI provider.

use super::*;
use crate::core::types::{model::ProviderCapability, RequestContext};
use crate::core::types::health::HealthStatus;
use crate::core::types::EmbeddingRequest;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use super::streaming;

mod config_tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = OciConfig::default();
        assert!(config.auth_token.is_none());
        assert!(config.compartment_id.is_none());
        assert!(config.region.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_config_build_chat_url() {
        let config = OciConfig {
            api_base: Some("https://inference.generativeai.us-chicago-1.oci.oraclecloud.com".to_string()),
            ..Default::default()
        };
        let url = config.build_chat_url();
        assert!(url.contains("generativeai"));
        assert!(url.contains("/actions/chat"));
    }

    #[test]
    fn test_config_build_url_from_region() {
        let config = OciConfig {
            region: Some("eu-frankfurt-1".to_string()),
            ..Default::default()
        };
        let base = config.get_api_base().unwrap();
        assert!(base.contains("eu-frankfurt-1"));
    }

    #[test]
    fn test_config_validation_missing_auth() {
        let config = OciConfig {
            compartment_id: Some("test-compartment".to_string()),
            region: Some("us-chicago-1".to_string()),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("authentication"));
    }

    #[test]
    fn test_config_validation_missing_compartment() {
        let config = OciConfig {
            auth_token: Some("test-token".to_string()),
            region: Some("us-chicago-1".to_string()),
            ..Default::default()
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("compartment"));
    }
}

mod error_tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = OciError::AuthenticationError("Test error".to_string());
        assert!(err.to_string().contains("Authentication"));
        assert!(err.to_string().contains("Test error"));
    }

    #[test]
    fn test_error_mapper_401() {
        let mapper = OciErrorMapper;
        let err: OciError =
            crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
                &mapper,
                401,
                r#"{"message": "Invalid credentials"}"#,
            );
        assert!(matches!(err, OciError::AuthenticationError(_)));
    }

    #[test]
    fn test_error_mapper_429() {
        let mapper = OciErrorMapper;
        let err: OciError =
            crate::core::traits::error_mapper::trait_def::ErrorMapper::map_http_error(
                &mapper,
                429,
                r#"{"message": "Rate limit exceeded"}"#,
            );
        assert!(matches!(err, OciError::RateLimitError(_)));
    }

    #[test]
    fn test_error_to_provider_error() {
        let oci_err = OciError::ModelNotFoundError("test-model".to_string());
        let provider_err: crate::core::providers::unified_provider::ProviderError = oci_err.into();
        assert!(matches!(
            provider_err,
            crate::core::providers::unified_provider::ProviderError::ModelNotFound { .. }
        ));
    }
}

mod model_info_tests {
    use super::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
    }

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("cohere.command-r-plus");
        assert!(model.is_some());
        let model = model.unwrap();
        assert_eq!(model.display_name, "Cohere Command R+");
    }

    #[test]
    fn test_supports_tools() {
        assert!(model_info::supports_tools("cohere.command-r-plus"));
        assert!(model_info::supports_tools("meta.llama-3.1-70b-instruct"));
        assert!(!model_info::supports_tools("cohere.command-light"));
    }

    #[test]
    fn test_supports_vision() {
        // OCI models don't currently support vision
        assert!(!model_info::supports_vision("cohere.command-r-plus"));
    }
}

mod provider_tests {
    use super::*;

    fn create_test_config() -> OciConfig {
        OciConfig {
            auth_token: Some("test-token".to_string()),
            compartment_id: Some("test-compartment".to_string()),
            api_base: Some("https://test.example.com".to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_provider_creation() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await;
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn test_provider_clone() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let cloned = provider.clone();
        assert_eq!(provider.name(), cloned.name());
    }

    #[tokio::test]
    async fn test_provider_models_not_empty() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        assert!(!provider.models().is_empty());
    }

    #[tokio::test]
    async fn test_provider_capabilities() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let caps = provider.capabilities();
        assert!(caps.contains(&ProviderCapability::ChatCompletion));
    }

    #[tokio::test]
    async fn test_health_check_healthy() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let health = provider.health_check().await;
        assert!(matches!(health, HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_calculate_cost() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let cost = provider
            .calculate_cost("cohere.command-r-plus", 1000, 500)
            .await;
        assert!(cost.is_ok());
        assert!(cost.unwrap() > 0.0);
    }

    #[tokio::test]
    async fn test_calculate_cost_unknown_model() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let cost = provider.calculate_cost("unknown-model", 1000, 500).await;
        assert!(cost.is_err());
    }

    #[tokio::test]
    async fn test_embeddings_not_supported() {
        let config = create_test_config();
        let provider = OciProvider::new(config).await.unwrap();
        let result = provider
            .embeddings(
                EmbeddingRequest {
                    model: "test".to_string(),
                    input: crate::core::types::EmbeddingInput::Single(
                        "test".to_string(),
                    ),
                    encoding_format: None,
                    dimensions: None,
                    user: None,
                },
                RequestContext::default(),
            )
            .await;
        assert!(result.is_err());
    }
}

mod streaming_tests {
    use super::*;
    use bytes::Bytes;
    use crate::core::types::MessageRole;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_stream_basic() {
        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = futures::stream::iter(test_data);
        let mut oci_stream = streaming::OciStream::new(mock_stream);

        let chunk = oci_stream.next().await;
        assert!(chunk.is_some());
        let chunk = chunk.unwrap().unwrap();
        assert_eq!(chunk.choices[0].delta.content.as_ref().unwrap(), "Hello");

        assert!(oci_stream.next().await.is_none());
    }

    #[tokio::test]
    async fn test_stream_multiple_chunks() {
        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"}}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" World\"}}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = futures::stream::iter(test_data);
        let mut oci_stream = streaming::OciStream::new(mock_stream);

        // Role chunk
        let chunk1 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.role, Some(MessageRole::Assistant));

        // Content chunks
        let chunk2 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), "Hello");

        let chunk3 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk3.choices[0].delta.content.as_ref().unwrap(), " World");

        assert!(oci_stream.next().await.is_none());
    }
}
