//! Unified SSE (Server-Sent Events) Parser
//!
//! A centralized SSE parsing system that eliminates code duplication across providers.
//! All providers can use this parser and only need to implement the transformation logic.
//!
//! Per-provider transformers are in submodules:
//! - `openai` - OpenAI-compatible format (reused by many providers)
//! - `anthropic` - Anthropic event-based format
//! - `gemini` - Gemini candidates/parts format
//! - `cohere` - Cohere v1/v2 format
//! - `databricks` - Databricks format (OpenAI + array content)

mod anthropic;
mod cohere;
mod databricks;
mod gemini;
mod openai;

pub use anthropic::AnthropicTransformer;
pub use cohere::CohereTransformer;
pub use databricks::DatabricksTransformer;
pub use gemini::GeminiTransformer;
pub use openai::OpenAICompatibleTransformer;

use bytes::Bytes;
use futures::Stream;
use std::collections::VecDeque;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{ChatChunk, FinishReason};

/// SSE Event Types
#[derive(Debug, Clone, PartialEq)]
pub enum SSEEventType {
    Data,
    Event,
    Id,
    Retry,
    Comment,
}

/// Parsed SSE Event
#[derive(Debug, Clone)]
pub struct SSEEvent {
    pub event_type: Option<String>,
    pub data: String,
    pub id: Option<String>,
    pub retry: Option<u64>,
}

impl SSEEvent {
    /// Parse SSE event from a line
    pub fn from_line(line: &str) -> Option<Self> {
        if line.is_empty() || line.starts_with(':') {
            return None;
        }

        if let Some(colon_pos) = line.find(':') {
            let field = &line[..colon_pos];
            let value = line[colon_pos + 1..].trim_start();

            match field {
                "data" => Some(SSEEvent {
                    event_type: None,
                    data: value.to_string(),
                    id: None,
                    retry: None,
                }),
                "event" => Some(SSEEvent {
                    event_type: Some(value.to_string()),
                    data: String::new(),
                    id: None,
                    retry: None,
                }),
                "id" => Some(SSEEvent {
                    event_type: None,
                    data: String::new(),
                    id: Some(value.to_string()),
                    retry: None,
                }),
                "retry" => {
                    if let Ok(retry_ms) = value.parse::<u64>() {
                        Some(SSEEvent {
                            event_type: None,
                            data: String::new(),
                            id: None,
                            retry: Some(retry_ms),
                        })
                    } else {
                        None
                    }
                }
                _ => None,
            }
        } else {
            None
        }
    }
}

/// Trait for provider-specific SSE transformation
pub trait SSETransformer: Send + Sync {
    /// Provider name for error reporting
    fn provider_name(&self) -> &'static str;

    /// Check if this is the end-of-stream marker
    fn is_end_marker(&self, data: &str) -> bool {
        data.trim() == "[DONE]"
    }

    /// Transform raw SSE data into ChatChunk
    fn transform_chunk(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError>;

    /// Parse finish reason from string (provider-specific)
    fn parse_finish_reason(&self, reason: &str) -> Option<FinishReason> {
        match reason {
            "stop" => Some(FinishReason::Stop),
            "length" | "max_tokens" => Some(FinishReason::Length),
            "tool_calls" | "function_call" => Some(FinishReason::ToolCalls),
            "content_filter" => Some(FinishReason::ContentFilter),
            _ => None,
        }
    }
}

/// Unified SSE Parser
pub struct UnifiedSSEParser<T: SSETransformer> {
    transformer: T,
    buffer: String,
    current_event: Option<SSEEvent>,
}

impl<T: SSETransformer> UnifiedSSEParser<T> {
    /// Create new SSE parser with a transformer
    pub fn new(transformer: T) -> Self {
        Self {
            transformer,
            buffer: String::new(),
            current_event: None,
        }
    }

