//! Utility functions and helper implementations
//!
//! This module contains utility methods for the rate limiter and RateLimitKey implementation.

use crate::utils::error::gateway_error::Result;
use std::collections::HashMap;
#[allow(unused_imports)]
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::engine::RateLimiter;
use super::types::{RateLimitKey, RateLimitResult};

impl RateLimiter {
    /// Record a request
    pub(super) async fn record_request(&self, key: &str, tokens: u32) -> Result<()> {
        let now = Instant::now();

        // Record in all relevant windows
        let mut windows = self.windows.write().await;

        // RPM window
        let rpm_key = format!("{}_rpm", key);
        if let Some(window) = windows.get_mut(&rpm_key) {
            window.requests.push(now);
        }

        // TPM window
        let tpm_key = format!("{}_tpm", key);
        if let Some(window) = windows.get_mut(&tpm_key) {
            window.tokens.push((now, tokens));
        }

        // RPD window
        let rpd_key = format!("{}_rpd", key);
        if let Some(window) = windows.get_mut(&rpd_key) {
            window.requests.push(now);
        }

        // TPD window
        let tpd_key = format!("{}_tpd", key);
        if let Some(window) = windows.get_mut(&tpd_key) {
            window.tokens.push((now, tokens));
        }

        Ok(())
    }

    /// Build key string from components
    pub(super) fn build_key_string(&self, key: &RateLimitKey) -> String {
        let mut parts = Vec::new();

        if let Some(user_id) = key.user_id {
            parts.push(format!("user:{}", user_id));
        }

        if let Some(team_id) = key.team_id {
            parts.push(format!("team:{}", team_id));
        }

        if let Some(api_key_id) = key.api_key_id {
            parts.push(format!("key:{}", api_key_id));
        }

        if let Some(ip) = &key.ip_address {
            parts.push(format!("ip:{}", ip));
        }

        parts.push(format!("type:{}", key.limit_type));

        parts.join(":")
    }

    /// Clean up old entries
    pub async fn cleanup(&self) {
        let now = Instant::now();
        let mut windows = self.windows.write().await;

        windows.retain(|_, window| {
            let window_start = now - window.window_size;
            window
                .requests
                .retain(|&timestamp| timestamp > window_start);
            window
                .tokens
                .retain(|(timestamp, _)| *timestamp > window_start);

            // Keep window if it has recent activity
            !window.requests.is_empty() || !window.tokens.is_empty()
        });
    }

    /// Get rate limit status
    pub async fn get_status(&self, key: &RateLimitKey) -> Result<HashMap<String, RateLimitResult>> {
        let key_str = self.build_key_string(key);
        let mut status = HashMap::new();

        let configs = self.configs.read().await;
        if let Some(config) = configs.get(&key_str).or_else(|| configs.get("default")) {
            if let Some(rpm) = config.rpm {
                let result = self
                    .check_sliding_window_requests(
                        &format!("{}_rpm", key_str),
                        Duration::from_secs(60),
                        rpm,
                    )
                    .await?;
                status.insert("rpm".to_string(), result);
            }

            if let Some(tpm) = config.tpm {
                let result = self
                    .check_sliding_window_tokens(
                        &format!("{}_tpm", key_str),
                        Duration::from_secs(60),
                        tpm,
                        0, // Don't consume tokens for status check
                    )
                    .await?;
                status.insert("tpm".to_string(), result);
            }
        }

        Ok(status)
    }
}

impl RateLimitKey {
    /// Create a new rate limit key
    pub fn new(limit_type: String) -> Self {
        Self {
            user_id: None,
            team_id: None,
            api_key_id: None,
            ip_address: None,
            limit_type,
        }
    }

    /// Set user ID
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set team ID
    pub fn with_team(mut self, team_id: Uuid) -> Self {
        self.team_id = Some(team_id);
        self
    }

    /// Set API key ID
    pub fn with_api_key(mut self, api_key_id: Uuid) -> Self {
        self.api_key_id = Some(api_key_id);
        self
    }

