//! Docker Model Runner Configuration

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

define_provider_config!(DockerModelRunnerConfig {});

impl DockerModelRunnerConfig {
    /// Create configuration from environment
    pub fn from_env() -> Self {
        Self::new("docker_model_runner")
    }

    /// Get the effective API base URL
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "http://localhost:8000".to_string())
    }
}

impl ProviderConfig for DockerModelRunnerConfig {
    fn validate(&self) -> Result<(), String> {
        if self.base.api_base.is_none() {
            return Err("Docker Model Runner requires api_base URL to be set".to_string());
        }
        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docker_model_runner_config() {
        let config = DockerModelRunnerConfig::new("docker_model_runner");
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_docker_model_runner_get_api_base_default() {
        let mut config = DockerModelRunnerConfig::new("docker_model_runner");
        config.base.api_base = None;
        assert_eq!(config.get_api_base(), "http://localhost:8000");
    }
}
