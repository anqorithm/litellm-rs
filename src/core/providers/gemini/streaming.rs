//! Gemini Streaming Module
//!
//! Independent streaming response processing, supporting SSE parsing and real-time data transformation

use std::pin::Pin;
use std::time::{SystemTime, UNIX_EPOCH};

use futures::{Stream, StreamExt};
use pin_project_lite::pin_project;
use reqwest::Response;
use serde_json::Value;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::{
    MessageRole,
    responses::{ChatChunk, ChatDelta, ChatStreamChoice, Usage},
};

use super::error::gemini_stream_error;

/// Get current timestamp in seconds since UNIX_EPOCH.
/// Returns 0 if system time is somehow before UNIX_EPOCH (should never happen).
#[inline]
fn current_timestamp_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// Get current timestamp in nanoseconds for unique ID generation.
/// Returns 0 if system time is somehow before UNIX_EPOCH (should never happen).
#[inline]
fn current_timestamp_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0)
}

/// SSE event types
#[derive(Debug, Clone)]
pub enum GeminiSSEEvent {
    /// Generation start
    GenerateContentResponse(Value),
    /// Error
    Error(Value),
    /// Ping event (heartbeat)
    Ping,
    /// Completion
    Done,
    /// Unknown event
    Unknown(String),
}

/// SSE parser
pub struct GeminiSSEParser;

impl GeminiSSEParser {
    /// Parse SSE line as event
    pub fn parse_event(line: &str) -> Option<GeminiSSEEvent> {
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        if line.starts_with("event:") {
            return None; // Handle event type
        }

        if line.starts_with("data:") {
            let data = line.strip_prefix("data:").unwrap_or("").trim();

            if data == "[DONE]" {
                return Some(GeminiSSEEvent::Done);
            }

            if data.is_empty() {
                return Some(GeminiSSEEvent::Ping);
            }

            // Try to parse JSON
            if let Ok(json) = serde_json::from_str::<Value>(data) {
                // Error response
                if json.get("error").is_some() {
                    return Some(GeminiSSEEvent::Error(json));
                }

                // Generate content response
                if json.get("candidates").is_some() {
                    return Some(GeminiSSEEvent::GenerateContentResponse(json));
                }

                Some(GeminiSSEEvent::Unknown(data.to_string()))
            } else {
                Some(GeminiSSEEvent::Unknown(data.to_string()))
            }
        } else {
            None
        }
    }

