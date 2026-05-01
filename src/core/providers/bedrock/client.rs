//! Bedrock HTTP Client
//!
//! Wrapper around base HTTP client with Bedrock-specific functionality
//! including AWS SigV4 signing and request routing.

use reqwest::{Client, Response};
use serde_json::Value;
use std::collections::HashMap;
use tracing::{debug, error};

use super::config::BedrockConfig;
use super::error::BedrockErrorMapper;
use super::sigv4::SigV4Signer;
use super::utils::{AwsAuth, validate_region};
use crate::core::providers::base::{BaseConfig, BaseHttpClient};
use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Bedrock HTTP client wrapper
#[derive(Debug, Clone)]
pub struct BedrockClient {
    base_client: BaseHttpClient,
    auth: AwsAuth,
    signer: SigV4Signer,
    error_mapper: BedrockErrorMapper,
}

impl BedrockClient {
    /// Create a new Bedrock client
    pub fn new(config: BedrockConfig) -> Result<Self, ProviderError> {
        // Validate region
        validate_region(&config.aws_region)?;

        // Create base HTTP client
        let base_config = BaseConfig {
            api_key: None,  // Bedrock uses AWS credentials
            api_base: None, // Dynamic based on region and model
            timeout: config.timeout_seconds,
            max_retries: config.max_retries,
            headers: HashMap::new(),
            organization: None,
            api_version: None,
        };

        let base_client = BaseHttpClient::new(base_config)
            .map_err(|e| ProviderError::configuration("bedrock", e.to_string()))?;

        // Create AWS auth
        let auth = AwsAuth::new(
            config.aws_access_key_id.clone(),
            config.aws_secret_access_key.clone(),
            config.aws_session_token.clone(),
            config.aws_region.clone(),
        );

        // Validate auth
        auth.validate()?;

        // Create SigV4 signer
        let signer = SigV4Signer::new(
            config.aws_access_key_id,
            config.aws_secret_access_key,
            config.aws_session_token,
            config.aws_region,
        );

        Ok(Self {
            base_client,
            auth,
            signer,
            error_mapper: BedrockErrorMapper,
        })
    }

    /// Get the underlying HTTP client
    pub fn inner(&self) -> &Client {
        self.base_client.inner()
    }

    /// Get AWS auth reference
    pub fn auth(&self) -> &AwsAuth {
        &self.auth
    }

    /// Build Bedrock API URL for a model and operation
    pub fn build_url(&self, model_id: &str, operation: &str) -> String {
        let region = &self.auth.credentials().region;

        // Different URL patterns for different operations
        match operation {
            "invoke" => {
                format!(
                    "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke",
                    region, model_id
                )
            }
            "invoke-with-response-stream" => {
                format!(
                    "https://bedrock-runtime.{}.amazonaws.com/model/{}/invoke-with-response-stream",
                    region, model_id
                )
            }
            "converse" => {
                format!(
                    "https://bedrock-runtime.{}.amazonaws.com/model/{}/converse",
                    region, model_id
                )
            }
            "converse-stream" => {
                format!(
                    "https://bedrock-runtime.{}.amazonaws.com/model/{}/converse-stream",
                    region, model_id
                )
            }
            "list-foundation-models" => {
                format!("https://bedrock.{}.amazonaws.com/foundation-models", region)
            }
            _ => {
                format!(
                    "https://bedrock-runtime.{}.amazonaws.com/model/{}/{}",
                    region, model_id, operation
                )
            }
        }
    }

    /// Create signed headers for AWS SigV4
    pub async fn create_signed_headers(
        &self,
        url: &str,
        body: &str,
        method: &str,
    ) -> Result<reqwest::header::HeaderMap, ProviderError> {
        let timestamp = chrono::Utc::now();
        let headers = HashMap::new(); // Start with empty headers

        let signed_headers = self
            .signer
            .sign_request(method, url, &headers, body, timestamp)
            .map_err(|e| {
                ProviderError::configuration("bedrock", format!("Signing failed: {}", e))
            })?;

        // Convert to reqwest HeaderMap
        let mut header_map = reqwest::header::HeaderMap::new();
        for (key, value) in signed_headers {
            if let (Ok(header_name), Ok(header_value)) = (
                reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                reqwest::header::HeaderValue::from_str(&value),
            ) {
                header_map.insert(header_name, header_value);
            }
        }

        Ok(header_map)
    }

