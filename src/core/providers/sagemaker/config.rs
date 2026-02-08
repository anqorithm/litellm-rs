//! AWS Sagemaker Provider Configuration
//!
//! Configuration for AWS Sagemaker endpoints including AWS credentials,
//! region settings, and endpoint-specific options.

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// AWS Sagemaker provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagemakerConfig {
    /// AWS access key ID
    pub aws_access_key_id: Option<String>,

    /// AWS secret access key
    pub aws_secret_access_key: Option<String>,

    /// AWS session token (optional, for temporary credentials)
    pub aws_session_token: Option<String>,

    /// AWS region (e.g., "us-west-2")
    pub aws_region: Option<String>,

    /// AWS profile name (optional, for profile-based credentials)
    pub aws_profile_name: Option<String>,

    /// Custom Sagemaker base URL (optional)
    pub sagemaker_base_url: Option<String>,

    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Maximum retries for failed requests
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,

    /// HuggingFace model name for prompt template
    pub hf_model_name: Option<String>,

    /// Whether to allow temperature of 0 (HF TGI requires strictly positive)
    #[serde(default)]
    pub allow_zero_temp: bool,
}

impl Default for SagemakerConfig {
    fn default() -> Self {
        Self {
            aws_access_key_id: None,
            aws_secret_access_key: None,
            aws_session_token: None,
            aws_region: None,
            aws_profile_name: None,
            sagemaker_base_url: None,
            timeout_seconds: default_timeout(),
            max_retries: default_max_retries(),
            hf_model_name: None,
            allow_zero_temp: false,
        }
    }
}

