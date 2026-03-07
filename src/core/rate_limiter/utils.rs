//! Utility functions for rate limiter

use super::limiter::RateLimiter;
use super::types::RateLimitResult;
use std::sync::Arc;
use std::time::{Duration, Instant};

impl RateLimiter {
    /// Cleanup expired entries (background, no global lock — DashMap per-shard locks)
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let window_start = now - self.window;

        self.entries.retain(|_, entry| {
            entry.timestamps.retain(|&t| t > window_start);
            !entry.timestamps.is_empty() || entry.tokens > 0.0
        });
    }

    /// Start background cleanup task
    pub fn start_cleanup_task(self: Arc<Self>) {
        let limiter = self.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                limiter.cleanup().await;
            }
        });
    }

    /// Get current status for a key
    pub async fn status(&self, key: &str) -> Option<RateLimitResult> {
        if !self.config.enabled {
            return None;
        }

        Some(self.check(key).await)
    }

    /// Check if rate limiting is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Get the configured limit
    pub fn limit(&self) -> u32 {
        self.config.default_rpm
    }
}
