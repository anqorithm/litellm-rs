//! Databricks Configuration
//!
//! Configuration for Databricks Foundation Model APIs.
//!
//! Authentication priority:
//! 1. OAuth M2M (DATABRICKS_CLIENT_ID + DATABRICKS_CLIENT_SECRET) - Recommended for production
//! 2. PAT (DATABRICKS_API_KEY) - Supported for development
//! 3. Direct API key parameter

use crate::core::traits::provider::ProviderConfig;
use crate::define_provider_config;

// Use the define_provider_config macro to create DatabricksConfig
define_provider_config!(DatabricksConfig {});

impl DatabricksConfig {
    /// Create configuration from environment variables
    pub fn from_env() -> Self {
        let mut config = Self::new("databricks");

        // Check for Databricks-specific environment variables
        if let Ok(api_base) = std::env::var("DATABRICKS_API_BASE") {
            config.base.api_base = Some(api_base);
        }
        if let Ok(api_key) = std::env::var("DATABRICKS_API_KEY") {
            config.base.api_key = Some(api_key);
        }

        config
    }

    /// Create configuration with API key and base URL
    pub fn with_credentials(api_key: impl Into<String>, api_base: impl Into<String>) -> Self {
        let mut config = Self::from_env();
        config.base.api_key = Some(api_key.into());
        config.base.api_base = Some(api_base.into());
        config
    }

    /// Create configuration with just API key (requires api_base to be set via env or later)
    pub fn with_api_key(api_key: impl Into<String>) -> Self {
        let mut config = Self::from_env();
        config.base.api_key = Some(api_key.into());
        config
    }

    /// Get the effective API base URL with serving-endpoints path
    pub fn get_serving_endpoint_base(&self) -> Option<String> {
        self.base.api_base.as_ref().map(|base| {
            let base = base.trim_end_matches('/');
            if base.ends_with("/serving-endpoints") {
                base.to_string()
            } else {
                format!("{}/serving-endpoints", base)
            }
        })
    }

    /// Check if OAuth M2M credentials are available
    pub fn has_oauth_credentials() -> bool {
        std::env::var("DATABRICKS_CLIENT_ID").is_ok()
            && std::env::var("DATABRICKS_CLIENT_SECRET").is_ok()
    }

    /// Build user agent string
    pub fn build_user_agent(custom_agent: Option<&str>) -> String {
        const VERSION: &str = env!("CARGO_PKG_VERSION");

        if let Some(agent) = custom_agent {
            let agent = agent.trim();
            // Extract partner name (part before / if present)
            let partner_name = if agent.contains('/') {
                agent.split('/').next().unwrap_or(agent)
            } else {
                agent
            };

            // Validate partner name: alphanumeric, underscore, hyphen only
            let is_valid = partner_name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-');

            if !partner_name.is_empty() && is_valid {
                return format!("{}_litellm-rs/{}", partner_name, VERSION);
            }
        }

        format!("litellm-rs/{}", VERSION)
    }
}

impl ProviderConfig for DatabricksConfig {
    fn validate(&self) -> Result<(), String> {
        // Need either API key or OAuth credentials
        if self.base.api_key.is_none() && !DatabricksConfig::has_oauth_credentials() {
            return Err(
                "Databricks requires either API key (DATABRICKS_API_KEY) or OAuth credentials \
                 (DATABRICKS_CLIENT_ID + DATABRICKS_CLIENT_SECRET)"
                    .to_string(),
            );
        }

        // Need API base
        if self.base.api_base.is_none() {
            return Err("Databricks API base URL (DATABRICKS_API_BASE) is required".to_string());
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
    fn test_databricks_config_default_timeout() {
        let config = DatabricksConfig::from_env();
        assert_eq!(config.base.timeout, 60);
    }

    #[test]
    fn test_databricks_config_with_credentials() {
        let config = DatabricksConfig::with_credentials(
            "dapi-test-key",
            "https://adb-123.azuredatabricks.net",
        );
        assert_eq!(config.base.api_key, Some("dapi-test-key".to_string()));
        assert_eq!(
            config.base.api_base,
            Some("https://adb-123.azuredatabricks.net".to_string())
        );
    }

    #[test]
    fn test_databricks_config_with_api_key() {
        let config = DatabricksConfig::with_api_key("dapi-test-key");
        assert_eq!(config.base.api_key, Some("dapi-test-key".to_string()));
    }

    #[test]
    fn test_databricks_validate_missing_api_key() {
        let mut config = DatabricksConfig::from_env();
        config.base.api_key = None;
        config.base.api_base = Some("https://test.databricks.net".to_string());

        // Will only fail if no OAuth credentials in environment
        if !DatabricksConfig::has_oauth_credentials() {
            let result = config.validate();
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_databricks_validate_missing_api_base() {
        let mut config = DatabricksConfig::with_api_key("dapi-test-key");
        config.base.api_base = None;
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("API base"));
    }

    #[test]
    fn test_databricks_validate_success() {
        let config = DatabricksConfig::with_credentials(
            "dapi-test-key",
            "https://adb-123.azuredatabricks.net",
        );
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_get_serving_endpoint_base() {
        let config =
            DatabricksConfig::with_credentials("test-key", "https://adb-123.azuredatabricks.net");
        let endpoint = config.get_serving_endpoint_base();
        assert_eq!(
            endpoint,
            Some("https://adb-123.azuredatabricks.net/serving-endpoints".to_string())
        );
    }

    #[test]
    fn test_get_serving_endpoint_base_already_has_path() {
        let config = DatabricksConfig::with_credentials(
            "test-key",
            "https://adb-123.azuredatabricks.net/serving-endpoints",
        );
        let endpoint = config.get_serving_endpoint_base();
        assert_eq!(
            endpoint,
            Some("https://adb-123.azuredatabricks.net/serving-endpoints".to_string())
        );
    }

    #[test]
    fn test_build_user_agent_default() {
        let agent = DatabricksConfig::build_user_agent(None);
        assert!(agent.starts_with("litellm-rs/"));
    }

    #[test]
    fn test_build_user_agent_with_partner() {
        let agent = DatabricksConfig::build_user_agent(Some("mycompany/1.0.0"));
        assert!(agent.starts_with("mycompany_litellm-rs/"));
    }

    #[test]
    fn test_build_user_agent_with_partner_no_version() {
        let agent = DatabricksConfig::build_user_agent(Some("acme"));
        assert!(agent.starts_with("acme_litellm-rs/"));
    }

    #[test]
    fn test_build_user_agent_invalid_partner() {
        let agent = DatabricksConfig::build_user_agent(Some("invalid partner!"));
        assert!(agent.starts_with("litellm-rs/"));
    }

    #[test]
    fn test_provider_config_trait() {
        let config = DatabricksConfig::with_credentials("test-key", "https://test.databricks.net");
        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://test.databricks.net"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(60));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_databricks_config_custom_timeout() {
        let mut config = DatabricksConfig::with_credentials("key", "https://test.databricks.net");
        config.base.timeout = 120;
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
    }
}
