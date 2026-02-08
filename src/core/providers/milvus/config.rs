//! Milvus Provider Configuration
//!
//! Configuration for Milvus vector database access including connection settings
//! and authentication options.
//!
//! Milvus is an open-source vector database designed for AI applications,
//! providing high-performance similarity search and vector storage.
//!
//! Reference: <https://milvus.io/docs/restful_api.md>

use crate::core::traits::provider::ProviderConfig;
use serde::{Deserialize, Serialize};

/// Default Milvus REST API port
const DEFAULT_PORT: u16 = 19530;

/// Default request timeout in seconds
const DEFAULT_TIMEOUT: u64 = 60;

/// Default maximum retries
const DEFAULT_MAX_RETRIES: u32 = 3;

/// Milvus provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilvusConfig {
    /// Milvus server host (required)
    /// Examples: "localhost", "milvus.example.com", "192.168.1.100"
    pub host: String,

    /// Milvus server port (default: 19530)
    #[serde(default = "default_port")]
    pub port: u16,

    /// Default collection name for operations (optional)
    /// If not specified, must be provided per-request
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection_name: Option<String>,

    /// Database name (optional, for multi-tenancy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub database: Option<String>,

    /// Username for authentication (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password for authentication (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// API token for authentication (alternative to username/password)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,

    /// Use HTTPS instead of HTTP
    #[serde(default)]
    pub use_https: bool,

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

fn default_port() -> u16 {
    DEFAULT_PORT
}

fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT
}

fn default_max_retries() -> u32 {
    DEFAULT_MAX_RETRIES
}

impl Default for MilvusConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: DEFAULT_PORT,
            collection_name: None,
            database: None,
            username: None,
            password: None,
            token: None,
            use_https: false,
            timeout: DEFAULT_TIMEOUT,
            max_retries: DEFAULT_MAX_RETRIES,
            debug: false,
        }
    }
}

impl ProviderConfig for MilvusConfig {
    fn validate(&self) -> Result<(), String> {
        // Host is required and must not be empty
        if self.host.is_empty() {
            return Err("Milvus host is required".to_string());
        }

        // Validate port range
        if self.port == 0 {
            return Err("Milvus port must be greater than 0".to_string());
        }

        // Validate timeout
        if self.timeout == 0 {
            return Err("Timeout must be greater than 0".to_string());
        }

        // If username is provided, password should also be provided (and vice versa)
        if self.username.is_some() != self.password.is_some() {
            return Err("Both username and password must be provided together".to_string());
        }

        Ok(())
    }

    fn api_key(&self) -> Option<&str> {
        // Milvus uses token-based auth, return token as api_key
        self.token.as_deref()
    }

    fn api_base(&self) -> Option<&str> {
        // Return None since we construct the URL from host/port
        None
    }

    fn timeout(&self) -> std::time::Duration {
        std::time::Duration::from_secs(self.timeout)
    }

    fn max_retries(&self) -> u32 {
        self.max_retries
    }
}

impl MilvusConfig {
    /// Create a new MilvusConfig with the specified host
    pub fn new(host: impl Into<String>) -> Self {
        Self {
            host: host.into(),
            ..Default::default()
        }
    }

    /// Create config with host and port
    pub fn with_host_port(host: impl Into<String>, port: u16) -> Self {
        Self {
            host: host.into(),
            port,
            ..Default::default()
        }
    }

    /// Set the collection name
    pub fn with_collection(mut self, collection: impl Into<String>) -> Self {
        self.collection_name = Some(collection.into());
        self
    }

    /// Set the database name
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    /// Set username and password authentication
    pub fn with_credentials(
        mut self,
        username: impl Into<String>,
        password: impl Into<String>,
    ) -> Self {
        self.username = Some(username.into());
        self.password = Some(password.into());
        self
    }

    /// Set token-based authentication
    pub fn with_token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Enable HTTPS
    pub fn with_https(mut self) -> Self {
        self.use_https = true;
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout = timeout_secs;
        self
    }