    /// Transform to chat chunk
    pub fn transform_to_chat_chunk(
        event: &GeminiSSEEvent,
        model: &str,
        chunk_id: &str,
    ) -> Result<Option<ChatChunk>, ProviderError> {
        match event {
            GeminiSSEEvent::GenerateContentResponse(response) => {
                let empty_candidates = vec![];
                let candidates = response
                    .get("candidates")
                    .and_then(|c| c.as_array())
                    .unwrap_or(&empty_candidates);

                let mut choices = Vec::new();

                for (index, candidate) in candidates.iter().enumerate() {
                    let empty_parts = vec![];
                    let content = candidate
                        .get("content")
                        .and_then(|c| c.get("parts"))
                        .and_then(|p| p.as_array())
                        .unwrap_or(&empty_parts);

                    // Extract text content delta
                    let mut text_parts = Vec::new();
                    for part in content {
                        if let Some(text) = part.get("text").and_then(|t| t.as_str()) {
                            text_parts.push(text);
                        }
                    }
                    let delta_content = text_parts.join("");

                    // Check
                    let finish_reason =
                        candidate
                            .get("finishReason")
                            .and_then(|r| r.as_str())
                            .map(|r| match r {
                                "STOP" => "stop",
                                "MAX_TOKENS" => "length",
                                "SAFETY" => "content_filter",
                                "RECITATION" => "content_filter",
                                _ => "stop",
                            });

                    // Create delta
                    let delta = ChatDelta {
                        role: if !delta_content.is_empty() || finish_reason.is_some() {
                            Some(MessageRole::Assistant)
                        } else {
                            None
                        },
                        content: if delta_content.is_empty() {
                            None
                        } else {
                            Some(delta_content)
                        },
                        thinking: None,
                        function_call: None,
                        tool_calls: None,
                    };

                    choices.push(ChatStreamChoice {
                        index: index as u32,
                        delta,
                        finish_reason: finish_reason.map(|s| match s {
                            "stop" => crate::core::types::responses::FinishReason::Stop,
                            "length" => crate::core::types::responses::FinishReason::Length,
                            "content_filter" => {
                                crate::core::types::responses::FinishReason::ContentFilter
                            }
                            _ => crate::core::types::responses::FinishReason::Stop,
                        }),
                        logprobs: None,
                    });
                }

                // Extract usage stats (if available)
                let usage = response.get("usageMetadata").map(|usage_metadata| Usage {
                    prompt_tokens: usage_metadata
                        .get("promptTokenCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    completion_tokens: usage_metadata
                        .get("candidatesTokenCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    total_tokens: usage_metadata
                        .get("totalTokenCount")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0) as u32,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                    thinking_usage: None,
                });

                if choices.is_empty() && usage.is_none() {
                    return Ok(None); // Skip empty chunk
                }

                Ok(Some(ChatChunk {
                    id: chunk_id.to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: current_timestamp_secs(),
                    model: model.to_string(),
                    choices,
                    usage,
                    system_fingerprint: None,
                }))
            }
            GeminiSSEEvent::Error(error) => {
                Err(super::error::GeminiErrorMapper::from_api_response(error))
            }
            GeminiSSEEvent::Done => {
                // Send final empty chunk to indicate completion
                Ok(Some(ChatChunk {
                    id: chunk_id.to_string(),
                    object: "chat.completion.chunk".to_string(),
                    created: current_timestamp_secs(),
                    model: model.to_string(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                            thinking: None,
                            function_call: None,
                            tool_calls: None,
                        },
                        finish_reason: Some(crate::core::types::responses::FinishReason::Stop),
                        logprobs: None,
                    }],
                    usage: None,
                    system_fingerprint: None,
                }))
            }
            GeminiSSEEvent::Ping => Ok(None), // Skip ping events
            GeminiSSEEvent::Unknown(_) => Ok(None), // Skip unknown events
        }
    }
}

pin_project! {
    /// Handle streaming response
    pub struct GeminiStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>,
    }
}