    /// Send a request to Bedrock API
    pub async fn send_request(
        &self,
        model_id: &str,
        operation: &str,
        body: &Value,
    ) -> Result<Response, ProviderError> {
        let url = self.build_url(model_id, operation);
        let body_str = serde_json::to_string(body)
            .map_err(|e| ProviderError::serialization("bedrock", e.to_string()))?;

        debug!(
            operation,
            url,
            body_bytes = body_str.len(),
            "Bedrock request prepared"
        );

        // Create signed headers
        let headers = self.create_signed_headers(&url, &body_str, "POST").await?;

        // Send request
        let response = self
            .inner()
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await
            .map_err(|e| self.error_mapper.map_network_error(&e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error_body_bytes = error_body.len();
            error!(status, body_bytes = error_body_bytes, "Bedrock API error");
            return Err(self.error_mapper.map_http_error(status, &error_body));
        }

        Ok(response)
    }

    /// Send a streaming request to Bedrock API
    pub async fn send_streaming_request(
        &self,
        model_id: &str,
        operation: &str,
        body: &Value,
    ) -> Result<Response, ProviderError> {
        let url = self.build_url(model_id, operation);
        let body_str = serde_json::to_string(body)
            .map_err(|e| ProviderError::serialization("bedrock", e.to_string()))?;

        debug!("Bedrock streaming request to {}", url);

        // Create signed headers
        let headers = self.create_signed_headers(&url, &body_str, "POST").await?;

        // Send streaming request
        let response = self
            .inner()
            .post(&url)
            .headers(headers)
            .body(body_str)
            .send()
            .await
            .map_err(|e| self.error_mapper.map_network_error(&e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error_body_bytes = error_body.len();
            error!(
                status,
                body_bytes = error_body_bytes,
                "Bedrock streaming API error"
            );
            return Err(self.error_mapper.map_http_error(status, &error_body));
        }

        Ok(response)
    }

    /// Send a GET request (for operations like listing models)
    pub async fn send_get_request(&self, operation: &str) -> Result<Response, ProviderError> {
        let url = self.build_url("", operation); // Empty model_id for non-model operations
        let body = ""; // Empty body for GET

        debug!("Bedrock GET request to {}", url);

        // Create signed headers
        let headers = self.create_signed_headers(&url, body, "GET").await?;

        // Send GET request
        let response = self
            .inner()
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| self.error_mapper.map_network_error(&e))?;

        // Check for errors
        if !response.status().is_success() {
            let status = response.status().as_u16();
            let error_body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            let error_body_bytes = error_body.len();
            error!(
                status,
                body_bytes = error_body_bytes,
                "Bedrock GET API error"
            );
            return Err(self.error_mapper.map_http_error(status, &error_body));
        }

        Ok(response)
    }

    /// Health check by listing foundation models
    pub async fn health_check(&self) -> Result<bool, ProviderError> {
        match self.send_get_request("list-foundation-models").await {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_config() -> BedrockConfig {
        BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key-1234567890".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }

    fn create_test_client() -> BedrockClient {
        BedrockClient::new(create_test_config()).unwrap()
    }

    // ==================== Client Creation Tests ====================

    #[tokio::test]
    async fn test_client_creation() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let client = BedrockClient::new(config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.auth().credentials().region, "us-east-1");
        assert!(!client.auth().is_temporary_credentials());
    }

    #[test]
    fn test_client_creation_with_session_token() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: Some("session-token-12345".to_string()),
            aws_region: "us-west-2".to_string(),
            timeout_seconds: 60,
            max_retries: 5,
        };

        let client = BedrockClient::new(config);
        assert!(client.is_ok());

        let client = client.unwrap();
        assert!(client.auth().is_temporary_credentials());
    }

    #[test]
    fn test_invalid_region() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "invalid-region".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let client = BedrockClient::new(config);
        assert!(client.is_err());
    }

    #[test]
    fn test_empty_access_key() {
        let config = BedrockConfig {
            aws_access_key_id: "".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let client = BedrockClient::new(config);
        assert!(client.is_err());
    }

    #[test]
    fn test_empty_secret_key() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let client = BedrockClient::new(config);
        assert!(client.is_err());
    }

    // ==================== URL Building Tests ====================

    #[test]
    fn test_url_building() {
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "us-east-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };

        let client = BedrockClient::new(config).unwrap();

        // Test invoke URL
        let url = client.build_url("anthropic.claude-3-opus-20240229", "invoke");
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-opus-20240229/invoke"
        );

