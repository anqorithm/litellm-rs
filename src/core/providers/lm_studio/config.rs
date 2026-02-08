//! LM Studio Provider Configuration
//!
//! Configuration for LM Studio API access including connection settings and model options.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// LM Studio provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LMStudioConfig {
    /// API key for LM Studio authentication (optional, typically not required for local usage)
    pub api_key: Option<String>,

    /// API base URL (default: configurable via LM_STUDIO_API_BASE env var)
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

impl Default for LMStudioConfig {
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

impl ProviderConfig for LMStudioConfig {
    fn validate(&self) -> Result<(), String> {
        // LM Studio doesn't require API key for local usage
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

impl LMStudioConfig {
    /// Get API key with environment variable fallback
    /// Returns "fake-api-key" if not set (LM Studio doesn't require an API key)
    pub fn get_api_key(&self) -> String {
        self.api_key
            .clone()
            .or_else(|| std::env::var("LM_STUDIO_API_KEY").ok())
            .unwrap_or_else(|| "fake-api-key".to_string())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("LM_STUDIO_API_BASE").ok())
    }

    /// Get chat completions endpoint
    pub fn get_chat_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!(
                "{}/v1/chat/completions",
                base.trim_end_matches('/')
            )),
            None => Err("LM_STUDIO_API_BASE not set".to_string()),
        }
    }

    /// Get embeddings endpoint
    pub fn get_embeddings_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!("{}/v1/embeddings", base.trim_end_matches('/'))),
            None => Err("LM_STUDIO_API_BASE not set".to_string()),
        }
    }

    /// Get models endpoint
    pub fn get_models_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!("{}/v1/models", base.trim_end_matches('/'))),
            None => Err("LM_STUDIO_API_BASE not set".to_string()),
        }
    }
}

fn default_timeout() -> u64 {
    120 // LM Studio can be slow for initial model loading
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lm_studio_config_default() {
        let config = LMStudioConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_lm_studio_config_get_api_key_default() {
        let config = LMStudioConfig::default();
        // Should return fake-api-key when not set
        assert_eq!(config.get_api_key(), "fake-api-key");
    }

    #[test]
    fn test_lm_studio_config_get_api_key_custom() {
        let config = LMStudioConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), "test-key");
    }

    #[test]
    fn test_lm_studio_config_get_api_base_custom() {
        let config = LMStudioConfig {
            api_base: Some("http://192.168.1.100:1234".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_api_base(),
            Some("http://192.168.1.100:1234".to_string())
        );
    }

    #[test]
    fn test_lm_studio_config_endpoints() {
        let config = LMStudioConfig {
            api_base: Some("http://localhost:1234".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_chat_endpoint().unwrap(),
            "http://localhost:1234/v1/chat/completions"
        );
        assert_eq!(
            config.get_embeddings_endpoint().unwrap(),
            "http://localhost:1234/v1/embeddings"
        );
        assert_eq!(
            config.get_models_endpoint().unwrap(),
            "http://localhost:1234/v1/models"
        );
    }

    #[test]
    fn test_lm_studio_config_endpoints_no_base() {
        let config = LMStudioConfig::default();
        assert!(config.get_chat_endpoint().is_err());
        assert!(config.get_embeddings_endpoint().is_err());
        assert!(config.get_models_endpoint().is_err());
    }

    #[test]
    fn test_lm_studio_config_provider_config_trait() {
        let config = LMStudioConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:1234".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("http://custom:1234"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_lm_studio_config_validation_ok() {
        let config = LMStudioConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_lm_studio_config_validation_zero_timeout() {
        let config = LMStudioConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_lm_studio_config_serialization() {
        let config = LMStudioConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:1234".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "http://custom:1234");
        assert_eq!(json["timeout"], 45);
    }

    #[test]
    fn test_lm_studio_config_deserialization() {
        let json = r#"{
            "api_base": "http://192.168.1.100:1234",
            "timeout": 60,
            "debug": true
        }"#;

        let config: LMStudioConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("http://192.168.1.100:1234".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
