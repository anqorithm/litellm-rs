//! Streaming Module for Snowflake
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! Also provides fake streaming support when needed.

use super::error::SnowflakeError;
use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatResponse, ChatStreamChoice};
use crate::core::types::{message::MessageContent, message::MessageRole};
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Snowflake uses OpenAI-compatible SSE format
pub type SnowflakeStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Snowflake stream
pub fn create_snowflake_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> SnowflakeStreamInner {
    let transformer = OpenAICompatibleTransformer::new("snowflake");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that converts ProviderError to SnowflakeError for backward compatibility
pub struct SnowflakeStream {
    inner: SnowflakeStreamInner,
}

impl SnowflakeStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_snowflake_stream(stream),
        }
    }
}

impl Stream for SnowflakeStream {
    type Item = Result<ChatChunk, SnowflakeError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::pin::Pin;
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(SnowflakeError::streaming_error(
                "snowflake",
                "chat",
                None,
                None,
                e.to_string(),
            )))),
            Poll::Ready(None) => Poll::Ready(None),
            Poll::Pending => Poll::Pending,
        }
    }
}

/// Create a fake stream from a complete response
pub async fn create_fake_stream(
    response: ChatResponse,
) -> Result<Pin<Box<dyn Stream<Item = Result<ChatChunk, SnowflakeError>> + Send>>, SnowflakeError> {
    // Convert response to chunks
    let chunks = response_to_chunks(response);
    let stream = futures::stream::iter(chunks.into_iter().map(Ok));
    Ok(Box::pin(stream))
}

/// Convert a complete ChatResponse to stream chunks
fn response_to_chunks(response: ChatResponse) -> Vec<ChatChunk> {
    let mut chunks = Vec::new();

    // Create initial chunk with role
    chunks.push(ChatChunk {
        id: response.id.clone(),
        object: "chat.completion.chunk".to_string(),
        created: response.created,
        model: response.model.clone(),
        system_fingerprint: response.system_fingerprint.clone(),
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
    });

    // Create content chunks
    if let Some(choice) = response.choices.first() {
        if let Some(content) = &choice.message.content {
            let text = match content {
                MessageContent::Text(text) => text.clone(),
                MessageContent::Parts(_) => content.to_string(),
            };

            // Split content into smaller chunks for more natural streaming
            let words: Vec<&str> = text.split_whitespace().collect();
            let chunk_size = 5; // Words per chunk

            for word_chunk in words.chunks(chunk_size) {
                let chunk_text = word_chunk.join(" ") + " ";
                chunks.push(ChatChunk {
                    id: response.id.clone(),
                    object: "chat.completion.chunk".to_string(),
                    created: response.created,
                    model: response.model.clone(),
                    system_fingerprint: response.system_fingerprint.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(chunk_text),
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                });
            }
        }

        // Add final chunk with finish_reason
        chunks.push(ChatChunk {
            id: response.id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: response.created,
            model: response.model.clone(),
            system_fingerprint: response.system_fingerprint.clone(),
            choices: vec![ChatStreamChoice {
                index: 0,
                delta: ChatDelta {
                    role: None,
                    content: None,
                    thinking: None,
                    tool_calls: None,
                    function_call: None,
                },
                finish_reason: choice.finish_reason.clone(),
                logprobs: None,
            }],
            usage: response.usage.clone(),
        });
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::chat::ChatMessage;
    use crate::core::types::responses::{ChatChoice, FinishReason, Usage};
    use bytes::Bytes;
    use futures::{stream, StreamExt};

    #[tokio::test]
    async fn test_create_snowflake_stream_parses_sse() {
        let payload = concat!(
            "data: {\"id\":\"chatcmpl-1\",\"object\":\"chat.completion.chunk\",",
            "\"created\":1726000000,\"model\":\"snowflake/arctic\",",
            "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",",
            "\"content\":\"hi\"},\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n"
        );

        let raw = stream::iter(vec![Ok::<Bytes, reqwest::Error>(Bytes::from(payload))]);
        let mut parsed = create_snowflake_stream(raw);
        let first = parsed.next().await.unwrap().unwrap();
        assert_eq!(first.model, "snowflake/arctic");
    }

    #[tokio::test]
    async fn test_snowflake_stream_wrapper_constructs() {
        let payload = concat!(
            "data: {\"id\":\"chatcmpl-2\",\"object\":\"chat.completion.chunk\",",
            "\"created\":1726000000,\"model\":\"snowflake/arctic\",",
            "\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",",
            "\"content\":\"hello\"},\"finish_reason\":null}]}\n\n",
            "data: [DONE]\n\n"
        );

        let raw = stream::iter(vec![Ok::<Bytes, reqwest::Error>(Bytes::from(payload))]);
        let mut wrapped = SnowflakeStream::new(raw);
        let first = wrapped.next().await.unwrap().unwrap();
        assert_eq!(first.model, "snowflake/arctic");
    }

    #[tokio::test]
    async fn test_create_fake_stream_emits_chunks() {
        let response = ChatResponse {
            id: "chatcmpl-snowflake-test".to_string(),
            object: "chat.completion".to_string(),
            created: 1_726_000_000,
            model: "snowflake/arctic".to_string(),
            choices: vec![ChatChoice {
                index: 0,
                message: ChatMessage {
                    role: MessageRole::Assistant,
                    content: Some(MessageContent::Text(
                        "hello from snowflake stream".to_string(),
                    )),
                    thinking: None,
                    name: None,
                    tool_calls: None,
                    tool_call_id: None,
                    function_call: None,
                },
                finish_reason: Some(FinishReason::Stop),
                logprobs: None,
            }],
            usage: Some(Usage {
                prompt_tokens: 5,
                completion_tokens: 4,
                total_tokens: 9,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                thinking_usage: None,
            }),
            system_fingerprint: None,
        };

        let mut stream = create_fake_stream(response).await.unwrap();
        let mut chunks = Vec::new();
        while let Some(chunk) = stream.next().await {
            chunks.push(chunk.unwrap());
        }

        assert!(chunks.len() >= 2);
        assert_eq!(
            chunks[0].choices[0].delta.role,
            Some(MessageRole::Assistant)
        );
        assert_eq!(
            chunks.last().unwrap().choices[0].finish_reason,
            Some(FinishReason::Stop)
        );
    }
}
