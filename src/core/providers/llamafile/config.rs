//! Llamafile Provider Configuration
//!
//! Configuration for Llamafile API access including connection settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default Llamafile server URL
const DEFAULT_LLAMAFILE_API_BASE: &str = "http://127.0.0.1:8080/v1";

/// Llamafile provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlamafileConfig {
    /// API key for Llamafile authentication (optional, typically not required)
    pub api_key: Option<String>,

    /// API base URL (default: <http://127.0.0.1:8080/v1>)
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

impl Default for LlamafileConfig {
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

impl ProviderConfig for LlamafileConfig {
    fn validate(&self) -> Result<(), String> {
        // Llamafile doesn't require API key for local usage
        // Validation can be relaxed compared to cloud providers

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

impl LlamafileConfig {
    /// Get API key with environment variable fallback
    /// Returns "fake-api-key" if not set (Llamafile doesn't require an API key)
    pub fn get_api_key(&self) -> String {
        self.api_key
            .clone()
            .or_else(|| std::env::var("LLAMAFILE_API_KEY").ok())
            .unwrap_or_else(|| "fake-api-key".to_string())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("LLAMAFILE_API_BASE").ok())
            .unwrap_or_else(|| DEFAULT_LLAMAFILE_API_BASE.to_string())
    }

    /// Get chat completions endpoint
    pub fn get_chat_endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.get_api_base().trim_end_matches('/')
        )
    }

    /// Get completions endpoint (legacy)
    pub fn get_completions_endpoint(&self) -> String {
        format!("{}/completions", self.get_api_base().trim_end_matches('/'))
    }

    /// Get models endpoint
    pub fn get_models_endpoint(&self) -> String {
        format!("{}/models", self.get_api_base().trim_end_matches('/'))
    }
}

fn default_timeout() -> u64 {
    120 // Llamafile can be slow for initial model loading
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_llamafile_config_default() {
        let config = LlamafileConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_llamafile_config_get_api_base_default() {
        let config = LlamafileConfig::default();
        assert_eq!(config.get_api_base(), "http://127.0.0.1:8080/v1");
    }

    #[test]
    fn test_llamafile_config_get_api_base_custom() {
        let config = LlamafileConfig {
            api_base: Some("http://192.168.1.100:8080/v1".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "http://192.168.1.100:8080/v1");
    }

    #[test]
    fn test_llamafile_config_get_api_key_default() {
        let config = LlamafileConfig::default();
        // Should return fake-api-key when not set
        assert_eq!(config.get_api_key(), "fake-api-key");
    }

    #[test]
    fn test_llamafile_config_get_api_key_custom() {
        let config = LlamafileConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), "test-key");
    }

    #[test]
    fn test_llamafile_config_endpoints() {
        let config = LlamafileConfig::default();
        assert_eq!(
            config.get_chat_endpoint(),
            "http://127.0.0.1:8080/v1/chat/completions"
        );
        assert_eq!(
            config.get_completions_endpoint(),
            "http://127.0.0.1:8080/v1/completions"
        );
        assert_eq!(
            config.get_models_endpoint(),
            "http://127.0.0.1:8080/v1/models"
        );
    }

    #[test]
    fn test_llamafile_config_endpoints_with_trailing_slash() {
        let config = LlamafileConfig {
            api_base: Some("http://localhost:8080/v1/".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_chat_endpoint(),
            "http://localhost:8080/v1/chat/completions"
        );
    }

    #[test]
    fn test_llamafile_config_provider_config_trait() {
        let config = LlamafileConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:8080/v1".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("http://custom:8080/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_llamafile_config_validation_ok() {
        let config = LlamafileConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_llamafile_config_validation_zero_timeout() {
        let config = LlamafileConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_llamafile_config_serialization() {
        let config = LlamafileConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:8080/v1".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "http://custom:8080/v1");
        assert_eq!(json["timeout"], 45);
    }

    #[test]
    fn test_llamafile_config_deserialization() {
        let json = r#"{
            "api_base": "http://192.168.1.100:8080/v1",
            "timeout": 60,
            "debug": true
        }"#;

        let config: LlamafileConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("http://192.168.1.100:8080/v1".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
