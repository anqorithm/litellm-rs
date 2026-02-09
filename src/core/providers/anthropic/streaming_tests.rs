use super::*;

// ==================== SSE Parser Tests ====================

#[test]
fn test_sse_parser_message_start() {
    let result = SSEParser::parse_event(
        "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_123\"}}",
    );
    assert!(matches!(result, Some(SSEEvent::MessageStart(_))));
}

#[test]
fn test_sse_parser_content_block_start() {
    let result = SSEParser::parse_event(
        "data: {\"type\":\"content_block_start\",\"index\":0,\"content_block\":{\"type\":\"text\"}}",
    );
    assert!(matches!(result, Some(SSEEvent::ContentBlockStart(_))));
}

#[test]
fn test_sse_parser_content_block_delta() {
    let result = SSEParser::parse_event(
        "data: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"Hello\"}}",
    );
    assert!(matches!(result, Some(SSEEvent::ContentBlockDelta(_))));
}

#[test]
fn test_sse_parser_content_block_stop() {
    let result = SSEParser::parse_event("data: {\"type\":\"content_block_stop\",\"index\":0}");
    assert!(matches!(result, Some(SSEEvent::ContentBlockStop(_))));
}

#[test]
fn test_sse_parser_message_delta() {
    let result = SSEParser::parse_event(
        "data: {\"type\":\"message_delta\",\"delta\":{\"stop_reason\":\"end_turn\"},\"usage\":{\"output_tokens\":100}}",
    );
    assert!(matches!(result, Some(SSEEvent::MessageDelta(_))));
}

#[test]
fn test_sse_parser_message_stop() {
    let result = SSEParser::parse_event("data: {\"type\":\"message_stop\"}");
    assert!(matches!(result, Some(SSEEvent::MessageStop(_))));
}

#[test]
fn test_sse_parser_error() {
    let result = SSEParser::parse_event(
        "data: {\"type\":\"error\",\"error\":{\"message\":\"Rate limit exceeded\"}}",
    );
    assert!(matches!(result, Some(SSEEvent::Error(_))));
}

#[test]
fn test_sse_parser_done_marker() {
    let result = SSEParser::parse_event("data: [DONE]");
    assert!(result.is_none());
}

#[test]
fn test_sse_parser_ping() {
    let result = SSEParser::parse_event("data: ");
    assert!(matches!(result, Some(SSEEvent::Ping)));
}

#[test]
fn test_sse_parser_empty_line() {
    let result = SSEParser::parse_event("");
    assert!(result.is_none());
}

#[test]
fn test_sse_parser_comment_line() {
    let result = SSEParser::parse_event(": this is a comment");
    assert!(result.is_none());
}

#[test]
fn test_sse_parser_event_line() {
    let result = SSEParser::parse_event("event: message_start");
    assert!(result.is_none());
}

#[test]
fn test_sse_parser_unknown_event_type() {
    let result = SSEParser::parse_event("data: {\"type\":\"unknown_event\",\"data\":{}}");
    assert!(matches!(result, Some(SSEEvent::Unknown(_))));
}

#[test]
fn test_sse_parser_invalid_json() {
    let result = SSEParser::parse_event("data: not valid json");
    assert!(matches!(result, Some(SSEEvent::Unknown(_))));
}

// ==================== Event Processing Tests ====================

#[test]
fn test_event_processing_content_delta() {
    let event = SSEEvent::ContentBlockDelta(serde_json::json!({
        "type": "content_block_delta",
        "delta": {
            "text": "Hello world"
        }
    }));

    let mut message_id = "msg_123".to_string();
    let result =
        AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 1234567890);

    assert!(result.is_ok());
    let chunk_opt = result.unwrap();
    assert!(chunk_opt.is_some());

    let chunk = chunk_opt.unwrap();
    assert_eq!(
        chunk.choices[0].delta.content,
        Some("Hello world".to_string())
    );
    assert_eq!(chunk.model, "claude-3-5-sonnet");
    assert_eq!(chunk.created, 1234567890);
}

#[test]
fn test_event_processing_message_start() {
    let event = SSEEvent::MessageStart(serde_json::json!({
        "type": "message_start",
        "message": {
            "id": "msg_test_123",
            "role": "assistant"
        }
    }));

    let mut message_id = String::new();
    let result =
        AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 1234567890);

    assert!(result.is_ok());
    let chunk_opt = result.unwrap();
    assert!(chunk_opt.is_some());

    let chunk = chunk_opt.unwrap();
    assert_eq!(chunk.choices[0].delta.role, Some(MessageRole::Assistant));
    assert_eq!(message_id, "msg_test_123");
}

#[test]
fn test_event_processing_message_delta_with_usage() {
    let event = SSEEvent::MessageDelta(serde_json::json!({
        "type": "message_delta",
        "delta": {
            "stop_reason": "end_turn"
        },
        "usage": {
            "input_tokens": 100,
            "output_tokens": 50
        }
    }));

    let mut message_id = "msg_123".to_string();
    let result =
        AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 1234567890);

    assert!(result.is_ok());
    let chunk_opt = result.unwrap();
    assert!(chunk_opt.is_some());

    let chunk = chunk_opt.unwrap();
    assert!(chunk.usage.is_some());
    let usage = chunk.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 100);
    assert_eq!(usage.completion_tokens, 50);
    assert_eq!(usage.total_tokens, 150);
}

