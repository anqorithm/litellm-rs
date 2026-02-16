//! Rate limiting types and data structures
//!
//! This module defines the core types used for rate limiting functionality.

use std::time::{Duration, Instant};
use uuid::Uuid;

/// Rate limit configuration
#[derive(Debug, Clone)]
pub struct LimiterConfig {
    /// Requests per minute
    pub rpm: Option<u32>,
    /// Tokens per minute
    pub tpm: Option<u32>,
    /// Requests per day
    pub rpd: Option<u32>,
    /// Tokens per day
    pub tpd: Option<u32>,
    /// Concurrent requests
    pub concurrent: Option<u32>,
    /// Burst allowance
    pub burst: Option<u32>,
}

/// Token bucket for rate limiting
#[derive(Debug, Clone)]
pub struct TokenBucket {
    /// Current token count
    pub tokens: f64,
    /// Maximum tokens (capacity)
    pub capacity: f64,
    /// Refill rate (tokens per second)
    pub refill_rate: f64,
    /// Last refill time
    pub last_refill: Instant,
}

/// Sliding window for request counting
#[derive(Debug, Clone)]
pub struct SlidingWindow {
    /// Window size in seconds
    pub window_size: Duration,
    /// Request timestamps
    pub requests: Vec<Instant>,
    /// Token counts with timestamps
    pub tokens: Vec<(Instant, u32)>,
}

/// Rate limit check result
#[derive(Debug, Clone)]
pub struct RateLimitResult {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Remaining requests in current window
    pub remaining_requests: Option<u32>,
    /// Remaining tokens in current window
    pub remaining_tokens: Option<u32>,
    /// Time until reset
    pub reset_time: Option<Duration>,
    /// Retry after duration
    pub retry_after: Option<Duration>,
    /// Rate limit type that was hit
    pub limit_type: Option<String>,
}

/// Rate limit key components
#[derive(Debug, Clone)]
pub struct RateLimitKey {
    /// User ID
    pub user_id: Option<Uuid>,
    /// Team ID
    pub team_id: Option<Uuid>,
    /// API Key ID
    pub api_key_id: Option<Uuid>,
    /// IP address
    pub ip_address: Option<String>,
    /// Rate limit type
    pub limit_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== LimiterConfig Tests ====================

    #[test]
    fn test_rate_limit_config_full() {
        let config = LimiterConfig {
            rpm: Some(100),
            tpm: Some(10000),
            rpd: Some(1000),
            tpd: Some(100000),
            concurrent: Some(10),
            burst: Some(20),
        };
        assert_eq!(config.rpm, Some(100));
        assert_eq!(config.tpm, Some(10000));
        assert_eq!(config.rpd, Some(1000));
        assert_eq!(config.tpd, Some(100000));
        assert_eq!(config.concurrent, Some(10));
        assert_eq!(config.burst, Some(20));
    }

