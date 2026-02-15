//! DataRobot Configuration
//!
//! Configuration for DataRobot AI platform

use crate::define_provider_config;

define_provider_config!(DataRobotConfig, provider: "datarobot");

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

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
