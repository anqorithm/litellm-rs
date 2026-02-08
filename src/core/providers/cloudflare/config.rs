//! Cloudflare Workers AI Configuration

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Cloudflare Workers AI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudflareConfig {
    /// Cloudflare account ID
    pub account_id: Option<String>,

    /// API token for authentication
    pub api_token: Option<String>,

    /// API base URL (defaults to <https://api.cloudflare.com/client/v4>)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Enable debug mode
    #[serde(default)]
    pub debug: bool,
}

impl Default for CloudflareConfig {
    fn default() -> Self {
        Self {
            account_id: std::env::var("CLOUDFLARE_ACCOUNT_ID").ok(),
            api_token: std::env::var("CLOUDFLARE_API_TOKEN").ok(),
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for CloudflareConfig {
    fn validate(&self) -> Result<(), String> {
        if self.account_id.is_none() {
            return Err("Cloudflare account ID is required".to_string());
        }

        if self.api_token.is_none() {
            return Err("Cloudflare API token is required".to_string());
        }

        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_token.as_deref()
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

impl CloudflareConfig {
    /// Get the API base URL
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_API_BASE").ok())
            .unwrap_or_else(|| "https://api.cloudflare.com/client/v4".to_string())
    }

    /// Get the account ID
    pub fn get_account_id(&self) -> Option<String> {
        self.account_id
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_ACCOUNT_ID").ok())
    }

    /// Get the API token
    pub fn get_api_token(&self) -> Option<String> {
        self.api_token
            .clone()
            .or_else(|| std::env::var("CLOUDFLARE_API_TOKEN").ok())
    }
}

fn default_timeout() -> u64 {
    30
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cloudflare_config_with_values() {
        let config = CloudflareConfig {
            account_id: Some("account-123".to_string()),
            api_token: Some("token-xyz".to_string()),
            api_base: Some("https://custom.cloudflare.com".to_string()),
            timeout: 45,
            max_retries: 5,
            debug: true,
        };

        assert_eq!(config.account_id, Some("account-123".to_string()));
        assert_eq!(config.api_token, Some("token-xyz".to_string()));
        assert_eq!(config.timeout, 45);
        assert_eq!(config.max_retries, 5);
        assert!(config.debug);
    }

    #[test]
    fn test_cloudflare_config_get_api_base_default() {
        let config = CloudflareConfig {
            account_id: Some("test".to_string()),
            api_token: Some("test".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert_eq!(
            config.get_api_base(),
            "https://api.cloudflare.com/client/v4"
        );
    }

    #[test]
    fn test_cloudflare_config_get_api_base_custom() {
        let config = CloudflareConfig {
            account_id: Some("test".to_string()),
            api_token: Some("test".to_string()),
            api_base: Some("https://custom.cloudflare.com".to_string()),
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert_eq!(config.get_api_base(), "https://custom.cloudflare.com");
    }

    #[test]
    fn test_cloudflare_config_get_account_id() {
        let config = CloudflareConfig {
            account_id: Some("my-account".to_string()),
            api_token: Some("test".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert_eq!(config.get_account_id(), Some("my-account".to_string()));
    }

    #[test]
    fn test_cloudflare_config_get_api_token() {
        let config = CloudflareConfig {
            account_id: Some("test".to_string()),
            api_token: Some("my-token".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert_eq!(config.get_api_token(), Some("my-token".to_string()));
    }

    #[test]
    fn test_cloudflare_config_provider_config_trait() {
        let config = CloudflareConfig {
            account_id: Some("account".to_string()),
            api_token: Some("token".to_string()),
            api_base: Some("https://custom.api.com".to_string()),
            timeout: 60,
            max_retries: 5,
            debug: false,
        };

        assert_eq!(config.api_key(), Some("token"));
        assert_eq!(config.api_base(), Some("https://custom.api.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_cloudflare_config_validation_missing_account() {
        let config = CloudflareConfig {
            account_id: None,
            api_token: Some("token".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cloudflare_config_validation_missing_token() {
        let config = CloudflareConfig {
            account_id: Some("account".to_string()),
            api_token: None,
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cloudflare_config_validation_zero_timeout() {
        let config = CloudflareConfig {
            account_id: Some("account".to_string()),
            api_token: Some("token".to_string()),
            api_base: None,
            timeout: 0,
            max_retries: 3,
            debug: false,
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_cloudflare_config_validation_success() {
        let config = CloudflareConfig {
            account_id: Some("account".to_string()),
            api_token: Some("token".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: false,
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_cloudflare_config_serialization() {
        let config = CloudflareConfig {
            account_id: Some("account-123".to_string()),
            api_token: Some("token-xyz".to_string()),
            api_base: None,
            timeout: 30,
            max_retries: 3,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["account_id"], "account-123");
        assert_eq!(json["api_token"], "token-xyz");
        assert_eq!(json["timeout"], 30);
        assert_eq!(json["debug"], true);
    }
}
