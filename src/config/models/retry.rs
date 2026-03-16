//! Retry configuration types

use super::defaults::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;

fn default_backoff_multiplier() -> f64 {
    2.0
}

/// Retry configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Initial delay (milliseconds)
    #[serde(default = "default_initial_delay_ms")]
    pub initial_delay_ms: u64,
    /// Maximum delay (milliseconds)
    #[serde(default = "default_max_delay_ms")]
    pub max_delay_ms: u64,
    /// Backoff multiplier for exponential backoff (set to 1.0 to disable)
    #[serde(default = "default_backoff_multiplier")]
    pub backoff_multiplier: f64,
    /// Add random jitter
    #[serde(default = "default_true")]
    pub jitter: bool,
    /// Retryable error types
    #[serde(default)]
    pub retryable_errors: Vec<String>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: default_max_retries(),
            initial_delay_ms: default_initial_delay_ms(),
            max_delay_ms: default_max_delay_ms(),
            backoff_multiplier: 2.0,
            jitter: true,
            retryable_errors: vec![
                "network_error".to_string(),
                "timeout_error".to_string(),
                "rate_limit_error".to_string(),
                "server_error".to_string(),
            ],
        }
    }
}

impl RetryConfig {
    /// Get initial delay as Duration
    pub fn initial_delay(&self) -> Duration {
        Duration::from_millis(self.initial_delay_ms)
    }

    /// Get max delay as Duration
    pub fn max_delay(&self) -> Duration {
        Duration::from_millis(self.max_delay_ms)
    }

    /// Calculate delay for a given retry attempt (0-indexed)
    pub fn delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.initial_delay_ms as f64 * self.backoff_multiplier.powi(attempt as i32);
        let capped = delay_ms.min(self.max_delay_ms as f64);
        Duration::from_millis(capped as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== RetryConfig Default Tests ====================

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(config.jitter);
        assert_eq!(config.retryable_errors.len(), 4);
    }

    #[test]
    fn test_retry_config_default_retryable_errors() {
        let config = RetryConfig::default();
        assert!(
            config
                .retryable_errors
                .contains(&"network_error".to_string())
        );
        assert!(
            config
                .retryable_errors
                .contains(&"timeout_error".to_string())
        );
        assert!(
            config
                .retryable_errors
                .contains(&"rate_limit_error".to_string())
        );
        assert!(
            config
                .retryable_errors
                .contains(&"server_error".to_string())
        );
    }

    // ==================== RetryConfig Structure Tests ====================

    #[test]
    fn test_retry_config_structure() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 200,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            jitter: false,
            retryable_errors: vec!["custom_error".to_string()],
        };
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_delay_ms, 200);
        assert_eq!(config.max_delay_ms, 60000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(!config.jitter);
        assert_eq!(config.retryable_errors.len(), 1);
    }

    #[test]
    fn test_retry_config_no_retries() {
        let config = RetryConfig {
            max_retries: 0,
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 1.0,
            jitter: false,
            retryable_errors: vec![],
        };
        assert_eq!(config.max_retries, 0);
        assert!((config.backoff_multiplier - 1.0).abs() < f64::EPSILON);
        assert!(config.retryable_errors.is_empty());
    }

    // ==================== RetryConfig Serialization Tests ====================

    #[test]
    fn test_retry_config_serialization() {
        let config = RetryConfig {
            max_retries: 3,
            initial_delay_ms: 100,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter: true,
            retryable_errors: vec!["error1".to_string(), "error2".to_string()],
        };
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["max_retries"], 3);
        assert_eq!(json["initial_delay_ms"], 100);
        assert_eq!(json["max_delay_ms"], 30000);
        assert!((json["backoff_multiplier"].as_f64().unwrap() - 2.0).abs() < f64::EPSILON);
        assert_eq!(json["jitter"], true);
        assert!(json["retryable_errors"].is_array());
    }

    #[test]
    fn test_retry_config_deserialization() {
        let json = r#"{
            "max_retries": 10,
            "initial_delay_ms": 500,
            "max_delay_ms": 120000,
            "backoff_multiplier": 1.0,
            "jitter": true,
            "retryable_errors": ["connection_refused", "dns_error"]
        }"#;
        let config: RetryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_retries, 10);
        assert_eq!(config.initial_delay_ms, 500);
        assert_eq!(config.max_delay_ms, 120000);
        assert!((config.backoff_multiplier - 1.0).abs() < f64::EPSILON);
        assert!(config.jitter);
        assert_eq!(config.retryable_errors.len(), 2);
    }

    #[test]
    fn test_retry_config_deserialization_defaults() {
        let json = r#"{}"#;
        let config: RetryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_delay_ms, 100);
        assert_eq!(config.max_delay_ms, 30000);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_deserialization_partial() {
        let json = r#"{"max_retries": 7}"#;
        let config: RetryConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.max_retries, 7);
        assert_eq!(config.initial_delay_ms, 100);
        assert!((config.backoff_multiplier - 2.0).abs() < f64::EPSILON);
    }

    // ==================== RetryConfig Clone Tests ====================

    #[test]
    fn test_retry_config_clone() {
        let config = RetryConfig {
            max_retries: 5,
            initial_delay_ms: 250,
            max_delay_ms: 60000,
            backoff_multiplier: 2.0,
            jitter: true,
            retryable_errors: vec!["error".to_string()],
        };
        let cloned = config.clone();
        assert_eq!(config.max_retries, cloned.max_retries);
        assert_eq!(config.initial_delay_ms, cloned.initial_delay_ms);
        assert_eq!(config.retryable_errors, cloned.retryable_errors);
    }
}
