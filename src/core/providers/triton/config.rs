//! Triton Provider Configuration
//!
//! Configuration for NVIDIA Triton Inference Server connection.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Triton provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TritonConfig {
    /// Triton server URL (required)
    /// Example: <http://localhost:8000>
    pub server_url: Option<String>,

    /// Model name deployed on Triton
    pub model_name: Option<String>,

    /// Specific model version (optional, uses latest if not specified)
    pub model_version: Option<String>,

    /// Request timeout in milliseconds
    #[serde(default = "default_timeout_ms")]
    pub timeout_ms: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,

    /// Custom headers for requests
    #[serde(default)]
    pub headers: std::collections::HashMap<String, String>,
}

impl Default for TritonConfig {
    fn default() -> Self {
        Self {
            server_url: None,
            model_name: None,
            model_version: None,
            timeout_ms: default_timeout_ms(),
            max_retries: default_max_retries(),
            debug: false,
            headers: std::collections::HashMap::new(),
        }
    }
}

impl ProviderConfig for TritonConfig {
    fn validate(&self) -> Result<(), String> {
        // Server URL can come from environment variable
        if self.server_url.is_none() && std::env::var("TRITON_SERVER_URL").is_err() {
            return Err(
                "Triton server URL not provided and TRITON_SERVER_URL environment variable not set"
                    .to_string(),
            );
        }

        // Validate timeout
        if self.timeout_ms == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        // Triton typically doesn't require API keys for basic usage
        // But custom authentication can be added via headers
        None
    }

    fn api_base(&self) -> Option<&str> {
        self.server_url.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.timeout_ms)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl TritonConfig {
    /// Create a new Triton config with the server URL
    pub fn new(server_url: impl Into<String>) -> Self {
        Self {
            server_url: Some(server_url.into()),
            ..Default::default()
        }
    }

    /// Create a new Triton config with server URL and model name
    pub fn with_model(server_url: impl Into<String>, model_name: impl Into<String>) -> Self {
        Self {
            server_url: Some(server_url.into()),
            model_name: Some(model_name.into()),
            ..Default::default()
        }
    }

    /// Get server URL with environment variable fallback
    pub fn get_server_url(&self) -> String {
        self.server_url
            .clone()
            .or_else(|| std::env::var("TRITON_SERVER_URL").ok())
            .unwrap_or_else(|| "http://localhost:8000".to_string())
    }

    /// Get model name with environment variable fallback
    pub fn get_model_name(&self) -> Option<String> {
        self.model_name
            .clone()
            .or_else(|| std::env::var("TRITON_MODEL_NAME").ok())
    }

    /// Get model version
    pub fn get_model_version(&self) -> Option<String> {
        self.model_version
            .clone()
            .or_else(|| std::env::var("TRITON_MODEL_VERSION").ok())
    }

    /// Set model name
    pub fn model_name(mut self, name: impl Into<String>) -> Self {
        self.model_name = Some(name.into());
        self
    }

    /// Set model version
    pub fn model_version(mut self, version: impl Into<String>) -> Self {
        self.model_version = Some(version.into());
        self
    }

    /// Set timeout in milliseconds
    pub fn timeout_ms(mut self, timeout: u64) -> Self {
        self.timeout_ms = timeout;
        self
    }

    /// Add custom header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

fn default_timeout_ms() -> u64 {
    30000 // 30 seconds
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triton_config_default() {
        let config = TritonConfig::default();
        assert!(config.server_url.is_none());
        assert!(config.model_name.is_none());
        assert!(config.model_version.is_none());
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_triton_config_new() {
        let config = TritonConfig::new("http://localhost:8000");
        assert_eq!(config.server_url, Some("http://localhost:8000".to_string()));
    }

    #[test]
    fn test_triton_config_with_model() {
        let config = TritonConfig::with_model("http://localhost:8000", "my-model");
        assert_eq!(config.server_url, Some("http://localhost:8000".to_string()));
        assert_eq!(config.model_name, Some("my-model".to_string()));
    }

    #[test]
    fn test_triton_config_builder() {
        let config = TritonConfig::new("http://localhost:8000")
            .model_name("llama-7b")
            .model_version("1")
            .timeout_ms(60000)
            .header("Authorization", "Bearer token");

        assert_eq!(config.server_url, Some("http://localhost:8000".to_string()));
        assert_eq!(config.model_name, Some("llama-7b".to_string()));
        assert_eq!(config.model_version, Some("1".to_string()));
        assert_eq!(config.timeout_ms, 60000);
        assert_eq!(
            config.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_triton_config_get_server_url_default() {
        let config = TritonConfig::default();
        assert_eq!(config.get_server_url(), "http://localhost:8000");
    }

    #[test]
    fn test_triton_config_get_server_url_custom() {
        let config = TritonConfig::new("http://triton.example.com:8000");
        assert_eq!(config.get_server_url(), "http://triton.example.com:8000");
    }

    #[test]
    fn test_triton_config_validation_no_url() {
        // Clear env var for this test
        // SAFETY: This is safe in a single-threaded test context
        unsafe { std::env::remove_var("TRITON_SERVER_URL") };
        let config = TritonConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_triton_config_validation_with_url() {
        let config = TritonConfig::new("http://localhost:8000");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_triton_config_validation_zero_timeout() {
        let config = TritonConfig {
            server_url: Some("http://localhost:8000".to_string()),
            timeout_ms: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_triton_config_provider_config_trait() {
        let config = TritonConfig {
            server_url: Some("http://localhost:8000".to_string()),
            timeout_ms: 60000,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), None); // Triton doesn't use API keys
        assert_eq!(config.api_base(), Some("http://localhost:8000"));
        assert_eq!(config.timeout(), std::time::Duration::from_millis(60000));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_triton_config_serialization() {
        let config = TritonConfig {
            server_url: Some("http://localhost:8000".to_string()),
            model_name: Some("llama-7b".to_string()),
            model_version: Some("1".to_string()),
            timeout_ms: 45000,
            max_retries: 2,
            debug: true,
            headers: std::collections::HashMap::new(),
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["server_url"], "http://localhost:8000");
        assert_eq!(json["model_name"], "llama-7b");
        assert_eq!(json["model_version"], "1");
        assert_eq!(json["timeout_ms"], 45000);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_triton_config_deserialization() {
        let json = r#"{
            "server_url": "http://triton.local:8000",
            "model_name": "gpt-j",
            "timeout_ms": 60000,
            "debug": true
        }"#;

        let config: TritonConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.server_url,
            Some("http://triton.local:8000".to_string())
        );
        assert_eq!(config.model_name, Some("gpt-j".to_string()));
        assert_eq!(config.timeout_ms, 60000);
        assert!(config.debug);
    }
}
