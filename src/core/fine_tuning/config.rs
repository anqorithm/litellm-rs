//! Fine-tuning configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Fine-tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FineTuningConfig {
    /// Whether fine-tuning is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Default provider for fine-tuning
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_provider: Option<String>,

    /// Provider-specific configurations
    #[serde(default)]
    pub providers: HashMap<String, ProviderFineTuningConfig>,

    /// Maximum concurrent jobs per user
    #[serde(default = "default_max_concurrent_jobs")]
    pub max_concurrent_jobs: u32,

    /// Job polling interval in seconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_seconds: u64,
}

fn default_enabled() -> bool {
    true
}

fn default_max_concurrent_jobs() -> u32 {
    5
}

fn default_poll_interval() -> u64 {
    60
}

impl Default for FineTuningConfig {
    fn default() -> Self {
        Self {
            enabled: default_enabled(),
            default_provider: None,
            providers: HashMap::new(),
            max_concurrent_jobs: default_max_concurrent_jobs(),
            poll_interval_seconds: default_poll_interval(),
        }
    }
}

impl FineTuningConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    pub fn default_provider(mut self, provider: impl Into<String>) -> Self {
        self.default_provider = Some(provider.into());
        self
    }

    pub fn add_provider(
        mut self,
        name: impl Into<String>,
        config: ProviderFineTuningConfig,
    ) -> Self {
        self.providers.insert(name.into(), config);
        self
    }

    pub fn max_concurrent_jobs(mut self, max: u32) -> Self {
        self.max_concurrent_jobs = max;
        self
    }
}

/// Provider-specific fine-tuning configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderFineTuningConfig {
    /// Whether this provider is enabled for fine-tuning
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// API key for the provider
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API base URL (for custom endpoints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// Organization ID (for OpenAI)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization_id: Option<String>,

    /// Supported models for fine-tuning
    #[serde(default)]
    pub supported_models: Vec<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Additional headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

fn default_timeout() -> u64 {
    300
}

impl Default for ProviderFineTuningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            api_key: None,
            api_base: None,
            organization_id: None,
            supported_models: vec![],
            timeout_seconds: default_timeout(),
            headers: HashMap::new(),
        }
    }
}

impl ProviderFineTuningConfig {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn api_base(mut self, base: impl Into<String>) -> Self {
        self.api_base = Some(base.into());
        self
    }

    pub fn organization_id(mut self, org: impl Into<String>) -> Self {
        self.organization_id = Some(org.into());
        self
    }

    pub fn supported_model(mut self, model: impl Into<String>) -> Self {
        self.supported_models.push(model.into());
        self
    }

    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fine_tuning_config_default() {
        let config = FineTuningConfig::default();
        assert!(config.enabled);
        assert!(config.default_provider.is_none());
        assert_eq!(config.max_concurrent_jobs, 5);
    }

    #[test]
    fn test_fine_tuning_config_builder() {
        let config = FineTuningConfig::new()
            .enabled(true)
            .default_provider("openai")
            .max_concurrent_jobs(10)
            .add_provider(
                "openai",
                ProviderFineTuningConfig::new()
                    .api_key("sk-test")
                    .supported_model("gpt-3.5-turbo"),
            );

        assert!(config.enabled);
        assert_eq!(config.default_provider, Some("openai".to_string()));
        assert_eq!(config.max_concurrent_jobs, 10);
        assert!(config.providers.contains_key("openai"));
    }

    #[test]
    fn test_provider_config_builder() {
        let config = ProviderFineTuningConfig::new()
            .api_key("sk-test")
            .api_base("https://api.openai.com/v1")
            .organization_id("org-123")
            .supported_model("gpt-3.5-turbo")
            .supported_model("gpt-4")
            .timeout(600)
            .header("X-Custom", "value");

        assert_eq!(config.api_key, Some("sk-test".to_string()));
        assert_eq!(
            config.api_base,
            Some("https://api.openai.com/v1".to_string())
        );
        assert_eq!(config.organization_id, Some("org-123".to_string()));
        assert_eq!(config.supported_models.len(), 2);
        assert_eq!(config.timeout_seconds, 600);
        assert_eq!(config.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_serialization() {
        let config = FineTuningConfig::new().default_provider("openai");

        let json = serde_json::to_string(&config).unwrap();
        let parsed: FineTuningConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.default_provider, config.default_provider);
    }
}
