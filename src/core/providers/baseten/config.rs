//! Baseten Provider Configuration
//!
//! Configuration for Baseten API access including authentication and model settings.

use crate::define_standalone_provider_config;
use regex::Regex;
use std::sync::LazyLock;

/// Default API base URL for Baseten Model API
pub const BASETEN_API_BASE: &str = "https://inference.baseten.co/v1";

/// Regex for matching dedicated deployment model IDs
static DEDICATED_DEPLOYMENT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9]{8}$").expect("static regex is valid"));

define_standalone_provider_config!(BasetenConfig,
    provider: "Baseten",
    env_prefix: "BASETEN",
    default_base_url: "https://inference.baseten.co/v1",
    default_timeout: 30,
    extra_fields: { debug: bool = false },
);

impl BasetenConfig {
    /// Check if model is a dedicated deployment (8-character alphanumeric code)
    pub fn is_dedicated_deployment(model: &str) -> bool {
        let model_id = model.strip_prefix("baseten/").unwrap_or(model);
        DEDICATED_DEPLOYMENT_REGEX.is_match(model_id)
    }

    /// Get the appropriate API base URL for the given model
    pub fn get_api_base_for_model(model: &str) -> String {
        if Self::is_dedicated_deployment(model) {
            let model_id = model.strip_prefix("baseten/").unwrap_or(model);
            format!(
                "https://model-{}.api.baseten.co/environments/production/sync/v1",
                model_id
            )
        } else {
            BASETEN_API_BASE.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::traits::provider::ProviderConfig;

    #[test]
    fn test_baseten_config_default() {
        let config = BasetenConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert_eq!(config.timeout, 30);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_baseten_config_get_api_base_default() {
        let config = BasetenConfig::default();
        assert_eq!(config.get_api_base(), BASETEN_API_BASE);
    }

    #[test]
    fn test_baseten_config_get_api_key() {
        let config = BasetenConfig::new("test-key");
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_is_dedicated_deployment() {
        assert!(BasetenConfig::is_dedicated_deployment("abc12345"));
        assert!(BasetenConfig::is_dedicated_deployment("baseten/abc12345"));
        assert!(BasetenConfig::is_dedicated_deployment("ABCD1234"));
        assert!(!BasetenConfig::is_dedicated_deployment("llama-3.1-70b"));
        assert!(!BasetenConfig::is_dedicated_deployment("ab12345"));
        assert!(!BasetenConfig::is_dedicated_deployment("abc123456"));
        assert!(!BasetenConfig::is_dedicated_deployment("abc-1234"));
    }

    #[test]
    fn test_get_api_base_for_model() {
        assert_eq!(
            BasetenConfig::get_api_base_for_model("llama-3.1-70b"),
            BASETEN_API_BASE
        );
        assert_eq!(
            BasetenConfig::get_api_base_for_model("abc12345"),
            "https://model-abc12345.api.baseten.co/environments/production/sync/v1"
        );
    }

    #[test]
    fn test_baseten_config_validation_with_key() {
        let config = BasetenConfig::new("test-key");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_baseten_config_validation_zero_timeout() {
        let config = BasetenConfig {
            api_key: Some("test-key".to_string()),
            timeout: 0,
            ..Default::default()
        };
        assert!(config.validate().is_err());
    }
}
