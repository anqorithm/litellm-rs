//! Provider-specific streaming implementations

use crate::utils::error::gateway_error::{GatewayError, Result};
use futures::stream::{BoxStream, StreamExt};

/// Helper to convert bytes to UTF-8 string efficiently
/// Validates UTF-8 in-place before allocating, avoiding allocation on error
#[inline]
fn bytes_to_utf8_string(bytes: &[u8]) -> std::result::Result<String, std::str::Utf8Error> {
    std::str::from_utf8(bytes).map(|s| s.to_owned())
}

/// OpenAI streaming implementation
pub struct OpenAIStreaming;

impl OpenAIStreaming {
    /// Create a stream from OpenAI SSE response
    pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
        let stream = response.bytes_stream().map(|chunk_result| {
            chunk_result
                .map_err(|e| GatewayError::Network(e.to_string()))
                .and_then(|chunk| {
                    bytes_to_utf8_string(&chunk)
                        .map_err(|e| GatewayError::Validation(e.to_string()))
                })
        });

        Box::pin(stream)
    }
}

/// Anthropic streaming implementation
pub struct AnthropicStreaming;

impl AnthropicStreaming {
    /// Create a stream from Anthropic SSE response
    pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
        let stream = response.bytes_stream().map(|chunk_result| {
            chunk_result
                .map_err(|e| GatewayError::network(e.to_string()))
                .and_then(|chunk| {
                    bytes_to_utf8_string(&chunk)
                        .map_err(|e| GatewayError::internal(format!("Parsing error: {}", e)))
                })
        });

        Box::pin(stream)
    }
}

/// Generic streaming implementation for other providers
pub struct GenericStreaming;

impl GenericStreaming {
    /// Create a stream from generic SSE response
    pub fn create_stream(response: reqwest::Response) -> BoxStream<'static, Result<String>> {
        let stream = response.bytes_stream().map(|chunk_result| {
            chunk_result
                .map_err(|e| GatewayError::network(e.to_string()))
                .and_then(|chunk| {
                    bytes_to_utf8_string(&chunk)
                        .map_err(|e| GatewayError::internal(format!("Parsing error: {}", e)))
                })
        });

        Box::pin(stream)
    }
}
