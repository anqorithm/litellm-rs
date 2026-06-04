//! Core rate limiter implementation

use super::types::{RateLimitEntry, RateLimitResult};
use crate::config::models::rate_limit::{RateLimitConfig, RateLimitStrategy};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
#[cfg(feature = "gateway")]
use tracing::warn;

/// Backend that recorded a rate-limit reservation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RateLimitRecordSource {
    Disabled,
    Local,
    Distributed,
}

/// A concrete rate-limit record that may later be released.
#[derive(Debug, Clone, Copy)]
pub(crate) struct RateLimitReservation {
    source: RateLimitRecordSource,
    recorded_at: Instant,
    expires_at: Option<Instant>,
}

impl RateLimitReservation {
    fn new(source: RateLimitRecordSource, recorded_at: Instant, reset_after_secs: u64) -> Self {
        let expires_at = match source {
            RateLimitRecordSource::Disabled => None,
            RateLimitRecordSource::Local | RateLimitRecordSource::Distributed => {
                Some(recorded_at + Duration::from_secs(reset_after_secs.max(1)))
            }
        };

        Self {
            source,
            recorded_at,
            expires_at,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(
        source: RateLimitRecordSource,
        recorded_at: Instant,
        reset_after_secs: u64,
    ) -> Self {
        Self::new(source, recorded_at, reset_after_secs)
    }

    #[cfg(test)]
    pub(crate) fn source(&self) -> RateLimitRecordSource {
        self.source
    }

    fn remaining_window_secs(&self) -> u64 {
        let Some(expires_at) = self.expires_at else {
            return 0;
        };

        let remaining = expires_at.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            0
        } else {
            remaining.as_secs().max(1)
        }
    }
}

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
    pub(super) fn token_bucket_reservation_window(&self) -> Duration {
        if self.config.default_rpm == 0 {
            return Duration::ZERO;
        }

        Duration::from_secs((60.0 / self.config.default_rpm as f64).ceil() as u64)
            .max(Duration::from_secs(1))
    }

    fn local_reservation_reset_after_secs(&self, result: &RateLimitResult) -> u64 {
        match self.config.strategy {
            RateLimitStrategy::TokenBucket => self.token_bucket_reservation_window().as_secs(),
            RateLimitStrategy::SlidingWindow => self.window.as_secs(),
            RateLimitStrategy::FixedWindow => result.reset_after_secs,
        }
    }

