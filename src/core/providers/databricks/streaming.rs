//! Databricks Streaming Support
//!
//! Server-Sent Events (SSE) streaming for Databricks chat completions.

use bytes::Bytes;
use futures::{Stream, StreamExt};

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::message::MessageRole;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice, FinishReason};

/// Parse a single SSE line into a ChatChunk
fn parse_sse_line(line: &str) -> Option<Result<ChatChunk, ProviderError>> {
    // Skip empty lines and comments
    if line.is_empty() || line.starts_with(':') {
        return None;
    }

    // Extract data from "data: {json}" format
    let data = if let Some(stripped) = line.strip_prefix("data: ") {
        stripped
    } else if let Some(stripped) = line.strip_prefix("data:") {
        stripped
    } else {
        return None;
    };

    // Skip [DONE] marker
    if data.trim() == "[DONE]" {
        return None;
    }

    // Parse JSON
    match serde_json::from_str::<serde_json::Value>(data) {
        Ok(json) => Some(parse_databricks_chunk(&json)),
        Err(e) => Some(Err(ProviderError::response_parsing(
            "databricks",
            format!("Failed to parse SSE JSON: {}", e),
        ))),
    }
}

/// Parse Databricks streaming response JSON into ChatChunk
fn parse_databricks_chunk(json: &serde_json::Value) -> Result<ChatChunk, ProviderError> {
    let id = json
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("chunk")
        .to_string();

    let created = json
        .get("created")
        .and_then(|v| v.as_i64())
        .unwrap_or_else(|| chrono::Utc::now().timestamp());

    let model = json
        .get("model")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let mut choices = Vec::new();

    if let Some(choices_array) = json.get("choices").and_then(|v| v.as_array()) {
        for choice in choices_array {
            let index = choice.get("index").and_then(|v| v.as_u64()).unwrap_or(0) as u32;

            let delta = if let Some(delta_obj) = choice.get("delta") {
                let role = delta_obj
                    .get("role")
                    .and_then(|v| v.as_str())
                    .map(|r| match r {
                        "assistant" => MessageRole::Assistant,
                        "user" => MessageRole::User,
                        "system" => MessageRole::System,
                        "tool" => MessageRole::Tool,
                        _ => MessageRole::Assistant,
                    });

                // Handle content - could be string or array (for Claude reasoning)
                let content = match delta_obj.get("content") {
                    Some(serde_json::Value::String(s)) => Some(s.clone()),
                    Some(serde_json::Value::Array(arr)) => {
                        // Extract text from content blocks
                        let mut text = String::new();
                        for item in arr {
                            if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                                text.push_str(t);
                            }
                        }
                        if text.is_empty() { None } else { Some(text) }
                    }
                    _ => None,
                };

                ChatDelta {
                    role,
                    content,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                }
            } else {
                ChatDelta {
                    role: None,
                    content: None,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                }
            };

            let finish_reason = choice
                .get("finish_reason")
                .and_then(|v| v.as_str())
                .and_then(|r| match r {
                    "stop" => Some(FinishReason::Stop),
                    "length" => Some(FinishReason::Length),
                    "tool_calls" => Some(FinishReason::ToolCalls),
                    "content_filter" => Some(FinishReason::ContentFilter),
                    _ => None,
                });

            choices.push(ChatStreamChoice {
                index,
                delta,
                finish_reason,
                logprobs: None,
            });
        }
    }

    Ok(ChatChunk {
        id,
        object: "chat.completion.chunk".to_string(),
        created,
        model,
        choices,
        usage: None,
        system_fingerprint: None,
    })
}

