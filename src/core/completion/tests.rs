//! Completion module tests

use super::*;
use crate::core::types::message::MessageRole;

#[test]
fn test_message_creation() {
    let msg = user_message("Hello, world!");
    assert_eq!(msg.role, MessageRole::User);
    if let Some(MessageContent::Text(content)) = msg.content {
        assert_eq!(content, "Hello, world!");
    } else {
        panic!("Expected text content");
    }
}

#[test]
fn test_completion_options_default() {
    let options = CompletionOptions::default();
    assert!(!options.stream);
    assert_eq!(options.extra_params.len(), 0);
}

#[test]
fn test_system_message_creation() {
    let msg = system_message("You are a helpful assistant.");
    assert_eq!(msg.role, MessageRole::System);
    if let Some(MessageContent::Text(content)) = msg.content {
        assert_eq!(content, "You are a helpful assistant.");
    } else {
        panic!("Expected text content");
    }
}

#[test]
fn test_assistant_message_creation() {
    let msg = assistant_message("I can help you with that.");
    assert_eq!(msg.role, MessageRole::Assistant);
    if let Some(MessageContent::Text(content)) = msg.content {
        assert_eq!(content, "I can help you with that.");
    } else {
        panic!("Expected text content");
    }
}