    /// Process raw bytes into SSE events
    ///
    /// Optimized to minimize allocations:
    /// - Uses `from_utf8_lossy` which returns `Cow<str>` (borrowed when valid UTF-8)
    /// - Processes lines without collecting into intermediate `Vec<String>`
    /// - Only allocates for incomplete lines that need to be buffered
    pub fn process_bytes(&mut self, bytes: &[u8]) -> Result<Vec<ChatChunk>, ProviderError> {
        // Append new bytes to buffer - from_utf8_lossy avoids allocation for valid UTF-8
        let text = String::from_utf8_lossy(bytes);
        self.buffer.push_str(&text);

        let mut chunks = Vec::new();

        // Find the last newline position
        let last_newline = self.buffer.rfind('\n');

        if let Some(pos) = last_newline {
            // Extract complete part and remaining incomplete part
            let complete_part = self.buffer[..=pos].to_string();
            let incomplete_part = self.buffer[pos + 1..].to_string();

            // Update buffer with incomplete part before processing
            // (This avoids borrow issues)
            self.buffer = incomplete_part;

            // Process complete lines
            for line in complete_part.lines() {
                if let Some(chunk) = self.process_line(line)? {
                    chunks.push(chunk);
                }
            }
        }
        // If no newline found, keep buffering (no action needed)

        Ok(chunks)
    }

    /// Process a single SSE line
    fn process_line(&mut self, line: &str) -> Result<Option<ChatChunk>, ProviderError> {
        // Empty line signals end of event
        if line.is_empty() {
            if let Some(event) = self.current_event.take() {
                return self.process_event(event);
            }
            return Ok(None);
        }

        // Parse SSE field
        if let Some(event) = SSEEvent::from_line(line) {
            // For data fields, accumulate or merge
            if !event.data.is_empty() {
                if let Some(ref mut current) = self.current_event {
                    // Append to existing data
                    if !current.data.is_empty() {
                        current.data.push('\n');
                    }
                    current.data.push_str(&event.data);
                } else {
                    self.current_event = Some(event);
                }
            } else if event.event_type.is_some() || event.id.is_some() || event.retry.is_some() {
                // Merge other fields
                if let Some(ref mut current) = self.current_event {
                    if event.event_type.is_some() {
                        current.event_type = event.event_type;
                    }
                    if event.id.is_some() {
                        current.id = event.id;
                    }
                    if event.retry.is_some() {
                        current.retry = event.retry;
                    }
                } else {
                    self.current_event = Some(event);
                }
            }
        }

        Ok(None)
    }

