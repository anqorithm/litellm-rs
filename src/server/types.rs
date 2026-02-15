//! Server types for monitoring and health checks
//!
//! This module provides types used for server monitoring and metrics.

/// Server health status
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServerHealth {
    /// Server status
    pub status: String,
    /// Server uptime in seconds
    pub uptime: u64,
    /// Number of active connections
    pub active_connections: u32,
    /// Memory usage in bytes
    pub memory_usage: u64,
    /// CPU usage percentage
    pub cpu_usage: f64,
    /// Storage health
    pub storage_health: crate::storage::StorageHealthStatus,
}

/// Request metrics for monitoring
#[derive(Debug, Clone)]
pub struct ServerRequestMetrics {
    /// Request ID
    pub request_id: String,
    /// HTTP method
    pub method: String,
    /// Request path
    pub path: String,
    /// Response status code
    pub status_code: u16,
    /// Response time in milliseconds
    pub response_time_ms: u64,
    /// Request size in bytes
    pub request_size: u64,
    /// Response size in bytes
    pub response_size: u64,
    /// User agent
    pub user_agent: Option<String>,
    /// Client IP address
    pub client_ip: Option<String>,
    /// User ID (if authenticated)
    pub user_id: Option<uuid::Uuid>,
    /// API key ID (if used)
    pub api_key_id: Option<uuid::Uuid>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::StorageHealthStatus;
    use uuid::Uuid;

    // ==================== ServerHealth Tests ====================

