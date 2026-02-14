//! Configuration
//!
//! Configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseConfig {
    /// API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API base URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_base: Option<String>,

    /// Request
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// maximumNumber of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Custom HTTP headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Organization ID (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub organization: Option<String>,

    /// API version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<String>,
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

impl Default for BaseConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        }
    }
}

impl BaseConfig {
    /// Configuration
    pub fn from_env(provider: &str) -> Self {
        let provider_upper = provider.to_uppercase();

        Self {
            api_key: std::env::var(format!("{}_API_KEY", provider_upper)).ok(),
            api_base: std::env::var(format!("{}_API_BASE", provider_upper)).ok(),
            timeout: std::env::var(format!("{}_TIMEOUT", provider_upper))
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(default_timeout()),
            max_retries: std::env::var(format!("{}_MAX_RETRIES", provider_upper))
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(default_max_retries()),
            headers: HashMap::new(),
            organization: std::env::var(format!("{}_ORGANIZATION", provider_upper)).ok(),
            api_version: std::env::var(format!("{}_API_VERSION", provider_upper)).ok(),
        }
    }

    /// Default
    pub fn for_provider(provider: &str) -> Self {
        let mut config = Self::from_env(provider);

        // Default
        if config.api_base.is_none() {
            config.api_base = Some(
                match provider {
                    "openai" => "https://api.openai.com/v1",
                    "anthropic" => "https://api.anthropic.com",
                    "azure" => "https://{instance}.openai.azure.com",
                    "mistral" => "https://api.mistral.ai/v1",
                    "deepseek" => "https://api.deepseek.com",
                    "moonshot" => "https://api.moonshot.cn/v1",
                    "deepinfra" => "https://api.deepinfra.com/v1/openai",
                    "vertex_ai" => "https://generativelanguage.googleapis.com",
                    "openrouter" => "https://openrouter.ai/api/v1",
                    "ai21" => "https://api.ai21.com/studio/v1",
                    "cerebras" => "https://api.cerebras.ai/v1",
                    "gigachat" => "https://gigachat.devices.sberbank.ru/api/v1",
                    "friendliai" => "https://api.friendli.ai/v1",
                    "nlp_cloud" => "https://api.nlpcloud.io/v1",
                    "volcengine" => "https://ark.cn-beijing.volces.com/api/v3",
                    "nebius" => "https://api.studio.nebius.ai/v1",
                    "nscale" => "https://inference.api.nscale.ai/v1",
                    _ => "https://api.openai.com/v1", // Default
                }
                .to_string(),
            );
        }

        // Default
        if provider == "anthropic" && config.api_version.is_none() {
            config.api_version = Some("2023-06-01".to_string());
        }

        // Azure requires API version
        if provider == "azure" && config.api_version.is_none() {
            config.api_version = Some("2024-02-01".to_string());
        }

        config
    }

    /// Configuration
    pub fn validate(&self, provider: &str) -> Result<(), String> {
        // Validation
        if self.api_key.is_none() {
            return Err(format!("{} API key is required", provider));
        }

        // Validation
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        // Validation
        if self.max_retries > 10 {
            return Err("Max retries should not exceed 10".to_string());
        }

        // Validation
        if let Some(api_key) = &self.api_key {
            match provider {
                "openai" if !api_key.starts_with("sk-") && !api_key.starts_with("proj-") => {
                    return Err("OpenAI API key should start with 'sk-' or 'proj-'".to_string());
                }
                "anthropic" if !api_key.starts_with("sk-ant-") => {
                    return Err("Anthropic API key should start with 'sk-ant-'".to_string());
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Configuration
    pub fn get_effective_api_key(&self, provider: &str) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var(format!("{}_API_KEY", provider.to_uppercase())).ok())
    }

    /// Get
    pub fn get_effective_api_base(&self, provider: &str) -> String {
        self.api_base.clone().unwrap_or_else(|| {
            std::env::var(format!("{}_API_BASE", provider.to_uppercase()))
                .unwrap_or_else(|_| Self::for_provider(provider).api_base.unwrap_or_default())
        })
    }

    /// Get
    pub fn get_chat_endpoint(&self) -> String {
        format!(
            "{}/chat/completions",
            self.api_base.as_ref().unwrap_or(&String::new())
        )
    }

    /// Get
    pub fn get_embeddings_endpoint(&self) -> String {
        format!(
            "{}/embeddings",
            self.api_base.as_ref().unwrap_or(&String::new())
        )
    }

    /// Convert to Duration
    pub fn timeout_duration(&self) -> Duration {
        Duration::from_secs(self.timeout)
    }
}

/// Configuration
#[macro_export]
macro_rules! define_provider_config {
    // Full version: generates struct, Default, new, from_env, builders, ProviderConfig impl
    ($name:ident, provider: $provider:expr) => {
        $crate::define_provider_config!($name { }, provider: $provider);
    };

    ($name:ident { $($field:ident: $type:ty = $default:expr),* $(,)? }, provider: $provider:expr) => {
        $crate::define_provider_config!($name { $($field: $type = $default),* });

        impl $name {
            pub fn from_env() -> Self {
                Self::new($provider)
            }

            pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
                self.base.api_key = Some(api_key.into());
                self
            }

            pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
                self.base.api_base = Some(base_url.into());
                self
            }

            pub fn with_timeout(mut self, timeout: u64) -> Self {
                self.base.timeout = timeout;
                self
            }
        }

        impl $crate::core::traits::provider::ProviderConfig for $name {
            fn validate(&self) -> Result<(), String> {
                self.base.validate($provider)
            }

            fn api_key(&self) -> Option<&str> {
                self.base.api_key.as_deref()
            }

            fn api_base(&self) -> Option<&str> {
                self.base.api_base.as_deref()
            }

            fn timeout(&self) -> std::time::Duration {
                self.base.timeout_duration()
            }

            fn max_retries(&self) -> u32 {
                self.base.max_retries
            }
        }
    };

