//! Server builder and run_server function
//!
//! This module provides the ServerBuilder for easier server configuration
//! and the run_server function for automatic configuration loading.

use crate::config::Config;
use crate::server::HttpServer;
use crate::utils::error::{GatewayError, Result};
use tracing::info;

/// Server builder for easier configuration
pub struct ServerBuilder {
    config: Option<Config>,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new() -> Self {
        Self { config: None }
    }

    /// Set configuration
    pub fn with_config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the HTTP server
    pub async fn build(self) -> Result<HttpServer> {
        let config = self
            .config
            .ok_or_else(|| GatewayError::Config("Configuration is required".to_string()))?;

        HttpServer::new(&config).await
    }
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Run the server with automatic configuration loading
pub async fn run_server() -> Result<()> {
    info!("🚀 Starting Rust LiteLLM Gateway");

    // Auto-load configuration file
    let config_path = "config/gateway.yaml";
    info!("📄 Loading configuration file: {}", config_path);

    let config = match Config::from_file(config_path).await {
        Ok(config) => {
            info!("✅ Configuration file loaded successfully");
            config
        }
        Err(e) => {
            info!(
                "⚠️  Configuration file loading failed, using default config: {}",
                e
            );
            info!("💡 Please ensure config/gateway.yaml exists with correct API keys");
            Config::default()
        }
    };

    // Ensure configuration is valid (including defaults)
    config.validate()?;

    // Create and start server
    let server = HttpServer::new(&config).await?;
    info!(
        "🌐 Server starting at: http://{}:{}",
        config.server().host,
        config.server().port
    );
    info!("📋 API Endpoints:");
    info!("   GET  /health - Health check");
    info!("   GET  /v1/models - Model list");
    info!("   POST /v1/chat/completions - Chat completions");
    info!("   POST /v1/completions - Text completions");
    info!("   POST /v1/embeddings - Text embeddings");

    server.start().await
}
