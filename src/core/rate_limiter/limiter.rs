//! Core rate limiter implementation

use super::types::{RateLimitEntry, RateLimitResult};
use crate::config::models::rate_limit::{RateLimitConfig, RateLimitStrategy};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
#[cfg(feature = "gateway")]
use tracing::warn;

/// Rate limiter implementation
pub struct RateLimiter {
    /// Rate limit configuration
    pub(super) config: RateLimitConfig,
    /// Rate limit entries by key (IP or API key) — per-key lock granularity
    pub(super) entries: Arc<DashMap<String, RateLimitEntry>>,
    /// Window duration
    pub(super) window: Duration,
    /// Optional distributed Redis backend
    #[cfg(feature = "gateway")]
    pub(super) redis: Option<Arc<crate::storage::redis::RedisPool>>,
}

impl RateLimiter {
    /// Create a new rate limiter
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            entries: Arc::new(DashMap::new()),
            window: Duration::from_secs(60), // 1 minute window
            #[cfg(feature = "gateway")]
            redis: None,
        }
    }

    /// Create a rate limiter with custom window
    pub fn with_window(config: RateLimitConfig, window: Duration) -> Self {
        Self {
            config,
            entries: Arc::new(DashMap::new()),
            window,
            #[cfg(feature = "gateway")]
            redis: None,
        }
    }

    /// Create a rate limiter backed by Redis for distributed enforcement.
    #[cfg(feature = "gateway")]
    pub fn with_redis(
        config: RateLimitConfig,
        redis: Arc<crate::storage::redis::RedisPool>,
    ) -> Self {
        Self {
            config,
            entries: Arc::new(DashMap::new()),
            window: Duration::from_secs(60),
            redis: if redis.is_noop() { None } else { Some(redis) },
        }
    }

    /// Check if a request should be allowed (read-only, does not record)
    ///
    /// WARNING: Using check() followed by record() has a race condition.
    /// Use check_and_record() for atomic check-then-record operations.
    pub async fn check(&self, key: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult {
                allowed: true,
                current_count: 0,
                limit: self.config.default_rpm,
                remaining: self.config.default_rpm,
                reset_after_secs: 0,
                retry_after_secs: None,
            };
        }

        #[cfg(feature = "gateway")]
        if let Some(redis) = &self.redis {
            match redis
                .rate_limit_status(key, self.config.default_rpm, self.window.as_secs())
                .await
            {
                Ok(result) => return result,
                Err(err) => {
                    warn!(
                        "Redis rate-limit status failed for key {}; falling back to in-process limiter: {}",
                        key, err
                    );
                }
            }
        }

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow => self.check_sliding_window_impl(key, false).await,
            RateLimitStrategy::TokenBucket => self.check_token_bucket_impl(key, false).await,
            RateLimitStrategy::FixedWindow => self.check_fixed_window_impl(key, false).await,
        }
    }

    /// Atomically check and record a request (prevents TOCTOU race condition)
    ///
    /// This is the preferred method for rate limiting as it performs both
    /// the check and record operations in a single lock acquisition.
    pub async fn check_and_record(&self, key: &str) -> RateLimitResult {
        if !self.config.enabled {
            return RateLimitResult {
                allowed: true,
                current_count: 0,
                limit: self.config.default_rpm,
                remaining: self.config.default_rpm,
                reset_after_secs: 0,
                retry_after_secs: None,
            };
        }

        #[cfg(feature = "gateway")]
        if let Some(redis) = &self.redis {
            match redis
                .rate_limit_check_and_record(key, self.config.default_rpm, self.window.as_secs())
                .await
            {
                Ok(result) => return result,
                Err(err) => {
                    warn!(
                        "Redis rate-limit check failed for key {}; falling back to in-process limiter: {}",
                        key, err
                    );
                }
            }
        }

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow => self.check_sliding_window_impl(key, true).await,
            RateLimitStrategy::TokenBucket => self.check_token_bucket_impl(key, true).await,
            RateLimitStrategy::FixedWindow => self.check_fixed_window_impl(key, true).await,
        }
    }

    /// Record a request (increments counter)
    ///
    /// WARNING: This is a separate operation from check() and has a race condition.
    /// Use check_and_record() instead to avoid race conditions.
    #[deprecated(note = "Use check_and_record() instead to avoid race conditions")]
    pub async fn record(&self, key: &str) {
        if !self.config.enabled {
            return;
        }

        let mut entry = self.entries.entry(key.to_string()).or_default();

        match self.config.strategy {
            RateLimitStrategy::SlidingWindow | RateLimitStrategy::FixedWindow => {
                entry.timestamps.push(std::time::Instant::now());
            }
            RateLimitStrategy::TokenBucket => {
                // Token is consumed in check_token_bucket
            }
        }
    }
}

impl Clone for RateLimiter {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            entries: self.entries.clone(),
            window: self.window,
            #[cfg(feature = "gateway")]
            redis: self.redis.clone(),
        }
    }
}
