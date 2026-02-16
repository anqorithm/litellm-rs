//! Tests for rate limiting functionality
//!
//! This module contains comprehensive tests for the rate limiter.

use super::engine::RateLimiter;
use super::types::{LimiterConfig, RateLimitKey, RateLimitResult, SlidingWindow, TokenBucket};
use std::time::{Duration, Instant};
use uuid::Uuid;

// ==================== RateLimiter Tests ====================

#[tokio::test]
async fn test_rate_limiter_creation() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(10),
        tpm: Some(1000),
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    limiter.add_config("test".to_string(), config).await;
}

#[tokio::test]
async fn test_rate_limit_check() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(2), // Very low limit for testing
        tpm: Some(100),
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let key = RateLimitKey::new("test".to_string()).with_user(Uuid::new_v4());

    limiter
        .add_config(limiter.build_key_string(&key), config)
        .await;

    // First request should be allowed
    let result = limiter.check_rate_limit(&key, 10).await.unwrap();
    assert!(result.allowed);

    // Second request should be allowed
    let result = limiter.check_rate_limit(&key, 10).await.unwrap();
    assert!(result.allowed);

    // Third request should be denied
    let result = limiter.check_rate_limit(&key, 10).await.unwrap();
    assert!(!result.allowed);
    assert_eq!(result.limit_type, Some("rpm".to_string()));
}

#[tokio::test]
async fn test_token_rate_limit() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: None,
        tpm: Some(50), // Low token limit
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let key = RateLimitKey::new("test".to_string()).with_user(Uuid::new_v4());

    limiter
        .add_config(limiter.build_key_string(&key), config)
        .await;

    // Request with 30 tokens should be allowed
    let result = limiter.check_rate_limit(&key, 30).await.unwrap();
    assert!(result.allowed);

    // Request with 25 tokens should be denied (30 + 25 > 50)
    let result = limiter.check_rate_limit(&key, 25).await.unwrap();
    assert!(!result.allowed);
    assert_eq!(result.limit_type, Some("tpm".to_string()));
}

#[tokio::test]
async fn test_rate_limiter_no_config() {
    let limiter = RateLimiter::new();
    let key = RateLimitKey::new("unconfigured".to_string());

    // Without config, should return error
    let result = limiter.check_rate_limit(&key, 100).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_rate_limiter_cleanup() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(10),
        tpm: None,
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let key = RateLimitKey::new("cleanup_test".to_string());
    limiter
        .add_config(limiter.build_key_string(&key), config)
        .await;

    // Make some requests
    for _ in 0..5 {
        let _ = limiter.check_rate_limit(&key, 0).await;
    }

    // Cleanup should run without error
    limiter.cleanup().await;
}

#[tokio::test]
async fn test_rate_limiter_get_status() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(10),
        tpm: Some(100),
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let key = RateLimitKey::new("status_test".to_string());
    limiter
        .add_config(limiter.build_key_string(&key), config)
        .await;

    // Get status
    let status = limiter.get_status(&key).await.unwrap();
    assert!(status.contains_key("rpm") || status.contains_key("tpm"));
}

// ==================== RateLimitKey Tests ====================

#[test]
fn test_key_building() {
    let limiter = RateLimiter::new();
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();

    let key = RateLimitKey::new("test".to_string())
        .with_user(user_id)
        .with_team(team_id)
        .with_ip("127.0.0.1".to_string());

    let key_str = limiter.build_key_string(&key);
    assert!(key_str.contains(&format!("user:{}", user_id)));
    assert!(key_str.contains(&format!("team:{}", team_id)));
    assert!(key_str.contains("ip:127.0.0.1"));
    assert!(key_str.contains("type:test"));
}

#[test]
fn test_rate_limit_key_new() {
    let key = RateLimitKey::new("api".to_string());
    assert_eq!(key.limit_type, "api");
    assert!(key.user_id.is_none());
    assert!(key.team_id.is_none());
    assert!(key.api_key_id.is_none());
    assert!(key.ip_address.is_none());
}

#[test]
fn test_rate_limit_key_with_user() {
    let user_id = Uuid::new_v4();
    let key = RateLimitKey::new("api".to_string()).with_user(user_id);
    assert_eq!(key.user_id, Some(user_id));
}