    #[test]
    fn test_rate_limit_config_minimal() {
        let config = LimiterConfig {
            rpm: Some(60),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        assert_eq!(config.rpm, Some(60));
        assert!(config.tpm.is_none());
    }

    #[test]
    fn test_rate_limit_config_clone() {
        let config = LimiterConfig {
            rpm: Some(100),
            tpm: Some(5000),
            rpd: None,
            tpd: None,
            concurrent: Some(5),
            burst: Some(10),
        };
        let cloned = config.clone();
        assert_eq!(config.rpm, cloned.rpm);
        assert_eq!(config.concurrent, cloned.concurrent);
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
        assert!((bucket.tokens - 100.0).abs() < f64::EPSILON);
        assert!((bucket.capacity - 100.0).abs() < f64::EPSILON);
        assert!((bucket.refill_rate - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_token_bucket_partial_fill() {
        let bucket = TokenBucket {
            tokens: 50.0,
            capacity: 100.0,
            refill_rate: 5.0,
            last_refill: Instant::now(),
        };
        assert!(bucket.tokens < bucket.capacity);
    }

    #[test]
    fn test_token_bucket_clone() {
        let bucket = TokenBucket {
            tokens: 75.0,
            capacity: 100.0,
            refill_rate: 10.0,
            last_refill: Instant::now(),
        };
        let cloned = bucket.clone();
        assert!((bucket.tokens - cloned.tokens).abs() < f64::EPSILON);
        assert!((bucket.capacity - cloned.capacity).abs() < f64::EPSILON);
    }

    // ==================== SlidingWindow Tests ====================

    #[test]
    fn test_sliding_window_creation() {
        let window = SlidingWindow {
            window_size: Duration::from_secs(60),
            requests: vec![],
            tokens: vec![],
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
            requests: vec![now, now, now],
            tokens: vec![(now, 100), (now, 200)],
        };
        assert_eq!(window.requests.len(), 3);
        assert_eq!(window.tokens.len(), 2);
    }

    #[test]
    fn test_sliding_window_clone() {
        let window = SlidingWindow {
            window_size: Duration::from_secs(120),
            requests: vec![Instant::now()],
            tokens: vec![],
        };
        let cloned = window.clone();
        assert_eq!(window.window_size, cloned.window_size);
        assert_eq!(window.requests.len(), cloned.requests.len());
    }

    // ==================== RateLimitResult Tests ====================

    #[test]
    fn test_rate_limit_result_allowed() {
        let result = RateLimitResult {
            allowed: true,
            remaining_requests: Some(99),
            remaining_tokens: Some(9900),
            reset_time: Some(Duration::from_secs(60)),
            retry_after: None,
            limit_type: None,
        };
        assert!(result.allowed);
        assert_eq!(result.remaining_requests, Some(99));
        assert!(result.retry_after.is_none());
    }

    #[test]
    fn test_rate_limit_result_denied() {
        let result = RateLimitResult {
            allowed: false,
            remaining_requests: Some(0),
            remaining_tokens: Some(0),
            reset_time: Some(Duration::from_secs(30)),
            retry_after: Some(Duration::from_secs(30)),
            limit_type: Some("rpm".to_string()),
        };
        assert!(!result.allowed);
        assert_eq!(result.remaining_requests, Some(0));
        assert!(result.retry_after.is_some());
        assert_eq!(result.limit_type, Some("rpm".to_string()));
    }

    #[test]
    fn test_rate_limit_result_clone() {
        let result = RateLimitResult {
            allowed: true,
            remaining_requests: Some(50),
            remaining_tokens: Some(5000),
            reset_time: None,
            retry_after: None,
            limit_type: None,
        };
        let cloned = result.clone();
        assert_eq!(result.allowed, cloned.allowed);
        assert_eq!(result.remaining_requests, cloned.remaining_requests);
    }

    // ==================== RateLimitKey Tests ====================

    #[test]
    fn test_rate_limit_key_user() {
        let user_id = Uuid::new_v4();
        let key = RateLimitKey {
            user_id: Some(user_id),
            team_id: None,
            api_key_id: None,
            ip_address: None,
            limit_type: "user".to_string(),
        };
        assert_eq!(key.user_id, Some(user_id));
        assert!(key.team_id.is_none());
        assert_eq!(key.limit_type, "user");
    }

    #[test]
    fn test_rate_limit_key_ip() {
        let key = RateLimitKey {
            user_id: None,
            team_id: None,
            api_key_id: None,
            ip_address: Some("192.168.1.1".to_string()),
            limit_type: "ip".to_string(),
        };
        assert_eq!(key.ip_address, Some("192.168.1.1".to_string()));
        assert_eq!(key.limit_type, "ip");
    }

    #[test]
    fn test_rate_limit_key_full() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();
        let key = RateLimitKey {
            user_id: Some(user_id),
            team_id: Some(team_id),
            api_key_id: Some(api_key_id),
            ip_address: Some("10.0.0.1".to_string()),
            limit_type: "combined".to_string(),
        };
        assert!(key.user_id.is_some());
        assert!(key.team_id.is_some());
        assert!(key.api_key_id.is_some());
        assert!(key.ip_address.is_some());
    }

    #[test]
    fn test_rate_limit_key_clone() {
        let key = RateLimitKey {
            user_id: Some(Uuid::new_v4()),
            team_id: None,
            api_key_id: None,
            ip_address: Some("127.0.0.1".to_string()),
            limit_type: "test".to_string(),
        };
        let cloned = key.clone();
        assert_eq!(key.user_id, cloned.user_id);
        assert_eq!(key.ip_address, cloned.ip_address);
        assert_eq!(key.limit_type, cloned.limit_type);
    }
}
