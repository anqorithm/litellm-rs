//! DataRobot Configuration
//!
//! Configuration for DataRobot AI platform

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

define_provider_config!(DataRobotConfig {});

impl DataRobotConfig {
    /// Create configuration from environment
    pub fn from_env() -> Self {
        Self::new("datarobot")
    }

    /// Get the effective API base URL
    pub fn get_api_base(&self) -> String {
        self.base
            .api_base
            .clone()
            .unwrap_or_else(|| "https://app.datarobot.com/api/v2".to_string())
    }
}

impl ProviderConfig for DataRobotConfig {
    fn validate(&self) -> Result<(), String> {
        self.base.validate("datarobot")
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
    fn test_datarobot_config() {
        let config = DataRobotConfig::new("datarobot");
        assert!(config.base.api_base.is_some());
    }

    #[test]
    fn test_datarobot_validate_missing_api_key() {
        let config = DataRobotConfig::new("datarobot");
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API key"));
    }

    #[test]
    fn test_datarobot_validate_success() {
        let mut config = DataRobotConfig::new("datarobot");
        config.base.api_key = Some("dr-test-key".to_string());
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_datarobot_get_api_base_default() {
        let mut config = DataRobotConfig::new("datarobot");
        config.base.api_base = None;
        assert_eq!(config.get_api_base(), "https://app.datarobot.com/api/v2");
    }
}
