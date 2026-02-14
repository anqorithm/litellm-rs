//! Anthropic Configuration
//!
//! Configuration

use std::collections::HashMap;
use std::env;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;

/// Configuration
#[derive(Debug, Clone)]
pub struct AnthropicConfig {
    /// API key
    pub api_key: Option<String>,
    /// Base URL
    pub base_url: String,
    /// APIversion
    pub api_version: String,
    /// Request
    pub request_timeout: u64,
    /// Connection
    pub connect_timeout: u64,
    /// maximumNumber of retries
    pub max_retries: u32,
    /// Retry delay base (milliseconds)
    pub retry_delay_base: u64,
    /// Proxy URL (optional)
    pub proxy_url: Option<String>,
    /// Request
    pub custom_headers: HashMap<String, String>,
    /// Enable multimodal support
    pub enable_multimodal: bool,
    /// Enable cache control
    pub enable_cache_control: bool,
    /// Enable computer tools
    pub enable_computer_use: bool,
    /// Enable experimental features
    pub enable_experimental: bool,
}

impl Default for AnthropicConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            base_url: "https://api.anthropic.com".to_string(),
            api_version: "2023-06-01".to_string(),
            request_timeout: 120,
            connect_timeout: 10,
            max_retries: 3,
            retry_delay_base: 1000,
            proxy_url: None,
            custom_headers: HashMap::new(),
            enable_multimodal: true,
            enable_cache_control: true,
            enable_computer_use: false, // Default disabled
            enable_experimental: false,
        }
    }
}

impl AnthropicConfig {
    /// Create
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            ..Default::default()
        }
    }

    /// Create
    pub fn new_test(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            base_url: "https://api.anthropic.com".to_string(),
            ..Default::default()
        }
    }

    /// Configuration
    pub fn from_env() -> Result<Self, ProviderError> {
        let api_key = env::var("ANTHROPIC_API_KEY")
            .or_else(|_| env::var("CLAUDE_API_KEY"))
            .map(Some)
            .map_err(|_| {
                ProviderError::configuration(
                    "anthropic",
                    "ANTHROPIC_API_KEY or CLAUDE_API_KEY environment variable is required",
                )
            })?;

        let mut config = Self {
            api_key,
            ..Default::default()
        };

        // Configuration
        if let Ok(base_url) = env::var("ANTHROPIC_BASE_URL") {
            config.base_url = base_url;
        }

        if let Ok(api_version) = env::var("ANTHROPIC_API_VERSION") {
            config.api_version = api_version;
        }

        if let Ok(timeout) = env::var("ANTHROPIC_TIMEOUT") {
            config.request_timeout = timeout.parse().unwrap_or(120);
        }

        if let Ok(proxy) = env::var("ANTHROPIC_PROXY") {
            config.proxy_url = Some(proxy);
        }

        // Feature switches
        if let Ok(multimodal) = env::var("ANTHROPIC_ENABLE_MULTIMODAL") {
            config.enable_multimodal = multimodal.parse().unwrap_or(true);
        }

        if let Ok(cache) = env::var("ANTHROPIC_ENABLE_CACHE") {
            config.enable_cache_control = cache.parse().unwrap_or(true);
        }

        if let Ok(computer) = env::var("ANTHROPIC_ENABLE_COMPUTER_USE") {
            config.enable_computer_use = computer.parse().unwrap_or(false);
        }

        if let Ok(experimental) = env::var("ANTHROPIC_ENABLE_EXPERIMENTAL") {
            config.enable_experimental = experimental.parse().unwrap_or(false);
        }

        Ok(config)
    }

    /// Settings
    pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
        self.api_key = Some(api_key.into());
        self
    }

    /// Settings
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Settings
    pub fn with_api_version(mut self, version: impl Into<String>) -> Self {
        self.api_version = version.into();
        self
    }

    /// Settings
    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// Settings
    pub fn with_proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Request
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.custom_headers.insert(key.into(), value.into());
        self
    }

    /// Enable multimodal support
    pub fn with_multimodal(mut self, enabled: bool) -> Self {
        self.enable_multimodal = enabled;
        self
    }

    /// Enable cache control
    pub fn with_cache_control(mut self, enabled: bool) -> Self {
        self.enable_cache_control = enabled;
        self
    }

    /// Enable computer tools
    pub fn with_computer_use(mut self, enabled: bool) -> Self {
        self.enable_computer_use = enabled;
        self
    }

    /// Enable experimental features
    pub fn with_experimental(mut self, enabled: bool) -> Self {
        self.enable_experimental = enabled;
        self
    }

    /// Get
    pub fn get_api_url(&self, endpoint: &str) -> String {
        format!("{}{}", self.base_url.trim_end_matches('/'), endpoint)
    }

    /// Check
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "multimodal" => self.enable_multimodal,
            "cache_control" => self.enable_cache_control,
            "computer_use" => self.enable_computer_use,
            "experimental" => self.enable_experimental,
            _ => false,
        }
    }
}

