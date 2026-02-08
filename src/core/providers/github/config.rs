//! GitHub Models Provider Configuration
//!
//! Configuration for GitHub Models API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default API base URL for GitHub Models
pub const GITHUB_MODELS_API_BASE: &str = "https://models.inference.ai.azure.com";

/// GitHub Models provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// API key for GitHub authentication (GitHub PAT)
    pub api_key: Option<String>,

    /// API base URL (default: <https://models.inference.ai.azure.com>)
    pub api_base: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for GitHubConfig {
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

impl ProviderConfig for GitHubConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var("GITHUB_TOKEN").is_err() {
            return Err(
                "GitHub API key not provided and GITHUB_TOKEN environment variable not set"
                    .to_string(),
            );
        }

        // Validate timeout
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

impl GitHubConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("GITHUB_TOKEN").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("GITHUB_MODELS_API_BASE").ok())
            .unwrap_or_else(|| GITHUB_MODELS_API_BASE.to_string())
    }
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_github_config_default() {
        let config = GitHubConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_github_config_get_api_base_default() {
        let config = GitHubConfig::default();
        assert_eq!(
            config.get_api_base(),
            "https://models.inference.ai.azure.com"
        );
    }

    #[test]
    fn test_github_config_get_api_base_custom() {
        let config = GitHubConfig {
            api_base: Some("https://custom.github.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.github.com");
    }

    #[test]
    fn test_github_config_get_api_key() {
        let config = GitHubConfig {
            api_key: Some("ghp_test123".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("ghp_test123".to_string()));
    }

    #[test]
    fn test_github_config_provider_config_trait() {
        let config = GitHubConfig {
            api_key: Some("ghp_test123".to_string()),
            api_base: Some("https://custom.github.com".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("ghp_test123"));
        assert_eq!(config.api_base(), Some("https://custom.github.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_github_config_validation_with_key() {
        let config = GitHubConfig {
            api_key: Some("ghp_test123".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_github_config_validation_zero_timeout() {
        let config = GitHubConfig {
            api_key: Some("ghp_test123".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_github_config_serialization() {
        let config = GitHubConfig {
            api_key: Some("ghp_test123".to_string()),
            api_base: Some("https://custom.github.com".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "ghp_test123");
        assert_eq!(json["api_base"], "https://custom.github.com");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert!(json["debug"].as_bool().unwrap());
    }

    #[test]
    fn test_github_config_deserialization() {
        let json = r#"{
            "api_key": "ghp_test123",
            "timeout": 60,
            "debug": true
        }"#;

        let config: GitHubConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("ghp_test123".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
