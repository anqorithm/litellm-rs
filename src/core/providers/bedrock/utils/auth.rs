//! AWS Authentication for Bedrock
//!
//! Handles AWS credential management, session tokens,
//! and authentication-related utilities.

use crate::core::providers::unified_provider::ProviderError;
use std::collections::HashMap;
use std::env;

/// AWS authentication credentials
#[derive(Debug, Clone)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub region: String,
}

/// AWS authentication handler
#[derive(Debug, Clone)]
pub struct AwsAuth {
    credentials: AwsCredentials,
}

impl AwsAuth {
    /// Create new AWS auth with explicit credentials
    pub fn new(
        access_key_id: String,
        secret_access_key: String,
        session_token: Option<String>,
        region: String,
    ) -> Self {
        Self {
            credentials: AwsCredentials {
                access_key_id,
                secret_access_key,
                session_token,
                region,
            },
        }
    }

    /// Create AWS auth from environment variables
    pub fn from_env() -> Result<Self, ProviderError> {
        let access_key_id = env::var("AWS_ACCESS_KEY_ID").map_err(|_| {
            ProviderError::configuration(
                "bedrock",
                "AWS_ACCESS_KEY_ID environment variable not found".to_string(),
            )
        })?;

        let secret_access_key = env::var("AWS_SECRET_ACCESS_KEY").map_err(|_| {
            ProviderError::configuration(
                "bedrock",
                "AWS_SECRET_ACCESS_KEY environment variable not found".to_string(),
            )
        })?;

        let session_token = env::var("AWS_SESSION_TOKEN").ok();

        let region = env::var("AWS_REGION")
            .or_else(|_| env::var("AWS_DEFAULT_REGION"))
            .unwrap_or_else(|_| "us-east-1".to_string());

        Ok(Self::new(
            access_key_id,
            secret_access_key,
            session_token,
            region,
        ))
    }

    /// Get credentials reference
    pub fn credentials(&self) -> &AwsCredentials {
        &self.credentials
    }

    /// Validate credentials format
    pub fn validate(&self) -> Result<(), ProviderError> {
        if self.credentials.access_key_id.is_empty() {
            return Err(ProviderError::configuration(
                "bedrock",
                "AWS access key ID cannot be empty".to_string(),
            ));
        }

        if self.credentials.secret_access_key.is_empty() {
            return Err(ProviderError::configuration(
                "bedrock",
                "AWS secret access key cannot be empty".to_string(),
            ));
        }

        if self.credentials.region.is_empty() {
            return Err(ProviderError::configuration(
                "bedrock",
                "AWS region cannot be empty".to_string(),
            ));
        }

        // Basic format validation for access key
        if !self.credentials.access_key_id.starts_with("AKIA")
            && !self.credentials.access_key_id.starts_with("ASIA")
        {
            return Err(ProviderError::configuration(
                "bedrock",
                "Invalid AWS access key format".to_string(),
            ));
        }

        Ok(())
    }

    /// Get special auth parameter mappings for Bedrock
    pub fn get_mapped_auth_params(&self) -> HashMap<String, String> {
        let mut params = HashMap::new();
        params.insert(
            "aws_access_key_id".to_string(),
            self.credentials.access_key_id.clone(),
        );
        params.insert(
            "aws_secret_access_key".to_string(),
            self.credentials.secret_access_key.clone(),
        );
        params.insert(
            "aws_region_name".to_string(),
            self.credentials.region.clone(),
        );

        if let Some(ref token) = self.credentials.session_token {
            params.insert("aws_session_token".to_string(), token.clone());
        }

        params
    }

    /// Check if credentials are temporary (have session token)
    pub fn is_temporary_credentials(&self) -> bool {
        self.credentials.session_token.is_some()
    }

    /// Get credential type for logging/debugging
    pub fn credential_type(&self) -> &'static str {
        if self.credentials.access_key_id.starts_with("AKIA") {
            "long-term"
        } else if self.credentials.access_key_id.starts_with("ASIA") {
            "temporary"
        } else {
            "unknown"
        }
    }
}

/// Authentication configuration for special cases
#[derive(Debug, Clone, Default)]
#[cfg(test)]
pub struct BedrockAuthConfig {
    /// Enable cross-region access
    pub cross_region_access: bool,
    /// Custom endpoint URL
    pub custom_endpoint: Option<String>,
    /// Additional headers
    pub additional_headers: HashMap<String, String>,
}