    // Env-required version: from_env() returns Result<Self, String> requiring API key
    ($name:ident, env_key: $env_key:expr, provider: $provider:expr) => {
        $crate::define_provider_config!($name { }, env_key: $env_key, provider: $provider);
    };

    ($name:ident { $($field:ident: $type:ty = $default:expr),* $(,)? }, env_key: $env_key:expr, provider: $provider:expr) => {
        $crate::define_provider_config!($name { $($field: $type = $default),* });

        impl $name {
            pub fn from_env() -> Result<Self, String> {
                let api_key = std::env::var($env_key)
                    .map_err(|_| concat!($env_key, " environment variable is required"))?;
                Ok(Self::new($provider).with_api_key(api_key))
            }

            pub fn with_api_key(mut self, api_key: impl Into<String>) -> Self {
                self.base.api_key = Some(api_key.into());
                self
            }

            pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
                self.base.api_base = Some(base_url.into());
                self
            }

            pub fn with_timeout(mut self, timeout: u64) -> Self {
                self.base.timeout = timeout;
                self
            }
        }

        impl $crate::core::traits::provider::ProviderConfig for $name {
            fn validate(&self) -> Result<(), String> {
                self.base.validate($provider)
            }

            fn api_key(&self) -> Option<&str> {
                self.base.api_key.as_deref()
            }

            fn api_base(&self) -> Option<&str> {
                self.base.api_base.as_deref()
            }

            fn timeout(&self) -> std::time::Duration {
                self.base.timeout_duration()
            }

            fn max_retries(&self) -> u32 {
                self.base.max_retries
            }
        }
    };

    ($name:ident { $($field:ident: $type:ty = $default:expr),* $(,)? }) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
        pub struct $name {
            #[serde(flatten)]
            pub base: $crate::core::providers::base::config::BaseConfig,
            $(
                #[serde(skip_serializing_if = "Option::is_none")]
                pub $field: $type,
            )*
        }

        impl Default for $name {
            fn default() -> Self {
                Self {
                    base: $crate::core::providers::base::config::BaseConfig::default(),
                    $($field: $default),*
                }
            }
        }

        impl $name {
            pub fn from_base(base: $crate::core::providers::base::config::BaseConfig) -> Self {
                Self {
                    base,
                    $($field: $default),*
                }
            }

            pub fn new(provider: &str) -> Self {
                Self::from_base($crate::core::providers::base::config::BaseConfig::for_provider(provider))
            }
        }

        impl AsRef<$crate::core::providers::base::config::BaseConfig> for $name {
            fn as_ref(&self) -> &$crate::core::providers::base::config::BaseConfig {
                &self.base
            }
        }

        impl AsMut<$crate::core::providers::base::config::BaseConfig> for $name {
            fn as_mut(&mut self) -> &mut $crate::core::providers::base::config::BaseConfig {
                &mut self.base
            }
        }
    };

    // Version without additional fields
    ($name:ident) => {
        $crate::define_provider_config!($name {});
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_base_config_default() {
        let config = BaseConfig::default();
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_provider_specific_defaults() {
        let openai = BaseConfig::for_provider("openai");
        assert_eq!(
            openai.api_base,
            Some("https://api.openai.com/v1".to_string())
        );

        let anthropic = BaseConfig::for_provider("anthropic");
        assert_eq!(anthropic.api_version, Some("2023-06-01".to_string()));
    }

    #[test]
    fn test_validation() {
        let mut config = BaseConfig::for_provider("openai");

        // Missing API key
        assert!(config.validate("openai").is_err());

        // Add valid API key
        config.api_key = Some("sk-test123".to_string());
        assert!(config.validate("openai").is_ok());

        // Invalid API key format
        config.api_key = Some("invalid-key".to_string());
        assert!(config.validate("openai").is_err());
    }
}
