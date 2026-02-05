//! Core functionality for the Gateway
//!
//! This module contains the core business logic and data structures.

#![allow(dead_code)]

pub mod a2a; // A2A (Agent-to-Agent) Protocol Gateway
pub mod agent; // Agent Coordinator for managing agent lifecycles
pub mod alerting; // Alerting system (Slack, webhooks)
pub mod analytics;
pub mod audio; // Audio API (transcription, translation, speech)
pub mod audit; // Audit logging system
// pub mod base_provider;  // Removed: unused dead code
pub mod batch;
pub mod budget; // Budget management system
// pub mod cache; // DualCache system (InMemory + Redis) - TODO: implement
pub mod cache_manager;
pub mod completion; // Core completion API
pub mod cost; // Unified cost calculation system
pub mod embedding; // Core embedding API (Python LiteLLM compatible)
pub mod fine_tuning; // Fine-tuning API
pub mod function_calling; // Function calling support for AI providers
pub mod guardrails; // Content safety and validation system
pub mod health; // Health monitoring system
pub mod integrations; // External integrations (Langfuse, etc.)
pub mod ip_access; // IP-based access control
pub mod keys; // API Key Management System
pub mod mcp; // MCP (Model Context Protocol) Gateway
pub mod models;
pub mod observability; // Advanced observability and monitoring
pub mod providers;
pub mod rate_limiter; // Rate limiting system
pub mod realtime; // Realtime WebSocket API
pub mod rerank; // Rerank API for RAG systems
pub mod router;
pub mod secret_managers; // Secret management system
pub mod security;
pub mod semantic_cache;
pub mod streaming;
pub mod teams; // Team management module
pub mod traits;
pub mod types;
// User and team management - disabled until database methods are implemented
// These modules require the following database methods to be implemented:
// - virtual_keys: store_virtual_key, get_virtual_key, update_virtual_key, etc.
// - user_management: get_user, create_user, get_team, create_team, etc.
// TODO: Implement database methods and enable these modules
// pub mod user_management;
// pub mod virtual_keys;
pub mod webhooks;

// Re-export commonly used types

// pub use engine::Gateway;

use crate::config::Config;
use crate::utils::error::Result;
use std::sync::Arc;
use tracing::{debug, info};

/// Main Gateway struct that orchestrates all components
#[derive(Clone)]
pub struct Gateway {
    /// Gateway configuration
    config: Arc<Config>,
    /// Request processing engine
    // engine: Arc<engine::RequestEngine>,
    // /// Provider pool
    // providers: Arc<providers::ProviderPool>,
    /// Storage layer
    storage: Arc<crate::storage::StorageLayer>,
    /// Authentication system
    auth: Arc<crate::auth::AuthSystem>,
    /// Monitoring system
    monitoring: Arc<crate::monitoring::system::MonitoringSystem>,
}

impl Gateway {
    /// Create a new Gateway instance
    pub async fn new(config: Config) -> Result<Self> {
        info!("Initializing Gateway");

        let config = Arc::new(config);

        // Initialize storage layer
        debug!("Initializing storage layer");
        let storage = Arc::new(crate::storage::StorageLayer::new(&config.gateway.storage).await?);

        // Initialize authentication system
        debug!("Initializing authentication system");
        let auth =
            Arc::new(crate::auth::AuthSystem::new(&config.gateway.auth, storage.clone()).await?);

        // Initialize monitoring system
        debug!("Initializing monitoring system");
        let monitoring = Arc::new(
            crate::monitoring::system::MonitoringSystem::new(
                &config.gateway.monitoring,
                storage.clone(),
            )
            .await?,
        );

        // Initialize provider pool
        // debug!("Initializing provider pool");
        // let providers = Arc::new(providers::ProviderPool::new(&config.gateway.providers).await?);

        // Initialize request engine
        debug!("Initializing request engine");
        // let engine = Arc::new(engine::RequestEngine::new(
        //     config.clone(),
        //     providers.clone(),
        //     storage.clone(),
        //     monitoring.clone(),
        // ).await?);

        info!("Gateway initialized successfully");

        Ok(Self {
            config,
            // engine,
            // providers,
            storage,
            auth,
            monitoring,
        })
    }

