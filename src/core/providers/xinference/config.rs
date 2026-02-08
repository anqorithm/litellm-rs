//! Xinference Provider Configuration

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Xinference provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XinferenceConfig {
    /// API key for authentication (optional for local deployments)
    pub api_key: Option<String>,

    /// Base URL of the Xinference server
    pub api_base: Option<String>,

    /// Request timeout in seconds (longer for local inference)
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for XinferenceConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for XinferenceConfig {
    fn validate(&self) -> Result<(), String> {
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }
        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        self.api_base.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl XinferenceConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("XINFERENCE_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("XINFERENCE_API_BASE").ok())
            .unwrap_or_else(|| "http://localhost:9997/v1".to_string())
    }
}

fn default_timeout() -> u64 {
    120 // Longer timeout for local inference
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = XinferenceConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_get_api_base_default() {
        let config = XinferenceConfig::default();
        assert_eq!(config.get_api_base(), "http://localhost:9997/v1");
    }

    #[test]
    fn test_config_validation() {
        let config = XinferenceConfig::default();
        assert!(config.validate().is_ok());

        let invalid = XinferenceConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(invalid.validate().is_err());
    }
}
