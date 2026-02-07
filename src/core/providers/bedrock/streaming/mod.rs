//! Streaming Module for Bedrock
//!
//! Handles AWS Event Stream parsing and streaming responses

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;
use bytes::Bytes;
use futures::{Stream, StreamExt};
use serde_json::Value;
use std::pin::Pin;
use std::task::{Context, Poll};

/// AWS Event Stream message
#[derive(Debug)]
pub struct EventStreamMessage {
    pub headers: Vec<EventStreamHeader>,
    pub payload: Bytes,
}

/// Event stream header
#[derive(Debug)]
pub struct EventStreamHeader {
    pub name: String,
    pub value: HeaderValue,
}

/// Header value types
#[derive(Debug)]
pub enum HeaderValue {
    String(String),
    ByteArray(Vec<u8>),
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    UUID(String),
    Timestamp(i64),
}

/// Bedrock streaming response
pub struct BedrockStream {
    inner: Pin<Box<dyn Stream<Item = Result<Bytes, ProviderError>> + Send>>,
    buffer: Vec<u8>,
    model_family: crate::core::providers::bedrock::model_config::BedrockModelFamily,
}

impl BedrockStream {
    /// Create a new Bedrock stream
    pub fn new(
        stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
        model_family: crate::core::providers::bedrock::model_config::BedrockModelFamily,
    ) -> Self {
        let mapped_stream = stream
            .map(|result| result.map_err(|e| ProviderError::network("bedrock", e.to_string())));

        Self {
            inner: Box::pin(mapped_stream),
            buffer: Vec::new(),
            model_family,
        }
    }

    /// Parse event stream message from bytes
    fn parse_event_message(data: &[u8]) -> Result<EventStreamMessage, ProviderError> {
        if data.len() < 16 {
            return Err(ProviderError::response_parsing(
                "bedrock",
                "Invalid event stream message",
            ));
        }

        // Parse prelude (12 bytes)
        let total_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let headers_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
        // let prelude_crc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        if data.len() < total_length {
            return Err(ProviderError::response_parsing(
                "bedrock",
                "Incomplete event stream message",
            ));
        }

        // Parse headers
        let mut headers = Vec::new();
        let mut offset = 12;
        let headers_end = 12 + headers_length;

        while offset < headers_end {
            if offset + 1 > data.len() {
                break;
            }

            let name_length = data[offset] as usize;
            offset += 1;

            if offset + name_length > data.len() {
                break;
            }

            let name = String::from_utf8_lossy(&data[offset..offset + name_length]).to_string();
            offset += name_length;

            if offset >= data.len() {
                break;
            }

            let header_type = data[offset];
            offset += 1;

            let value = match header_type {
                5 => {
                    // String type
                    if offset + 2 > data.len() {
                        break;
                    }
                    let string_length =
                        u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
                    offset += 2;
                    if offset + string_length > data.len() {
                        break;
                    }
                    let string_value =
                        String::from_utf8_lossy(&data[offset..offset + string_length]).to_string();
                    offset += string_length;
                    HeaderValue::String(string_value)
                }
                _ => {
                    // Skip unknown header types
                    HeaderValue::String(String::new())
                }
            };

            headers.push(EventStreamHeader { name, value });
        }

        // Extract payload
        let payload_start = headers_end;
        let payload_end = total_length - 4; // Exclude message CRC
        let payload = if payload_start < payload_end && payload_end <= data.len() {
            Bytes::copy_from_slice(&data[payload_start..payload_end])
        } else {
            Bytes::new()
        };