#[test]
fn test_rate_limit_key_with_team() {
    let team_id = Uuid::new_v4();
    let key = RateLimitKey::new("api".to_string()).with_team(team_id);
    assert_eq!(key.team_id, Some(team_id));
}

#[test]
fn test_rate_limit_key_with_api_key() {
    let api_key_id = Uuid::new_v4();
    let key = RateLimitKey::new("api".to_string()).with_api_key(api_key_id);
    assert_eq!(key.api_key_id, Some(api_key_id));
}

#[test]
fn test_rate_limit_key_with_ip() {
    let key = RateLimitKey::new("api".to_string()).with_ip("192.168.1.1".to_string());
    assert_eq!(key.ip_address, Some("192.168.1.1".to_string()));
}

#[test]
fn test_rate_limit_key_chaining() {
    let user_id = Uuid::new_v4();
    let team_id = Uuid::new_v4();
    let api_key_id = Uuid::new_v4();

    let key = RateLimitKey::new("api".to_string())
        .with_user(user_id)
        .with_team(team_id)
        .with_api_key(api_key_id)
        .with_ip("10.0.0.1".to_string());

    assert_eq!(key.user_id, Some(user_id));
    assert_eq!(key.team_id, Some(team_id));
    assert_eq!(key.api_key_id, Some(api_key_id));
    assert_eq!(key.ip_address, Some("10.0.0.1".to_string()));
    assert_eq!(key.limit_type, "api");
}

#[test]
fn test_rate_limit_key_partial() {
    let limiter = RateLimiter::new();
    let user_id = Uuid::new_v4();

    // Key with only user
    let key = RateLimitKey::new("chat".to_string()).with_user(user_id);
    let key_str = limiter.build_key_string(&key);
    assert!(key_str.contains(&format!("user:{}", user_id)));
    assert!(key_str.contains("type:chat"));
    assert!(!key_str.contains("team:"));
    assert!(!key_str.contains("key:"));
    assert!(!key_str.contains("ip:"));
}

// ==================== LimiterConfig Tests ====================

#[test]
fn test_rate_limit_config_clone() {
    let config = LimiterConfig {
        rpm: Some(100),
        tpm: Some(10000),
        rpd: Some(1000),
        tpd: Some(100000),
        concurrent: Some(10),
        burst: Some(20),
    };

    let cloned = config.clone();
    assert_eq!(cloned.rpm, config.rpm);
    assert_eq!(cloned.tpm, config.tpm);
    assert_eq!(cloned.rpd, config.rpd);
    assert_eq!(cloned.tpd, config.tpd);
    assert_eq!(cloned.concurrent, config.concurrent);
    assert_eq!(cloned.burst, config.burst);
}

