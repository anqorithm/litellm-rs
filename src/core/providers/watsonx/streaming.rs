//! Streaming Module for Watsonx
//!
//! Uses the unified SSE parser for consistent streaming across providers.
//! Also provides fake streaming support when needed.

use super::error::WatsonxError;
use crate::core::providers::base::sse::{OpenAICompatibleTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;
use bytes::Bytes;
use futures::Stream;
use std::pin::Pin;

/// Watsonx uses OpenAI-compatible SSE format
pub type WatsonxStreamInner = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    OpenAICompatibleTransformer,
>;

/// Helper function to create Watsonx stream
pub fn create_watsonx_stream(
    stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static,
) -> WatsonxStreamInner {
    let transformer = OpenAICompatibleTransformer::new("watsonx");
    UnifiedSSEStream::new(Box::pin(stream), transformer)
}

/// Wrapper stream that converts ProviderError to WatsonxError for backward compatibility
pub struct WatsonxStream {
    inner: WatsonxStreamInner,
}

impl WatsonxStream {
    pub fn new(stream: impl Stream<Item = Result<Bytes, reqwest::Error>> + Send + 'static) -> Self {
        Self {
            inner: create_watsonx_stream(stream),
        }
    }
}

impl Stream for WatsonxStream {
    type Item = Result<ChatChunk, WatsonxError>;

    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        use std::pin::Pin;
        use std::task::Poll;

        match Pin::new(&mut self.inner).poll_next(cx) {
            Poll::Ready(Some(Ok(chunk))) => Poll::Ready(Some(Ok(chunk))),
            Poll::Ready(Some(Err(e))) => Poll::Ready(Some(Err(ProviderError::streaming_error(
                "watsonx",
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