impl ProviderConfig for SagemakerConfig {
    fn validate(&self) -> Result<(), String> {
        // Check for AWS credentials
        let has_explicit_creds =
            self.aws_access_key_id.is_some() && self.aws_secret_access_key.is_some();
        let has_env_creds = std::env::var("AWS_ACCESS_KEY_ID").is_ok()
            && std::env::var("AWS_SECRET_ACCESS_KEY").is_ok();
        let has_profile = self.aws_profile_name.is_some();

        if !has_explicit_creds && !has_env_creds && !has_profile {
            return Err(
                "AWS credentials not provided. Set aws_access_key_id/aws_secret_access_key, \
                 AWS_ACCESS_KEY_ID/AWS_SECRET_ACCESS_KEY environment variables, or aws_profile_name"
                    .to_string(),
            );
        }

        // Validate region
        let region = self.get_region();
        if region.is_empty() {
            return Err(
                "AWS region not provided. Set aws_region or AWS_REGION environment variable"
                    .to_string(),
            );
        }

        // Validate timeout
        if self.timeout_seconds == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        if self.max_retries > 10 {
            return Err("Max retries should not exceed 10".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        // Sagemaker uses AWS credentials instead of API key
        None
    }

    fn api_base(&self) -> Option<&str> {
        self.sagemaker_base_url.as_deref()
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout_seconds)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl SagemakerConfig {
    /// Get AWS access key ID with environment variable fallback
    pub fn get_access_key_id(&self) -> Option<String> {
        self.aws_access_key_id
            .clone()
            .or_else(|| std::env::var("AWS_ACCESS_KEY_ID").ok())
    }

    /// Get AWS secret access key with environment variable fallback
    pub fn get_secret_access_key(&self) -> Option<String> {
        self.aws_secret_access_key
            .clone()
            .or_else(|| std::env::var("AWS_SECRET_ACCESS_KEY").ok())
    }

    /// Get AWS session token with environment variable fallback
    pub fn get_session_token(&self) -> Option<String> {
        self.aws_session_token
            .clone()
            .or_else(|| std::env::var("AWS_SESSION_TOKEN").ok())
    }

    /// Get AWS region with environment variable fallback
    pub fn get_region(&self) -> String {
        self.aws_region
            .clone()
            .or_else(|| std::env::var("AWS_REGION_NAME").ok())
            .or_else(|| std::env::var("AWS_REGION").ok())
            .unwrap_or_else(|| "us-west-2".to_string())
    }

    /// Check if temporary credentials are being used
    pub fn is_temporary_credentials(&self) -> bool {
        self.get_session_token().is_some()
    }

    /// Build the Sagemaker endpoint URL
    pub fn build_endpoint_url(&self, endpoint_name: &str, stream: bool) -> String {
        if let Some(base_url) = &self.sagemaker_base_url {
            return base_url.clone();
        }

        let region = self.get_region();
        if stream {
            format!(
                "https://runtime.sagemaker.{}.amazonaws.com/endpoints/{}/invocations-response-stream",
                region, endpoint_name
            )
        } else {
            format!(
                "https://runtime.sagemaker.{}.amazonaws.com/endpoints/{}/invocations",
                region, endpoint_name
            )
        }
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
    fn test_sagemaker_config_default() {
        let config = SagemakerConfig::default();
        assert!(config.aws_access_key_id.is_none());
        assert!(config.aws_secret_access_key.is_none());
        assert!(config.aws_session_token.is_none());
        assert!(config.aws_region.is_none());
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 3);
        assert!(!config.allow_zero_temp);
    }

    #[test]
    fn test_sagemaker_config_get_region_default() {
        let config = SagemakerConfig::default();
        // Will return default if no env vars are set
        let region = config.get_region();
        assert!(!region.is_empty());
    }

    #[test]
    fn test_sagemaker_config_get_region_custom() {
        let config = SagemakerConfig {
            aws_region: Some("eu-west-1".to_string()),
            ..Default::default()
        };
        assert_eq!(config.get_region(), "eu-west-1");
    }

    #[test]
    fn test_sagemaker_config_build_endpoint_url() {
        let config = SagemakerConfig {
            aws_region: Some("us-east-1".to_string()),
            ..Default::default()
        };

        let url = config.build_endpoint_url("my-endpoint", false);
        assert_eq!(
            url,
            "https://runtime.sagemaker.us-east-1.amazonaws.com/endpoints/my-endpoint/invocations"
        );

        let stream_url = config.build_endpoint_url("my-endpoint", true);
        assert_eq!(
            stream_url,
            "https://runtime.sagemaker.us-east-1.amazonaws.com/endpoints/my-endpoint/invocations-response-stream"
        );
    }

    #[test]
    fn test_sagemaker_config_build_endpoint_url_custom_base() {
        let config = SagemakerConfig {
            sagemaker_base_url: Some("https://custom.sagemaker.com/invoke".to_string()),
            ..Default::default()
        };

        let url = config.build_endpoint_url("my-endpoint", false);
        assert_eq!(url, "https://custom.sagemaker.com/invoke");
    }

    #[test]
    fn test_sagemaker_config_is_temporary_credentials() {
        let config = SagemakerConfig {
            aws_session_token: Some("token123".to_string()),
            ..Default::default()
        };
        assert!(config.is_temporary_credentials());

        let config_no_token = SagemakerConfig::default();
        assert!(!config_no_token.is_temporary_credentials());
    }

    #[test]
    fn test_sagemaker_config_validation_with_credentials() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            aws_region: Some("us-east-1".to_string()),
            ..Default::default()
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_sagemaker_config_validation_zero_timeout() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            aws_region: Some("us-east-1".to_string()),
            timeout_seconds: 0,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Timeout"));
    }

    #[test]
    fn test_sagemaker_config_validation_max_retries_too_high() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            aws_region: Some("us-east-1".to_string()),
            max_retries: 11,
            ..Default::default()
        };
        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("retries"));
    }

    #[test]
    fn test_sagemaker_config_provider_config_trait() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            sagemaker_base_url: Some("https://custom.com".to_string()),
            timeout_seconds: 90,
            max_retries: 5,
            ..Default::default()
        };

        assert!(config.api_key().is_none()); // Sagemaker doesn't use API key
        assert_eq!(config.api_base(), Some("https://custom.com"));
        assert_eq!(config.timeout(), std::time::Duration::from_secs(90));
        assert_eq!(config.max_retries(), 5);
    }

    #[test]
    fn test_sagemaker_config_clone() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            aws_region: Some("us-east-1".to_string()),
            ..Default::default()
        };

        let cloned = config.clone();
        assert_eq!(cloned.aws_access_key_id, config.aws_access_key_id);
        assert_eq!(cloned.aws_secret_access_key, config.aws_secret_access_key);
        assert_eq!(cloned.aws_region, config.aws_region);
    }

    #[test]
    fn test_sagemaker_config_serialization() {
        let config = SagemakerConfig {
            aws_access_key_id: Some("AKIATEST".to_string()),
            aws_secret_access_key: Some("secret".to_string()),
            aws_region: Some("us-west-2".to_string()),
            timeout_seconds: 45,
            max_retries: 2,
            allow_zero_temp: true,
            ..Default::default()
        };

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["aws_access_key_id"], "AKIATEST");
        assert_eq!(json["aws_region"], "us-west-2");
        assert_eq!(json["timeout_seconds"], 45);
        assert_eq!(json["allow_zero_temp"], true);
    }

    #[test]
    fn test_sagemaker_config_deserialization() {
        let json = r#"{
            "aws_access_key_id": "AKIATEST",
            "aws_secret_access_key": "secret",
            "aws_region": "eu-central-1",
            "timeout_seconds": 120
        }"#;

        let config: SagemakerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.aws_access_key_id, Some("AKIATEST".to_string()));
        assert_eq!(config.aws_region, Some("eu-central-1".to_string()));
        assert_eq!(config.timeout_seconds, 120);
    }
}
