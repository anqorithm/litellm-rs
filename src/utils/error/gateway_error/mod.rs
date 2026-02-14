//! Error handling for the Gateway
//!
//! This module defines all error types used throughout the gateway.

#![allow(missing_docs)]

mod conversions;
mod helpers;
mod response;
#[cfg(test)]
mod tests;
mod types;

// Re-export all public types for backward compatibility
pub use response::{ErrorDetail, ErrorResponse};
pub use types::{GatewayError, Result};