#[test]
fn test_event_processing_message_delta_stop_reasons() {
    // Test end_turn
    let event = SSEEvent::MessageDelta(serde_json::json!({
        "type": "message_delta",
        "delta": { "stop_reason": "end_turn" }
    }));
    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);
    let chunk = result.unwrap().unwrap();
    assert_eq!(
        chunk.choices[0].finish_reason,
        Some(crate::core::types::responses::FinishReason::Stop)
    );

    // Test max_tokens
    let event = SSEEvent::MessageDelta(serde_json::json!({
        "type": "message_delta",
        "delta": { "stop_reason": "max_tokens" }
    }));
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);
    let chunk = result.unwrap().unwrap();
    assert_eq!(
        chunk.choices[0].finish_reason,
        Some(crate::core::types::responses::FinishReason::Length)
    );

    // Test tool_use
    let event = SSEEvent::MessageDelta(serde_json::json!({
        "type": "message_delta",
        "delta": { "stop_reason": "tool_use" }
    }));
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);
    let chunk = result.unwrap().unwrap();
    assert_eq!(
        chunk.choices[0].finish_reason,
        Some(crate::core::types::responses::FinishReason::ToolCalls)
    );
}

#[test]
fn test_event_processing_message_stop() {
    let event = SSEEvent::MessageStop(serde_json::json!({
        "type": "message_stop"
    }));

    let mut message_id = "msg_123".to_string();
    let result =
        AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 1234567890);

    assert!(result.is_ok());
    let chunk_opt = result.unwrap();
    assert!(chunk_opt.is_some());

    let chunk = chunk_opt.unwrap();
    assert!(chunk.choices.is_empty());
}

#[test]
fn test_event_processing_content_block_start_skip() {
    let event = SSEEvent::ContentBlockStart(serde_json::json!({
        "type": "content_block_start",
        "index": 0
    }));

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Should skip
}

#[test]
fn test_event_processing_content_block_stop_skip() {
    let event = SSEEvent::ContentBlockStop(serde_json::json!({
        "type": "content_block_stop",
        "index": 0
    }));

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Should skip
}

#[test]
fn test_event_processing_ping_skip() {
    let mut message_id = "msg_123".to_string();
    let result =
        AnthropicStream::process_event(SSEEvent::Ping, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Should skip
}

#[test]
fn test_event_processing_unknown_skip() {
    let event = SSEEvent::Unknown("unknown_event".to_string());

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    assert!(result.unwrap().is_none()); // Should skip
}

#[test]
fn test_event_processing_error() {
    let event = SSEEvent::Error(serde_json::json!({
        "type": "error",
        "error": {
            "message": "Rate limit exceeded"
        }
    }));

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_err());
}

// ==================== StreamUtils Tests ====================

#[test]
fn test_validate_stream_chunk_valid() {
    let chunk = ChatChunk {
        id: "chunk_123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "claude-3-5-sonnet".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
    };

    let result = StreamUtils::validate_stream_chunk(&chunk);
    assert!(result.is_ok());
}

#[test]
fn test_validate_stream_chunk_missing_id() {
    let chunk = ChatChunk {
        id: "".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "claude-3-5-sonnet".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
    };

    let result = StreamUtils::validate_stream_chunk(&chunk);
    assert!(result.is_err());
}

#[test]
fn test_validate_stream_chunk_missing_model() {
    let chunk = ChatChunk {
        id: "chunk_123".to_string(),
        object: "chat.completion.chunk".to_string(),
        created: 1234567890,
        model: "".to_string(),
        choices: vec![],
        usage: None,
        system_fingerprint: None,
    };

    let result = StreamUtils::validate_stream_chunk(&chunk);
    assert!(result.is_err());
}

// ==================== SSEEvent Clone Tests ====================

#[test]
fn test_sse_event_clone() {
    let event = SSEEvent::ContentBlockDelta(serde_json::json!({"type": "test"}));
    let cloned = event.clone();

    if let (SSEEvent::ContentBlockDelta(orig), SSEEvent::ContentBlockDelta(cloned_val)) =
        (&event, &cloned)
    {
        assert_eq!(orig, cloned_val);
    } else {
        panic!("Clone failed");
    }
}

// ==================== Edge Cases ====================

#[test]
fn test_content_delta_empty_text() {
    let event = SSEEvent::ContentBlockDelta(serde_json::json!({
        "type": "content_block_delta",
        "delta": {
            "text": ""
        }
    }));

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    let chunk = result.unwrap().unwrap();
    assert_eq!(chunk.choices[0].delta.content, Some("".to_string()));
}

#[test]
fn test_content_delta_missing_text() {
    let event = SSEEvent::ContentBlockDelta(serde_json::json!({
        "type": "content_block_delta",
        "delta": {}
    }));

    let mut message_id = "msg_123".to_string();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    let chunk = result.unwrap().unwrap();
    assert_eq!(chunk.choices[0].delta.content, Some("".to_string()));
}

#[test]
fn test_message_start_missing_message() {
    let event = SSEEvent::MessageStart(serde_json::json!({
        "type": "message_start"
    }));

    let mut message_id = String::new();
    let result = AnthropicStream::process_event(event, "claude-3-5-sonnet", &mut message_id, 0);

    assert!(result.is_ok());
    // message_id should remain empty since there's no message field
    assert!(message_id.is_empty());
}
