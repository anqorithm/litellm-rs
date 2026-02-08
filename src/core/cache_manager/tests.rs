//! Cache manager tests

#[cfg(test)]
mod tests {
    use crate::core::cache_manager::manager::CacheManager;
    use crate::core::cache_manager::types::{CacheConfig, CacheKey};
    use crate::core::models::openai::*;
    use crate::utils::error::error::Result;

    #[tokio::test]
    async fn test_cache_manager() -> Result<()> {
        let config = CacheConfig::default();
        let cache = CacheManager::new(config)?;

        let request = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let key = CacheKey::from_request(&request, None);

        // Should be empty initially
        let initial_result = cache.get(&key).await?;
        assert!(initial_result.is_none());

        // Store a response
        let response = ChatCompletionResponse {
            id: "test".to_string(),
            object: "chat.completion".to_string(),
            created: 1234567890,
            model: "gpt-4".to_string(),
            choices: vec![],
            usage: None,
            system_fingerprint: None,
        };

        cache.put(key.clone(), response.clone()).await?;

        // Should find the cached response
        let cached = cache.get(&key).await?;
        assert!(cached.is_some());
        if let Some(cached_response) = cached {
            assert_eq!(cached_response.id, response.id);
        }

        Ok(())
    }

    #[test]
    fn test_cache_key_generation() {
        let request1 = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let request2 = ChatCompletionRequest {
            model: "gpt-4".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            }],
            ..Default::default()
        };

        let key1 = CacheKey::from_request(&request1, None);
        let key2 = CacheKey::from_request(&request2, None);

        assert_eq!(key1, key2);
    }
}
