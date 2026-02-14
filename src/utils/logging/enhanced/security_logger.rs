//! Security-aware logging utilities

use crate::utils::logging::enhanced::async_logger::async_logger;
use std::collections::HashMap;
use tracing::Level;
use uuid::Uuid;

/// Security-aware logging utilities
#[allow(dead_code)]
pub struct SecurityLogger;

#[allow(dead_code)]
impl SecurityLogger {
    /// Log authentication events
    pub fn log_auth_event(
        event_type: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        success: bool,
        details: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "event_type".to_string(),
            serde_json::Value::String(event_type.to_string()),
        );
        fields.insert("success".to_string(), serde_json::Value::Bool(success));

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(ua) = user_agent {
            // Truncate user agent to prevent log injection
            let safe_ua = ua.chars().take(200).collect::<String>();
            fields.insert("user_agent".to_string(), serde_json::Value::String(safe_ua));
        }

        if let Some(details) = details {
            fields.insert(
                "details".to_string(),
                serde_json::Value::String(details.to_string()),
            );
        }

        let level = if success { Level::INFO } else { Level::WARN };
        let message = format!(
            "Authentication {}: {}",
            if success { "success" } else { "failure" },
            event_type
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, user_id);
        }
    }

    /// Log authorization events
    pub fn log_authz_event(
        user_id: Uuid,
        resource: &str,
        action: &str,
        granted: bool,
        reason: Option<&str>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "resource".to_string(),
            serde_json::Value::String(resource.to_string()),
        );
        fields.insert(
            "action".to_string(),
            serde_json::Value::String(action.to_string()),
        );
        fields.insert("granted".to_string(), serde_json::Value::Bool(granted));

        if let Some(reason) = reason {
            fields.insert(
                "reason".to_string(),
                serde_json::Value::String(reason.to_string()),
            );
        }

        let level = if granted { Level::DEBUG } else { Level::WARN };
        let message = format!(
            "Authorization {}: {} on {}",
            if granted { "granted" } else { "denied" },
            action,
            resource
        );

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", &message, fields, None, Some(user_id));
        }
    }

    /// Log security violations
    pub fn log_security_violation(
        violation_type: &str,
        severity: &str,
        description: &str,
        user_id: Option<Uuid>,
        ip_address: Option<&str>,
        additional_data: Option<HashMap<String, serde_json::Value>>,
    ) {
        let mut fields = HashMap::new();
        fields.insert(
            "violation_type".to_string(),
            serde_json::Value::String(violation_type.to_string()),
        );
        fields.insert(
            "severity".to_string(),
            serde_json::Value::String(severity.to_string()),
        );

        if let Some(ip) = ip_address {
            fields.insert(
                "ip_address".to_string(),
                serde_json::Value::String(ip.to_string()),
            );
        }

        if let Some(data) = additional_data {
            for (key, value) in data {
                fields.insert(key, value);
            }
        }

        let level = match severity.to_lowercase().as_str() {
            "critical" | "high" => Level::ERROR,
            "medium" => Level::WARN,
            _ => Level::INFO,
        };

        if let Some(logger) = async_logger() {
            logger.log_structured(level, "security", description, fields, None, user_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::logging::enhanced::async_logger::init_async_logger;
    use crate::utils::logging::enhanced::types::AsyncLoggerConfig;
    use std::sync::Once;

    static INIT: Once = Once::new();

    fn setup_logger() {
        INIT.call_once(|| {
            // Initialize with a test configuration
            let config = AsyncLoggerConfig {
                buffer_size: 1000,
                drop_on_overflow: true,
                sample_rate: 1.0,
                max_message_length: 1024,
            };
            init_async_logger(config);
        });
    }

    #[tokio::test]
    async fn test_log_auth_event_success_with_all_fields() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_auth_event(
            "login",
            Some(user_id),
            Some("192.168.1.1"),
            Some("Mozilla/5.0"),
            true,
            Some("Successful login"),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_auth_event_failure_with_minimal_fields() {
        setup_logger();

        SecurityLogger::log_auth_event("login", None, None, None, false, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_auth_event_with_ip_address_only() {
        setup_logger();

        SecurityLogger::log_auth_event("api_key_auth", None, Some("10.0.0.1"), None, true, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_auth_event_with_user_agent_only() {
        setup_logger();

        SecurityLogger::log_auth_event("jwt_auth", None, None, Some("curl/7.68.0"), true, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_auth_event_with_long_user_agent() {
        setup_logger();

        // Create a user agent string longer than 200 characters
        let long_user_agent = "A".repeat(300);
        SecurityLogger::log_auth_event(
            "token_auth",
            None,
            None,
            Some(&long_user_agent),
            true,
            None,
        );

        // Test passes if no panic occurs - user agent should be truncated
    }

    #[tokio::test]
    async fn test_log_auth_event_with_details() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_auth_event(
            "password_reset",
            Some(user_id),
            Some("172.16.0.1"),
            Some("PostmanRuntime/7.29.0"),
            false,
            Some("Invalid reset token"),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_auth_event_different_event_types() {
        setup_logger();

        let event_types = vec!["login", "logout", "token_refresh", "2fa", "sso"];
        for event_type in event_types {
            SecurityLogger::log_auth_event(event_type, None, None, None, true, None);
        }

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_authz_event_granted() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_authz_event(user_id, "/api/users", "read", true, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_authz_event_denied() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_authz_event(
            user_id,
            "/api/admin",
            "write",
            false,
            Some("Insufficient permissions"),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_authz_event_with_reason() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_authz_event(
            user_id,
            "/api/data",
            "delete",
            false,
            Some("User not in admin role"),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_authz_event_different_actions() {
        setup_logger();

        let user_id = Uuid::new_v4();
        let actions = vec!["read", "write", "delete", "update", "execute"];
        for action in actions {
            SecurityLogger::log_authz_event(user_id, "/api/resource", action, true, None);
        }

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_authz_event_different_resources() {
        setup_logger();

        let user_id = Uuid::new_v4();
        let resources = vec![
            "/api/users",
            "/api/posts",
            "/api/settings",
            "/admin/dashboard",
        ];
        for resource in resources {
            SecurityLogger::log_authz_event(user_id, resource, "read", true, None);
        }

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_critical() {
        setup_logger();

        SecurityLogger::log_security_violation(
            "sql_injection",
            "critical",
            "SQL injection attempt detected",
            None,
            Some("203.0.113.1"),
            None,
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_high() {
        setup_logger();

        let user_id = Uuid::new_v4();
        SecurityLogger::log_security_violation(
            "brute_force",
            "high",
            "Multiple failed login attempts",
            Some(user_id),
            Some("198.51.100.1"),
            None,
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_medium() {
        setup_logger();

        SecurityLogger::log_security_violation(
            "rate_limit",
            "medium",
            "Rate limit exceeded",
            None,
            Some("192.0.2.1"),
            None,
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_low() {
        setup_logger();

        SecurityLogger::log_security_violation(
            "invalid_input",
            "low",
            "Invalid input format",
            None,
            None,
            None,
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_unknown_severity() {
        setup_logger();

        SecurityLogger::log_security_violation(
            "unknown_event",
            "unknown",
            "Unknown security event",
            None,
            None,
            None,
        );

        // Test passes if no panic occurs - should default to INFO level
    }

    #[tokio::test]
    async fn test_log_security_violation_with_additional_data() {
        setup_logger();

        let mut additional_data = HashMap::new();
        additional_data.insert(
            "request_path".to_string(),
            serde_json::Value::String("/api/admin".to_string()),
        );
        additional_data.insert(
            "method".to_string(),
            serde_json::Value::String("POST".to_string()),
        );
        additional_data.insert(
            "status_code".to_string(),
            serde_json::Value::Number(403.into()),
        );

        let user_id = Uuid::new_v4();
        SecurityLogger::log_security_violation(
            "unauthorized_access",
            "high",
            "Unauthorized access attempt",
            Some(user_id),
            Some("10.1.1.1"),
            Some(additional_data),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_with_complex_additional_data() {
        setup_logger();

        let mut additional_data = HashMap::new();
        additional_data.insert(
            "payload".to_string(),
            serde_json::json!({"key": "value", "nested": {"data": [1, 2, 3]}}),
        );
        additional_data.insert(
            "headers".to_string(),
            serde_json::json!({"content-type": "application/json"}),
        );

        SecurityLogger::log_security_violation(
            "suspicious_payload",
            "medium",
            "Suspicious payload detected",
            None,
            Some("192.168.100.50"),
            Some(additional_data),
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_case_insensitive_severity() {
        setup_logger();

        let severities = vec![
            ("CRITICAL", "critical"),
            ("High", "high"),
            ("MEDIUM", "medium"),
            ("Low", "low"),
        ];

        for (severity, violation_type) in severities {
            SecurityLogger::log_security_violation(
                violation_type,
                severity,
                &format!("Test {} severity", severity),
                None,
                None,
                None,
            );
        }

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_different_types() {
        setup_logger();

        let violation_types = vec![
            "xss_attack",
            "csrf_missing",
            "path_traversal",
            "command_injection",
            "xxe_attack",
        ];

        for violation_type in violation_types {
            SecurityLogger::log_security_violation(
                violation_type,
                "high",
                &format!("{} detected", violation_type),
                None,
                Some("192.168.1.100"),
                None,
            );
        }

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_log_security_violation_empty_additional_data() {
        setup_logger();

        let additional_data = HashMap::new();
        SecurityLogger::log_security_violation(
            "test",
            "medium",
            "Test with empty additional data",
            None,
            None,
            Some(additional_data),
        );

        // Test passes if no panic occurs
    }

    #[test]
    fn test_all_functions_without_logger() {
        // Don't call setup_logger() to test behavior when logger is not initialized
        // This tests the None case in the if let Some(logger) = async_logger() branches

        let user_id = Uuid::new_v4();

        // Test log_auth_event
        SecurityLogger::log_auth_event("test", Some(user_id), None, None, true, None);

        // Test log_authz_event
        SecurityLogger::log_authz_event(user_id, "/test", "read", true, None);

        // Test log_security_violation
        SecurityLogger::log_security_violation("test", "low", "test", None, None, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_user_agent_truncation_boundary() {
        setup_logger();

        // Test exactly 200 characters
        let exact_200 = "A".repeat(200);
        SecurityLogger::log_auth_event("test", None, None, Some(&exact_200), true, None);

        // Test 199 characters
        let under_200 = "B".repeat(199);
        SecurityLogger::log_auth_event("test", None, None, Some(&under_200), true, None);

        // Test 201 characters (should be truncated)
        let over_200 = "C".repeat(201);
        SecurityLogger::log_auth_event("test", None, None, Some(&over_200), true, None);

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_edge_cases_with_special_characters() {
        setup_logger();

        let user_id = Uuid::new_v4();

        // Test with special characters in various fields
        SecurityLogger::log_auth_event(
            "login\n\r\t",
            Some(user_id),
            Some("192.168.1.1:8080"),
            Some("Mozilla/5.0 (Unicode: \u{1F600})"),
            true,
            Some("Details with \"quotes\" and 'apostrophes'"),
        );

        SecurityLogger::log_authz_event(
            user_id,
            "/api/resource?param=value&other=test",
            "read|write",
            true,
            Some("Reason: <tag>content</tag>"),
        );

        SecurityLogger::log_security_violation(
            "test<>violation",
            "medium",
            "Description with & and % and #",
            Some(user_id),
            Some("2001:0db8:85a3:0000:0000:8a2e:0370:7334"),
            None,
        );

        // Test passes if no panic occurs
    }

    #[tokio::test]
    async fn test_concurrent_logging() {
        setup_logger();

        let mut handles = vec![];

        for i in 0..10 {
            let handle = tokio::spawn(async move {
                let user_id = Uuid::new_v4();
                SecurityLogger::log_auth_event(
                    &format!("concurrent_test_{}", i),
                    Some(user_id),
                    Some("127.0.0.1"),
                    None,
                    true,
                    None,
                );
            });
            handles.push(handle);
        }

        for handle in handles {
            handle.await.expect("Task should complete");
        }

        // Test passes if all concurrent logs complete without panic
    }
}
