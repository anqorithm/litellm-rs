//! xAI Provider Configuration

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// xAI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XAIConfig {
    /// API key for authentication
    pub api_key: Option<String>,

    /// API base URL (defaults to <https://api.x.ai>)
    pub api_base: Option<String>,

    /// Organization ID (optional)
    pub organization_id: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,

    /// Enable web search capability for Grok models
    #[serde(default = "default_web_search")]
    pub enable_web_search: bool,
}

impl Default for XAIConfig {
    fn default() -> Self {
        Self {
            api_key: std::env::var("XAI_API_KEY").ok(),
            api_base: None,
            organization_id: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            enable_web_search: default_web_search(),
        }
    }
}

impl ProviderConfig for XAIConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() {
            return Err("XAI API key is required".to_string());
        }

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

impl XAIConfig {
    /// Get the API base URL
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("XAI_API_BASE").ok())
            .unwrap_or_else(|| "https://api.x.ai/v1".to_string())
    }

    /// Get the API key
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("XAI_API_KEY").ok())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

fn default_web_search() -> bool {
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = XAIConfig {
            api_key: None,
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(config.enable_web_search);
    }

    #[test]
    fn test_validate_missing_api_key() {
        let config = XAIConfig {
            api_key: None,
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_validate_zero_timeout() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: None,
            organization_id: None,
            timeout: 0,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Timeout"));
    }

    #[test]
    fn test_validate_success() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_provider_config_trait() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.api.com".to_string()),
            organization_id: None,
            timeout: 60,
            max_retries: 5,
            debug: false,
            enable_web_search: true,
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.api.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_get_api_base_default() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        // Note: This test assumes XAI_API_BASE is not set in the environment
        // When api_base is None and env var is not set, should return default
        assert_eq!(config.get_api_base(), "https://api.x.ai/v1");
    }

    #[test]
    fn test_get_api_base_custom() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.api.com".to_string()),
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        assert_eq!(config.get_api_base(), "https://custom.api.com");
    }

    #[test]
    fn test_get_api_key() {
        let config = XAIConfig {
            api_key: Some("my-api-key".to_string()),
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        assert_eq!(config.get_api_key(), Some("my-api-key".to_string()));
    }

    #[test]
    fn test_debug_mode() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: None,
            organization_id: None,
            timeout: 30,
            max_retries: 3,
            debug: true,
            enable_web_search: true,
        };

        assert!(config.debug);
    }

    #[test]
    fn test_organization_id() {
        let config = XAIConfig {
            api_key: Some("test-key".to_string()),
            api_base: None,
            organization_id: Some("org-123".to_string()),
            timeout: 30,
            max_retries: 3,
            debug: false,
            enable_web_search: true,
        };

        assert_eq!(config.organization_id, Some("org-123".to_string()));
    }
}
