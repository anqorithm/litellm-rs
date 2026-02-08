//! Core health checker implementation

use crate::storage::StorageLayer;
use crate::utils::error::error::Result;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;
use tracing::debug;

use super::types::{ComponentHealth, HealthData, HealthStatus, HealthSummary};

/// Health checker for monitoring system component health
#[derive(Debug)]
pub struct HealthChecker {
    /// Storage layer for health data
    pub(super) storage: Arc<StorageLayer>,
    /// Consolidated health data - single lock for related data
    pub(super) health_data: Arc<RwLock<HealthData>>,
    /// Whether health checking is active - using AtomicBool for lock-free access
    pub(super) active: AtomicBool,
}

impl HealthChecker {
    /// Create a new health checker
    pub async fn new(storage: Arc<StorageLayer>) -> Result<Self> {
        let initial_health = HealthStatus {
            overall_healthy: true,
            last_check: chrono::Utc::now(),
            components: HashMap::new(),
            uptime_seconds: 0,
            summary: HealthSummary {
                total_components: 0,
                healthy_components: 0,
                unhealthy_components: 0,
                health_percentage: 100.0,
            },
        };

        Ok(Self {
            storage,
            health_data: Arc::new(RwLock::new(HealthData {
                components: HashMap::new(),
                overall: initial_health,
            })),
            active: AtomicBool::new(false),
        })
    }

    /// Start health checking
    pub async fn start(&self) -> Result<()> {
        debug!("Starting health checker");

        self.active.store(true, Ordering::Release);

        // Start health check tasks
        self.start_health_check_tasks().await;

        Ok(())
    }

    /// Stop health checking
    pub async fn stop(&self) -> Result<()> {
        debug!("Stopping health checker");
        self.active.store(false, Ordering::Release);
        Ok(())
    }

    /// Check if health checker is active
    #[inline]
    pub fn is_active(&self) -> bool {
        self.active.load(Ordering::Acquire)
    }

    /// Get current health status
    pub async fn get_status(&self) -> Result<HealthStatus> {
        let data = self.health_data.read();
        Ok(data.overall.clone())
    }

    /// Check all components
    pub async fn check_all(&self) -> Result<HealthStatus> {
        debug!("Running comprehensive health check");

        let start_time = Instant::now();
        let mut components = HashMap::new();

        // Check storage layer
        let storage_health = self.check_storage().await;
        components.insert("storage".to_string(), storage_health);

        // Check database
        let database_health = self.check_database().await;
        components.insert("database".to_string(), database_health);

        // Check Redis
        let redis_health = self.check_redis().await;
        components.insert("redis".to_string(), redis_health);

        // Check file storage
        let file_storage_health = self.check_file_storage().await;
        components.insert("file_storage".to_string(), file_storage_health);

        // Check vector database (if configured)
        if self.storage.vector().is_some() {
            let vector_health = self.check_vector_database().await;
            components.insert("vector_database".to_string(), vector_health);
        }

        // Calculate overall health
        let healthy_components = components.values().filter(|c| c.healthy).count();
        let total_components = components.len();
        let overall_healthy = healthy_components == total_components;
        let health_percentage = if total_components > 0 {
            (healthy_components as f64 / total_components as f64) * 100.0
        } else {
            100.0 // No components means healthy by default
        };

        let health_status = HealthStatus {
            overall_healthy,
            last_check: chrono::Utc::now(),
            components: components.clone(),
            uptime_seconds: start_time.elapsed().as_secs(),
            summary: HealthSummary {
                total_components,
                healthy_components,
                unhealthy_components: total_components - healthy_components,
                health_percentage,
            },
        };

        // Update stored health status - single lock for both updates
        {
            let mut data = self.health_data.write();
            data.overall = health_status.clone();
            data.components = components;
        }

        Ok(health_status)
    }

    /// Get component health by name
    pub async fn get_component_health(&self, component_name: &str) -> Option<ComponentHealth> {
        let data = self.health_data.read();
        data.components.get(component_name).cloned()
    }

    /// Check if a specific component is healthy
    pub async fn is_component_healthy(&self, component_name: &str) -> bool {
        if let Some(component) = self.get_component_health(component_name).await {
            component.healthy
        } else {
            false
        }
    }

    /// Get unhealthy components
    pub async fn get_unhealthy_components(&self) -> Vec<ComponentHealth> {
        let data = self.health_data.read();
        data.components
            .values()
            .filter(|component| !component.healthy)
            .cloned()
            .collect()
    }
}

impl Clone for HealthChecker {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            health_data: self.health_data.clone(),
            active: AtomicBool::new(self.active.load(Ordering::Acquire)),
        }
    }
}