        // Test streaming URL
        let url = client.build_url(
            "amazon.titan-text-express-v1",
            "invoke-with-response-stream",
        );
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/amazon.titan-text-express-v1/invoke-with-response-stream"
        );

        // Test converse URL
        let url = client.build_url("anthropic.claude-3-sonnet-20240229", "converse");
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-sonnet-20240229/converse"
        );
    }

    #[test]
    fn test_url_building_converse_stream() {
        let client = create_test_client();

        let url = client.build_url("anthropic.claude-3-haiku-20240307", "converse-stream");
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/anthropic.claude-3-haiku-20240307/converse-stream"
        );
    }

    #[test]
    fn test_url_building_list_foundation_models() {
        let client = create_test_client();

        let url = client.build_url("", "list-foundation-models");
        assert_eq!(
            url,
            "https://bedrock.us-east-1.amazonaws.com/foundation-models"
        );
    }

    #[test]
    fn test_url_building_custom_operation() {
        let client = create_test_client();

        let url = client.build_url("some-model", "custom-operation");
        assert_eq!(
            url,
            "https://bedrock-runtime.us-east-1.amazonaws.com/model/some-model/custom-operation"
        );
    }

    #[test]
    fn test_url_building_different_regions() {
        // us-west-2
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "us-west-2".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };
        let client = BedrockClient::new(config).unwrap();
        let url = client.build_url("anthropic.claude-3-opus-20240229", "invoke");
        assert!(url.contains("us-west-2"));

        // eu-west-1
        let config = BedrockConfig {
            aws_access_key_id: "AKIATEST123456789012".to_string(),
            aws_secret_access_key: "test-secret-key".to_string(),
            aws_session_token: None,
            aws_region: "eu-west-1".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };
        let client = BedrockClient::new(config).unwrap();
        let url = client.build_url("anthropic.claude-3-opus-20240229", "invoke");
        assert!(url.contains("eu-west-1"));
    }

    // ==================== Auth Access Tests ====================

    #[test]
    fn test_auth_access() {
        let client = create_test_client();

        let auth = client.auth();
        assert_eq!(auth.credentials().region, "us-east-1");
        assert_eq!(auth.credentials().access_key_id, "AKIATEST123456789012");
    }

    #[test]
    fn test_inner_client_access() {
        let client = create_test_client();

        let _inner = client.inner();
        // Just verify we can access the inner client
    }

    // ==================== Signed Headers Tests ====================

    #[tokio::test]
    async fn test_create_signed_headers() {
        let client = create_test_client();

        let headers = client
            .create_signed_headers(
                "https://bedrock-runtime.us-east-1.amazonaws.com/model/test/invoke",
                r#"{"test": "body"}"#,
                "POST",
            )
            .await;

        assert!(headers.is_ok());
        let headers = headers.unwrap();

        // Should have authorization header
        assert!(headers.contains_key("authorization"));
        // Should have x-amz-date header
        assert!(headers.contains_key("x-amz-date"));
        // Should have host header
        assert!(headers.contains_key("host"));
    }

    #[tokio::test]
    async fn test_create_signed_headers_get() {
        let client = create_test_client();

        let headers = client
            .create_signed_headers(
                "https://bedrock.us-east-1.amazonaws.com/foundation-models",
                "",
                "GET",
            )
            .await;

        assert!(headers.is_ok());
    }

    // ==================== Clone/Debug Tests ====================

    #[test]
    fn test_client_clone() {
        let client = create_test_client();
        let cloned = client.clone();

        assert_eq!(
            client.auth().credentials().region,
            cloned.auth().credentials().region
        );
        assert_eq!(
            client.auth().credentials().access_key_id,
            cloned.auth().credentials().access_key_id
        );
    }

    #[test]
    fn test_client_debug() {
        let client = create_test_client();
        let debug_str = format!("{:?}", client);

        assert!(debug_str.contains("BedrockClient"));
    }

    // ==================== Multiple Region Tests ====================

    #[test]
    fn test_supported_regions() {
        let regions = vec![
            "us-east-1",
            "us-west-2",
            "eu-west-1",
            "eu-central-1",
            "ap-northeast-1",
            "ap-southeast-1",
        ];

        for region in regions {
            let config = BedrockConfig {
                aws_access_key_id: "AKIATEST123456789012".to_string(),
                aws_secret_access_key: "test-secret-key".to_string(),
                aws_session_token: None,
                aws_region: region.to_string(),
                timeout_seconds: 30,
                max_retries: 3,
            };

            let client = BedrockClient::new(config);
            assert!(client.is_ok(), "Region {} should be supported", region);
        }
    }

    // ==================== URL Building Edge Cases ====================

    #[test]
    fn test_url_building_special_model_ids() {
        let client = create_test_client();

        // Model with version suffix
        let url = client.build_url("meta.llama3-70b-instruct-v1:0", "invoke");
        assert!(url.contains("meta.llama3-70b-instruct-v1:0"));

        // Model with dots
        let url = client.build_url("ai21.jamba-1-5-large-v1:0", "invoke");
        assert!(url.contains("ai21.jamba-1-5-large-v1:0"));
    }
}