        Ok(EventStreamMessage { headers, payload })
    }

    /// Parse chunk based on model family
    fn parse_chunk(&self, payload: &[u8]) -> Result<Option<ChatChunk>, ProviderError> {
        let json_str = String::from_utf8_lossy(payload);
        let value: Value = serde_json::from_str(&json_str)
            .map_err(|e| ProviderError::response_parsing("bedrock", e.to_string()))?;

        // Parse based on model family
        match self.model_family {
            crate::core::providers::bedrock::model_config::BedrockModelFamily::Claude => {
                self.parse_claude_chunk(&value)
            }
            crate::core::providers::bedrock::model_config::BedrockModelFamily::Nova => {
                self.parse_nova_chunk(&value)
            }
            crate::core::providers::bedrock::model_config::BedrockModelFamily::TitanText => {
                self.parse_titan_chunk(&value)
            }
            _ => {
                // Generic parsing for other models
                self.parse_generic_chunk(&value)
            }
        }
    }

    /// Parse Claude streaming chunk
    fn parse_claude_chunk(&self, value: &Value) -> Result<Option<ChatChunk>, ProviderError> {
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice};

        // Claude uses specific event types
        let event_type = value.get("type").and_then(|v| v.as_str());

        match event_type {
            Some("content_block_delta") => {
                let delta = value
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                Ok(Some(ChatChunk {
                    id: format!("bedrock-{}", uuid::Uuid::new_v4()),
                    object: "chat.completion.chunk".to_string(),
                    created: chrono::Utc::now().timestamp(),
                    model: String::new(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(delta.to_string()),
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
            Some("message_stop") => Ok(Some(ChatChunk {
                id: format!("bedrock-{}", uuid::Uuid::new_v4()),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: String::new(),
                choices: vec![ChatStreamChoice {
                    index: 0,
                    delta: ChatDelta {
                        role: None,
                        content: None,
                        thinking: None,
                        tool_calls: None,
                        function_call: None,
                    },
                    finish_reason: Some(crate::core::types::responses::FinishReason::Stop),
                    logprobs: None,
                }],
                usage: None,
                system_fingerprint: None,
            })),
            _ => Ok(None),
        }
    }

    /// Parse Nova streaming chunk
    fn parse_nova_chunk(&self, value: &Value) -> Result<Option<ChatChunk>, ProviderError> {
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice};

        if let Some(content) = value
            .get("contentBlockDelta")
            .and_then(|c| c.get("delta"))
            .and_then(|d| d.get("text"))
            .and_then(|t| t.as_str())
        {
            Ok(Some(ChatChunk {
                id: format!("bedrock-{}", uuid::Uuid::new_v4()),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: String::new(),
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
        } else {
            Ok(None)
        }
    }

    /// Parse Titan streaming chunk
    fn parse_titan_chunk(&self, value: &Value) -> Result<Option<ChatChunk>, ProviderError> {
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice};

        if let Some(content) = value.get("outputText").and_then(|t| t.as_str()) {
            Ok(Some(ChatChunk {
                id: format!("bedrock-{}", uuid::Uuid::new_v4()),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: String::new(),
                choices: vec![ChatStreamChoice {
                    index: 0,
                    delta: ChatDelta {
                        role: None,
                        content: Some(content.to_string()),
                        thinking: None,
                        tool_calls: None,
                        function_call: None,
                    },
                    finish_reason: if value.get("completionReason").is_some() {
                        Some(crate::core::types::responses::FinishReason::Stop)
                    } else {
                        None
                    },
                    logprobs: None,
                }],
                usage: None,
                system_fingerprint: None,
            }))
        } else {
            Ok(None)
        }
    }

    /// Parse generic streaming chunk
    fn parse_generic_chunk(&self, value: &Value) -> Result<Option<ChatChunk>, ProviderError> {
        use crate::core::types::responses::{ChatDelta, ChatStreamChoice};

        // Try to find content in common locations
        let content = value
            .get("completion")
            .or_else(|| value.get("generation"))
            .or_else(|| value.get("text"))
            .and_then(|t| t.as_str());

        if let Some(text) = content {
            Ok(Some(ChatChunk {
                id: format!("bedrock-{}", uuid::Uuid::new_v4()),
                object: "chat.completion.chunk".to_string(),
                created: chrono::Utc::now().timestamp(),
                model: String::new(),
                choices: vec![ChatStreamChoice {
                    index: 0,
                    delta: ChatDelta {
                        role: None,
                        content: Some(text.to_string()),
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
        } else {
            Ok(None)
        }
    }
}

impl Stream for BedrockStream {
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // Poll the inner stream for more data
        match self.inner.as_mut().poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                // Add bytes to buffer
                self.buffer.extend_from_slice(&bytes);

                // Try to parse an event message
                if self.buffer.len() >= 16 {
                    // Check if we have a complete message
                    let total_length = u32::from_be_bytes([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        self.buffer[3],
                    ]) as usize;

                    if self.buffer.len() >= total_length {
                        // Extract the message
                        let message_data = self.buffer[..total_length].to_vec();
                        self.buffer.drain(..total_length);

                        // Parse the message
                        match Self::parse_event_message(&message_data) {
                            Ok(message) => {
                                // Parse the payload as a chunk
                                match self.parse_chunk(&message.payload) {
                                    Ok(Some(chunk)) => Poll::Ready(Some(Ok(chunk))),
                                    Ok(None) => {
                                        // No chunk from this message, poll again
                                        cx.waker().wake_by_ref();
                                        Poll::Pending
                                    }
                                    Err(e) => Poll::Ready(Some(Err(e))),
                                }
                            }
                            Err(e) => Poll::Ready(Some(Err(e))),
                        }
                    } else {
                        // Need more data
                        cx.waker().wake_by_ref();
                        Poll::Pending
                    }
                } else {
                    // Need more data
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(e))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::bedrock::model_config::BedrockModelFamily;

    // ==================== HeaderValue Tests ====================

    #[test]
    fn test_header_value_string() {
        let value = HeaderValue::String("test".to_string());
        assert!(matches!(value, HeaderValue::String(_)));
    }

    #[test]
    fn test_header_value_byte_array() {
        let value = HeaderValue::ByteArray(vec![1, 2, 3]);
        assert!(matches!(value, HeaderValue::ByteArray(_)));
    }

    #[test]
    fn test_header_value_boolean() {
        let value = HeaderValue::Boolean(true);
        assert!(matches!(value, HeaderValue::Boolean(true)));
    }

    #[test]
    fn test_header_value_numeric_types() {
        let _ = HeaderValue::Byte(1);
        let _ = HeaderValue::Short(256);
        let _ = HeaderValue::Integer(65536);
        let _ = HeaderValue::Long(1_000_000_000);
        let _ = HeaderValue::Timestamp(1234567890);
    }

    #[test]
    fn test_header_value_uuid() {
        let value = HeaderValue::UUID("550e8400-e29b-41d4-a716-446655440000".to_string());
        assert!(matches!(value, HeaderValue::UUID(_)));
    }

    // ==================== EventStreamHeader Tests ====================

    #[test]
    fn test_event_stream_header() {
        let header = EventStreamHeader {
            name: ":message-type".to_string(),
            value: HeaderValue::String("event".to_string()),
        };
        assert_eq!(header.name, ":message-type");
    }

    // ==================== EventStreamMessage Tests ====================

    #[test]
    fn test_event_stream_message() {
        let message = EventStreamMessage {
            headers: vec![EventStreamHeader {
                name: ":event-type".to_string(),
                value: HeaderValue::String("chunk".to_string()),
            }],
            payload: Bytes::from(r#"{"text": "hello"}"#),
        };
        assert_eq!(message.headers.len(), 1);
        assert!(!message.payload.is_empty());
    }

    // ==================== parse_event_message Tests ====================

    #[test]
    fn test_parse_event_message_too_short() {
        let data = vec![0, 0, 0, 0, 0, 0, 0, 0]; // Only 8 bytes
        let result = BedrockStream::parse_event_message(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_event_message_incomplete() {
        // total_length says 100 but we only have 20 bytes
        let mut data = vec![0u8; 20];
        data[0..4].copy_from_slice(&100u32.to_be_bytes()); // total_length = 100
        data[4..8].copy_from_slice(&0u32.to_be_bytes()); // headers_length = 0

        let result = BedrockStream::parse_event_message(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_event_message_minimal() {
        // Minimum valid message:
        // - 4 bytes: total_length (16 bytes min for prelude + 4 for CRC = 20 if no headers/payload)
        // - 4 bytes: headers_length
        // - 4 bytes: prelude CRC
        // - (headers if any)
        // - (payload if any)
        // - 4 bytes: message CRC
        //
        // For a minimal message with no headers and no payload:
        // total_length = 12 (prelude) + 4 (message CRC) = 16
        let total_length: u32 = 16;
        let headers_length: u32 = 0;
        let prelude_crc: u32 = 0;
        let message_crc: u32 = 0;

        let mut data = Vec::new();
        data.extend_from_slice(&total_length.to_be_bytes());
        data.extend_from_slice(&headers_length.to_be_bytes());
        data.extend_from_slice(&prelude_crc.to_be_bytes());
        data.extend_from_slice(&message_crc.to_be_bytes());

        let result = BedrockStream::parse_event_message(&data);
        assert!(result.is_ok());

        let message = result.unwrap();
        assert!(message.headers.is_empty());
        // Payload is from headers_end (12 + 0 = 12) to total_length - 4 (16 - 4 = 12)
        // So payload start == payload end, meaning empty payload
        assert!(message.payload.is_empty());
    }

    // ==================== Claude Chunk Parsing Tests ====================

    fn create_test_stream_claude() -> BedrockStream {
        let stream = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        BedrockStream::new(stream, BedrockModelFamily::Claude)
    }

    #[test]
    fn test_parse_claude_content_block_delta() {
        let stream = create_test_stream_claude();
        let json = serde_json::json!({
            "type": "content_block_delta",
            "delta": {
                "text": "Hello, world!"
            }
        });

        let result = stream.parse_claude_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());

        let chunk = chunk.unwrap();
        assert_eq!(chunk.choices.len(), 1);
        assert_eq!(
            chunk.choices[0].delta.content,
            Some("Hello, world!".to_string())
        );
    }

    #[test]
    fn test_parse_claude_message_stop() {
        let stream = create_test_stream_claude();
        let json = serde_json::json!({
            "type": "message_stop"
        });

        let result = stream.parse_claude_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());

        let chunk = chunk.unwrap();
        assert!(chunk.choices[0].finish_reason.is_some());
    }

    #[test]
    fn test_parse_claude_unknown_event() {
        let stream = create_test_stream_claude();
        let json = serde_json::json!({
            "type": "message_start"
        });

        let result = stream.parse_claude_chunk(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_parse_claude_empty_delta() {
        let stream = create_test_stream_claude();
        let json = serde_json::json!({
            "type": "content_block_delta",
            "delta": {}
        });

        let result = stream.parse_claude_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("".to_string())
        );
    }

    // ==================== Nova Chunk Parsing Tests ====================

    fn create_test_stream_nova() -> BedrockStream {
        let stream = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        BedrockStream::new(stream, BedrockModelFamily::Nova)
    }

    #[test]
    fn test_parse_nova_content_block_delta() {
        let stream = create_test_stream_nova();
        let json = serde_json::json!({
            "contentBlockDelta": {
                "delta": {
                    "text": "Nova response"
                }
            }
        });

        let result = stream.parse_nova_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("Nova response".to_string())
        );
    }

    #[test]
    fn test_parse_nova_no_content() {
        let stream = create_test_stream_nova();
        let json = serde_json::json!({
            "messageStart": {}
        });

        let result = stream.parse_nova_chunk(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ==================== Titan Chunk Parsing Tests ====================

    fn create_test_stream_titan() -> BedrockStream {
        let stream = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        BedrockStream::new(stream, BedrockModelFamily::TitanText)
    }

    #[test]
    fn test_parse_titan_output_text() {
        let stream = create_test_stream_titan();
        let json = serde_json::json!({
            "outputText": "Titan response"
        });

        let result = stream.parse_titan_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("Titan response".to_string())
        );
    }

    #[test]
    fn test_parse_titan_with_completion_reason() {
        let stream = create_test_stream_titan();
        let json = serde_json::json!({
            "outputText": "Final text",
            "completionReason": "FINISH"
        });

        let result = stream.parse_titan_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert!(chunk.unwrap().choices[0].finish_reason.is_some());
    }

    #[test]
    fn test_parse_titan_no_output() {
        let stream = create_test_stream_titan();
        let json = serde_json::json!({
            "usage": {
                "inputTokens": 10
            }
        });

        let result = stream.parse_titan_chunk(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ==================== Generic Chunk Parsing Tests ====================

    fn create_test_stream_generic() -> BedrockStream {
        let stream = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        BedrockStream::new(stream, BedrockModelFamily::Mistral)
    }

    #[test]
    fn test_parse_generic_completion() {
        let stream = create_test_stream_generic();
        let json = serde_json::json!({
            "completion": "Generic completion"
        });

        let result = stream.parse_generic_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("Generic completion".to_string())
        );
    }

    #[test]
    fn test_parse_generic_generation() {
        let stream = create_test_stream_generic();
        let json = serde_json::json!({
            "generation": "Generated text"
        });

        let result = stream.parse_generic_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("Generated text".to_string())
        );
    }

    #[test]
    fn test_parse_generic_text() {
        let stream = create_test_stream_generic();
        let json = serde_json::json!({
            "text": "Simple text"
        });

        let result = stream.parse_generic_chunk(&json);
        assert!(result.is_ok());

        let chunk = result.unwrap();
        assert!(chunk.is_some());
        assert_eq!(
            chunk.unwrap().choices[0].delta.content,
            Some("Simple text".to_string())
        );
    }

    #[test]
    fn test_parse_generic_no_content() {
        let stream = create_test_stream_generic();
        let json = serde_json::json!({
            "metadata": {}
        });

        let result = stream.parse_generic_chunk(&json);
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    // ==================== parse_chunk Routing Tests ====================

    #[test]
    fn test_parse_chunk_routes_to_claude() {
        let stream = create_test_stream_claude();
        let payload = br#"{"type": "content_block_delta", "delta": {"text": "test"}}"#;

        let result = stream.parse_chunk(payload);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_chunk_routes_to_nova() {
        let stream = create_test_stream_nova();
        let payload = br#"{"contentBlockDelta": {"delta": {"text": "test"}}}"#;

        let result = stream.parse_chunk(payload);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_chunk_routes_to_titan() {
        let stream = create_test_stream_titan();
        let payload = br#"{"outputText": "test"}"#;

        let result = stream.parse_chunk(payload);
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_parse_chunk_invalid_json() {
        let stream = create_test_stream_claude();
        let payload = b"not valid json";

        let result = stream.parse_chunk(payload);
        assert!(result.is_err());
    }

    // ==================== BedrockStream Creation Tests ====================

    #[test]
    fn test_bedrock_stream_creation() {
        let stream = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        let bedrock_stream = BedrockStream::new(stream, BedrockModelFamily::Claude);
        assert!(bedrock_stream.buffer.is_empty());
    }

    #[test]
    fn test_bedrock_stream_different_models() {
        let stream1 = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        let _ = BedrockStream::new(stream1, BedrockModelFamily::Claude);

        let stream2 = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        let _ = BedrockStream::new(stream2, BedrockModelFamily::Nova);

        let stream3 = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        let _ = BedrockStream::new(stream3, BedrockModelFamily::TitanText);

        let stream4 = futures::stream::empty::<Result<Bytes, reqwest::Error>>();
        let _ = BedrockStream::new(stream4, BedrockModelFamily::Mistral);
    }
}