    #[test]
    fn test_server_health_creation() {
        let health = ServerHealth {
            status: "healthy".to_string(),
            uptime: 3600,
            active_connections: 100,
            memory_usage: 1024 * 1024 * 512, // 512MB
            cpu_usage: 25.5,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        assert_eq!(health.status, "healthy");
        assert_eq!(health.uptime, 3600);
        assert_eq!(health.active_connections, 100);
    }

    #[test]
    fn test_server_health_unhealthy() {
        let health = ServerHealth {
            status: "unhealthy".to_string(),
            uptime: 100,
            active_connections: 0,
            memory_usage: 1024 * 1024 * 1024, // 1GB
            cpu_usage: 95.0,
            storage_health: StorageHealthStatus {
                database: false,
                redis: false,
                files: true,
                vector: false,
                overall: false,
            },
        };

        assert_eq!(health.status, "unhealthy");
        assert!(!health.storage_health.overall);
    }

    #[test]
    fn test_server_health_clone() {
        let health = ServerHealth {
            status: "healthy".to_string(),
            uptime: 1000,
            active_connections: 50,
            memory_usage: 500000,
            cpu_usage: 10.0,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        let cloned = health.clone();
        assert_eq!(cloned.status, health.status);
        assert_eq!(cloned.uptime, health.uptime);
    }

    #[test]
    fn test_server_health_debug() {
        let health = ServerHealth {
            status: "healthy".to_string(),
            uptime: 100,
            active_connections: 10,
            memory_usage: 100000,
            cpu_usage: 5.0,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        let debug_str = format!("{:?}", health);
        assert!(debug_str.contains("ServerHealth"));
        assert!(debug_str.contains("healthy"));
    }

    #[test]
    fn test_server_health_serialization() {
        let health = ServerHealth {
            status: "healthy".to_string(),
            uptime: 7200,
            active_connections: 200,
            memory_usage: 1024 * 1024 * 256,
            cpu_usage: 30.0,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        let json = serde_json::to_value(&health).unwrap();
        assert_eq!(json["status"], "healthy");
        assert_eq!(json["uptime"], 7200);
        assert_eq!(json["active_connections"], 200);
    }

    #[test]
    fn test_server_health_high_load() {
        let health = ServerHealth {
            status: "degraded".to_string(),
            uptime: 86400,
            active_connections: 10000,
            memory_usage: 8 * 1024 * 1024 * 1024, // 8GB
            cpu_usage: 85.0,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        assert_eq!(health.status, "degraded");
        assert!(health.cpu_usage > 80.0);
    }

    #[test]
    fn test_server_health_zero_uptime() {
        let health = ServerHealth {
            status: "starting".to_string(),
            uptime: 0,
            active_connections: 0,
            memory_usage: 50 * 1024 * 1024,
            cpu_usage: 0.1,
            storage_health: StorageHealthStatus {
                database: true,
                redis: true,
                files: true,
                vector: true,
                overall: true,
            },
        };

        assert_eq!(health.uptime, 0);
        assert_eq!(health.active_connections, 0);
    }

    // ==================== RequestMetrics Tests ====================

    #[test]
    fn test_request_metrics_creation() {
        let metrics = ServerRequestMetrics {
            request_id: "req-123".to_string(),
            method: "GET".to_string(),
            path: "/api/v1/users".to_string(),
            status_code: 200,
            response_time_ms: 150,
            request_size: 256,
            response_size: 1024,
            user_agent: Some("Mozilla/5.0".to_string()),
            client_ip: Some("192.168.1.1".to_string()),
            user_id: Some(Uuid::new_v4()),
            api_key_id: Some(Uuid::new_v4()),
        };

        assert_eq!(metrics.request_id, "req-123");
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.status_code, 200);
    }

    #[test]
    fn test_request_metrics_minimal() {
        let metrics = ServerRequestMetrics {
            request_id: "req-456".to_string(),
            method: "POST".to_string(),
            path: "/".to_string(),
            status_code: 201,
            response_time_ms: 50,
            request_size: 100,
            response_size: 50,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        assert!(metrics.user_agent.is_none());
        assert!(metrics.client_ip.is_none());
        assert!(metrics.user_id.is_none());
        assert!(metrics.api_key_id.is_none());
    }

    #[test]
    fn test_request_metrics_clone() {
        let user_id = Uuid::new_v4();
        let metrics = ServerRequestMetrics {
            request_id: "req-789".to_string(),
            method: "PUT".to_string(),
            path: "/api/resource".to_string(),
            status_code: 200,
            response_time_ms: 100,
            request_size: 500,
            response_size: 200,
            user_agent: Some("Custom Agent".to_string()),
            client_ip: None,
            user_id: Some(user_id),
            api_key_id: None,
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.request_id, metrics.request_id);
        assert_eq!(cloned.user_id, metrics.user_id);
    }

    #[test]
    fn test_request_metrics_debug() {
        let metrics = ServerRequestMetrics {
            request_id: "req-abc".to_string(),
            method: "DELETE".to_string(),
            path: "/api/item/1".to_string(),
            status_code: 204,
            response_time_ms: 30,
            request_size: 0,
            response_size: 0,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        let debug_str = format!("{:?}", metrics);
        assert!(debug_str.contains("RequestMetrics"));
        assert!(debug_str.contains("DELETE"));
    }

    #[test]
    fn test_request_metrics_different_methods() {
        let methods = vec!["GET", "POST", "PUT", "PATCH", "DELETE", "HEAD", "OPTIONS"];

        for method in methods {
            let metrics = ServerRequestMetrics {
                request_id: format!("req-{}", method.to_lowercase()),
                method: method.to_string(),
                path: "/test".to_string(),
                status_code: 200,
                response_time_ms: 10,
                request_size: 0,
                response_size: 0,
                user_agent: None,
                client_ip: None,
                user_id: None,
                api_key_id: None,
            };

            assert_eq!(metrics.method, method);
        }
    }

    #[test]
    fn test_request_metrics_error_status() {
        let metrics = ServerRequestMetrics {
            request_id: "req-error".to_string(),
            method: "GET".to_string(),
            path: "/api/not-found".to_string(),
            status_code: 404,
            response_time_ms: 5,
            request_size: 0,
            response_size: 100,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        assert!(metrics.status_code >= 400);
    }

    #[test]
    fn test_request_metrics_server_error() {
        let metrics = ServerRequestMetrics {
            request_id: "req-server-error".to_string(),
            method: "POST".to_string(),
            path: "/api/process".to_string(),
            status_code: 500,
            response_time_ms: 100,
            request_size: 1000,
            response_size: 50,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        assert!(metrics.status_code >= 500);
    }

    #[test]
    fn test_request_metrics_slow_request() {
        let metrics = ServerRequestMetrics {
            request_id: "req-slow".to_string(),
            method: "GET".to_string(),
            path: "/api/heavy-computation".to_string(),
            status_code: 200,
            response_time_ms: 30000, // 30 seconds
            request_size: 100,
            response_size: 10000,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        assert!(metrics.response_time_ms > 10000);
    }

    #[test]
    fn test_request_metrics_large_payload() {
        let metrics = ServerRequestMetrics {
            request_id: "req-large".to_string(),
            method: "POST".to_string(),
            path: "/api/upload".to_string(),
            status_code: 200,
            response_time_ms: 5000,
            request_size: 10 * 1024 * 1024, // 10MB
            response_size: 100,
            user_agent: None,
            client_ip: None,
            user_id: None,
            api_key_id: None,
        };

        assert!(metrics.request_size > 1024 * 1024);
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_server_health_with_partial_storage() {
        let health = ServerHealth {
            status: "degraded".to_string(),
            uptime: 1000,
            active_connections: 50,
            memory_usage: 500 * 1024 * 1024,
            cpu_usage: 40.0,
            storage_health: StorageHealthStatus {
                database: true,
                redis: false, // Redis is down
                files: true,
                vector: true,
                overall: false, // Overall is unhealthy due to Redis
            },
        };

        assert!(!health.storage_health.redis);
        assert!(!health.storage_health.overall);
    }

    #[test]
    fn test_request_metrics_authenticated() {
        let user_id = Uuid::new_v4();
        let api_key_id = Uuid::new_v4();

        let metrics = ServerRequestMetrics {
            request_id: "req-auth".to_string(),
            method: "GET".to_string(),
            path: "/api/protected".to_string(),
            status_code: 200,
            response_time_ms: 50,
            request_size: 100,
            response_size: 500,
            user_agent: Some("API Client/1.0".to_string()),
            client_ip: Some("10.0.0.1".to_string()),
            user_id: Some(user_id),
            api_key_id: Some(api_key_id),
        };

        assert!(metrics.user_id.is_some());
        assert!(metrics.api_key_id.is_some());
    }

    #[test]
    fn test_request_metrics_status_classification() {
        let status_codes = vec![
            (200, "success"),
            (201, "success"),
            (301, "redirect"),
            (400, "client_error"),
            (401, "client_error"),
            (403, "client_error"),
            (404, "client_error"),
            (500, "server_error"),
            (502, "server_error"),
            (503, "server_error"),
        ];

        for (code, expected_type) in status_codes {
            let actual_type = if code >= 500 {
                "server_error"
            } else if code >= 400 {
                "client_error"
            } else if code >= 300 {
                "redirect"
            } else {
                "success"
            };

            assert_eq!(
                actual_type, expected_type,
                "Status code {} classification",
                code
            );
        }
    }
}
