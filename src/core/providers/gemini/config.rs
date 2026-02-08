//! Gemini Configuration Module
//!
//! Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::provider::ProviderConfig;

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeminiConfig {
    /// API key (Google AI Studio)
    pub api_key: Option<String>,

    /// Project ID (Vertex AI)
    pub project_id: Option<String>,

    /// Region (Vertex AI)
    pub location: Option<String>,

    /// Service account JSON (Vertex AI)
    pub service_account_json: Option<String>,

    /// Use Vertex AI or Google AI Studio
    pub use_vertex_ai: bool,

    /// Base URL
    pub base_url: String,

    /// API version
    pub api_version: String,

    /// Request timeout
    pub request_timeout: u64,

    /// Connection timeout
    pub connect_timeout: u64,

    /// Maximum number of retries
    pub max_retries: u32,

    /// Retry delay (milliseconds)
    pub retry_delay_ms: u64,

    /// Enable caching
    pub enable_caching: bool,

    /// Cache TTL (seconds)
    pub cache_ttl_seconds: u64,

    /// Enable search grounding
    pub enable_search_grounding: bool,

    /// Safety settings
    pub safety_settings: Option<Vec<SafetySetting>>,

    /// Custom headers
    pub custom_headers: HashMap<String, String>,

    /// Proxy URL
    pub proxy_url: Option<String>,

    /// Enable debug logging
    pub debug: bool,
}

/// Settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetySetting {
    pub category: String,
    pub threshold: String,
}

impl GeminiConfig {
    /// Create
    pub fn new_google_ai(api_key: impl Into<String>) -> Self {
        Self {
            api_key: Some(api_key.into()),
            project_id: None,
            location: None,
            service_account_json: None,
            use_vertex_ai: false,
            base_url: "https://generativelanguage.googleapis.com".to_string(),
            api_version: "v1beta".to_string(),
            request_timeout: 600,
            connect_timeout: 10,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_caching: true,
            cache_ttl_seconds: 3600,
            enable_search_grounding: false,
            safety_settings: None,
            custom_headers: HashMap::new(),
            proxy_url: None,
            debug: false,
        }
    }

    /// Create
    pub fn new_vertex_ai(project_id: impl Into<String>, location: impl Into<String>) -> Self {
        let location_str = location.into();
        Self {
            api_key: None,
            project_id: Some(project_id.into()),
            location: Some(location_str.clone()),
            service_account_json: None,
            use_vertex_ai: true,
            base_url: format!("https://{}-aiplatform.googleapis.com", location_str),
            api_version: "v1".to_string(),
            request_timeout: 600,
            connect_timeout: 10,
            max_retries: 3,
            retry_delay_ms: 1000,
            enable_caching: true,
            cache_ttl_seconds: 3600,
            enable_search_grounding: false,
            safety_settings: None,
            custom_headers: HashMap::new(),
            proxy_url: None,
            debug: false,
        }
    }

    /// Create
    pub fn from_env() -> Result<Self, ProviderError> {
        // Try Google AI Studio first
        if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            return Ok(Self::new_google_ai(api_key));
        }

        if let Ok(api_key) = std::env::var("GEMINI_API_KEY") {
            return Ok(Self::new_google_ai(api_key));
        }

        // Try Vertex AI
        if let (Ok(project_id), Ok(location)) = (
            std::env::var("GOOGLE_CLOUD_PROJECT"),
            std::env::var("GOOGLE_CLOUD_LOCATION"),
        ) {
            let mut config = Self::new_vertex_ai(project_id, location);

            // Optional service account
            if let Ok(sa_json) = std::env::var("GOOGLE_APPLICATION_CREDENTIALS") {
                config.service_account_json = Some(sa_json);
            }

            return Ok(config);
        }

        Err(ProviderError::configuration(
            "gemini",
            "No valid Gemini configuration found in environment variables",
        ))
    }

    /// Settings
    pub fn with_safety_settings(mut self, settings: Vec<SafetySetting>) -> Self {
        self.safety_settings = Some(settings);
        self
    }

    /// Settings
    pub fn with_search_grounding(mut self, enabled: bool) -> Self {
        self.enable_search_grounding = enabled;
        self
    }

    /// Settings
    pub fn with_caching(mut self, enabled: bool, ttl_seconds: u64) -> Self {
        self.enable_caching = enabled;
        self.cache_ttl_seconds = ttl_seconds;
        self
    }

    /// Settings
    pub fn with_proxy(mut self, proxy_url: impl Into<String>) -> Self {
        self.proxy_url = Some(proxy_url.into());
        self
    }

    /// Settings
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    /// Create
    #[cfg(test)]
    pub fn new_test(api_key: impl Into<String>) -> Self {
        let mut config = Self::new_google_ai(api_key);
        config.request_timeout = 5;
        config.max_retries = 0;
        config
    }

    /// Get
    pub fn get_endpoint(&self, model: &str, operation: &str) -> String {
        if self.use_vertex_ai {
            // Vertex AI endpoint format
            format!(
                "{}/v1/projects/{}/locations/{}/publishers/google/models/{}:{}",
                self.base_url,
                self.project_id.as_ref().unwrap_or(&"".to_string()),
                self.location.as_ref().unwrap_or(&"".to_string()),
                model,
                operation
            )
        } else {
            // Google AI Studio endpoint format
            match operation {
                "streamGenerateContent" => format!(
                    "{}/{}/models/{}:streamGenerateContent?key={}",
                    self.base_url,
                    self.api_version,
                    model,
                    self.api_key.as_ref().unwrap_or(&"".to_string())
                ),
                _ => format!(
                    "{}/{}/models/{}:{}?key={}",
                    self.base_url,
                    self.api_version,
                    model,
                    operation,
                    self.api_key.as_ref().unwrap_or(&"".to_string())
                ),
            }
        }
    }

    /// Check
    pub fn is_feature_enabled(&self, feature: &str) -> bool {
        match feature {
            "caching" => self.enable_caching,
            "search_grounding" => self.enable_search_grounding,
            "debug" => self.debug,
            _ => false,
        }
    }
}

