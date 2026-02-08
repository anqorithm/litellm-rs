//! OCI Generative AI Provider Configuration
//!
//! Configuration for Oracle Cloud Infrastructure Generative AI access.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default OCI Generative AI API base URL pattern
pub const DEFAULT_API_BASE: &str = "https://inference.generativeai.{region}.oci.oraclecloud.com";

/// OCI provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OciConfig {
    /// Authentication token (Bearer token or OCI API key)
    pub auth_token: Option<String>,

    /// OCI compartment OCID
    pub compartment_id: Option<String>,

    /// OCI region (e.g., us-chicago-1, eu-frankfurt-1)
    pub region: Option<String>,

    /// API base URL (overrides region-based URL)
    pub api_base: Option<String>,

    /// User OCID for API key authentication
    pub user_ocid: Option<String>,

    /// Tenancy OCID for API key authentication
    pub tenancy_ocid: Option<String>,

    /// Fingerprint for API key authentication
    pub fingerprint: Option<String>,

    /// Private key path for API key authentication
    pub private_key_path: Option<String>,

    /// Private key content for API key authentication
    pub private_key: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,

    /// Maximum number of retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// Whether to enable debug logging
    #[serde(default)]
    pub debug: bool,
}

impl Default for OciConfig {
    fn default() -> Self {
        Self {
            auth_token: None,
            compartment_id: None,
            region: None,
            api_base: None,
            user_ocid: None,
            tenancy_ocid: None,
            fingerprint: None,
            private_key_path: None,
            private_key: None,
            timeout: default_timeout(),
            max_retries: default_max_retries(),
            debug: false,
        }
    }
}

impl ProviderConfig for OciConfig {
    fn validate(&self) -> Result<(), String> {
        // Check for auth token or API key credentials
        let has_token = self.auth_token.is_some() || std::env::var("OCI_AUTH_TOKEN").is_ok();
        let has_api_key = (self.user_ocid.is_some() || std::env::var("OCI_USER_OCID").is_ok())
            && (self.fingerprint.is_some() || std::env::var("OCI_FINGERPRINT").is_ok())
            && (self.private_key.is_some()
                || self.private_key_path.is_some()
                || std::env::var("OCI_PRIVATE_KEY").is_ok()
                || std::env::var("OCI_PRIVATE_KEY_PATH").is_ok());

        if !has_token && !has_api_key {
            return Err(
                "OCI authentication not configured. Set OCI_AUTH_TOKEN or configure API key \
                authentication with OCI_USER_OCID, OCI_FINGERPRINT, and OCI_PRIVATE_KEY."
                    .to_string(),
            );
        }

        // Check for compartment ID
        let has_compartment =
            self.compartment_id.is_some() || std::env::var("OCI_COMPARTMENT_ID").is_ok();
        if !has_compartment {
            return Err(
                "OCI compartment ID not configured. Set OCI_COMPARTMENT_ID environment variable \
                or pass compartment_id in configuration."
                    .to_string(),
            );
        }

        // Check for region or api_base
        let has_region = self.region.is_some() || std::env::var("OCI_REGION").is_ok();
        let has_api_base = self.api_base.is_some() || std::env::var("OCI_API_BASE").is_ok();

        if !has_region && !has_api_base {
            return Err(
                "OCI region or API base URL not configured. Set OCI_REGION or OCI_API_BASE \
                environment variable."
                    .to_string(),
            );
        }

        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        self.auth_token.as_deref()
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

impl OciConfig {
    /// Get auth token with environment variable fallback
    pub fn get_auth_token(&self) -> Option<String> {
        self.auth_token
            .clone()
            .or_else(|| std::env::var("OCI_AUTH_TOKEN").ok())
    }

    /// Get compartment ID with environment variable fallback
    pub fn get_compartment_id(&self) -> Option<String> {
        self.compartment_id
            .clone()
            .or_else(|| std::env::var("OCI_COMPARTMENT_ID").ok())
    }

    /// Get region with environment variable fallback
    pub fn get_region(&self) -> Option<String> {
        self.region
            .clone()
            .or_else(|| std::env::var("OCI_REGION").ok())
    }

    /// Get API base URL, building from region if not explicitly set
    pub fn get_api_base(&self) -> Option<String> {
        self.api_base
            .clone()
            .or_else(|| std::env::var("OCI_API_BASE").ok())
            .or_else(|| {
                self.get_region()
                    .map(|r| DEFAULT_API_BASE.replace("{region}", &r))
            })
    }

    /// Build the chat completions URL
    pub fn build_chat_url(&self) -> String {
        let base = self.get_api_base().unwrap_or_else(|| {
            "https://inference.generativeai.us-chicago-1.oci.oraclecloud.com".to_string()
        });
        format!("{}/20231130/actions/chat", base.trim_end_matches('/'))
    }

    /// Build URL for a specific model
    pub fn build_model_url(&self, _model: &str) -> String {
        self.build_chat_url()
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
    fn test_oci_config_default() {
        let config = OciConfig::default();
        assert!(config.auth_token.is_none());
        assert!(config.compartment_id.is_none());
        assert!(config.region.is_none());
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_oci_config_build_chat_url() {
        let config = OciConfig {
            api_base: Some(
                "https://inference.generativeai.us-chicago-1.oci.oraclecloud.com".to_string(),
            ),
            ..Default::default()
        };
        let url = config.build_chat_url();
        assert!(url.contains("generativeai"));
        assert!(url.contains("/actions/chat"));
    }

    #[test]
    fn test_oci_config_provider_config_trait() {
        let config = OciConfig {
            auth_token: Some("test-token".to_string()),
            api_base: Some("https://test.example.com".to_string()),
            timeout: 120,
            max_retries: 5,
            ..Default::default()
        };

        assert_eq!(config.api_key(), Some("test-token"));
        assert_eq!(config.api_base(), Some("https://test.example.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(120));
        assert_eq!(config.max_retries(), 5);
    }
}
