//! Infinity Provider Configuration
//!
//! Configuration for Infinity embedding and reranking server access.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Infinity provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InfinityConfig {
    /// API key for Infinity authentication (optional - may not be required for self-hosted)
    pub api_key: Option<String>,

    /// API base URL (required - no default as Infinity is self-hosted)
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

impl Default for InfinityConfig {
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

impl ProviderConfig for InfinityConfig {
    fn validate(&self) -> Result<(), String> {
        // API base is required for Infinity (self-hosted server)
        if self.api_base.is_none() && std::env::var("INFINITY_API_BASE").is_err() {
            return Err(
                "Infinity API base not provided and INFINITY_API_BASE environment variable not set. \
                 Infinity is a self-hosted server, so api_base must be specified."
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

impl InfinityConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("INFINITY_API_KEY").ok())
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("INFINITY_API_BASE").ok())
    }

    /// Get embeddings endpoint URL
    pub fn get_embeddings_url(&self) -> Option<String> {
        self.get_api_base().map(|base| {
            let base = base.trim_end_matches('/');
            if base.ends_with("/embeddings") {
                base.to_string()
            } else {
                format!("{}/embeddings", base)
            }
        })
    }

    /// Get rerank endpoint URL
    pub fn get_rerank_url(&self) -> Option<String> {
        self.get_api_base().map(|base| {
            let base = base.trim_end_matches('/');
            if base.ends_with("/rerank") {
                base.to_string()
            } else {
                format!("{}/rerank", base)
            }
        })
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
    fn test_infinity_config_default() {
        let config = InfinityConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_infinity_config_get_api_base() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_api_base(),
            Some("http://localhost:8080".to_string())
        );
    }

    #[test]
    fn test_infinity_config_get_embeddings_url() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_embeddings_url(),
            Some("http://localhost:8080/embeddings".to_string())
        );

        // Test with trailing slash
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080/".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_embeddings_url(),
            Some("http://localhost:8080/embeddings".to_string())
        );

        // Test when already has /embeddings
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080/embeddings".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_embeddings_url(),
            Some("http://localhost:8080/embeddings".to_string())
        );
    }

    #[test]
    fn test_infinity_config_get_rerank_url() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_rerank_url(),
            Some("http://localhost:8080/rerank".to_string())
        );

        // Test with trailing slash
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080/".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_rerank_url(),
            Some("http://localhost:8080/rerank".to_string())
        );

        // Test when already has /rerank
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080/rerank".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_rerank_url(),
            Some("http://localhost:8080/rerank".to_string())
        );
    }

    #[test]
    fn test_infinity_config_get_api_key() {
        let config = InfinityConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_infinity_config_provider_config_trait() {
        let config = InfinityConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://localhost:8080".to_string()),
            timeout: 60,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("http://localhost:8080"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_infinity_config_validation_with_api_base() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_infinity_config_validation_zero_timeout() {
        let config = InfinityConfig {
            api_base: Some("http://localhost:8080".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_infinity_config_serialization() {
        let config = InfinityConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("http://localhost:8080".to_string()),
            timeout: 45,
            max_retries: 2,
            debug: true,
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "http://localhost:8080");
        assert_eq!(json["timeout"], 45);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_infinity_config_deserialization() {
        let json = r#"{
            "api_base": "http://localhost:8080",
            "timeout": 60,
            "debug": true
        }"#;

        let config: InfinityConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_base, Some("http://localhost:8080".to_string()));
        assert_eq!(config.timeout, 60);
        assert!(config.debug);
    }
}
