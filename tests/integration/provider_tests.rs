//! Provider integration tests
//!
//! Tests provider creation, configuration, and basic functionality.
//! E2E tests that require real API keys are marked with #[ignore].

#[cfg(test)]
mod tests {
    use litellm_rs::core::providers::openai_like::OpenAILikeProvider;
    use litellm_rs::core::providers::registry;
    use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;

    /// Test Groq provider creation via catalog
    #[tokio::test]
    async fn test_groq_provider_creation() {
        let def = registry::get_definition("groq").expect("groq should be in catalog");
        let config = def.to_openai_like_config(Some("test-key"), None);
        let provider = OpenAILikeProvider::new(config).await;
        assert!(provider.is_ok());
    }

    /// Test Groq provider name via catalog
    #[tokio::test]
    async fn test_groq_provider_name() {
        let def = registry::get_definition("groq").unwrap();
        let config = def.to_openai_like_config(Some("test-key"), None);
        let provider = OpenAILikeProvider::new(config).await.unwrap();
        assert_eq!(provider.name(), "groq");
    }

    // E2E tests that require real API keys
    // Run with: cargo test --all-features -- --ignored

    /// E2E test for Groq chat completion via catalog (requires GROQ_API_KEY)
    #[tokio::test]
    #[ignore]
    async fn test_groq_real_chat_completion() {
        use litellm_rs::core::types::context::RequestContext;
        use litellm_rs::core::types::{
            chat::ChatMessage, chat::ChatRequest, message::MessageContent, message::MessageRole,
        };

        let api_key =
            std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY environment variable not set");

        let def = registry::get_definition("groq").unwrap();
        let config = def.to_openai_like_config(Some(&api_key), None);
        let provider = OpenAILikeProvider::new(config).await.unwrap();

        let request = ChatRequest {
            model: "llama-3.1-8b-instant".to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "Say 'Hello' and nothing else".to_string(),
                )),
                ..Default::default()
            }],
            max_tokens: Some(10),
            ..Default::default()
        };

        let context = RequestContext::default();
        let response = provider.chat_completion(request, context).await;
        assert!(
            response.is_ok(),
            "Chat completion failed: {:?}",
            response.err()
        );

        let response = response.unwrap();
        assert!(!response.choices.is_empty());
        assert!(response.choices[0].message.content.is_some());
    }
}
