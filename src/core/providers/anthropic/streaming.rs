//! Anthropic Streaming Module
//!
//! Uses the unified SSE parser with AnthropicTransformer for event-based streaming.

use std::pin::Pin;

use bytes::Bytes;
use futures::Stream;
use pin_project_lite::pin_project;

use crate::core::providers::base::sse::{AnthropicTransformer, UnifiedSSEStream};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::ChatChunk;

/// Anthropic SSE stream type
pub type AnthropicSSEStream = UnifiedSSEStream<
    Pin<Box<dyn Stream<Item = Result<Bytes, reqwest::Error>> + Send>>,
    AnthropicTransformer,
>;

pin_project! {
    /// Anthropic streaming processor
    pub struct AnthropicStream {
        #[pin]
        inner: Pin<Box<dyn Stream<Item = Result<ChatChunk, ProviderError>> + Send>>,
    }
}

impl AnthropicStream {
    /// Create stream from response
    pub fn from_response(response: reqwest::Response, model: String) -> Self {
        Self::from_response_impl(response, model, false)
    }

    /// Create stream for a json_schema structured-output request.
    ///
    /// Enables json_schema_mode on the transformer so that the `tool_use`
    /// stop_reason is normalised to `Stop` instead of `ToolCalls`.
    pub fn from_response_json_schema(response: reqwest::Response, model: String) -> Self {
        Self::from_response_impl(response, model, true)
    }

    fn from_response_impl(
        response: reqwest::Response,
        model: String,
        json_schema_mode: bool,
    ) -> Self {
        let transformer = if json_schema_mode {
            AnthropicTransformer::new(model).with_json_schema_mode()
        } else {
            AnthropicTransformer::new(model)
        };
        let stream = UnifiedSSEStream::new(Box::pin(response.bytes_stream()), transformer);
        Self {
            inner: Box::pin(stream),
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