impl Default for GeminiConfig {
    fn default() -> Self {
        Self::new_google_ai("")
    }
}

impl ProviderConfig for GeminiConfig {
    fn validate(&self) -> Result<(), String> {
        if self.use_vertex_ai {
            // Vertex AI validation - use pattern matching to avoid unwrap
            match &self.project_id {
                Some(id) if !id.is_empty() => {}
                _ => return Err("Project ID is required for Vertex AI".to_string()),
            }

            match &self.location {
                Some(loc) if !loc.is_empty() => {}
                _ => return Err("Location is required for Vertex AI".to_string()),
            }
        } else {
            // Google AI Studio validation - use pattern matching
            let api_key = match &self.api_key {
                Some(key) if !key.is_empty() => key,
                _ => return Err("API key is required for Google AI Studio".to_string()),
            };

            if api_key.len() < 20 {
                return Err("API key appears to be too short".to_string());
            }
        }

        // General validation
        if self.request_timeout == 0 {
            return Err("Request timeout must be greater than 0".to_string());
        }

        if self.connect_timeout == 0 {
            return Err("Connect timeout must be greater than 0".to_string());
        }

        if self.connect_timeout > self.request_timeout {
            return Err("Connect timeout cannot be greater than request timeout".to_string());
        }

        if self.max_retries > 10 {
            return Err("Max retries cannot exceed 10".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.api_key.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        Some(&self.base_url)
    }

    fn timeout(&self) -> Duration {
        Duration::from_secs(self.request_timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

/// Configuration builder
pub struct GeminiConfigBuilder {
    config: GeminiConfig,
}

impl GeminiConfigBuilder {
    /// Create
    pub fn google_ai(api_key: impl Into<String>) -> Self {
        Self {
            config: GeminiConfig::new_google_ai(api_key),
        }
    }

    /// Create
    pub fn vertex_ai(project_id: impl Into<String>, location: impl Into<String>) -> Self {
        Self {
            config: GeminiConfig::new_vertex_ai(project_id, location),
        }
    }

    /// Settings
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.config.base_url = base_url.into();
        self
    }

    /// Settings
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.config.request_timeout = timeout_secs;
        self
    }

    /// Settings
    pub fn with_retries(mut self, max_retries: u32) -> Self {
        self.config.max_retries = max_retries;
        self
    }

    /// Settings
    pub fn with_caching(mut self, enabled: bool) -> Self {
        self.config.enable_caching = enabled;
        self
    }

    /// Settings
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.config.debug = debug;
        self
    }

    /// Configuration
    pub fn build(self) -> Result<GeminiConfig, ProviderError> {
        self.config
            .validate()
            .map_err(|e| ProviderError::configuration("gemini", e))?;
        Ok(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_google_ai_config() {
        let config = GeminiConfig::new_google_ai("test-api-key-1234567890123456");
        assert!(!config.use_vertex_ai);
        assert_eq!(
            config.api_key,
            Some("test-api-key-1234567890123456".to_string())
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_vertex_ai_config() {
        let config = GeminiConfig::new_vertex_ai("test-project", "us-central1");
        assert!(config.use_vertex_ai);
        assert_eq!(config.project_id, Some("test-project".to_string()));
        assert_eq!(config.location, Some("us-central1".to_string()));
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation() {
        let mut config = GeminiConfig::new_google_ai("");
        assert!(config.validate().is_err());

        config.api_key = Some("valid-api-key-12345678901234567890".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_endpoint_generation() {
        let config = GeminiConfig::new_google_ai("test-key-1234567890123456");
        let endpoint = config.get_endpoint("gemini-pro", "generateContent");
        assert!(endpoint.contains("generativelanguage.googleapis.com"));
        assert!(endpoint.contains("gemini-pro:generateContent"));
        assert!(endpoint.contains("key=test-key-1234567890123456"));
    }

    #[test]
    fn test_builder_pattern() {
        let config = GeminiConfigBuilder::google_ai("test-key-1234567890123456")
            .with_timeout(300)
            .with_retries(5)
            .with_debug(true)
            .build()
            .unwrap();

        assert_eq!(config.request_timeout, 300);
        assert_eq!(config.max_retries, 5);
        assert!(config.debug);
    }
}
