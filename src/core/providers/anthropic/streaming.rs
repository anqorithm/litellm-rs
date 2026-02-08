//! Anthropic Streaming Module
//!
//! Independent streaming response processing with SSE parsing and real-time data conversion

use std::pin::Pin;

use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use reqwest::Response;
use serde_json::Value;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    message::MessageRole,
    responses::{ChatChunk, ChatDelta, ChatStreamChoice, Usage},
};

use super::error::anthropic_stream_error;

/// SSE event types
#[derive(Debug, Clone)]
pub enum SSEEvent {
    /// Message start
    MessageStart(Value),
    /// Content block start
    ContentBlockStart(Value),
    /// Content block delta
    ContentBlockDelta(Value),
    /// Content block stop
    ContentBlockStop(Value),
    /// Message delta
    MessageDelta(Value),
    /// Message stop
    MessageStop(Value),
    /// Error event
    Error(Value),
    /// Ping event (heartbeat)
    Ping,
    /// Unknown event
    Unknown(String),
}

/// SSE parser
pub struct SSEParser;

impl SSEParser {
    /// Parse SSE line to event
    pub fn parse_event(line: &str) -> Option<SSEEvent> {
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        if line.starts_with("event:") {
            return None; // Already handled by event type
        }

        if line.starts_with("data:") {
            let data = line.strip_prefix("data:").unwrap_or("").trim();

            if data == "[DONE]" {
                return None;
            }

            if data.is_empty() {
                return Some(SSEEvent::Ping);
            }

            // Try to parse JSON
            if let Ok(json) = serde_json::from_str::<Value>(data) {
                let event_type = json.get("type").and_then(|t| t.as_str()).unwrap_or("");

                match event_type {
                    "message_start" => Some(SSEEvent::MessageStart(json)),
                    "content_block_start" => Some(SSEEvent::ContentBlockStart(json)),
                    "content_block_delta" => Some(SSEEvent::ContentBlockDelta(json)),
                    "content_block_stop" => Some(SSEEvent::ContentBlockStop(json)),
                    "message_delta" => Some(SSEEvent::MessageDelta(json)),
                    "message_stop" => Some(SSEEvent::MessageStop(json)),
                    "error" => Some(SSEEvent::Error(json)),
                    _ => Some(SSEEvent::Unknown(event_type.to_string())),
                }
            } else {
                Some(SSEEvent::Unknown(data.to_string()))
            }
        } else {
            None
        }
    }
}

pin_project! {
    /// Anthropic streaming processor
    pub struct AnthropicStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>,
    }
}

