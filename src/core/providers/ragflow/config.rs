//! RAGFlow Provider Configuration

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagflowConfig {
    pub api_key: Option<String>,
    pub api_base: Option<String>,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    #[serde(default)]
    pub debug: bool,
}

impl Default for RagflowConfig {
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

impl ProviderConfig for RagflowConfig {
    fn validate(&self) -> Result<(), String> {
        if self.api_key.is_none() && std::env::var("RAGFLOW_API_KEY").is_err() {
            return Err(
                "RAGFlow API key not provided and RAGFLOW_API_KEY environment variable not set"
                    .to_string(),
            );
        }

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

impl RagflowConfig {
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key
            .clone()
            .or_else(|| std::env::var("RAGFLOW_API_KEY").ok())
    }

    pub fn get_api_base(&self) -> String {
        self.api_base
            .clone()
            .or_else(|| std::env::var("RAGFLOW_API_BASE").ok())
            .unwrap_or_else(|| "http://localhost:9380/v1".to_string())
    }
}

fn default_timeout() -> u64 {
    60
}

fn default_max_retries() -> u32 {
    3
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ragflow_config_default() {
        let config = RagflowConfig::default();
        assert!(config.api_key.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }
}
