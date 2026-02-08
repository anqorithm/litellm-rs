//! Hosted vLLM Provider Configuration
//!
//! Configuration for connecting to hosted vLLM inference servers.
//! vLLM servers provide an OpenAI-compatible API, so configuration is minimal.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Hosted vLLM provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostedVLLMConfig {
    /// API base URL (required - vLLM is self-hosted)
    /// Example: "<http://localhost:8000/v1"> or "<https://your-vllm-server.com/v1">
    pub api_base: Option<String>,

    /// API key for vLLM authentication (optional, depends on deployment)
    pub api_key: Option<String>,

    /// Default model to use (optional - can be specified per request)
    pub model: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,

    /// Whether to skip model validation (useful for custom models)
    #[serde(default = "default_skip_validation")]
    pub skip_model_validation: bool,

    /// Custom headers to include in requests
    #[serde(default)]
    pub custom_headers: std::collections::HashMap<String, String>,
}

impl Default for HostedVLLMConfig {
    fn default() -> Self {
        Self {
            api_base: None,
            api_key: None,
            model: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
            skip_model_validation: default_skip_validation(),
            custom_headers: std::collections::HashMap::new(),
        }
    }
}

impl ProviderConfig for HostedVLLMConfig {
    fn validate(&self) -> Result<(), String> {
        // Hosted vLLM requires API base URL
        if self.api_base.is_none() && std::env::var("HOSTED_VLLM_API_BASE").is_err() {
            return Err(
                "Hosted vLLM API base URL not provided and HOSTED_VLLM_API_BASE environment variable not set"
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

impl HostedVLLMConfig {
    /// Create a new hosted vLLM config with the given API base URL
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

    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            api_base: std::env::var("HOSTED_VLLM_API_BASE").ok(),
            api_key: std::env::var("HOSTED_VLLM_API_KEY").ok(),
            model: std::env::var("HOSTED_VLLM_MODEL").ok(),
            ..Default::default()
        }
    }

    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("HOSTED_VLLM_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("HOSTED_VLLM_API_BASE").ok())
    }

    /// Set the default model to use
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the API key
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Set debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Set timeout in seconds
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add a custom header
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    /// Set skip model validation
    pub fn with_skip_model_validation(mut self, skip: bool) -> Self {
        self.skip_model_validation = skip;
        self
    }
}

fn default_timeout() -> u64 {
    120 // vLLM requests can take longer for large batches
}

fn default_max_retries() -> u32 {
    3
}

fn default_skip_validation() -> bool {
    true // vLLM typically serves custom models
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hosted_vllm_config_default() {
        let config = HostedVLLMConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 120);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
        assert!(config.skip_model_validation);
    }

    #[test]
    fn test_hosted_vllm_config_new() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1");
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
    }

    #[test]
    fn test_hosted_vllm_config_with_credentials() {
        let config = HostedVLLMConfig::with_credentials(
            "http://localhost:8000/v1",
            Some("test-key".to_string()),
        );
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(config.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_hosted_vllm_config_builder_methods() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1")
            .with_model("meta-llama/Llama-3.1-8B-Instruct")
            .with_api_key("my-key")
            .with_timeout(60)
            .with_max_retries(5)
            .with_debug(true)
            .with_header("X-Custom-Header", "value");

        assert_eq!(
            config.model,
            Some("meta-llama/Llama-3.1-8B-Instruct".to_string())
        );
        assert_eq!(config.api_key, Some("my-key".to_string()));
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 5);
        assert!(config.debug);
        assert_eq!(
            config.custom_headers.get("X-Custom-Header"),
            Some(&"value".to_string())
        );
    }

    #[test]
    fn test_hosted_vllm_config_validation_with_base() {
        let config = HostedVLLMConfig::new("http://localhost:8000/v1");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_hosted_vllm_config_validation_zero_timeout() {
        let config = HostedVLLMConfig {
            api_base: Some("http://localhost:8000/v1".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hosted_vllm_config_provider_config_trait() {
        let config = HostedVLLMConfig {
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
    fn test_hosted_vllm_config_serialization() {
        let config = HostedVLLMConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://localhost:8000/v1".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
            model: Some("meta-llama/Llama-3.1-8B-Instruct".to_string()),
            skip_model_validation: true,
            custom_headers: std::collections::HashMap::new(),
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
    fn test_hosted_vllm_config_deserialization() {
        let json = r#"{
            "api_base": "http://localhost:8000/v1",
            "timeout": 60,
            "debug": true
        }"#;

        let config: HostedVLLMConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.api_base,
            Some("http://localhost:8000/v1".to_string())
        );
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
