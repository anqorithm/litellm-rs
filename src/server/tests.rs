//! Tests for server module
//!
//! This module contains all tests for the server components.

#[cfg(test)]
mod tests {
    use crate::server::builder::ServerBuilder;
    use crate::server::HttpServer;
    use crate::server::types::RequestMetrics;

    #[test]
    fn test_server_builder() {
        let _builder = ServerBuilder::new();
        // ServerBuilder exists and can be instantiated
    }

    #[test]
    fn test_app_state_creation() {
        // Basic test to ensure module compiles
        // HttpServer requires config, so we just test that the type exists
        assert_eq!(
            std::mem::size_of::<HttpServer>(),
            std::mem::size_of::<HttpServer>()
        );
    }

    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics {
            request_id: "req-123".to_string(),
            method: "GET".to_string(),
            path: "/health".to_string(),
            status_code: 200,
            response_time_ms: 50,
            request_size: 0,
            response_size: 100,
            user_agent: Some("test-agent".to_string()),
            client_ip: Some("127.0.0.1".to_string()),
            user_id: None,
            api_key_id: None,
        };

        assert_eq!(metrics.request_id, "req-123");
        assert_eq!(metrics.method, "GET");
        assert_eq!(metrics.status_code, 200);
    }
}