    /// Process a complete SSE event
    fn process_event(&self, event: SSEEvent) -> Result<Option<ChatChunk>, ProviderError> {
        // Skip empty events
        if event.data.is_empty() {
            return Ok(None);
        }

        // Check for end marker
        if self.transformer.is_end_marker(&event.data) {
            return Ok(None);
        }

        // Transform to ChatChunk
        self.transformer.transform_chunk(&event.data)
    }
}

/// Maximum number of chunks allowed in the buffer to prevent OOM from slow clients
/// or malicious actors. At ~1KB per chunk, 10,000 chunks ≈ 10MB upper bound.
const MAX_CHUNK_BUFFER_SIZE: usize = 10_000;

/// Streaming wrapper that uses UnifiedSSEParser
///
/// Uses `VecDeque` for buffered chunks to enable O(1) pop_front instead of O(n) Vec::remove(0).
pub struct UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone,
{
    inner: S,
    parser: UnifiedSSEParser<T>,
    chunk_buffer: VecDeque<ChatChunk>,
}

impl<S, T> UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone,
{
    pub fn new(stream: S, transformer: T) -> Self {
        Self {
            inner: stream,
            parser: UnifiedSSEParser::new(transformer),
            chunk_buffer: VecDeque::new(),
        }
    }
}

impl<S, T> Stream for UnifiedSSEStream<S, T>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Send + Unpin,
    T: SSETransformer + Clone + Unpin,
{
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // Return buffered chunks first - O(1) with VecDeque
        if let Some(chunk) = this.chunk_buffer.pop_front() {
            return Poll::Ready(Some(Ok(chunk)));
        }

        // Poll inner stream for more data
        match Pin::new(&mut this.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(bytes))) => {
                match this.parser.process_bytes(&bytes) {
                    Ok(chunks) => {
                        if chunks.is_empty() {
                            // No chunks yet, poll again
                            cx.waker().wake_by_ref();
                            Poll::Pending
                        } else {
                            // Guard against unbounded buffer growth (slow client / malicious actor)
                            if this.chunk_buffer.len() + chunks.len() > MAX_CHUNK_BUFFER_SIZE {
                                return Poll::Ready(Some(Err(ProviderError::network(
                                    this.parser.transformer.provider_name(),
                                    format!(
                                        "SSE chunk buffer exceeded limit of {} chunks",
                                        MAX_CHUNK_BUFFER_SIZE
                                    ),
                                ))));
                            }
                            // Buffer chunks and return first one
                            this.chunk_buffer.extend(chunks);
                            if let Some(chunk) = this.chunk_buffer.pop_front() {
                                Poll::Ready(Some(Ok(chunk)))
                            } else {
                                cx.waker().wake_by_ref();
                                Poll::Pending
                            }
                        }
                    }
                    Err(e) => Poll::Ready(Some(Err(e))),
                }
            }
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ProviderError::network(
                this.parser.transformer.provider_name(),
                format!("Stream error: {}", e),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create an OpenAI-compatible SSE stream from an HTTP response.
///
/// Replaces the manual `.scan()` + `.flat_map()` pattern that was duplicated
/// across many providers. Uses `UnifiedSSEStream` which handles buffering,
/// parsing, and error mapping internally.
pub fn create_provider_sse_stream(
    response: reqwest::Response,
    provider_name: &'static str,
) -> Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>> {
    let transformer = OpenAICompatibleTransformer::new(provider_name);
    let stream = UnifiedSSEStream::new(Box::pin(response.bytes_stream()), transformer);
    Box::pin(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sse_event_parsing() {
        let event = SSEEvent::from_line("data: test data").unwrap();
        assert_eq!(event.data, "test data");

        let event = SSEEvent::from_line("event: message").unwrap();
        assert_eq!(event.event_type, Some("message".to_string()));

        let event = SSEEvent::from_line("id: 123").unwrap();
        assert_eq!(event.id, Some("123".to_string()));

        let event = SSEEvent::from_line("retry: 5000").unwrap();
        assert_eq!(event.retry, Some(5000));

        // Comments should be ignored
        assert!(SSEEvent::from_line(": comment").is_none());
    }

    #[test]
    fn test_openai_transformer() {
        let transformer = OpenAICompatibleTransformer::new("test");

        // Test end marker
        assert!(transformer.is_end_marker("[DONE]"));
        assert!(!transformer.is_end_marker("data: {\"test\": 1}"));

        // Test JSON transformation
        let json_data = r#"{
            "id": "test-id",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "gpt-4",
            "choices": [{
                "index": 0,
                "delta": {"content": "Hello"},
                "finish_reason": null
            }]
        }"#;

        let result = transformer.transform_chunk(json_data).unwrap().unwrap();
        assert_eq!(result.id, "test-id");
        assert_eq!(result.model, "gpt-4");
        assert_eq!(result.choices[0].delta.content, Some("Hello".to_string()));
    }

    #[test]
    fn test_openai_transformer_reasoning_content_to_thinking() {
        let transformer = OpenAICompatibleTransformer::new("test");

        let json_data = r#"{
            "id": "test-id-reasoning",
            "object": "chat.completion.chunk",
            "created": 1234567890,
            "model": "deepseek-r1",
            "choices": [{
                "index": 0,
                "delta": {
                    "content": "Answer",
                    "reasoning_content": "chain-of-thought"
                },
                "finish_reason": null
            }]
        }"#;

        let result = transformer.transform_chunk(json_data).unwrap().unwrap();
        assert_eq!(
            result.choices[0]
                .delta
                .thinking
                .as_ref()
                .and_then(|t| t.content.as_ref())
                .map(String::as_str),
            Some("chain-of-thought")
        );
    }

    #[test]
    fn test_sse_parser_multiline() {
        let transformer = OpenAICompatibleTransformer::new("test");
        let mut parser = UnifiedSSEParser::new(transformer);

        // Simulate receiving data in chunks
        let chunk1 = b"data: {\"id\": \"test\", ";
        let chunk2 = b"\"choices\": [{\"delta\": {\"content\": \"Hi\"}, \"index\": 0}], ";
        let chunk3 = b"\"model\": \"gpt-4\", \"created\": 123}\n\n";

        let results1 = parser.process_bytes(chunk1).unwrap();
        assert!(results1.is_empty()); // Not complete yet

        let results2 = parser.process_bytes(chunk2).unwrap();
        assert!(results2.is_empty()); // Still not complete

        let results3 = parser.process_bytes(chunk3).unwrap();
        assert_eq!(results3.len(), 1);
        assert_eq!(results3[0].choices[0].delta.content, Some("Hi".to_string()));
    }
}