    /// Get the base URL for the Milvus REST API
    pub fn get_api_base(&self) -> String {
        let scheme = if self.use_https { "https" } else { "http" };
        format!("{}://{}:{}", scheme, self.host, self.port)
    }

    /// Get the full URL for a specific endpoint
    pub fn get_endpoint_url(&self, endpoint: &str) -> String {
        let base = self.get_api_base();
        if endpoint.starts_with('/') {
            format!("{}{}", base, endpoint)
        } else {
            format!("{}/{}", base, endpoint)
        }
    }

    /// Get authentication headers
    pub fn get_auth_headers(&self) -> Vec<(String, String)> {
        let mut headers = Vec::new();

        // Token-based auth takes precedence
        if let Some(ref token) = self.token {
            headers.push(("Authorization".to_string(), format!("Bearer {}", token)));
        } else if let (Some(username), Some(password)) = (&self.username, &self.password) {
            // Basic auth
            let credentials = base64_encode(&format!("{}:{}", username, password));
            headers.push((
                "Authorization".to_string(),
                format!("Basic {}", credentials),
            ));
        }

        headers
    }

    /// Get the effective collection name
    pub fn get_collection_name(&self) -> Option<&str> {
        self.collection_name.as_deref()
    }

    /// Create config from environment variables
    pub fn from_env() -> Self {
        Self {
            host: std::env::var("MILVUS_HOST").unwrap_or_else(|_| "localhost".to_string()),
            port: std::env::var("MILVUS_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(DEFAULT_PORT),
            collection_name: std::env::var("MILVUS_COLLECTION").ok(),
            database: std::env::var("MILVUS_DATABASE").ok(),
            username: std::env::var("MILVUS_USERNAME").ok(),
            password: std::env::var("MILVUS_PASSWORD").ok(),
            token: std::env::var("MILVUS_TOKEN")
                .or_else(|_| std::env::var("MILVUS_API_KEY"))
                .ok(),
            use_https: std::env::var("MILVUS_USE_HTTPS")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
            timeout: std::env::var("MILVUS_TIMEOUT")
                .ok()
                .and_then(|t| t.parse().ok())
                .unwrap_or(DEFAULT_TIMEOUT),
            max_retries: std::env::var("MILVUS_MAX_RETRIES")
                .ok()
                .and_then(|r| r.parse().ok())
                .unwrap_or(DEFAULT_MAX_RETRIES),
            debug: std::env::var("MILVUS_DEBUG")
                .map(|v| v.to_lowercase() == "true" || v == "1")
                .unwrap_or(false),
        }
    }
}

/// Simple base64 encoding for basic auth
fn base64_encode(input: &str) -> String {
    use base64::{Engine, engine::general_purpose::STANDARD};
    STANDARD.encode(input.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milvus_config_default() {
        let config = MilvusConfig::default();
        assert_eq!(config.host, "localhost");
        assert_eq!(config.port, 19530);
        assert!(config.collection_name.is_none());
        assert!(config.username.is_none());
        assert!(config.password.is_none());
        assert!(config.token.is_none());
        assert!(!config.use_https);
        assert_eq!(config.timeout, 60);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_milvus_config_new() {
        let config = MilvusConfig::new("milvus.example.com");
        assert_eq!(config.host, "milvus.example.com");
        assert_eq!(config.port, 19530);
    }

    #[test]
    fn test_milvus_config_with_host_port() {
        let config = MilvusConfig::with_host_port("milvus.example.com", 19531);
        assert_eq!(config.host, "milvus.example.com");
        assert_eq!(config.port, 19531);
    }

    #[test]
    fn test_milvus_config_builder_pattern() {
        let config = MilvusConfig::new("milvus.example.com")
            .with_collection("my_collection")
            .with_database("my_database")
            .with_credentials("user", "pass")
            .with_https()
            .with_timeout(120);

        assert_eq!(config.host, "milvus.example.com");
        assert_eq!(config.collection_name, Some("my_collection".to_string()));
        assert_eq!(config.database, Some("my_database".to_string()));
        assert_eq!(config.username, Some("user".to_string()));
        assert_eq!(config.password, Some("pass".to_string()));
        assert!(config.use_https);
        assert_eq!(config.timeout, 120);
    }

    #[test]
    fn test_milvus_config_get_api_base() {
        let config = MilvusConfig::new("milvus.example.com");
        assert_eq!(config.get_api_base(), "http://milvus.example.com:19530");

        let config_https = MilvusConfig::new("milvus.example.com").with_https();
        assert_eq!(
            config_https.get_api_base(),
            "https://milvus.example.com:19530"
        );
    }

    #[test]
    fn test_milvus_config_get_endpoint_url() {
        let config = MilvusConfig::new("milvus.example.com");
        assert_eq!(
            config.get_endpoint_url("/v1/vector/insert"),
            "http://milvus.example.com:19530/v1/vector/insert"
        );
        assert_eq!(
            config.get_endpoint_url("v1/vector/search"),
            "http://milvus.example.com:19530/v1/vector/search"
        );
    }

    #[test]
    fn test_milvus_config_validation_valid() {
        let config = MilvusConfig::new("milvus.example.com");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_milvus_config_validation_empty_host() {
        let config = MilvusConfig::new("");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_milvus_config_validation_zero_port() {
        let mut config = MilvusConfig::new("milvus.example.com");
        config.port = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_milvus_config_validation_zero_timeout() {
        let mut config = MilvusConfig::new("milvus.example.com");
        config.timeout = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_milvus_config_validation_partial_credentials() {
        let mut config = MilvusConfig::new("milvus.example.com");
        config.username = Some("user".to_string());
        // password not set
        assert!(config.validate().is_err());

        let mut config2 = MilvusConfig::new("milvus.example.com");
        config2.password = Some("pass".to_string());
        // username not set
        assert!(config2.validate().is_err());
    }

    #[test]
    fn test_milvus_config_validation_full_credentials() {
        let config = MilvusConfig::new("milvus.example.com").with_credentials("user", "pass");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_milvus_config_auth_headers_token() {
        let config = MilvusConfig::new("milvus.example.com").with_token("my-secret-token");
        let headers = config.get_auth_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert!(headers[0].1.starts_with("Bearer "));
    }

    #[test]
    fn test_milvus_config_auth_headers_basic() {
        let config = MilvusConfig::new("milvus.example.com").with_credentials("user", "pass");
        let headers = config.get_auth_headers();
        assert_eq!(headers.len(), 1);
        assert_eq!(headers[0].0, "Authorization");
        assert!(headers[0].1.starts_with("Basic "));
    }

    #[test]
    fn test_milvus_config_auth_headers_none() {
        let config = MilvusConfig::new("milvus.example.com");
        let headers = config.get_auth_headers();
        assert!(headers.is_empty());
    }

    #[test]
    fn test_milvus_config_provider_config_trait() {
        let config = MilvusConfig::new("milvus.example.com")
            .with_token("my-token")
            .with_timeout(90);

        assert_eq!(config.api_key(), Some("my-token"));
        assert_eq!(config.api_base(), None);
        assert_eq!(config.timeout(), std::time::Duration::from_secs(90));
        assert_eq!(config.max_retries(), 3);
    }

    #[test]
    fn test_milvus_config_serialization() {
        let config = MilvusConfig::new("milvus.example.com")
            .with_collection("test_collection")
            .with_https();

        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["host"], "milvus.example.com");
        assert_eq!(json["collection_name"], "test_collection");
        assert_eq!(json["use_https"], true);
    }

    #[test]
    fn test_milvus_config_deserialization() {
        let json = r#"{
            "host": "milvus.example.com",
            "port": 19531,
            "collection_name": "my_collection",
            "use_https": true
        }"#;

        let config: MilvusConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.host, "milvus.example.com");
        assert_eq!(config.port, 19531);
        assert_eq!(config.collection_name, Some("my_collection".to_string()));
        assert!(config.use_https);
    }
}
