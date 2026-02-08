//! Tests for semantic caching

#[cfg(test)]
mod tests {
    use super::super::cache::SemanticCache;
    use super::super::types::{EmbeddingProvider, SemanticCacheConfig};
    use super::super::utils::extract_prompt_text;
    use super::super::validation::should_cache_request;
    use crate::core::models::openai::ChatCompletionRequest;
    use crate::core::models::openai::{ChatMessage, MessageContent, MessageRole};
    use crate::storage::vector::VectorStore;
    use crate::utils::error::error::Result;
    use std::sync::Arc;

    #[test]
    fn test_semantic_cache_config_default() {
        let config = SemanticCacheConfig::default();
        assert_eq!(config.similarity_threshold, 0.85);
        assert_eq!(config.max_cache_size, 10000);
        assert_eq!(config.default_ttl_seconds, 3600);
    }

    #[tokio::test]
    async fn test_extract_prompt_text() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant".to_string(),
                )),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello world".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
        ];

        let prompt_text = extract_prompt_text(&messages);
        assert!(prompt_text.contains("You are a helpful assistant"));
        assert!(prompt_text.contains("Hello world"));
    }

    #[tokio::test]
    async fn test_should_cache_request() {
        let config = SemanticCacheConfig::default();

        let mut request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            max_tokens: None,
            max_completion_tokens: None,
            temperature: Some(0.1),
            top_p: None,
            n: None,
            stream: Some(false),
            stream_options: None,
            stop: None,
            presence_penalty: None,
            frequency_penalty: None,
            logit_bias: None,
            user: None,
            functions: None,
            function_call: None,
            tools: None,
            tool_choice: None,
            response_format: None,
            seed: None,
            logprobs: None,
            top_logprobs: None,
            modalities: None,
            audio: None,
        };

        // Should cache low temperature request
        assert!(should_cache_request(&config, &request));

        // Should not cache high temperature request
        request.temperature = Some(0.9);
        assert!(!should_cache_request(&config, &request));

        // Should not cache streaming request (by default)
        request.temperature = Some(0.1);
        request.stream = Some(true);
        assert!(!should_cache_request(&config, &request));
    }

    async fn create_test_cache() -> SemanticCache {
        // For testing purposes, create a dummy cache
        let config = SemanticCacheConfig {
            similarity_threshold: 0.85,
            max_cache_size: 1000,
            default_ttl_seconds: 3600,
            embedding_model: "text-embedding-ada-002".to_string(),
            enable_streaming_cache: false,
            min_prompt_length: 10,
            cache_hit_boost: 1.1,
        };

        // Create a simple test implementation
        SemanticCache::new(
            config,
            Arc::new(TestVectorStore),
            Arc::new(TestEmbeddingProvider),
        )
        .await
        .unwrap()
    }

    // Simple test implementations
    struct TestVectorStore;
    struct TestEmbeddingProvider;

    #[async_trait::async_trait]
    impl VectorStore for TestVectorStore {
        async fn search(
            &self,
            _vector: Vec<f32>,
            _limit: usize,
        ) -> Result<Vec<crate::storage::vector::SearchResult>> {
            Ok(vec![])
        }

        async fn insert(&self, _vectors: Vec<crate::storage::vector::VectorData>) -> Result<()> {
            Ok(())
        }

        async fn delete(&self, _ids: Vec<String>) -> Result<()> {
            Ok(())
        }
    }

    #[async_trait::async_trait]
    impl EmbeddingProvider for TestEmbeddingProvider {
        async fn generate_embedding(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.1; 1536])
        }

        fn embedding_dimension(&self) -> usize {
            1536
        }
    }
}