impl AnthropicStream {
    /// Create stream from response
    pub fn from_response(response: Response, model: String) -> Self {
        let stream = async_stream::stream! {
            let mut response_stream = response.bytes_stream();
            let mut buffer = String::new();
            let mut message_id = String::new();
            let created_time = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            while let Some(chunk_result) = response_stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        let chunk_str = String::from_utf8_lossy(&chunk);
                        buffer.push_str(&chunk_str);

                        // Handle lines
                        while let Some(newline_pos) = buffer.find('\n') {
                            let line = buffer[..newline_pos].trim().to_string();
                            buffer = buffer[newline_pos + 1..].to_string();

                            if let Some(event) = SSEParser::parse_event(&line) {
                                match Self::process_event(event, &model, &mut message_id, created_time) {
                                    Ok(Some(chat_chunk)) => yield Ok(chat_chunk),
                                    Ok(None) => continue,
                                    Err(e) => yield Err(e),
                                }
                            }
                        }
                    }
                    Err(e) => {
                        yield Err(anthropic_stream_error(format!("Stream error: {}", e)));
                        break;
                    }
                }
            }
        };

        Self {
            inner: Box::pin(stream),
        }
    }

    /// Process SSE event
    fn process_event(
        event: SSEEvent,
        model: &str,
        message_id: &mut String,
        created_time: i64,
    ) -> Result<Option<ChatChunk>, ProviderError> {
        match event {
            SSEEvent::MessageStart(data) => {
                // Extract message ID
                if let Some(message) = data.get("message") {
                    if let Some(id) = message.get("id").and_then(|v| v.as_str()) {
                        *message_id = id.to_string();
                    }
                }

                Ok(Some(ChatChunk {
                    id: message_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: created_time,
                    model: model.to_string(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: Some(MessageRole::Assistant),
                            content: None,
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                    system_fingerprint: None,
                }))
            }

            SSEEvent::ContentBlockDelta(data) => {
                let content = data
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                Ok(Some(ChatChunk {
                    id: message_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: created_time,
                    model: model.to_string(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(content.to_string()),
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                    system_fingerprint: None,
                }))
            }

            SSEEvent::MessageDelta(data) => {
                // Extract usage information and stop_reason
                let usage = data.get("usage").map(|u| Usage {
                    prompt_tokens: u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                        as u32,
                    completion_tokens: u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                        as u32,
                    total_tokens: (u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0)
                        + u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0))
                        as u32,
                    completion_tokens_details: None,
                    prompt_tokens_details: None,
                    thinking_usage: None,
                });

                let finish_reason = data
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|r| r.as_str())
                    .map(|reason| match reason {
                        "end_turn" => crate::core::types::responses::FinishReason::Stop,
                        "max_tokens" => crate::core::types::responses::FinishReason::Length,
                        "tool_use" => crate::core::types::responses::FinishReason::ToolCalls,
                        _ => crate::core::types::responses::FinishReason::Stop,
                    });

                Ok(Some(ChatChunk {
                    id: message_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: created_time,
                    model: model.to_string(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason,
                        logprobs: None,
                    }],
                    usage,
                    system_fingerprint: None,
                }))
            }

            SSEEvent::MessageStop(_) => {
                // Final end chunk
                Ok(Some(ChatChunk {
                    id: message_id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: created_time,
                    model: model.to_string(),
                    choices: vec![],
                    usage: None,
                    system_fingerprint: None,
                }))
            }

            SSEEvent::ContentBlockStart(_) | SSEEvent::ContentBlockStop(_) => {
                // These events don't need to generate chunks
                Ok(None)
            }

            SSEEvent::Error(error_data) => {
                let error_message = error_data
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown streaming error");

                Err(anthropic_stream_error(error_message))
            }

            SSEEvent::Ping => Ok(None),

            SSEEvent::Unknown(_) => Ok(None),
        }
    }
}

impl Stream for AnthropicStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

/// Streaming utilities
pub struct StreamUtils;

impl StreamUtils {
    /// Collect stream to response
    pub async fn collect_stream_to_response(
        mut stream: AnthropicStream,
    ) -> Result<crate::core::types::responses::ChatResponse, ProviderError> {
        let mut content_parts = Vec::new();
        let mut final_usage = None;
        let mut response_id = String::new();
        let mut model = String::new();
        let mut created = 0;

        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if response_id.is_empty() {
                        response_id = chunk.id.clone();
                        model = chunk.model.clone();
                        created = chunk.created;
                    }

                    for choice in chunk.choices {
                        if let Some(content) = choice.delta.content {
                            content_parts.push(content);
                        }
                    }

                    if let Some(usage) = chunk.usage {
                        final_usage = Some(usage);
                    }
                }
                Err(e) => return Err(e),
            }
        }

        let final_content = content_parts.join("");
        let message = crate::core::types::ChatMessage {
            role: MessageRole::Assistant,
            content: if final_content.is_empty() {
                None
            } else {
                Some(crate::core::types::message::MessageContent::Text(
                    final_content,
                ))
            },
            thinking: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        };

        let choice = crate::core::types::responses::ChatChoice {
            index: 0,
            message,
            finish_reason: Some(crate::core::types::responses::FinishReason::Stop),
            logprobs: None,
        };

        Ok(crate::core::types::responses::ChatResponse {
            id: response_id,
            object: "chat.completion".to_string(),
            created,
            model,
            choices: vec![choice],
            usage: final_usage,
            system_fingerprint: None,
        })
    }

    /// Validate stream chunk
    pub fn validate_stream_chunk(chunk: &ChatChunk) -> Result<(), ProviderError> {
        if chunk.id.is_empty() {
            return Err(anthropic_stream_error("Missing chunk ID"));
        }

        if chunk.model.is_empty() {
            return Err(anthropic_stream_error("Missing model in chunk"));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
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
}