/// Map special authentication parameters
#[cfg(test)]
pub fn map_special_auth_params(
    non_default_params: &HashMap<String, String>,
    optional_params: &mut HashMap<String, String>,
) {
    let mappings = [
        ("region_name", "aws_region_name"),
        ("aws_region", "aws_region_name"),
        ("access_key", "aws_access_key_id"),
        ("secret_key", "aws_secret_access_key"),
        ("session_token", "aws_session_token"),
    ];

    for (from_param, to_param) in &mappings {
        if let Some(value) = non_default_params.get(*from_param) {
            optional_params.insert(to_param.to_string(), value.clone());
        }
    }
}

/// Extract AWS credentials from various parameter formats
#[cfg(test)]
pub fn extract_credentials_from_params(
    params: &HashMap<String, String>,
) -> Result<AwsCredentials, ProviderError> {
    let access_key_id = params
        .get("aws_access_key_id")
        .or_else(|| params.get("access_key"))
        .ok_or_else(|| {
            ProviderError::configuration(
                "bedrock",
                "AWS access key ID not found in parameters".to_string(),
            )
        })?
        .clone();

    let secret_access_key = params
        .get("aws_secret_access_key")
        .or_else(|| params.get("secret_key"))
        .ok_or_else(|| {
            ProviderError::configuration(
                "bedrock",
                "AWS secret access key not found in parameters".to_string(),
            )
        })?
        .clone();

    let session_token = params
        .get("aws_session_token")
        .or_else(|| params.get("session_token"))
        .cloned();

    let region = params
        .get("aws_region_name")
        .or_else(|| params.get("region"))
        .or_else(|| params.get("aws_region"))
        .cloned()
        .unwrap_or_else(|| "us-east-1".to_string());

    Ok(AwsCredentials {
        access_key_id,
        secret_access_key,
        session_token,
        region,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aws_auth_creation() {
        let auth = AwsAuth::new(
            "AKIATEST123456789012".to_string(),
            "test-secret-key".to_string(),
            None,
            "us-east-1".to_string(),
        );

        assert_eq!(auth.credentials().access_key_id, "AKIATEST123456789012");
        assert_eq!(auth.credentials().region, "us-east-1");
        assert!(!auth.is_temporary_credentials());
        assert_eq!(auth.credential_type(), "long-term");
    }

    #[test]
    fn test_temporary_credentials() {
        let auth = AwsAuth::new(
            "ASIATEST123456789012".to_string(),
            "test-secret-key".to_string(),
            Some("session-token".to_string()),
            "us-west-2".to_string(),
        );

        assert!(auth.is_temporary_credentials());
        assert_eq!(auth.credential_type(), "temporary");
    }

    #[test]
    fn test_credential_validation() {
        let auth = AwsAuth::new(
            "AKIATEST123456789012".to_string(),
            "test-secret-key".to_string(),
            None,
            "us-east-1".to_string(),
        );

        assert!(auth.validate().is_ok());

        // Test invalid access key format
        let invalid_auth = AwsAuth::new(
            "INVALID_KEY".to_string(),
            "test-secret-key".to_string(),
            None,
            "us-east-1".to_string(),
        );

        assert!(invalid_auth.validate().is_err());
    }

    #[test]
    fn test_mapped_auth_params() {
        let auth = AwsAuth::new(
            "AKIATEST123456789012".to_string(),
            "test-secret-key".to_string(),
            Some("session-token".to_string()),
            "us-east-1".to_string(),
        );

        let params = auth.get_mapped_auth_params();
        assert_eq!(
            params.get("aws_access_key_id").unwrap(),
            "AKIATEST123456789012"
        );
        assert_eq!(
            params.get("aws_secret_access_key").unwrap(),
            "test-secret-key"
        );
        assert_eq!(params.get("aws_session_token").unwrap(), "session-token");
        assert_eq!(params.get("aws_region_name").unwrap(), "us-east-1");
    }

    #[test]
    fn test_extract_credentials_from_params() {
        let mut params = HashMap::new();
        params.insert(
            "aws_access_key_id".to_string(),
            "AKIATEST123456789012".to_string(),
        );
        params.insert(
            "aws_secret_access_key".to_string(),
            "test-secret-key".to_string(),
        );
        params.insert("aws_region_name".to_string(), "us-west-2".to_string());

        let credentials = extract_credentials_from_params(&params).unwrap();
        assert_eq!(credentials.access_key_id, "AKIATEST123456789012");
        assert_eq!(credentials.secret_access_key, "test-secret-key");
        assert_eq!(credentials.region, "us-west-2");
        assert!(credentials.session_token.is_none());
    }
}