/// Create a streaming response from Databricks SSE stream
pub fn create_databricks_stream(
    byte_stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> impl Stream<Item = Result<ChatChunk, ProviderError>> + Send + 'static {
    let mut buffer = String::new();

    byte_stream.flat_map(move |result| {
        let chunks: Vec<Result<ChatChunk, ProviderError>> = match result {
            Ok(bytes) => {
                // Append new bytes to buffer
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    buffer.push_str(text);
                }

                // Process complete lines
                let mut results = Vec::new();
                while let Some(pos) = buffer.find('\n') {
                    let line = buffer[..pos].trim().to_string();
                    buffer = buffer[pos + 1..].to_string();

                    if let Some(chunk_result) = parse_sse_line(&line) {
                        results.push(chunk_result);
                    }
                }
                results
            }
            Err(e) => {
                vec![Err(ProviderError::network(
                    "databricks",
                    format!("Stream error: {}", e),
                ))]
            }
        };

        futures::stream::iter(chunks)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_sse_line_empty() {
        assert!(parse_sse_line("").is_none());
        assert!(parse_sse_line("   ").is_none());
    }

    #[test]
    fn test_parse_sse_line_comment() {
        assert!(parse_sse_line(": comment").is_none());
        assert!(parse_sse_line(":ping").is_none());
    }

    #[test]
    fn test_parse_sse_line_done() {
        assert!(parse_sse_line("data: [DONE]").is_none());
    }

    #[test]
    fn test_parse_sse_line_valid() {
        let line = r#"data: {"id":"chatcmpl-123","object":"chat.completion.chunk","created":1234567890,"model":"dbrx","choices":[{"index":0,"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let result = parse_sse_line(line);
        assert!(result.is_some());

        let chunk = result.unwrap().unwrap();
        assert_eq!(chunk.id, "chatcmpl-123");
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(chunk.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_parse_sse_line_with_role() {
        let line = r#"data: {"id":"test","created":0,"model":"test","choices":[{"index":0,"delta":{"role":"assistant","content":"Hi"},"finish_reason":null}]}"#;
        let result = parse_sse_line(line).unwrap().unwrap();

        assert_eq!(result.choices[0].delta.role, Some(MessageRole::Assistant));
        assert_eq!(result.choices[0].delta.content, Some("Hi".to_string()));
    }

    #[test]
    fn test_parse_sse_line_with_finish_reason() {
        let line = r#"data: {"id":"test","created":0,"model":"test","choices":[{"index":0,"delta":{},"finish_reason":"stop"}]}"#;
        let result = parse_sse_line(line).unwrap().unwrap();

        assert_eq!(result.choices[0].finish_reason, Some(FinishReason::Stop));
    }

    #[test]
    fn test_parse_sse_line_invalid_json() {
        let line = "data: {invalid json}";
        let result = parse_sse_line(line);
        assert!(result.is_some());
        assert!(result.unwrap().is_err());
    }

    #[test]
    fn test_parse_databricks_chunk_minimal() {
        let json: serde_json::Value = serde_json::json!({
            "choices": []
        });

        let result = parse_databricks_chunk(&json).unwrap();
        assert_eq!(result.object, "chat.completion.chunk");
        assert!(result.choices.is_empty());
    }

    #[test]
    fn test_parse_databricks_chunk_with_content() {
        let json: serde_json::Value = serde_json::json!({
            "id": "chunk-123",
            "created": 1700000000,
            "model": "dbrx-instruct",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Hello world"
                }
            }]
        });

        let result = parse_databricks_chunk(&json).unwrap();
        assert_eq!(result.id, "chunk-123");
        assert_eq!(result.model, "dbrx-instruct");
        assert_eq!(result.choices.len(), 1);
        assert_eq!(
            result.choices[0].delta.content,
            Some("Hello world".to_string())
        );
    }

    #[test]
    fn test_parse_databricks_chunk_with_array_content() {
        // Claude-style content array
        let json: serde_json::Value = serde_json::json!({
            "id": "chunk-123",
            "created": 1700000000,
            "model": "claude-3-opus",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": [
                        {"type": "text", "text": "Hello "},
                        {"type": "text", "text": "world"}
                    ]
                }
            }]
        });

        let result = parse_databricks_chunk(&json).unwrap();
        assert_eq!(
            result.choices[0].delta.content,
            Some("Hello world".to_string())
        );
    }
}