impl GeminiStream {
    /// Create from response
    pub fn from_response(response: Response, model: String) -> Self {
        let chunk_id = format!("gemini-stream-{}", current_timestamp_nanos());

        let stream = futures::stream::unfold(
            (response.bytes_stream(), String::new(), chunk_id, model),
            |state| async move {
                let (mut lines, mut buffer, chunk_id, model) = state;

                // Handle line buffer
                if let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim_end_matches('\r').to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if let Some(event) = GeminiSSEParser::parse_event(&line) {
                        match GeminiSSEParser::transform_to_chat_chunk(&event, &model, &chunk_id) {
                            Ok(Some(chunk)) => {
                                let new_state = (lines, buffer, chunk_id, model);
                                return Some((Ok(chunk), new_state));
                            }
                            Ok(None) => {
                                // Handle continuation
                                let chunk_id_cloned = chunk_id.clone();
                                let model_cloned = model.clone();
                                let new_state = (lines, buffer, chunk_id, model);
                                return Some((
                                    Ok(ChatChunk {
                                        id: chunk_id_cloned,
                                        object: "chat.completion.chunk".to_string(),
                                        created: std::time::SystemTime::now()
                                            .duration_since(std::time::UNIX_EPOCH)
                                            .unwrap()
                                            .as_secs()
                                            as i64,
                                        model: model_cloned,
                                        choices: vec![],
                                        usage: None,
                                        system_fingerprint: None,
                                    }),
                                    new_state,
                                ));
                            }
                            Err(e) => {
                                let new_state = (lines, buffer, chunk_id, model);
                                return Some((Err(e), new_state));
                            }
                        }
                    }
                }

                // Read more data
                match lines.next().await {
                    Some(Ok(bytes)) => {
                        let text = String::from_utf8_lossy(&bytes);
                        buffer.push_str(&text);
                        let chunk_id_cloned = chunk_id.clone();
                        let model_cloned = model.clone();
                        let new_state = (lines, buffer, chunk_id, model);

                        // Handle stream data
                        Some((
                            Ok(ChatChunk {
                                id: chunk_id_cloned,
                                object: "chat.completion.chunk".to_string(),
                                created: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs() as i64,
                                model: model_cloned,
                                choices: vec![],
                                usage: None,
                                system_fingerprint: None,
                            }),
                            new_state,
                        ))
                    }
                    Some(Err(e)) => {
                        let new_state = (lines, buffer, chunk_id, model);
                        Some((
                            Err(gemini_stream_error(format!("Stream read error: {}", e))),
                            new_state,
                        ))
                    }
                    None => {
                        // Handle final buffer
                        if !buffer.trim().is_empty() {
                            if let Some(event) = GeminiSSEParser::parse_event(&buffer) {
                                let new_state = (lines, buffer, chunk_id, model);
                                match GeminiSSEParser::transform_to_chat_chunk(
                                    &event,
                                    &new_state.3,
                                    &new_state.2,
                                ) {
                                    Ok(Some(chunk)) => Some((Ok(chunk), new_state)),
                                    Ok(None) => None,
                                    Err(e) => Some((Err(e), new_state)),
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                }
            },
        );

        Self {
            inner: Box::pin(stream),
        }
    }

    /// Create from test data
    #[cfg(test)]
    pub fn from_test_data(data: Vec<String>, model: String) -> Self {
        let chunk_id = format!(
            "gemini-test-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let stream = futures::stream::iter(data)
            .then(move |line| {
                let model = model.clone();
                let chunk_id = chunk_id.clone();
                async move {
                    if let Some(event) = GeminiSSEParser::parse_event(&line) {
                        match GeminiSSEParser::transform_to_chat_chunk(&event, &model, &chunk_id) {
                            Ok(Some(chunk)) => Some(Ok(chunk)),
                            Ok(None) => None,
                            Err(e) => Some(Err(e)),
                        }
                    } else {
                        None
                    }
                }
            })
            .filter_map(|item| async move { item });

        Self {
            inner: Box::pin(stream),
        }
    }
}

impl Stream for GeminiStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.inner.poll_next(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sse_parsing() {
        let line = r#"data: {"candidates": [{"content": {"parts": [{"text": "Hello"}]}, "finishReason": null}]}"#;
        let event = GeminiSSEParser::parse_event(line);

        assert!(event.is_some());
        match event.unwrap() {
            GeminiSSEEvent::GenerateContentResponse(response) => {
                assert!(response.get("candidates").is_some());
            }
            _ => panic!("Expected GenerateContentResponse"),
        }
    }

    #[test]
    fn test_done_parsing() {
        let line = "data: [DONE]";
        let event = GeminiSSEParser::parse_event(line);

        assert!(event.is_some());
        assert!(matches!(event.unwrap(), GeminiSSEEvent::Done));
    }

    #[test]
    fn test_error_parsing() {
        let line = r#"data: {"error": {"code": 400, "message": "Bad request"}}"#;
        let event = GeminiSSEParser::parse_event(line);

        assert!(event.is_some());
        match event.unwrap() {
            GeminiSSEEvent::Error(error) => {
                assert!(error.get("error").is_some());
            }
            _ => panic!("Expected Error"),
        }
    }

    #[test]
    fn test_chunk_transformation() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Hello, world!"}]
                },
                "finishReason": null
            }],
            "usageMetadata": {
                "promptTokenCount": 10,
                "candidatesTokenCount": 5,
                "totalTokenCount": 15
            }
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(chunk.model, "gemini-pro");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            chunk.choices[0].delta.content.as_ref().unwrap(),
            "Hello, world!"
        );
        assert!(chunk.usage.is_some());
    }

    #[tokio::test]
    async fn test_stream_creation() {
        let test_data = vec![
            r#"data: {"candidates": [{"content": {"parts": [{"text": "Hello"}]}}]}"#.to_string(),
            r#"data: {"candidates": [{"content": {"parts": [{"text": " world!"}]}}]}"#.to_string(),
            "data: [DONE]".to_string(),
        ];

        let stream = GeminiStream::from_test_data(test_data, "gemini-pro".to_string());
        let chunks: Vec<_> = stream.collect().await;

        assert_eq!(chunks.len(), 3); // 2 content chunks + 1 completion chunk

        // Check results
        assert!(chunks[0].is_ok());
        assert!(chunks[1].is_ok());
        assert!(chunks[2].is_ok());

        let chunk1 = chunks[0].as_ref().unwrap();
        let chunk2 = chunks[1].as_ref().unwrap();
        let chunk3 = chunks[2].as_ref().unwrap();

        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");
        assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), " world!");
        assert_eq!(
            chunk3.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::Stop
        );
    }

    #[test]
    fn test_sse_empty_line() {
        let event = GeminiSSEParser::parse_event("");
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_comment_line() {
        let event = GeminiSSEParser::parse_event(": this is a comment");
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_event_type_line() {
        let event = GeminiSSEParser::parse_event("event: message");
        assert!(event.is_none());
    }

    #[test]
    fn test_sse_ping_event() {
        let event = GeminiSSEParser::parse_event("data: ");
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), GeminiSSEEvent::Ping));
    }

    #[test]
    fn test_sse_unknown_json() {
        let line = r#"data: {"unknown_field": "value"}"#;
        let event = GeminiSSEParser::parse_event(line);
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), GeminiSSEEvent::Unknown(_)));
    }

    #[test]
    fn test_sse_invalid_json() {
        let line = "data: not valid json";
        let event = GeminiSSEParser::parse_event(line);
        assert!(event.is_some());
        assert!(matches!(event.unwrap(), GeminiSSEEvent::Unknown(_)));
    }

    #[test]
    fn test_chunk_transformation_with_finish_reason_stop() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "Done."}]
                },
                "finishReason": "STOP"
            }]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(
            chunk.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::Stop
        );
    }

    #[test]
    fn test_chunk_transformation_with_finish_reason_max_tokens() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "..."}]
                },
                "finishReason": "MAX_TOKENS"
            }]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(
            chunk.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::Length
        );
    }

    #[test]
    fn test_chunk_transformation_with_finish_reason_safety() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "..."}]
                },
                "finishReason": "SAFETY"
            }]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(
            chunk.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::ContentFilter
        );
    }

    #[test]
    fn test_chunk_transformation_with_finish_reason_recitation() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [{"text": "..."}]
                },
                "finishReason": "RECITATION"
            }]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(
            chunk.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::ContentFilter
        );
    }

    #[test]
    fn test_chunk_transformation_ping_event() {
        let event = GeminiSSEEvent::Ping;
        let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_chunk_transformation_unknown_event() {
        let event = GeminiSSEEvent::Unknown("some data".to_string());
        let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_chunk_transformation_done_event() {
        let event = GeminiSSEEvent::Done;
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            chunk.choices[0].finish_reason.as_ref().unwrap(),
            &crate::core::types::responses::FinishReason::Stop
        );
    }

    #[test]
    fn test_chunk_transformation_empty_candidates() {
        let response = json!({
            "candidates": []
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let result = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none()); // Empty candidates should be skipped
    }

    #[test]
    fn test_chunk_transformation_multiple_parts() {
        let response = json!({
            "candidates": [{
                "content": {
                    "parts": [
                        {"text": "Hello"},
                        {"text": " "},
                        {"text": "world"}
                    ]
                }
            }]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(
            chunk.choices[0].delta.content.as_ref().unwrap(),
            "Hello world"
        );
    }

    #[test]
    fn test_chunk_transformation_multiple_candidates() {
        let response = json!({
            "candidates": [
                {
                    "content": {
                        "parts": [{"text": "Response 1"}]
                    }
                },
                {
                    "content": {
                        "parts": [{"text": "Response 2"}]
                    }
                }
            ]
        });

        let event = GeminiSSEEvent::GenerateContentResponse(response);
        let chunk = GeminiSSEParser::transform_to_chat_chunk(&event, "gemini-pro", "test-id")
            .unwrap()
            .unwrap();

        assert_eq!(chunk.choices.len(), 2);
        assert_eq!(chunk.choices[0].index, 0);
        assert_eq!(chunk.choices[1].index, 1);
        assert_eq!(
            chunk.choices[0].delta.content.as_ref().unwrap(),
            "Response 1"
        );
        assert_eq!(
            chunk.choices[1].delta.content.as_ref().unwrap(),
            "Response 2"
        );
    }

    #[test]
    fn test_current_timestamp_secs() {
        let ts = current_timestamp_secs();
        assert!(ts > 0);
    }

    #[test]
    fn test_current_timestamp_nanos() {
        let ts = current_timestamp_nanos();
        assert!(ts > 0);
    }
}
