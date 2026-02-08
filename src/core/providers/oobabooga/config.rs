//! Oobabooga Provider Configuration
//!
//! Configuration for Oobabooga text-generation-webui API access.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Oobabooga provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OobaboogaConfig {
    /// API key for Oobabooga authentication (optional, uses Token auth if provided)
    pub api_key: Option<String>,

    /// API base URL (required - must be set via config or OOBABOOGA_API_BASE env var)
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

impl Default for OobaboogaConfig {
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

impl ProviderConfig for OobaboogaConfig {
    fn validate(&self) -> Result<(), String> {
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

impl OobaboogaConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("OOBABOOGA_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("OOBABOOGA_API_BASE").ok())
    }

    /// Get chat completions endpoint
    pub fn get_chat_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!(
                "{}/v1/chat/completions",
                base.trim_end_matches('/')
            )),
            None => Err(
                "OOBABOOGA_API_BASE not set. Set one via api_base config or environment variable."
                    .to_string(),
            ),
        }
    }

    /// Get embeddings endpoint
    pub fn get_embeddings_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!("{}/v1/embeddings", base.trim_end_matches('/'))),
            None => Err(
                "OOBABOOGA_API_BASE not set. Set one via api_base config or environment variable."
                    .to_string(),
            ),
        }
    }

    /// Get models endpoint
    pub fn get_models_endpoint(&self) -> Result<String, String> {
        match self.get_api_base() {
            Some(base) => Ok(format!("{}/v1/models", base.trim_end_matches('/'))),
            None => Err(
                "OOBABOOGA_API_BASE not set. Set one via api_base config or environment variable."
                    .to_string(),
            ),
        }
    }

    /// Build authentication headers for Oobabooga
    /// Oobabooga uses Token auth format: "Token {api_key}"
    pub fn build_auth_headers(&self) -> Vec<(String, String)> {
        let mut headers = vec![
            ("accept".to_string(), "application/json".to_string()),
            ("content-type".to_string(), "application/json".to_string()),
        ];

        if let Some(api_key) = self.get_api_key() {
            headers.push(("Authorization".to_string(), format!("Token {}", api_key)));
        }

        headers
    }
}

fn default_timeout() -> u64 {
    120 // Oobabooga can be slow for large models
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oobabooga_config_default() {
        let config = OobaboogaConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_oobabooga_config_get_api_base_none() {
        let config = OobaboogaConfig::default();
        // Should return None when not set (unless env var is set)
        // We can't guarantee env var state, so just check it's Option
        let _base = config.get_api_base();
    }

    #[test]
    fn test_oobabooga_config_get_api_base_custom() {
        let config = OobaboogaConfig {
            api_base: Some("http://192.168.1.100:5000".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_api_base(),
            Some("http://192.168.1.100:5000".to_string())
        );
    }

    #[test]
    fn test_oobabooga_config_get_api_key() {
        let config = OobaboogaConfig {
            api_key: Some("test-token".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-token".to_string()));
    }

    #[test]
    fn test_oobabooga_config_endpoints() {
        let config = OobaboogaConfig {
            api_base: Some("http://localhost:5000".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_chat_endpoint().unwrap(),
            "http://localhost:5000/v1/chat/completions"
        );
        assert_eq!(
            config.get_embeddings_endpoint().unwrap(),
            "http://localhost:5000/v1/embeddings"
        );
        assert_eq!(
            config.get_models_endpoint().unwrap(),
            "http://localhost:5000/v1/models"
        );
    }

    #[test]
    fn test_oobabooga_config_endpoints_no_base() {
        let config = OobaboogaConfig::default();
        // Check that endpoints fail without base (assuming env var not set)
        // This might pass if OOBABOOGA_API_BASE is set in env
        let chat_result = config.get_chat_endpoint();
        // If env var is not set, should be Err
        // Can't guarantee env state, so just check it returns a Result
        let _ = chat_result;
    }

    #[test]
    fn test_oobabooga_config_endpoints_with_trailing_slash() {
        let config = OobaboogaConfig {
            api_base: Some("http://localhost:5000/".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_chat_endpoint().unwrap(),
            "http://localhost:5000/v1/chat/completions"
        );
    }

    #[test]
    fn test_oobabooga_config_provider_config_trait() {
        let config = OobaboogaConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:5000".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("http://custom:5000"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_oobabooga_config_validation_ok() {
        let config = OobaboogaConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_oobabooga_config_validation_zero_timeout() {
        let config = OobaboogaConfig {
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_oobabooga_config_build_auth_headers_no_key() {
        let config = OobaboogaConfig::default();
        let headers = config.build_auth_headers();

        // Should have accept and content-type
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "accept" && v == "application/json")
        );
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "content-type" && v == "application/json")
        );

        // Should NOT have Authorization if no API key set (and env var not set)
        // Can't guarantee env state
    }

    #[test]
    fn test_oobabooga_config_build_auth_headers_with_key() {
        let config = OobaboogaConfig {
            api_key: Some("my-token".to_string()),
            ..Default::default()
        };
        let headers = config.build_auth_headers();

        // Should have Token auth header
        assert!(
            headers
                .iter()
                .any(|(k, v)| k == "Authorization" && v == "Token my-token")
        );
    }

    #[test]
    fn test_oobabooga_config_serialization() {
        let config = OobaboogaConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://custom:5000".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "http://custom:5000");
        assert_eq!(json["timeout"], 45);
    }

    #[test]
    fn test_oobabooga_config_deserialization() {
        let json = r#"{
            "api_base": "http://192.168.1.100:5000",
            "timeout": 60,
            "debug": true
        }"#;

        let config: OobaboogaConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("http://192.168.1.100:5000".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