    /// Set IP address
    pub fn with_ip(mut self, ip_address: String) -> Self {
        self.ip_address = Some(ip_address);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::LimiterConfig;
    use super::*;

    // ==================== RateLimitKey Builder Tests ====================

    #[test]
    fn test_rate_limit_key_new() {
        let key = RateLimitKey::new("global".to_string());
        assert!(key.user_id.is_none());
        assert!(key.team_id.is_none());
        assert!(key.api_key_id.is_none());
        assert!(key.ip_address.is_none());
        assert_eq!(key.limit_type, "global");
    }

    #[test]
    fn test_rate_limit_key_with_user() {
        let user_id = Uuid::new_v4();
        let key = RateLimitKey::new("user".to_string()).with_user(user_id);
        assert_eq!(key.user_id, Some(user_id));
        assert_eq!(key.limit_type, "user");
    }

    #[test]
    fn test_rate_limit_key_with_team() {
        let team_id = Uuid::new_v4();
        let key = RateLimitKey::new("team".to_string()).with_team(team_id);
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
        let key = RateLimitKey::new("ip".to_string()).with_ip("192.168.1.1".to_string());
        assert_eq!(key.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_rate_limit_key_builder_chain() {
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        let key = RateLimitKey::new("combined".to_string())
            .with_user(user_id)
            .with_team(team_id)
            .with_api_key(api_key_id)
            .with_ip("10.0.0.1".to_string());

        assert_eq!(key.user_id, Some(user_id));
        assert_eq!(key.team_id, Some(team_id));
        assert_eq!(key.api_key_id, Some(api_key_id));
        assert_eq!(key.ip_address, Some("10.0.0.1".to_string()));
        assert_eq!(key.limit_type, "combined");
    }

    #[test]
    fn test_rate_limit_key_ipv6() {
        let key = RateLimitKey::new("ip".to_string())
            .with_ip("2001:0db8:85a3:0000:0000:8a2e:0370:7334".to_string());
        assert!(key.ip_address.is_some());
        assert!(key.ip_address.unwrap().contains("2001"));
    }

    #[test]
    fn test_rate_limit_key_localhost() {
        let key = RateLimitKey::new("ip".to_string()).with_ip("127.0.0.1".to_string());
        assert_eq!(key.ip_address, Some("127.0.0.1".to_string()));
    }

    // ==================== RateLimiter Build Key String Tests ====================

    #[tokio::test]
    async fn test_build_key_string_minimal() {
        let limiter = RateLimiter::new();
        let key = RateLimitKey::new("global".to_string());
        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains("type:global"));
    }

    #[tokio::test]
    async fn test_build_key_string_with_user() {
        let limiter = RateLimiter::new();
        let user_id = Uuid::new_v4();
        let key = RateLimitKey::new("user".to_string()).with_user(user_id);
        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains(&format!("user:{}", user_id)));
        assert!(key_str.contains("type:user"));
    }

    #[tokio::test]
    async fn test_build_key_string_with_team() {
        let limiter = RateLimiter::new();
        let team_id = Uuid::new_v4();
        let key = RateLimitKey::new("team".to_string()).with_team(team_id);
        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains(&format!("team:{}", team_id)));
    }

