//! Tests for rate limiter

#[cfg(test)]
use super::limiter::RateLimiter;
use crate::config::models::rate_limit::{RateLimitConfig, RateLimitStrategy};
use std::time::Duration;

fn test_config(enabled: bool, rpm: u32) -> RateLimitConfig {
    RateLimitConfig {
        enabled,
        default_rpm: rpm,
        default_tpm: 100000,
        strategy: RateLimitStrategy::SlidingWindow,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_rate_limiter_disabled() {
    let limiter = RateLimiter::new(test_config(false, 10));

    for _ in 0..100 {
        let result = limiter.check_and_record("test-key").await;
        assert!(result.allowed);
    }
}

#[tokio::test]
async fn test_sliding_window_allows_within_limit() {
    let limiter = RateLimiter::new(test_config(true, 10));

    for i in 0..10 {
        let result = limiter.check_and_record("test-key").await;
        assert!(result.allowed, "Request {} should be allowed", i);
    }
}

#[tokio::test]
async fn test_sliding_window_blocks_over_limit() {
    let limiter = RateLimiter::new(test_config(true, 5));

    // Fill up the limit using atomic check_and_record
    for _ in 0..5 {
        let result = limiter.check_and_record("test-key").await;
        assert!(result.allowed);
    }

    // This should be blocked
    let result = limiter.check_and_record("test-key").await;
    assert!(!result.allowed);
    assert!(result.retry_after_secs.is_some());
}

#[tokio::test]
async fn test_different_keys_independent() {
    let limiter = RateLimiter::new(test_config(true, 2));

    // Fill up limit for key1 using atomic method
    limiter.check_and_record("key1").await;
    limiter.check_and_record("key1").await;

    // key1 should be blocked
    let result = limiter.check_and_record("key1").await;
    assert!(!result.allowed);

    // key2 should still work
    let result = limiter.check_and_record("key2").await;
    assert!(result.allowed);
}

#[tokio::test]
async fn test_token_bucket() {
    let config = RateLimitConfig {
        enabled: true,
        default_rpm: 60, // 1 per second
        default_tpm: 100000,
        strategy: RateLimitStrategy::TokenBucket,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    // Should allow initial requests (bucket starts full)
    let result = limiter.check_and_record("test-key").await;
    assert!(result.allowed);
}

#[tokio::test]
async fn test_fixed_window() {
    let config = RateLimitConfig {
        enabled: true,
        default_rpm: 5,
        default_tpm: 100000,
        strategy: RateLimitStrategy::FixedWindow,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    for _ in 0..5 {
        let result = limiter.check_and_record("test-key").await;
        assert!(result.allowed);
    }

    // Should be blocked
    let result = limiter.check_and_record("test-key").await;
    assert!(!result.allowed);
}

#[tokio::test]
async fn test_remaining_count() {
    let limiter = RateLimiter::new(test_config(true, 5));

    // First check (no record) should show 5 remaining
    let result = limiter.check("test-key").await;
    assert_eq!(result.remaining, 5);

    // After check_and_record, remaining should be 4
    let result = limiter.check_and_record("test-key").await;
    assert_eq!(result.remaining, 4);

    // Do two more
    limiter.check_and_record("test-key").await;
    limiter.check_and_record("test-key").await;

    // Should have 2 remaining
    let result = limiter.check("test-key").await;
    assert_eq!(result.remaining, 2);
}

#[tokio::test]
async fn test_atomic_check_and_record() {
    let limiter = RateLimiter::new(test_config(true, 3));

    // Use atomic method - should record and decrement in one operation
    let r1 = limiter.check_and_record("atomic-key").await;
    assert!(r1.allowed);
    assert_eq!(r1.remaining, 2); // 3-1=2 after recording

    let r2 = limiter.check_and_record("atomic-key").await;
    assert!(r2.allowed);
    assert_eq!(r2.remaining, 1);

    let r3 = limiter.check_and_record("atomic-key").await;
    assert!(r3.allowed);
    assert_eq!(r3.remaining, 0);

    // 4th request should be blocked
    let r4 = limiter.check_and_record("atomic-key").await;
    assert!(!r4.allowed);
}

#[tokio::test]
async fn test_cleanup() {
    let limiter = RateLimiter::with_window(test_config(true, 100), Duration::from_millis(50));

    // Use atomic method
    limiter.check_and_record("key1").await;
    limiter.check_and_record("key2").await;

    // Wait for window to expire
    tokio::time::sleep(Duration::from_millis(100)).await;

    limiter.cleanup().await;

    // After cleanup, should have full limit again
    let result = limiter.check("key1").await;
    assert!(result.allowed);
    assert_eq!(result.remaining, 100);
}