impl ProviderConfig for AnthropicConfig {
    fn validate(&self) -> Result<(), String> {
        let api_key = self.api_key.as_ref().ok_or("API key is required")?;
        if api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        if !api_key.starts_with("sk-ant-") {
            return Err(
                "Invalid Anthropic API key format. Keys should start with 'sk-ant-'".to_string(),
            );
        }

        if api_key.len() < 20 {
            return Err("API key appears to be too short".to_string());
        }

        // Validation
        if self.base_url.is_empty() {
            return Err("Base URL cannot be empty".to_string());
        }

        if !self.base_url.starts_with("http://") && !self.base_url.starts_with("https://") {
            return Err("Base URL must start with http:// or https://".to_string());
        }

        // Settings
        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.connect_timeout == 0 {
            return Err("Connect timeout must be greater than 0".to_string());
        }

        if self.connect_timeout > self.request_timeout {
            return Err("Connect timeout cannot be greater than request timeout".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.base_url)
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.request_timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Configuration
pub struct AnthropicConfigBuilder {
    config: AnthropicConfig,
}

impl AnthropicConfigBuilder {
    /// Create
    pub fn new() -> Self {
        Self {
            config: AnthropicConfig::default(),
        }
    }

    /// Build
    pub fn from_env() -> Result<Self, ProviderError> {
        Ok(Self {
            config: AnthropicConfig::from_env()?,
        })
    }

    /// Settings
    pub fn api_key(mut self, api_key: impl Into<String>) -> Self {
        self.config.api_key = Some(api_key.into());
        self
    }

    /// Settings
    pub fn base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    /// Settings
    pub fn timeout(mut self, timeout: u64) -> Self {
        self.config.request_timeout = timeout;
        self
    }

    /// Enable multimodal
    pub fn multimodal(mut self, enabled: bool) -> Self {
        self.config.enable_multimodal = enabled;
        self
    }

    /// Enable experimental features
    pub fn experimental(mut self, enabled: bool) -> Self {
        self.config.enable_experimental = enabled;
        self.config.enable_computer_use = enabled; // Computer tools are part of experimental features
        self
    }

    /// Configuration
    pub fn build(self) -> Result<AnthropicConfig, ProviderError> {
        self.config
            .validate()
            .map_err(|e| ProviderError::configuration("anthropic", e))?;
        Ok(self.config)
    }
}

impl Default for AnthropicConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AnthropicConfig::default();
        assert_eq!(config.base_url, "https://api.anthropic.com");
        assert_eq!(config.api_version, "2023-06-01");
        assert!(config.enable_multimodal);
        assert!(config.enable_cache_control);
        assert!(!config.enable_computer_use);
    }

    #[test]
    fn test_config_validation() {
        let mut config = AnthropicConfig::default();

        // Should fail without API key
        assert!(config.validate().is_err());

        // Should pass with valid API key
        config.api_key = Some("sk-ant-api03-test123456".to_string());
        assert!(config.validate().is_ok());

        // Should fail with invalid API key format
        config.api_key = Some("invalid-key".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let config = AnthropicConfigBuilder::new()
            .api_key("sk-ant-test1234567890123")
            .base_url("https://custom.api.com")
            .timeout(60)
            .multimodal(false)
            .build();

        assert!(config.is_ok());
        let config = config.unwrap();
        assert_eq!(config.api_key, Some("sk-ant-test1234567890123".to_string()));
        assert_eq!(config.base_url, "https://custom.api.com");
        assert_eq!(config.request_timeout, 60);
        assert!(!config.enable_multimodal);
    }

    #[test]
    fn test_feature_check() {
        let config = AnthropicConfig::default();
        assert!(config.is_feature_enabled("multimodal"));
        assert!(config.is_feature_enabled("cache_control"));
        assert!(!config.is_feature_enabled("computer_use"));
        assert!(!config.is_feature_enabled("unknown_feature"));
    }

    #[test]
    fn test_api_url_generation() {
        let config = AnthropicConfig::default();
        assert_eq!(
            config.get_api_url("/v1/messages"),
            "https://api.anthropic.com/v1/messages"
        );

        // Test trailing slash removal
        let config = config.with_base_url("https://api.anthropic.com/");
        assert_eq!(
            config.get_api_url("/v1/messages"),
            "https://api.anthropic.com/v1/messages"
        );
    }
}
