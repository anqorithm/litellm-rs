//! Streaming Module for OCI Generative AI
//!
//! OCI uses an OpenAI-compatible SSE format for streaming responses.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::MessageRole;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice, FinishReason};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

/// OCI SSE stream processor
pub struct OciStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    inner: S,
    buffer: String,
    done: bool,
}

impl<S> OciStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    /// Create a new OCI stream
    pub fn new(stream: S) -> Self {
        Self {
            inner: stream,
            buffer: String::new(),
            done: false,
        }
    }

    /// Process SSE data line
    fn process_data(&self, data: &str) -> Option<Result<ChatChunk, ProviderError>> {
        // Check for done signal
        if data == "[DONE]" {
            return None;
        }

        // Parse JSON response
        match serde_json::from_str::<serde_json::Value>(data) {
            Ok(json) => Some(self.parse_chunk(json)),
            Err(e) => Some(Err(ProviderError::response_parsing(
                "oci",
                format!("Failed to parse streaming response: {}", e),
            ))),
        }
    }

    /// Parse response chunk into ChatChunk
    fn parse_chunk(&self, json: serde_json::Value) -> Result<ChatChunk, ProviderError> {
        let id = json
            .get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("oci-stream")
            .to_string();

        let model = json
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let created = json
            .get("created")
            .and_then(|v| v.as_i64())
            .unwrap_or_else(|| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64
            });

        // Parse choices
        let choices = if let Some(choices_arr) = json.get("choices").and_then(|v| v.as_array()) {
            choices_arr
                .iter()
                .enumerate()
                .map(|(i, choice)| {
                    let delta = choice.get("delta").cloned().unwrap_or_default();
                    let content = delta
                        .get("content")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let role = delta
                        .get("role")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string());
                    let finish_reason =
                        choice
                            .get("finish_reason")
                            .and_then(|v| v.as_str())
                            .map(|s| match s {
                                "stop" => FinishReason::Stop,
                                "length" => FinishReason::Length,
                                "tool_calls" => FinishReason::ToolCalls,
                                "content_filter" => FinishReason::ContentFilter,
                                "function_call" => FinishReason::FunctionCall,
                                _ => FinishReason::Stop,
                            });

                    // Parse tool_calls if present
                    let tool_calls = delta
                        .get("tool_calls")
                        .and_then(|tc| serde_json::from_value(tc.clone()).ok());

                    ChatStreamChoice {
                        index: choice
                            .get("index")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(i as u64) as u32,
                        delta: ChatDelta {
                            role: role.and_then(|r| match r.as_str() {
                                "assistant" => Some(MessageRole::Assistant),
                                "user" => Some(MessageRole::User),
                                "system" => Some(MessageRole::System),
                                "tool" => Some(MessageRole::Tool),
                                _ => None,
                            }),
                            content,
                            thinking: None,
                            tool_calls,
                            function_call: None,
                        },
                        finish_reason,
                        logprobs: None,
                    }
                })
                .collect()
        } else {
            vec![]
        };

        Ok(ChatChunk {
            id,
            object: "chat.completion.chunk".to_string(),
            created,
            model,
            choices,
            usage: None,
            system_fingerprint: json
                .get("system_fingerprint")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

impl<S> Stream for OciStream<S>
where
    S: Stream<Item = Result<Bytes, reqwest::Error>> + Unpin,
{
    type Item = Result<ChatChunk, ProviderError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.done {
            return Poll::Ready(None);
        }

        loop {
            // Check buffer for complete events
            if let Some(pos) = self.buffer.find("\n\n") {
                let event = self.buffer[..pos].to_string();
                self.buffer = self.buffer[pos + 2..].to_string();

                // Process SSE event
                for line in event.lines() {
                    if let Some(data) = line.strip_prefix("data: ") {
                        let data = data.trim();
                        if data == "[DONE]" {
                            self.done = true;
                            return Poll::Ready(None);
                        }
                        if let Some(result) = self.process_data(data) {
                            return Poll::Ready(Some(result));
                        }
                    }
                }
                continue;
            }

            // Need more data
            match Pin::new(&mut self.inner).poll_next(cx) {
                Poll::Ready(Some(Ok(bytes))) => {
                    self.buffer.push_str(&String::from_utf8_lossy(&bytes));
                }
                Poll::Ready(Some(Err(e))) => {
                    return Poll::Ready(Some(Err(ProviderError::network("oci", e.to_string()))));
                }
                Poll::Ready(None) => {
                    self.done = true;
                    // Process any remaining data in buffer
                    if !self.buffer.is_empty() {
                        for line in self.buffer.lines() {
                            if let Some(data) = line.strip_prefix("data: ") {
                                let data = data.trim();
                                if data != "[DONE]" {
                                    if let Some(result) = self.process_data(data) {
                                        return Poll::Ready(Some(result));
                                    }
                                }
                            }
                        }
                    }
                    return Poll::Ready(None);
                }
                Poll::Pending => return Poll::Pending,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use futures::stream;

    #[tokio::test]
    async fn test_oci_stream_basic() {
        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut oci_stream = OciStream::new(mock_stream);

        let chunk1 = oci_stream.next().await;
        assert!(chunk1.is_some());
        let chunk1 = chunk1.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.content.as_ref().unwrap(), "Hello");

        let end = oci_stream.next().await;
        assert!(end.is_none());
    }

    #[tokio::test]
    async fn test_oci_stream_multiple_chunks() {
        let test_data = vec![
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\"}}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\"Hello\"}}]}\n\n",
            )),
            Ok(Bytes::from(
                "data: {\"id\":\"test-1\",\"model\":\"cohere.command-r-plus\",\"choices\":[{\"index\":0,\"delta\":{\"content\":\" World\"}}]}\n\n",
            )),
            Ok(Bytes::from("data: [DONE]\n\n")),
        ];

        let mock_stream = stream::iter(test_data);
        let mut oci_stream = OciStream::new(mock_stream);

        // Role chunk
        let chunk1 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk1.choices[0].delta.role, Some(MessageRole::Assistant));

        // Content chunks
        let chunk2 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk2.choices[0].delta.content.as_ref().unwrap(), "Hello");

        let chunk3 = oci_stream.next().await.unwrap().unwrap();
        assert_eq!(chunk3.choices[0].delta.content.as_ref().unwrap(), " World");

        // Done
        assert!(oci_stream.next().await.is_none());
    }

    #[test]
    fn test_parse_chunk_with_tool_calls() {
        let stream = OciStream::new(futures::stream::empty::<Result<Bytes, reqwest::Error>>());
        let json = serde_json::json!({
            "id": "test",
            "model": "cohere.command-r-plus",
            "choices": [{
                "index": 0,
                "delta": {
                    "tool_calls": [{
                        "index": 0,
                        "id": "call_123",
                        "type": "function",
                        "function": {
                            "name": "get_weather",
                            "arguments": "{}"
                        }
                    }]
                }
            }]
        });

        let chunk = stream.parse_chunk(json).unwrap();
        assert!(chunk.choices[0].delta.tool_calls.is_some());
    }
}
