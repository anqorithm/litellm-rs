//! Tests for token counter functionality

#[cfg(test)]
use crate::core::models::openai::{ChatMessage, MessageContent, MessageRole};
use crate::utils::ai::counter::token_counter::TokenCounter;

#[test]
fn test_text_token_estimation() {
    let counter = TokenCounter::new();
    let config = counter.get_model_config("gpt-3.5-turbo").unwrap();

    let tokens = counter.estimate_text_tokens(config, "Hello, world!");
    assert!(tokens > 0);
    assert!(tokens < 10); // Should be reasonable for short text
}

#[test]
fn test_chat_token_counting() {
    let counter = TokenCounter::new();
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text("Hello, how are you?".to_string())),
        name: None,
        function_call: None,
        tool_calls: None,
        tool_call_id: None,
        audio: None,
    }];

    let estimate = counter
        .count_chat_tokens("gpt-3.5-turbo", &messages)
        .unwrap();
    assert!(estimate.input_tokens > 0);
    assert!(estimate.is_approximate);
}

#[test]
fn test_context_window_check() {
    let counter = TokenCounter::new();

    // Should fit
    assert!(
        counter
            .check_context_window("gpt-3.5-turbo", 1000, Some(1000))
            .unwrap()
    );

    // Should not fit
    assert!(
        !counter
            .check_context_window("gpt-3.5-turbo", 3000, Some(2000))
            .unwrap()
    );
}

#[test]
fn test_model_family_extraction() {
    let counter = TokenCounter::new();

    assert_eq!(counter.extract_model_family("gpt-4-turbo"), "gpt-4");
    assert_eq!(
        counter.extract_model_family("gpt-3.5-turbo-16k"),
        "gpt-3.5-turbo"
    );
    assert_eq!(counter.extract_model_family("claude-3-opus"), "claude-3");
    assert_eq!(counter.extract_model_family("unknown-model"), "default");
}
