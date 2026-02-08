//! OpenRouter Provider Configuration
//!
//! Configuration management for OpenRouter API integration

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenRouter provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterConfig {
    /// API key for OpenRouter
    pub api_key: String,
    /// Base URL for OpenRouter API
    pub base_url: String,
    /// Site URL for OpenRouter (optional)
    pub site_url: Option<String>,
    /// Site Name for OpenRouter (optional)
    pub site_name: Option<String>,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum number of retries
    pub max_retries: u32,
    /// Additional provider-specific parameters
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl Default for OpenRouterConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            base_url: "https://openrouter.ai/api/v1".to_string(),
            site_url: None,
            site_name: None,
            timeout_seconds: 30,
            max_retries: 3,
            extra_params: HashMap::new(),
        }
    }
}

impl ProviderConfig for OpenRouterConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_empty() {
            return Err("OpenRouter API key is required".to_string());
        }

        if self.api_key.len() < 10 {
            return Err("OpenRouter API key appears to be invalid (too short)".to_string());
        }

        if self.base_url.is_empty() {
            return Err("OpenRouter base URL is required".to_string());
        }

        if !self.base_url.starts_with("http") {
            return Err("OpenRouter base URL must start with http:// or https://".to_string());
        }

        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        Some(&self.api_key)
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.base_url)
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl OpenRouterConfig {
    /// Create new OpenRouter configuration
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        let api_key = std::env::var("OPENROUTER_API_KEY").unwrap_or_default();
        let base_url = std::env::var("OPENROUTER_BASE_URL")
            .unwrap_or_else(|_| "https://openrouter.ai/api/v1".to_string());
        let site_url = std::env::var("OPENROUTER_SITE_URL").ok();
        let site_name = std::env::var("OPENROUTER_SITE_NAME").ok();
        let timeout_seconds = std::env::var("OPENROUTER_TIMEOUT")
            .ok()
            .and_then(|t| t.parse().ok())
            .unwrap_or(30);
        let max_retries = std::env::var("OPENROUTER_MAX_RETRIES")
            .ok()
            .and_then(|r| r.parse().ok())
            .unwrap_or(3);

        Self {
            api_key,
            base_url,
            site_url,
            site_name,
            timeout_seconds,
            max_retries,
            extra_params: HashMap::new(),
        }
    }

    /// Set site URL for OpenRouter request headers
    pub fn with_site_url(mut self, site_url: impl Into<String>) -> Self {
        self.site_url = Some(site_url.into());
        self
    }

    /// Set site name for OpenRouter request headers  
    pub fn with_site_name(mut self, site_name: impl Into<String>) -> Self {
        self.site_name = Some(site_name.into());
        self
    }

    /// Set base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Set maximum retries
    pub fn with_max_retries(mut self, max_retries: u32) -> Self {
        self.max_retries = max_retries;
        self
    }

    /// Add extra parameter
    pub fn with_extra_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra_params.insert(key.into(), value);
        self
    }

    /// Get request headers for OpenRouter API
    pub fn get_headers(&self) -> HashMap<String, String> {
        // Pre-allocate capacity for known headers
        let mut headers = HashMap::with_capacity(4);

        headers.insert(
            "Authorization".to_string(),
            format!("Bearer {}", self.api_key),
        );
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        if let Some(site_url) = &self.site_url {
            headers.insert("HTTP-Referer".to_string(), site_url.clone());
        }

        if let Some(site_name) = &self.site_name {
            headers.insert("X-Title".to_string(), site_name.clone());
        }

        headers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let mut config = OpenRouterConfig::default();

        // Should fail with empty API key
        assert!(config.validate().is_err());

        // Should fail with short API key
        config.api_key = "short".to_string();
        assert!(config.validate().is_err());

        // Should pass with valid API key
        config.api_key = "or-valid-api-key-12345".to_string();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_builder_methods() {
        let config = OpenRouterConfig::new("test-key")
            .with_site_url("https://example.com")
            .with_site_name("Test Site")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.site_url, Some("https://example.com".to_string()));
        assert_eq!(config.site_name, Some("Test Site".to_string()));
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_headers() {
        let config = OpenRouterConfig::new("test-key")
            .with_site_url("https://example.com")
            .with_site_name("Test Site");

        let headers = config.get_headers();

        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer test-key".to_string())
        );
        assert_eq!(
            headers.get("HTTP-Referer"),
            Some(&"https://example.com".to_string())
        );
        assert_eq!(headers.get("X-Title"), Some(&"Test Site".to_string()));
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_default_config() {
        let config = OpenRouterConfig::default();
        assert_eq!(config.base_url, "https://openrouter.ai/api/v1");
        assert_eq!(config.timeout_seconds, 30);
        assert_eq!(config.max_retries, 3);
        assert!(config.api_key.is_empty());
        assert!(config.site_url.is_none());
        assert!(config.site_name.is_none());
        assert!(config.extra_params.is_empty());
    }

    #[test]
    fn test_validation_empty_base_url() {
        let mut config = OpenRouterConfig::new("or-valid-api-key-12345");
        config.base_url = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_invalid_base_url() {
        let mut config = OpenRouterConfig::new("or-valid-api-key-12345");
        config.base_url = "invalid-url".to_string();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_timeout() {
        let mut config = OpenRouterConfig::new("or-valid-api-key-12345");
        config.timeout_seconds = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_provider_config_trait() {
        let config = OpenRouterConfig::new("test-api-key");
        assert_eq!(config.api_key(), Some("test-api-key"));
        assert_eq!(config.api_base(), Some("https://openrouter.ai/api/v1"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(30));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_with_base_url() {
        let config = OpenRouterConfig::new("test-key").with_base_url("https://custom.api.com/v1");
        assert_eq!(config.base_url, "https://custom.api.com/v1");
    }

    #[test]
    fn test_with_extra_param() {
        let config = OpenRouterConfig::new("test-key")
            .with_extra_param("model_routing", serde_json::json!(["openai", "anthropic"]));

        assert!(config.extra_params.contains_key("model_routing"));
        assert_eq!(
            config.extra_params.get("model_routing").unwrap(),
            &serde_json::json!(["openai", "anthropic"])
        );
    }

    #[test]
    fn test_headers_without_optional() {
        let config = OpenRouterConfig::new("test-key");

        let headers = config.get_headers();

        // Should have authorization and content-type
        assert!(headers.contains_key("Authorization"));
        assert!(headers.contains_key("Content-Type"));

        // Should NOT have optional headers
        assert!(!headers.contains_key("HTTP-Referer"));
        assert!(!headers.contains_key("X-Title"));
    }
}
