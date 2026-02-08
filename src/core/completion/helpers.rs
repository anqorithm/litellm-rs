//! Helper functions for message creation

use crate::core::types::{ChatMessage, message::MessageContent, message::MessageRole};

/// Convert messages to chat messages (no-op since Message is an alias)
pub fn convert_messages_to_chat_messages(messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    messages
}

/// Helper function to create user message
pub fn user_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text(content.into())),
        ..Default::default()
    }
}

/// Helper function to create system message
pub fn system_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::System,
        content: Some(MessageContent::Text(content.into())),
        ..Default::default()
    }
}

/// Helper function to create assistant message
pub fn assistant_message(content: impl Into<String>) -> ChatMessage {
    ChatMessage {
        role: MessageRole::Assistant,
        content: Some(MessageContent::Text(content.into())),
        ..Default::default()
    }
}

/// Helper function to create assistant message with thinking
pub fn assistant_message_with_thinking(
    content: impl Into<String>,
    thinking: impl Into<String>,
) -> ChatMessage {
    use crate::core::types::ThinkingContent;

    ChatMessage {
        role: MessageRole::Assistant,
        content: Some(MessageContent::Text(content.into())),
        thinking: Some(ThinkingContent::text(thinking)),
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_message() {
        let msg = user_message("Hello");
        assert_eq!(msg.role, MessageRole::User);
        match msg.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "Hello"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_system_message() {
        let msg = system_message("You are helpful");
        assert_eq!(msg.role, MessageRole::System);
        match msg.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "You are helpful"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_assistant_message() {
        let msg = assistant_message("Hi there!");
        assert_eq!(msg.role, MessageRole::Assistant);
        match msg.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "Hi there!"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_assistant_message_with_thinking() {
        let msg = assistant_message_with_thinking("Answer", "Let me think...");
        assert_eq!(msg.role, MessageRole::Assistant);
        assert!(msg.thinking.is_some());
        match msg.content {
            Some(MessageContent::Text(text)) => assert_eq!(text, "Answer"),
            _ => panic!("Expected text content"),
        }
    }

    #[test]
    fn test_convert_messages() {
        let messages = vec![user_message("Hello"), assistant_message("Hi!")];
        let converted = convert_messages_to_chat_messages(messages.clone());
        assert_eq!(converted.len(), 2);
    }
}
