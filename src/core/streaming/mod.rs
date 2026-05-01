//! Streaming response handling for AI providers
//!
//! This module provides Server-Sent Events (SSE) streaming support for real-time AI responses.

#[cfg(feature = "gateway")]
use crate::utils::error::gateway_error::Result;
#[cfg(feature = "gateway")]
use actix_web::http::header::{CACHE_CONTROL, CONTENT_TYPE};
#[cfg(feature = "gateway")]
use actix_web::{HttpResponse, web};
#[cfg(feature = "gateway")]
use futures::stream::Stream;

// Module declarations
pub mod handler;
pub mod types;
pub mod utils;

#[cfg(test)]
mod tests;

/// Create a Server-Sent Events response for Actix-web
#[cfg(feature = "gateway")]
pub fn create_sse_response<S>(stream: S) -> HttpResponse
where
    S: Stream<Item = Result<web::Bytes>> + Send + 'static,
{
    HttpResponse::Ok()
        .insert_header((CONTENT_TYPE, "text/event-stream"))
        .insert_header((CACHE_CONTROL, "no-cache"))
        .insert_header(("Connection", "keep-alive"))
        .streaming(stream)
}
