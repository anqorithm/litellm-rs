//! Error handling for the Gateway
//!
//! This module defines all error types used throughout the gateway.

#![allow(missing_docs)]

mod conversions;
mod helpers;
#[cfg(feature = "gateway")]
mod response;
#[cfg(test)]
mod tests;
mod types;

#[cfg(feature = "gateway")]
pub use response::{GatewayErrorDetail, GatewayErrorResponse, with_gateway_error_request_id};
pub use types::{GatewayError, Result};
