//! Sliding window rate limiting implementation
//!
//! This module contains sliding window algorithms for tracking requests and tokens.

use crate::utils::error::Result;
use std::time::{Duration, Instant};

use super::engine::RateLimiter;
use super::types::{RateLimitResult, SlidingWindow};

impl RateLimiter {
    /// Check sliding window for requests
    pub(super) async fn check_sliding_window_requests(
        &self,
        key: &str,
        window_size: Duration,
        limit: u32,
    ) -> Result<RateLimitResult> {
        let mut windows = self.windows.write().await;
        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow {
                window_size,
                requests: Vec::new(),
                tokens: Vec::new(),
            });

        let now = Instant::now();
        let window_start = now - window_size;

        // Remove old requests
        window
            .requests
            .retain(|&timestamp| timestamp > window_start);

        let current_count = window.requests.len() as u32;
        let allowed = current_count < limit;
        let remaining = limit.saturating_sub(current_count);

        let retry_after = if !allowed {
            window
                .requests
                .first()
                .map(|&first| window_size - (now - first))
        } else {
            None
        };

        Ok(RateLimitResult {
            allowed,
            remaining_requests: Some(remaining),
            remaining_tokens: None,
            reset_time: Some(window_size),
            retry_after,
            limit_type: None,
        })
    }

    /// Check sliding window for tokens
    pub(super) async fn check_sliding_window_tokens(
        &self,
        key: &str,
        window_size: Duration,
        limit: u32,
        tokens: u32,
    ) -> Result<RateLimitResult> {
        let mut windows = self.windows.write().await;
        let window = windows
            .entry(key.to_string())
            .or_insert_with(|| SlidingWindow {
                window_size,
                requests: Vec::new(),
                tokens: Vec::new(),
            });

        let now = Instant::now();
        let window_start = now - window_size;

        // Remove old token records
        window
            .tokens
            .retain(|(timestamp, _)| *timestamp > window_start);

        let current_tokens: u32 = window.tokens.iter().map(|(_, count)| count).sum();
        let allowed = current_tokens + tokens <= limit;
        let remaining = limit.saturating_sub(current_tokens);

        let retry_after = if !allowed {
            window
                .tokens
                .first()
                .map(|(first, _)| window_size - (now - *first))
        } else {
            None
        };

        Ok(RateLimitResult {
            allowed,
            remaining_requests: None,
            remaining_tokens: Some(remaining),
            reset_time: Some(window_size),
            retry_after,
            limit_type: None,
        })
    }
}
