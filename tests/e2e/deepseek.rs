//! E2E tests for DeepSeek provider
//!
//! These tests make real API calls and require DEEPSEEK_API_KEY.
//! Run with: DEEPSEEK_API_KEY=xxx cargo test --all-features -- --ignored deepseek

#[cfg(test)]
mod tests {
    use litellm_rs::core::providers::deepseek::DeepSeekProvider;
    use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
    use litellm_rs::core::types::context::RequestContext;
    use litellm_rs::core::types::{
        chat::ChatMessage, chat::ChatRequest, message::MessageContent, message::MessageRole,
    };

    /// Helper to create a simple chat request
    fn create_chat_request(model: &str, content: &str) -> ChatRequest {
        ChatRequest {
            model: model.to_string(),
            messages: vec![ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(content.to_string())),
                ..Default::default()
            }],
            max_tokens: Some(100),
            temperature: Some(0.0),
            ..Default::default()
        }
    }

    /// E2E test for DeepSeek chat completion
    #[tokio::test]
    #[ignore]
    async fn test_deepseek_chat_completion() {
        let provider = DeepSeekProvider::from_env().expect("Failed to create DeepSeek provider");

        let request = create_chat_request(
            "deepseek-chat",
            "Say 'Hello from DeepSeek!' and nothing else.",
        );
        let context = RequestContext::default();

        let response = provider.chat_completion(request, context).await;

        assert!(
            response.is_ok(),
            "DeepSeek chat completion failed: {:?}",
            response.err()
        );

        let response = response.unwrap();
        assert!(!response.choices.is_empty(), "Expected at least one choice");

        if let Some(content) = &response.choices[0].message.content {
            let text = match content {
                MessageContent::Text(t) => t.as_str(),
                MessageContent::Parts(_) => "",
            };
            println!("DeepSeek response: {}", text);
            assert!(!text.is_empty(), "Expected non-empty response");
        }
    }

    /// E2E test for DeepSeek streaming
    #[tokio::test]
    #[ignore]
    async fn test_deepseek_streaming() {
        use futures::StreamExt;

        let provider = DeepSeekProvider::from_env().expect("Failed to create DeepSeek provider");

        let request = create_chat_request("deepseek-chat", "Count from 1 to 5.");
        let context = RequestContext::default();

        let stream_result = provider.chat_completion_stream(request, context).await;

        assert!(
            stream_result.is_ok(),
            "DeepSeek streaming failed: {:?}",
            stream_result.err()
        );

        let mut stream = stream_result.unwrap();
        let mut chunk_count = 0;
        let mut full_content = String::new();

        while let Some(result) = stream.next().await {
            match result {
                Ok(chunk) => {
                    chunk_count += 1;
                    if let Some(choice) = chunk.choices.first() {
                        if let Some(content) = &choice.delta.content {
                            full_content.push_str(content);
                        }
                    }
                }
                Err(e) => {
                    panic!("Stream error: {:?}", e);
                }
            }
        }

        println!(
            "DeepSeek streaming: {} chunks, content: {}",
            chunk_count, full_content
        );
        assert!(chunk_count > 0, "Expected at least one chunk");
        assert!(
            !full_content.is_empty(),
            "Expected non-empty streamed content"
        );
    }

    /// E2E test for DeepSeek with system message
    #[tokio::test]
    #[ignore]
    async fn test_deepseek_system_message() {
        let provider = DeepSeekProvider::from_env().expect("Failed to create DeepSeek provider");

        let request = ChatRequest {
            model: "deepseek-chat".to_string(),
            messages: vec![
                ChatMessage {
                    role: MessageRole::System,
                    content: Some(MessageContent::Text(
                        "You are a helpful assistant that responds in exactly one word."
                            .to_string(),
                    )),
                    ..Default::default()
                },
                ChatMessage {
                    role: MessageRole::User,
                    content: Some(MessageContent::Text("What is 2+2?".to_string())),
                    ..Default::default()
                },
            ],
            max_tokens: Some(10),
            temperature: Some(0.0),
            ..Default::default()
        };

        let context = RequestContext::default();
        let response = provider.chat_completion(request, context).await;

        assert!(response.is_ok(), "DeepSeek failed: {:?}", response.err());

        let response = response.unwrap();
        if let Some(content) = &response.choices[0].message.content {
            let text = match content {
                MessageContent::Text(t) => t.as_str(),
                MessageContent::Parts(_) => "",
            };
            println!("DeepSeek system message response: {}", text);
            // Should contain "4" or "Four"
            assert!(
                text.contains("4") || text.to_lowercase().contains("four"),
                "Expected '4' in response: {}",
                text
            );
        }
    }

    /// E2E test for DeepSeek-Coder model
    #[tokio::test]
    #[ignore]
    async fn test_deepseek_coder() {
        let provider = DeepSeekProvider::from_env().expect("Failed to create DeepSeek provider");

        let request = create_chat_request(
            "deepseek-coder",
            "Write a Python function that adds two numbers. Only output the code, no explanation.",
        );
        let context = RequestContext::default();

        let response = provider.chat_completion(request, context).await;

        assert!(
            response.is_ok(),
            "DeepSeek Coder failed: {:?}",
            response.err()
        );

        let response = response.unwrap();
        if let Some(content) = &response.choices[0].message.content {
            let text = match content {
                MessageContent::Text(t) => t.as_str(),
                MessageContent::Parts(_) => "",
            };
            println!("DeepSeek Coder response: {}", text);
            // Should contain Python function definition
            assert!(
                text.contains("def") || text.contains("return"),
                "Expected Python code in response: {}",
                text
            );
        }
    }
}
