//! Individual component health check implementations

use std::collections::HashMap;
use std::time::Instant;

use super::checker::HealthChecker;
use super::types::ComponentHealth;

impl HealthChecker {
    /// Check storage layer health
    pub(super) async fn check_storage(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.health_check().await {
            Ok(storage_status) => ComponentHealth {
                name: "storage".to_string(),
                healthy: storage_status.overall,
                status: if storage_status.overall {
                    "healthy"
                } else {
                    "degraded"
                }
                .to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: serde_json::to_value(&storage_status)
                    .unwrap_or_default()
                    .as_object()
                    .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    .unwrap_or_default(),
            },
            Err(e) => ComponentHealth {
                name: "storage".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check database health
    pub(super) async fn check_database(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.db().health_check().await {
            Ok(()) => {
                let stats = self.storage.db().stats().await;
                let mut metadata = HashMap::new();
                metadata.insert(
                    "pool_size".to_string(),
                    serde_json::Value::Number(stats.size.into()),
                );
                metadata.insert(
                    "idle_connections".to_string(),
                    serde_json::Value::Number(stats.idle.into()),
                );

                ComponentHealth {
                    name: "database".to_string(),
                    healthy: true,
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                    metadata,
                }
            }
            Err(e) => ComponentHealth {
                name: "database".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check Redis health
    pub(super) async fn check_redis(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.redis().health_check().await {
            Ok(()) => ComponentHealth {
                name: "redis".to_string(),
                healthy: true,
                status: "healthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: HashMap::new(),
            },
            Err(e) => ComponentHealth {
                name: "redis".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check file storage health
    pub(super) async fn check_file_storage(&self) -> ComponentHealth {
        let start_time = Instant::now();

        match self.storage.files().health_check().await {
            Ok(()) => ComponentHealth {
                name: "file_storage".to_string(),
                healthy: true,
                status: "healthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: None,
                metadata: HashMap::new(),
            },
            Err(e) => ComponentHealth {
                name: "file_storage".to_string(),
                healthy: false,
                status: "unhealthy".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: start_time.elapsed().as_millis() as u64,
                error: Some(e.to_string()),
                metadata: HashMap::new(),
            },
        }
    }

    /// Check vector database health
    pub(super) async fn check_vector_database(&self) -> ComponentHealth {
        let start_time = Instant::now();

        if let Some(vector_store) = self.storage.vector() {
            match vector_store.health_check().await {
                Ok(()) => ComponentHealth {
                    name: "vector_database".to_string(),
                    healthy: true,
                    status: "healthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: None,
                    metadata: HashMap::new(),
                },
                Err(e) => ComponentHealth {
                    name: "vector_database".to_string(),
                    healthy: false,
                    status: "unhealthy".to_string(),
                    last_check: chrono::Utc::now(),
                    response_time_ms: start_time.elapsed().as_millis() as u64,
                    error: Some(e.to_string()),
                    metadata: HashMap::new(),
                },
            }
        } else {
            ComponentHealth {
                name: "vector_database".to_string(),
                healthy: true,
                status: "not_configured".to_string(),
                last_check: chrono::Utc::now(),
                response_time_ms: 0,
                error: None,
                metadata: HashMap::new(),
            }
        }
    }

}
