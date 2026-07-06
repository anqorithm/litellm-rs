//! Error handling for the Gateway
//!
//! This module defines all error types used throughout the gateway.

#![allow(missing_docs)]

mod conversions;
mod helpers;
#[cfg(feature = "gateway")]
mod http_mapping;
#[cfg(feature = "gateway")]
mod response;
#[cfg(test)]
mod tests;
mod types;

#[cfg(feature = "gateway")]
pub use http_mapping::{HttpErrorFacts, HttpErrorHeaders, gateway_http_error_facts};
#[cfg(feature = "gateway")]
pub use response::{
    GatewayErrorDetail, GatewayErrorResponse, current_error_response_request_id,
    with_error_response_request_id,
};
pub use types::{GatewayError, Result};
