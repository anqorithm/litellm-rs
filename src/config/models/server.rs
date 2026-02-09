//! Server configuration

use super::*;
use serde::{Deserialize, Serialize};
use tracing::warn;

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host
    #[serde(default = "default_host")]
    pub host: String,
    /// Server port
    #[serde(default = "default_port")]
    pub port: u16,
    /// Number of worker threads
    pub workers: Option<usize>,
    /// Request timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    /// Maximum request body size in bytes
    #[serde(default = "default_max_body_size")]
    pub max_body_size: usize,
    /// Enable development mode
    #[serde(default)]
    pub dev_mode: bool,
    /// TLS configuration
    pub tls: Option<TlsConfig>,
    /// CORS configuration
    #[serde(default)]
    pub cors: CorsConfig,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
            port: default_port(),
            workers: None,
            timeout: default_timeout(),
            max_body_size: default_max_body_size(),
            dev_mode: false,
            tls: None,
            cors: CorsConfig::default(),
        }
    }
}

impl ServerConfig {
    /// Merge server configurations
    pub fn merge(mut self, other: Self) -> Self {
        if other.host != default_host() {
            self.host = other.host;
        }
        if other.port != default_port() {
            self.port = other.port;
        }
        if other.workers.is_some() {
            self.workers = other.workers;
        }
        if other.timeout != default_timeout() {
            self.timeout = other.timeout;
        }
        if other.max_body_size != default_max_body_size() {
            self.max_body_size = other.max_body_size;
        }
        if other.dev_mode {
            self.dev_mode = other.dev_mode;
        }
        if other.tls.is_some() {
            self.tls = other.tls;
        }
        self.cors = self.cors.merge(other.cors);
        self
    }

    /// Get the server address
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls.is_some()
    }

    /// Get the number of workers (defaults to CPU count)
    pub fn worker_count(&self) -> usize {
        self.workers
            .unwrap_or_else(|| std::thread::available_parallelism().map_or(1, |n| n.get()))
    }

    /// Validate server configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.port == 0 {
            return Err("Port cannot be 0".to_string());
        }

        if self.timeout == 0 {
            return Err("Timeout cannot be 0".to_string());
        }

        if self.max_body_size == 0 {
            return Err("Max body size cannot be 0".to_string());
        }

        if let Some(tls) = &self.tls {
            tls.validate()?;
        }

        Ok(())
    }
}

/// TLS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Certificate file path
    pub cert_file: String,
    /// Private key file path
    pub key_file: String,
    /// CA certificate file path (optional)
    pub ca_file: Option<String>,
    /// Require client certificates
    #[serde(default)]
    pub require_client_cert: bool,
}

impl TlsConfig {
    /// Validate TLS configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.cert_file.is_empty() {
            return Err("TLS certificate file path is required".to_string());
        }

        if self.key_file.is_empty() {
            return Err("TLS private key file path is required".to_string());
        }

        // Check if files exist
        if !std::path::Path::new(&self.cert_file).exists() {
            return Err(format!(
                "TLS certificate file not found: {}",
                self.cert_file
            ));
        }

        if !std::path::Path::new(&self.key_file).exists() {
            return Err(format!("TLS private key file not found: {}", self.key_file));
        }

        if let Some(ca_file) = &self.ca_file {
            if !std::path::Path::new(ca_file).exists() {
                return Err(format!("TLS CA file not found: {}", ca_file));
            }
        }

        Ok(())
    }
}

/// CORS configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Enable CORS
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Allowed origins (empty means allow all)
    #[serde(default)]
    pub allowed_origins: Vec<String>,
    /// Allowed methods
    #[serde(default = "default_cors_methods")]
    pub allowed_methods: Vec<String>,
    /// Allowed headers
    #[serde(default = "default_cors_headers")]
    pub allowed_headers: Vec<String>,
    /// Max age for preflight requests
    #[serde(default = "default_cors_max_age")]
    pub max_age: u32,
    /// Allow credentials
    #[serde(default)]
    pub allow_credentials: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_origins: vec![], // Empty means restrictive by default
            allowed_methods: default_cors_methods(),
            allowed_headers: default_cors_headers(),
            max_age: default_cors_max_age(),
            allow_credentials: false,
        }
    }
}

impl CorsConfig {
    /// Merge CORS configurations
    pub fn merge(mut self, other: Self) -> Self {
        if !other.enabled {
            self.enabled = other.enabled;
        }
        if !other.allowed_origins.is_empty() {
            self.allowed_origins = other.allowed_origins;
        }
        if other.allowed_methods != default_cors_methods() {
            self.allowed_methods = other.allowed_methods;
        }
        if other.allowed_headers != default_cors_headers() {
            self.allowed_headers = other.allowed_headers;
        }
        if other.max_age != default_cors_max_age() {
            self.max_age = other.max_age;
        }
        if other.allow_credentials {
            self.allow_credentials = other.allow_credentials;
        }
        self
    }

    /// Check if CORS allows all origins (insecure)
    pub fn allows_all_origins(&self) -> bool {
        self.allowed_origins.is_empty() || self.allowed_origins.contains(&"*".to_string())
    }

    /// Validate CORS configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            if self.allows_all_origins() && self.allow_credentials {
                return Err("CORS cannot allow all origins (*) when credentials are enabled for security reasons".to_string());
            }

            // Warn about insecure configurations
            if self.allows_all_origins() {
                warn!("CORS allows all origins. This may be insecure for production.");
            }
        }
        Ok(())
    }
}

fn default_true() -> bool {
    true
}

fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
        "OPTIONS".to_string(),
    ]
}

fn default_cors_headers() -> Vec<String> {
    vec![
        "authorization".to_string(),
        "content-type".to_string(),
        "x-requested-with".to_string(),
    ]
}

fn default_cors_max_age() -> u32 {
    3600
}

#[cfg(test)]
mod server_tests;
