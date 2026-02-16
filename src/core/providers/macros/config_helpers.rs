//! Macros for provider implementation
//!
//! These macros help reduce boilerplate code when implementing providers,
//! following Rust's principle of zero-cost abstractions.

use crate::core::providers::unified_provider::ProviderError;

// ==================== Configuration Extraction Helpers ====================
// These helpers reduce boilerplate for extracting required/optional config values

/// Extract a required string value from configuration JSON
///
/// # Example
/// ```rust
/// # use litellm_rs::core::providers::macros::require_config_str;
/// # fn example() -> Result<(), litellm_rs::ProviderError> {
/// let config = serde_json::json!({"api_key": "sk-123"});
/// let api_key = require_config_str(&config, "api_key", "openai")?;
/// assert_eq!(api_key, "sk-123");
/// # Ok(())
/// # }
/// ```
#[inline]
pub fn require_config_str<'a>(
    config: &'a serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<&'a str, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional string value from configuration JSON
#[inline]
pub fn get_config_str<'a>(config: &'a serde_json::Value, key: &str) -> Option<&'a str> {
    config.get(key).and_then(|v| v.as_str())
}

/// Extract a required u64 value from configuration JSON
#[inline]
pub fn require_config_u64(
    config: &serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<u64, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_u64())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional u64 value from configuration JSON with a default
#[inline]
pub fn get_config_u64_or(config: &serde_json::Value, key: &str, default: u64) -> u64 {
    config.get(key).and_then(|v| v.as_u64()).unwrap_or(default)
}

/// Extract a required bool value from configuration JSON
#[inline]
pub fn require_config_bool(
    config: &serde_json::Value,
    key: &str,
    provider: &'static str,
) -> Result<bool, ProviderError> {
    config
        .get(key)
        .and_then(|v| v.as_bool())
        .ok_or_else(|| ProviderError::configuration(provider, format!("{} is required", key)))
}

/// Extract an optional bool value from configuration JSON with a default
#[inline]
pub fn get_config_bool_or(config: &serde_json::Value, key: &str, default: bool) -> bool {
    config.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}

/// Macro to extract required configuration value with provider context
///
/// # Example
/// ```rust
/// # use litellm_rs::require_config;
/// # fn example() -> Result<(), litellm_rs::ProviderError> {
/// let config = serde_json::json!({"api_key": "sk-123", "timeout": 30});
/// // Extract required string
/// let api_key = require_config!(&config, "api_key", str, "openai")?;
///
/// // Extract required u64
/// let timeout = require_config!(&config, "timeout", u64, "openai")?;
/// # Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! require_config {
    ($config:expr, $key:literal, str, $provider:literal) => {
        $crate::core::providers::macros::require_config_str($config, $key, $provider)
    };
    ($config:expr, $key:literal, u64, $provider:literal) => {
        $crate::core::providers::macros::require_config_u64($config, $key, $provider)
    };
    ($config:expr, $key:literal, bool, $provider:literal) => {
        $crate::core::providers::macros::require_config_bool($config, $key, $provider)
    };
}
