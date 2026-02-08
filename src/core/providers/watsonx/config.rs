//! Watsonx Provider Configuration
//!
//! Configuration for IBM Watsonx API access including authentication and project settings.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default Watsonx API version
pub const DEFAULT_API_VERSION: &str = "2024-05-31";

/// Default IAM token URL
pub const DEFAULT_IAM_URL: &str = "https://iam.cloud.ibm.com/identity/token";

/// Watsonx provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatsonxConfig {
    /// API key for Watsonx authentication
    pub api_key: Option<String>,

    /// API base URL (e.g., <https://us-south.ml.cloud.ibm.com>)
    pub api_base: Option<String>,

    /// Project ID for Watsonx.ai
    pub project_id: Option<String>,

    /// Deployment space ID (alternative to project_id)
    pub space_id: Option<String>,

    /// Region name (e.g., us-south, eu-de)
    pub region: Option<String>,

    /// API version to use
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// IAM token URL for authentication
    #[serde(default = "default_iam_url")]
    pub iam_url: String,

    /// Pre-generated token (optional, alternative to api_key)
    pub token: Option<String>,

    /// Zen API key for on-premise deployments
    pub zen_api_key: Option<String>,

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

impl Default for WatsonxConfig {
    fn default() -> Self {
        Self {
            api_key: None,
            api_base: None,
            project_id: None,
            space_id: None,
            region: None,
            api_version: default_api_version(),
            iam_url: default_iam_url(),
            token: None,
            zen_api_key: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for WatsonxConfig {
    fn validate(&self) -> Result<(), String> {
        // Check for API key or token
        let has_api_key = self.api_key.is_some() || std::env::var("WATSONX_API_KEY").is_ok();
        let has_token = self.token.is_some() || std::env::var("WATSONX_TOKEN").is_ok();
        let has_zen_key = self.zen_api_key.is_some() || std::env::var("WATSONX_ZENAPIKEY").is_ok();

        if !has_api_key && !has_token && !has_zen_key {
            return Err("Watsonx API key, token, or Zen API key not provided. \
                Set WATSONX_API_KEY, WATSONX_TOKEN, or WATSONX_ZENAPIKEY environment variable \
                or pass in configuration."
                .to_string());
        }

        // Check for project ID
        let has_project = self.project_id.is_some() || std::env::var("WATSONX_PROJECT_ID").is_ok();
        let has_space = self.space_id.is_some() || std::env::var("WATSONX_SPACE_ID").is_ok();

        if !has_project && !has_space {
            return Err("Watsonx project_id or space_id not provided. \
                Set WATSONX_PROJECT_ID or WATSONX_SPACE_ID environment variable \
                or pass in configuration."
                .to_string());
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

impl WatsonxConfig {
    /// Get API key with environment variable fallback
    pub fn get_api_key(&self) -> Option<String> {
        self.api_key.clone().or_else(|| {
            std::env::var("WATSONX_API_KEY")
                .ok()
                .or_else(|| std::env::var("WX_API_KEY").ok())
                .or_else(|| std::env::var("WATSONX_APIKEY").ok())
        })
    }

    /// Get API base with environment variable fallback
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base.clone().or_else(|| {
            std::env::var("WATSONX_API_BASE")
                .ok()
                .or_else(|| std::env::var("WATSONX_URL").ok())
                .or_else(|| std::env::var("WX_URL").ok())
                .or_else(|| std::env::var("WML_URL").ok())
        })
    }

    /// Get project ID with environment variable fallback
    pub fn get_project_id(&self) -> Option<String> {
        self.project_id.clone().or_else(|| {
            std::env::var("WATSONX_PROJECT_ID")
                .ok()
                .or_else(|| std::env::var("WX_PROJECT_ID").ok())
                .or_else(|| std::env::var("PROJECT_ID").ok())
        })
    }

    /// Get space ID with environment variable fallback
    pub fn get_space_id(&self) -> Option<String> {
        self.space_id.clone().or_else(|| {
            std::env::var("WATSONX_SPACE_ID")
                .ok()
                .or_else(|| std::env::var("WX_SPACE_ID").ok())
                .or_else(|| std::env::var("SPACE_ID").ok())
        })
    }

    /// Get region with environment variable fallback
    pub fn get_region(&self) -> Option<String> {
        self.region.clone().or_else(|| {
            std::env::var("WATSONX_REGION")
                .ok()
                .or_else(|| std::env::var("WX_REGION").ok())
        })
    }

    /// Get token with environment variable fallback
    pub fn get_token(&self) -> Option<String> {
        self.token
            .clone()
            .or_else(|| std::env::var("WATSONX_TOKEN").ok())
    }

    /// Get Zen API key with environment variable fallback
    pub fn get_zen_api_key(&self) -> Option<String> {
        self.zen_api_key
            .clone()
            .or_else(|| std::env::var("WATSONX_ZENAPIKEY").ok())
    }

    /// Get IAM URL with environment variable fallback
    pub fn get_iam_url(&self) -> String {
        std::env::var("WATSONX_IAM_URL").unwrap_or_else(|_| self.iam_url.clone())
    }

    /// Build the complete API URL for a given endpoint
    pub fn build_url(&self, endpoint: &str, _stream: bool) -> Result<String, String> {
        let base_url = self.get_api_base().ok_or_else(|| {
            "Watsonx API base URL not set. Set WATSONX_API_BASE environment variable \
            or pass api_base in configuration."
                .to_string()
        })?;

        let base_url = base_url.trim_end_matches('/');

        // Build endpoint URL
        let full_url = format!("{}{}", base_url, endpoint);

        // Add API version
        let url_with_version = format!("{}?version={}", full_url, self.api_version);

        Ok(url_with_version)
    }
}

fn default_api_version() -> String {
    DEFAULT_API_VERSION.to_string()
}

fn default_iam_url() -> String {
    DEFAULT_IAM_URL.to_string()
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
    fn test_watsonx_config_default() {
        let config = WatsonxConfig::default();
        assert!(config.api_key.is_none());
        assert!(config.api_base.is_none());
        assert!(config.project_id.is_none());
        assert_eq!(config.api_version, DEFAULT_API_VERSION);
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
        assert!(!config.debug);
    }

    #[test]
    fn test_watsonx_config_get_api_base() {
        let config = WatsonxConfig {
            api_base: Some("https://us-south.ml.cloud.ibm.com".to_string()),
            ..Default::default()
        };
        assert_eq!(
            config.get_api_base(),
            Some("https://us-south.ml.cloud.ibm.com".to_string())
        );
    }

    #[test]
    fn test_watsonx_config_get_api_key() {
        let config = WatsonxConfig {
            api_key: Some("test-key".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_api_key(), Some("test-key".to_string()));
    }

    #[test]
    fn test_watsonx_config_get_project_id() {
        let config = WatsonxConfig {
            project_id: Some("test-project".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_project_id(), Some("test-project".to_string()));
    }

    #[test]
    fn test_watsonx_config_provider_config_trait() {
        let config = WatsonxConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://us-south.ml.cloud.ibm.com".to_string()),
            timeout: 120,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-key"));
        assert_eq!(config.api_base(), Some("https://us-south.ml.cloud.ibm.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_watsonx_config_build_url() {
        let config = WatsonxConfig {
            api_base: Some("https://us-south.ml.cloud.ibm.com".to_string()),
            api_version: "2024-05-31".to_string(),
            ..Default::default()
        };

        let url = config.build_url("/ml/v1/text/chat", false).unwrap();
        assert!(url.contains("https://us-south.ml.cloud.ibm.com"));
        assert!(url.contains("/ml/v1/text/chat"));
        assert!(url.contains("version=2024-05-31"));
    }

    #[test]
    fn test_watsonx_config_serialization() {
        let config = WatsonxConfig {
            api_key: Some("test-key".to_string()),
            api_base: Some("https://us-south.ml.cloud.ibm.com".to_string()),
            project_id: Some("project-123".to_string()),
            timeout: 90,
            max_retries: 2,
            debug: true,
            ..Default::default()
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["api_key"], "test-key");
        assert_eq!(json["api_base"], "https://us-south.ml.cloud.ibm.com");
        assert_eq!(json["project_id"], "project-123");
        assert_eq!(json["timeout"], 90);
        assert_eq!(json["max_retries"], 2);
        assert_eq!(json["debug"], true);
    }

    #[test]
    fn test_watsonx_config_deserialization() {
        let json = r#"{
            "api_key": "test-key",
            "project_id": "project-123",
            "timeout": 120,
            "debug": true
        }"#;

        let config: WatsonxConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.api_key, Some("test-key".to_string()));
        assert_eq!(config.project_id, Some("project-123".to_string()));
        assert_eq!(config.timeout, 120);
        assert!(config.debug);
    }
}
