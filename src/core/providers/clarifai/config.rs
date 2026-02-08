//! Clarifai Provider Configuration
//!
//! Configuration for Clarifai API access including authentication and model settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default API base URL for Clarifai
pub const CLARIFAI_API_BASE: &str = "https://api.clarifai.com/v2/ext/openai/v1";

/// Clarifai provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClarifaiConfig {
    /// API key for Clarifai authentication
    pub api_key: Option<String>,

    /// API base URL (default: <https://api.clarifai.com/v2/ext/openai/v1>)
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

impl Default for ClarifaiConfig {
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

impl ProviderConfig for ClarifaiConfig {
    fn validate(&self) -> Result<(), String> {
        // API key can come from environment variable
        if self.api_key.is_none() && std::env::var("CLARIFAI_API_KEY").is_err() {
            return Err(
                "Clarifai API key not provided and CLARIFAI_API_KEY environment variable not set"
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

impl ClarifaiConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("CLARIFAI_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("CLARIFAI_API_BASE").ok())
            .unwrap_or_else(|| CLARIFAI_API_BASE.to_string())
    }

    /// Parse Clarifai model string format and return the full model URL
    ///
    /// Clarifai model format: `user_id.app_id.model_id`
    /// Returns: `<https://clarifai.com/{user_id}/{app_id}/models/{model_id}`>
    pub fn get_model_url(model: &str) -> Option<String> {
        let parts: Vec<&str> = model.split('.').collect();
        if parts.len() == 3 {
            let user_id = parts[0];
            let app_id = parts[1];
            let model_id = parts[2];
            Some(format!(
                "https://clarifai.com/{}/{}/models/{}",
                user_id, app_id, model_id
            ))
        } else {
            None
        }
    }

    /// Check if a model string is in valid Clarifai format
    pub fn is_valid_model_format(model: &str) -> bool {
        let parts: Vec<&str> = model.split('.').collect();
        parts.len() == 3 && parts.iter().all(|p| !p.is_empty())
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
    fn test_clarifai_config_default() {
        let config = ClarifaiConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_clarifai_config_get_api_base_default() {
        let config = ClarifaiConfig::default();
        assert_eq!(config.get_api_base(), CLARIFAI_API_BASE);
    }

    #[test]
    fn test_clarifai_config_get_api_base_custom() {
        let config = ClarifaiConfig {
            api_base: Some("https://custom.clarifai.com".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_base(), "https://custom.clarifai.com");
    }

    #[test]
    fn test_clarifai_config_get_api_key() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_get_model_url() {
        // Valid model format
        let url = ClarifaiConfig::get_model_url("user123.app456.model789");
        assert_eq!(
            url,
            Some("https://clarifai.com/user123/app456/models/model789".to_string())
        );

        // Invalid format - wrong number of parts
        assert!(ClarifaiConfig::get_model_url("user.app").is_none());
        assert!(ClarifaiConfig::get_model_url("user.app.model.extra").is_none());
        assert!(ClarifaiConfig::get_model_url("singlepart").is_none());
    }

    #[test]
    fn test_is_valid_model_format() {
        // Valid formats
        assert!(ClarifaiConfig::is_valid_model_format("user.app.model"));
        assert!(ClarifaiConfig::is_valid_model_format(
            "openai.chat-completion.gpt-4"
        ));

        // Invalid formats
        assert!(!ClarifaiConfig::is_valid_model_format("user.app"));
        assert!(!ClarifaiConfig::is_valid_model_format(
            "user.app.model.extra"
        ));
        assert!(!ClarifaiConfig::is_valid_model_format("singlepart"));
        assert!(!ClarifaiConfig::is_valid_model_format("user..model")); // empty app
        assert!(!ClarifaiConfig::is_valid_model_format(".app.model")); // empty user
        assert!(!ClarifaiConfig::is_valid_model_format("user.app.")); // empty model
    }

    #[test]
    fn test_clarifai_config_provider_config_trait() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.clarifai.com".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://custom.clarifai.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_clarifai_config_validation_with_key() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_clarifai_config_validation_zero_timeout() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_clarifai_config_serialization() {
        let config = ClarifaiConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://custom.clarifai.com".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://custom.clarifai.com");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_clarifai_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "timeout": 60,
            "debug": true
        }"#;

        let config: ClarifaiConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