    /// Start the Gateway server
    pub async fn run(self) -> Result<()> {
        info!("Starting Gateway server");

        // Start background services
        self.start_background_services().await?;

        // Start HTTP server
        // crate::server::start_server(self).await
        Ok(())
    }

    /// Start background services
    async fn start_background_services(&self) -> Result<()> {
        debug!("Starting background services");

        // // Start health checker
        // let health_checker = crate::monitoring::health::HealthChecker::new(
        //     self.providers.clone(),
        //     self.monitoring.clone(),
        // );
        // tokio::spawn(async move {
        //     health_checker.run().await;
        // });

        // // Start metrics collector
        // let metrics_collector = crate::monitoring::metrics::MetricsCollector::new(
        //     self.storage.clone(),
        //     self.monitoring.clone(),
        // );
        // tokio::spawn(async move {
        //     metrics_collector.run().await;
        // });

        // // Start cost calculator
        // let cost_calculator = crate::utils::cost::CostCalculator::new(
        //     self.storage.clone(),
        //     self.monitoring.clone(),
        // );
        // tokio::spawn(async move {
        //     cost_calculator.run().await;
        // });

        debug!("Background services started");
        Ok(())
    }

    /// Get gateway configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    // /// Get request engine
    // pub fn engine(&self) -> &engine::RequestEngine {
    //     &self.engine
    // }

    // /// Get provider pool
    // pub fn providers(&self) -> &providers::ProviderPool {
    //     &self.providers
    // }

    /// Get storage layer
    pub fn storage(&self) -> &crate::storage::StorageLayer {
        &self.storage
    }

    /// Get authentication system
    pub fn auth(&self) -> &crate::auth::AuthSystem {
        &self.auth
    }

    /// Get monitoring system
    pub fn monitoring(&self) -> &crate::monitoring::system::MonitoringSystem {
        &self.monitoring
    }

    /// Graceful shutdown
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Gateway");

        // Stop background services
        // TODO: Implement graceful shutdown for background services

        // Close storage connections
        self.storage.close().await?;

        info!("Gateway shutdown completed");
        Ok(())
    }

    /// Health check
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let mut status = HealthStatus {
            status: "healthy".to_string(),
            timestamp: chrono::Utc::now(),
            components: std::collections::HashMap::new(),
        };

        // Check storage health
        match self.storage.health_check().await {
            Ok(_) => {
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth {
                        status: "healthy".to_string(),
                        message: None,
                    },
                );
            }
            Err(e) => {
                status.status = "unhealthy".to_string();
                status.components.insert(
                    "storage".to_string(),
                    ComponentHealth {
                        status: "unhealthy".to_string(),
                        message: Some(e.to_string()),
                    },
                );
            }
        }

        // Check provider health
        // TODO: Re-enable when providers field is restored
        // let provider_health = self.providers.health_check().await;
        status.components.insert(
            "providers".to_string(),
            ComponentHealth {
                status: "unknown".to_string(),
                message: Some("Provider health check disabled".to_string()),
            },
        );

        Ok(status)
    }
}

/// Health status response
#[derive(Debug, Clone, serde::Serialize)]
pub struct HealthStatus {
    /// Overall system status
    pub status: String,
    /// Timestamp when health was checked
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Health status of individual components
    pub components: std::collections::HashMap<String, ComponentHealth>,
}

/// Component health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct ComponentHealth {
    /// Component status
    pub status: String,
    /// Optional status message
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {

    use crate::config::Config;

    #[tokio::test]
    async fn test_gateway_creation() {
        let _config = Config::default();

        // This test would require proper setup of all dependencies
        // For now, we'll just test that the config is properly stored
        // let gateway = Gateway::new(config).await.unwrap();
        // assert_eq!(gateway.config().server().port, 8000);
    }
}