#[test]
fn test_rate_limit_config_debug() {
    let config = LimiterConfig {
        rpm: Some(100),
        tpm: None,
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let debug_str = format!("{:?}", config);
    assert!(debug_str.contains("rpm"));
    assert!(debug_str.contains("100"));
}

// ==================== RateLimitResult Tests ====================

#[test]
fn test_rate_limit_result_allowed() {
    let result = RateLimitResult {
        allowed: true,
        remaining_requests: Some(9),
        remaining_tokens: Some(900),
        reset_time: Some(Duration::from_secs(60)),
        retry_after: None,
        limit_type: None,
    };

    assert!(result.allowed);
    assert_eq!(result.remaining_requests, Some(9));
    assert_eq!(result.remaining_tokens, Some(900));
    assert!(result.retry_after.is_none());
}

#[test]
fn test_rate_limit_result_denied() {
    let result = RateLimitResult {
        allowed: false,
        remaining_requests: Some(0),
        remaining_tokens: None,
        reset_time: Some(Duration::from_secs(60)),
        retry_after: Some(Duration::from_secs(30)),
        limit_type: Some("rpm".to_string()),
    };

    assert!(!result.allowed);
    assert_eq!(result.remaining_requests, Some(0));
    assert_eq!(result.limit_type, Some("rpm".to_string()));
    assert_eq!(result.retry_after, Some(Duration::from_secs(30)));
}

#[test]
fn test_rate_limit_result_clone() {
    let result = RateLimitResult {
        allowed: true,
        remaining_requests: Some(5),
        remaining_tokens: Some(500),
        reset_time: Some(Duration::from_secs(30)),
        retry_after: None,
        limit_type: None,
    };

    let cloned = result.clone();
    assert_eq!(cloned.allowed, result.allowed);
    assert_eq!(cloned.remaining_requests, result.remaining_requests);
    assert_eq!(cloned.remaining_tokens, result.remaining_tokens);
}

// ==================== TokenBucket Tests ====================

#[test]
fn test_token_bucket_creation() {
    let bucket = TokenBucket {
        tokens: 100.0,
        capacity: 100.0,
        refill_rate: 10.0,
        last_refill: Instant::now(),
    };

    assert_eq!(bucket.tokens, 100.0);
    assert_eq!(bucket.capacity, 100.0);
    assert_eq!(bucket.refill_rate, 10.0);
}

#[test]
fn test_token_bucket_clone() {
    let bucket = TokenBucket {
        tokens: 50.0,
        capacity: 100.0,
        refill_rate: 5.0,
        last_refill: Instant::now(),
    };

    let cloned = bucket.clone();
    assert_eq!(cloned.tokens, bucket.tokens);
    assert_eq!(cloned.capacity, bucket.capacity);
    assert_eq!(cloned.refill_rate, bucket.refill_rate);
}

// ==================== SlidingWindow Tests ====================

#[test]
fn test_sliding_window_creation() {
    let window = SlidingWindow {
        window_size: Duration::from_secs(60),
        requests: Vec::new(),
        tokens: Vec::new(),
    };

    assert_eq!(window.window_size, Duration::from_secs(60));
    assert!(window.requests.is_empty());
    assert!(window.tokens.is_empty());
}

#[test]
fn test_sliding_window_with_requests() {
    let now = Instant::now();
    let window = SlidingWindow {
        window_size: Duration::from_secs(60),
        requests: vec![now],
        tokens: vec![(now, 100)],
    };

    assert_eq!(window.requests.len(), 1);
    assert_eq!(window.tokens.len(), 1);
    assert_eq!(window.tokens[0].1, 100);
}

#[test]
fn test_sliding_window_clone() {
    let now = Instant::now();
    let window = SlidingWindow {
        window_size: Duration::from_secs(120),
        requests: vec![now],
        tokens: vec![(now, 50)],
    };

    let cloned = window.clone();
    assert_eq!(cloned.window_size, window.window_size);
    assert_eq!(cloned.requests.len(), window.requests.len());
    assert_eq!(cloned.tokens.len(), window.tokens.len());
}

// ==================== Integration Tests ====================

#[tokio::test]
async fn test_multiple_keys() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(5),
        tpm: None,
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let user1 = Uuid::new_v4();
    let user2 = Uuid::new_v4();

    let key1 = RateLimitKey::new("api".to_string()).with_user(user1);
    let key2 = RateLimitKey::new("api".to_string()).with_user(user2);

    limiter
        .add_config(limiter.build_key_string(&key1), config.clone())
        .await;
    limiter
        .add_config(limiter.build_key_string(&key2), config)
        .await;

    // Both users should have independent limits
    for _ in 0..5 {
        let result1 = limiter.check_rate_limit(&key1, 0).await.unwrap();
        let result2 = limiter.check_rate_limit(&key2, 0).await.unwrap();
        assert!(result1.allowed);
        assert!(result2.allowed);
    }

    // Both should be rate limited now
    let result1 = limiter.check_rate_limit(&key1, 0).await.unwrap();
    let result2 = limiter.check_rate_limit(&key2, 0).await.unwrap();
    assert!(!result1.allowed);
    assert!(!result2.allowed);
}

#[tokio::test]
async fn test_combined_rpm_and_tpm_limits() {
    let limiter = RateLimiter::new();

    let config = LimiterConfig {
        rpm: Some(10),  // High request limit
        tpm: Some(100), // Low token limit
        rpd: None,
        tpd: None,
        concurrent: None,
        burst: None,
    };

    let key = RateLimitKey::new("combined".to_string());
    limiter
        .add_config(limiter.build_key_string(&key), config)
        .await;

    // First request with 60 tokens should be allowed
    let result = limiter.check_rate_limit(&key, 60).await.unwrap();
    assert!(result.allowed);

    // Second request with 50 tokens should be denied (60 + 50 > 100)
    let result = limiter.check_rate_limit(&key, 50).await.unwrap();
    assert!(!result.allowed);
    assert_eq!(result.limit_type, Some("tpm".to_string()));
}