    fn disabled_result(&self) -> RateLimitResult {
        RateLimitResult {
            allowed: true,
            current_count: 0,
            limit: self.config.default_rpm,
            remaining: self.config.default_rpm,
            reset_after_secs: 0,
            retry_after_secs: None,
        }
    }

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
            return self.disabled_result();
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
            RateLimitStrategy::SlidingWindow => {
                self.check_sliding_window_impl(key, false, Instant::now())
                    .await
            }
            RateLimitStrategy::TokenBucket => {
                self.check_token_bucket_impl(key, false, Instant::now())
                    .await
            }
            RateLimitStrategy::FixedWindow => {
                self.check_fixed_window_impl(key, false, Instant::now())
                    .await
            }
        }
    }

    /// Atomically check and record a request (prevents TOCTOU race condition)
    ///
    /// This is the preferred method for rate limiting as it performs both
    /// the check and record operations in a single lock acquisition.
    pub async fn check_and_record(&self, key: &str) -> RateLimitResult {
        self.check_and_record_with_source(key).await.0
    }

    /// Atomically check and record a request, returning the backend used.
    pub(crate) async fn check_and_record_with_source(
        &self,
        key: &str,
    ) -> (RateLimitResult, RateLimitReservation) {
        let recorded_at = Instant::now();

        if !self.config.enabled {
            return (
                self.disabled_result(),
                RateLimitReservation::new(RateLimitRecordSource::Disabled, recorded_at, 0),
            );
        }

        #[cfg(feature = "gateway")]
        if let Some(redis) = &self.redis {
            match redis
                .rate_limit_check_and_record(key, self.config.default_rpm, self.window.as_secs())
                .await
            {
                Ok(result) => {
                    let reservation = RateLimitReservation::new(
                        RateLimitRecordSource::Distributed,
                        recorded_at,
                        result.reset_after_secs,
                    );
                    return (result, reservation);
                }
                Err(err) => {
                    warn!(
                        "Redis rate-limit check failed for key {}; falling back to in-process limiter: {}",
                        key, err
                    );
                }
            }
        }

        let result = self.check_and_record_local(key, recorded_at).await;
        let reset_after_secs = self.local_reservation_reset_after_secs(&result);
        let reservation =
            RateLimitReservation::new(RateLimitRecordSource::Local, recorded_at, reset_after_secs);
        (result, reservation)
    }

    async fn check_and_record_local(&self, key: &str, recorded_at: Instant) -> RateLimitResult {
        match self.config.strategy {
            RateLimitStrategy::SlidingWindow => {
                self.check_sliding_window_impl(key, true, recorded_at).await
            }
            RateLimitStrategy::TokenBucket => {
                self.check_token_bucket_impl(key, true, recorded_at).await
            }
            RateLimitStrategy::FixedWindow => {
                self.check_fixed_window_impl(key, true, recorded_at).await
            }
        }
    }

    /// Release one request previously reserved by `check_and_record`.
    pub async fn release(&self, key: &str) {
        #[cfg(feature = "gateway")]
        if self.redis.is_some() {
            if let Some(redis) = &self.redis
                && let Err(err) = redis
                    .rate_limit_release(key, self.window.as_secs().max(1))
                    .await
            {
                warn!("Redis rate-limit release failed for key {}: {}", key, err);
            }
            return;
        }

        self.release_local(key, None);
    }

    /// Release one request previously reserved by the given backend.
    pub(crate) async fn release_recorded(&self, key: &str, reservation: RateLimitReservation) {
        if !self.config.enabled {
            return;
        }

        match reservation.source {
            RateLimitRecordSource::Disabled => {}
            RateLimitRecordSource::Local => {
                if reservation.remaining_window_secs() == 0 {
                    return;
                }

                self.release_local(key, Some(reservation.recorded_at));
            }
            RateLimitRecordSource::Distributed => {
                let remaining_window_secs = reservation.remaining_window_secs();
                if remaining_window_secs == 0 {
                    return;
                }

                #[cfg(feature = "gateway")]
                if let Some(redis) = &self.redis
                    && let Err(err) = redis.rate_limit_release(key, remaining_window_secs).await
                {
                    warn!("Redis rate-limit release failed for key {}: {}", key, err);
                }
            }
        }
    }

    fn release_local(&self, key: &str, recorded_at: Option<Instant>) {
        let limit = self.config.default_rpm as f64;
        let strategy = self.config.strategy.clone();
        let reservation_window = self.token_bucket_reservation_window();

        let _removed_entry = self.entries.remove_if_mut(key, |_, entry| {
            match &strategy {
                RateLimitStrategy::SlidingWindow | RateLimitStrategy::FixedWindow => {
                    if let Some(recorded_at) = recorded_at {
                        if let Some(position) =
                            entry.timestamps.iter().position(|&ts| ts == recorded_at)
                        {
                            entry.timestamps.remove(position);
                        }
                    } else {
                        entry.timestamps.pop();
                    }
                }
                RateLimitStrategy::TokenBucket => {
                    let should_refund = if let Some(recorded_at) = recorded_at {
                        let now = Instant::now();
                        entry
                            .timestamps
                            .retain(|&ts| now.saturating_duration_since(ts) < reservation_window);

                        if let Some(position) =
                            entry.timestamps.iter().position(|&ts| ts == recorded_at)
                        {
                            entry.timestamps.remove(position);
                            true
                        } else {
                            false
                        }
                    } else {
                        true
                    };

                    if should_refund {
                        entry.tokens = (entry.tokens + 1.0).min(limit);
                    }
                }
            }

            match &strategy {
                RateLimitStrategy::SlidingWindow | RateLimitStrategy::FixedWindow => {
                    entry.timestamps.is_empty()
                }
                RateLimitStrategy::TokenBucket => {
                    entry.timestamps.is_empty() && entry.tokens >= limit
                }
            }
        });
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