    #[tokio::test]
    async fn test_build_key_string_with_api_key() {
        let limiter = RateLimiter::new();
        let api_key_id = Uuid::new_v4();
        let key = RateLimitKey::new("api".to_string()).with_api_key(api_key_id);
        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains(&format!("key:{}", api_key_id)));
    }

    #[tokio::test]
    async fn test_build_key_string_with_ip() {
        let limiter = RateLimiter::new();
        let key = RateLimitKey::new("ip".to_string()).with_ip("192.168.1.100".to_string());
        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains("ip:192.168.1.100"));
    }

    #[tokio::test]
    async fn test_build_key_string_full() {
        let limiter = RateLimiter::new();
        let user_id = Uuid::new_v4();
        let team_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        let key = RateLimitKey::new("combined".to_string())
            .with_user(user_id)
            .with_team(team_id)
            .with_api_key(api_key_id)
            .with_ip("10.0.0.5".to_string());

        let key_str = limiter.build_key_string(&key);

        assert!(key_str.contains(&format!("user:{}", user_id)));
        assert!(key_str.contains(&format!("team:{}", team_id)));
        assert!(key_str.contains(&format!("key:{}", api_key_id)));
        assert!(key_str.contains("ip:10.0.0.5"));
        assert!(key_str.contains("type:combined"));
        assert!(key_str.contains(":")); // Parts are colon-separated
    }

    // ==================== RateLimiter Cleanup Tests ====================

    #[tokio::test]
    async fn test_cleanup_empty_limiter() {
        let limiter = RateLimiter::new();
        limiter.cleanup().await;
        // Should not panic
    }

    #[tokio::test]
    async fn test_cleanup_with_windows() {
        let limiter = RateLimiter::new();

        // Add a config and do a rate limit check to create windows
        let config = LimiterConfig {
            rpm: Some(100),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        limiter.add_config("default".to_string(), config).await;

        // After cleanup, should still work
        limiter.cleanup().await;
    }

    // ==================== RateLimiter Get Status Tests ====================

    #[tokio::test]
    async fn test_get_status_no_config() {
        let limiter = RateLimiter::new();
        let key = RateLimitKey::new("test".to_string());

        let status = limiter.get_status(&key).await;
        assert!(status.is_ok());
        assert!(status.unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_get_status_with_rpm_config() {
        let limiter = RateLimiter::new();

        let config = LimiterConfig {
            rpm: Some(60),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        limiter.add_config("default".to_string(), config).await;

        let key = RateLimitKey::new("test".to_string());
        let status = limiter.get_status(&key).await.unwrap();

        assert!(status.contains_key("rpm"));
    }

    #[tokio::test]
    async fn test_get_status_with_tpm_config() {
        let limiter = RateLimiter::new();

        let config = LimiterConfig {
            rpm: None,
            tpm: Some(10000),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        limiter.add_config("default".to_string(), config).await;

        let key = RateLimitKey::new("test".to_string());
        let status = limiter.get_status(&key).await.unwrap();

        assert!(status.contains_key("tpm"));
    }

    #[tokio::test]
    async fn test_get_status_with_full_config() {
        let limiter = RateLimiter::new();

        let config = LimiterConfig {
            rpm: Some(100),
            tpm: Some(50000),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        limiter.add_config("default".to_string(), config).await;

        let key = RateLimitKey::new("test".to_string());
        let status = limiter.get_status(&key).await.unwrap();

        assert!(status.contains_key("rpm"));
        assert!(status.contains_key("tpm"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_rate_limit_key_empty_limit_type() {
        let key = RateLimitKey::new(String::new());
        assert!(key.limit_type.is_empty());
    }

    #[test]
    fn test_rate_limit_key_special_characters_in_ip() {
        // Unusual but valid scenarios
        let key = RateLimitKey::new("test".to_string()).with_ip("::1".to_string()); // IPv6 localhost
        assert_eq!(key.ip_address, Some("::1".to_string()));
    }

    #[tokio::test]
    async fn test_build_key_string_deterministic() {
        let limiter = RateLimiter::new();
        let user_id = Uuid::new_v4();

        let key1 = RateLimitKey::new("test".to_string()).with_user(user_id);
        let key2 = RateLimitKey::new("test".to_string()).with_user(user_id);

        let str1 = limiter.build_key_string(&key1);
        let str2 = limiter.build_key_string(&key2);

        assert_eq!(str1, str2);
    }

    #[tokio::test]
    async fn test_limiter_multiple_configs() {
        let limiter = RateLimiter::new();

        let config1 = LimiterConfig {
            rpm: Some(100),
            tpm: None,
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };
        let config2 = LimiterConfig {
            rpm: Some(50),
            tpm: Some(10000),
            rpd: None,
            tpd: None,
            concurrent: None,
            burst: None,
        };

        limiter.add_config("config1".to_string(), config1).await;
        limiter.add_config("config2".to_string(), config2).await;

        // Both configs should be stored
        let configs = limiter.configs.read().await;
        assert!(configs.contains_key("config1"));
        assert!(configs.contains_key("config2"));
    }
}
