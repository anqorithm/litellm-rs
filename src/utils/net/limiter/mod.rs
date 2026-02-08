//! Rate limiting utilities for the Gateway
//!
//! This module provides rate limiting functionality using token bucket and sliding window algorithms.

// Module declarations
mod engine;
mod types;
mod utils;
mod window;

#[cfg(test)]
mod tests;

// Re-exports
pub use engine::RateLimiter;
pub use types::{RateLimitConfig, RateLimitKey, RateLimitResult, SlidingWindow, TokenBucket};
