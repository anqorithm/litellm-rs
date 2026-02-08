//! E2E tests for chat completion
//!
//! These tests make real API calls and require API keys.
//! Run with: cargo test --all-features -- --ignored

#[cfg(test)]
mod tests {
    use litellm_rs::core::types::{ChatMessage, ChatRequest, MessageContent, MessageRole};

    /// Helper to extract text from MessageContent
    fn extract_text(content: &MessageContent) -> &str {
        match content {
            MessageContent::Text(text) => text.as_str(),
            MessageContent::Parts(_) => "",
        }
    }

    /// Helper to create a simple chat request
    fn create_simple_request(model: &str, content: &str) -> ChatRequest {
        ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(content.to_string())),
                ..Default::default()
            }],
            max_tokens: Some(50),
            ..Default::default()
        }
    }

    /// E2E test using the high-level completion API
    #[tokio::test]
    #[ignore]
    async fn test_completion_api() {
        use litellm_rs::{completion, user_message};

        let response = completion(
            "groq/llama-3.1-8b-instant",
            vec![user_message("Say 'test passed' and nothing else")],
            None,
        )
        .await;

        assert!(response.is_ok(), "Completion failed: {:?}", response.err());
        let response = response.unwrap();
        assert!(!response.choices.is_empty());
    }

    /// E2E test for streaming completion
    #[tokio::test]
    #[ignore]
    async fn test_streaming_completion() {
        use futures::StreamExt;
        use litellm_rs::{completion_stream, user_message};

        let mut stream = completion_stream(
            "groq/llama-3.1-8b-instant",
            vec![user_message("Count to 3")],
            None,
        )
        .await
        .expect("Failed to create stream");

        let mut chunk_count = 0;
        while let Some(result) = stream.next().await {
            assert!(result.is_ok(), "Stream chunk failed: {:?}", result.err());
            chunk_count += 1;
        }

        assert!(chunk_count > 0, "No chunks received");
    }

    /// E2E test with system message
    #[tokio::test]
    #[ignore]
    async fn test_system_message() {
        use litellm_rs::{completion, system_message, user_message};

        let response = completion(
            "groq/llama-3.1-8b-instant",
            vec![
                system_message("You are a helpful assistant that responds briefly."),
                user_message("What is 2+2?"),
            ],
            None,
        )
        .await;

        assert!(response.is_ok());
        let response = response.unwrap();
        assert!(!response.choices.is_empty());

        // Response should contain "4"
        if let Some(content) = response.choices[0].message.content.as_ref() {
            let text = extract_text(content);
            assert!(text.contains("4"), "Expected '4' in response: {}", text);
        }
    }

    /// E2E test for multi-turn conversation
    #[tokio::test]
    #[ignore]
    async fn test_multi_turn_conversation() {
        use litellm_rs::{assistant_message, completion, user_message};

        let response = completion(
            "groq/llama-3.1-8b-instant",
            vec![
                user_message("My name is Alice."),
                assistant_message("Nice to meet you, Alice!"),
                user_message("What is my name?"),
            ],
            None,
        )
        .await;

        assert!(response.is_ok());
        let response = response.unwrap();

        // Response should contain "Alice"
        if let Some(content) = response.choices[0].message.content.as_ref() {
            let text = extract_text(content);
            assert!(
                text.to_lowercase().contains("alice"),
                "Expected 'Alice' in response: {}",
                text
            );
        }
    }

    /// E2E test for temperature control
    #[tokio::test]
    #[ignore]
    async fn test_temperature_control() {
        use litellm_rs::{CompletionOptions, completion, user_message};

        // Low temperature (deterministic)
        let options = CompletionOptions {
            temperature: Some(0.0),
            ..Default::default()
        };

        let response1 = completion(
            "groq/llama-3.1-8b-instant",
            vec![user_message("What is the capital of France?")],
            Some(options.clone()),
        )
        .await;

        let response2 = completion(
            "groq/llama-3.1-8b-instant",
            vec![user_message("What is the capital of France?")],
            Some(options),
        )
        .await;

        assert!(response1.is_ok());
        assert!(response2.is_ok());

        // With temperature=0, responses should be very similar
        let r1 = response1.unwrap();
        let r2 = response2.unwrap();

        // Both should contain "Paris"
        if let Some(content1) = r1.choices[0].message.content.as_ref() {
            let text1 = extract_text(content1);
            assert!(text1.contains("Paris"), "Expected 'Paris' in response");
        }
        if let Some(content2) = r2.choices[0].message.content.as_ref() {
            let text2 = extract_text(content2);
            assert!(text2.contains("Paris"), "Expected 'Paris' in response");
        }
    }

    /// E2E test for max_tokens limit
    #[tokio::test]
    #[ignore]
    async fn test_max_tokens_limit() {
        use litellm_rs::core::completion::FinishReason;
        use litellm_rs::{CompletionOptions, completion, user_message};

        let options = CompletionOptions {
            max_tokens: Some(5),
            ..Default::default()
        };

        let response = completion(
            "groq/llama-3.1-8b-instant",
            vec![user_message("Write a long story about a dragon")],
            Some(options),
        )
        .await;

        assert!(response.is_ok());
        let response = response.unwrap();

        // Check that the response was truncated (finish_reason should be 'length' or 'stop')
        let finish_reason = &response.choices[0].finish_reason;
        assert!(
            matches!(
                finish_reason,
                Some(FinishReason::Length) | Some(FinishReason::Stop)
            ),
            "Unexpected finish_reason: {:?}",
            finish_reason
        );
    }

    /// E2E test for Groq provider chat completion (requires GROQ_API_KEY)
    #[tokio::test]
    #[ignore]
    async fn test_groq_provider_chat_completion() {
        use litellm_rs::core::providers::groq::GroqProvider;
        use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
        use litellm_rs::core::types::context::RequestContext;

        let api_key =
            std::env::var("GROQ_API_KEY").expect("GROQ_API_KEY environment variable not set");

        let provider = GroqProvider::with_api_key(&api_key).await.unwrap();

        let request = create_simple_request("llama-3.1-8b-instant", "Hello");
        let context = RequestContext::default();
        let response = provider.chat_completion(request, context).await;

        assert!(
            response.is_ok(),
            "Chat completion failed: {:?}",
            response.err()
        );
        let response = response.unwrap();
        assert!(!response.choices.is_empty());
    }
}
