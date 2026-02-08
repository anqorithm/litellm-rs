//! vLLM Provider Configuration
//!
//! Configuration for vLLM API access including authentication and model settings.
//! vLLM is typically self-hosted, so API base URL is required.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// vLLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VLLMConfig {
    /// API key for vLLM authentication (optional, depends on deployment)
    pub api_key: Option<String>,

    /// API base URL (required - vLLM is self-hosted)
    /// Example: "<http://localhost:8000/v1"> or "<https://your-vllm-server.com/v1">
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

    /// Model to use (vLLM serves specific models)
    pub model: Option<String>,

    /// Whether to skip model validation (useful for custom models)
    #[serde(default)]
    pub skip_model_validation: bool,
}

impl Default for VLLMConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            model: None,
            skip_model_validation: true, // vLLM serves custom models
        }
    }
}

impl ProviderConfig for VLLMConfig {
    fn validate(&self) -> Result<(), String> {
        // vLLM requires API base URL
        if self.api_base.is_none() && std::env::var("VLLM_API_BASE").is_err() {
            return Err(
                "vLLM API base URL not provided and VLLM_API_BASE environment variable not set"
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

impl VLLMConfig {
    /// Create a new vLLM config with the given API base URL
    pub fn new(api_base: impl Into<String>) -> Self {
        Self {
            api_base: Some(api_base.into()),
            ..Default::default()
        }
    }

    /// Create config with API base and optional API key
    pub fn with_credentials(api_base: impl Into<String>, api_key: Option<String>) -> Self {
        Self {
            api_base: Some(api_base.into()),
            api_key,
            ..Default::default()
        }
    }

    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("VLLM_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("VLLM_API_BASE").ok())
    }

    /// Set the model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }
}

fn default_timeout() -> u64 {
    120 // vLLM requests can be slower for large batches
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vllm_config_default() {
        let config = VLLMConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(config.skip_model_validation);
    }

    #[test]
    fn test_vllm_config_new() {
        let config = VLLMConfig::new("http://localhost:8000/v1");
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
    }

    #[test]
    fn test_vllm_config_with_credentials() {
        let config =
            VLLMConfig::with_credentials("http://localhost:8000/v1", Some("test-key".to_string()));
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_vllm_config_builder_methods() {
        let config = VLLMConfig::new("http://localhost:8000/v1")
            .with_model("meta-llama/Llama-3.1-8B-Instruct")
            .with_timeout(60)
            .with_debug(true);

        assert_eq!(
            config.model,
            Some("meta-llama/Llama-3.1-8B-Instruct".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }

    #[test]
    fn test_vllm_config_validation_no_base() {
        // Clear env var for this test (safely using unsafe block required in newer Rust)
        // SAFETY: This is a single-threaded test and we're just clearing an env var
        unsafe {
            std::env::remove_var("VLLM_API_BASE");
        }
        let config = VLLMConfig::default();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vllm_config_validation_with_base() {
        let config = VLLMConfig::new("http://localhost:8000/v1");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vllm_config_validation_zero_timeout() {
        let config = VLLMConfig {
            api_base: Some("http://localhost:8000/v1".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_vllm_config_provider_config_trait() {
        let config = VLLMConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://localhost:8000/v1".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("http://localhost:8000/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_vllm_config_serialization() {
        let config = VLLMConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://localhost:8000/v1".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
            model: Some("meta-llama/Llama-3.1-8B-Instruct".to_string()),
            skip_model_validation: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "http://localhost:8000/v1");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
        assert_eq!(json["model"], "meta-llama/Llama-3.1-8B-Instruct");
    }

    #[test]
    fn test_vllm_config_deserialization() {
        let json = r#"{
            "api_base": "http://localhost:8000/v1",
            "timeout": 60,
            "debug": true
        }"#;

        let config: VLLMConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
