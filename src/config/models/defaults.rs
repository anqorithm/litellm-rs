//! Default value functions for configuration

use super::monitoring::LogFormat;

pub fn default_true() -> bool {
    true
}

pub fn default_weight() -> f64 {
    1.0
}

pub fn default_timeout_seconds() -> u64 {
    30
}

pub fn default_max_retries() -> u32 {
    3
}

pub fn default_health_check_interval() -> u64 {
    30
}

pub fn default_health_check_timeout() -> u64 {
    5
}

pub fn default_health_threshold() -> u32 {
    2
}

pub fn default_unhealthy_threshold() -> u32 {
    3
}

pub fn default_failure_threshold() -> u32 {
    5
}

pub fn default_recovery_timeout() -> u64 {
    60
}

pub fn default_half_open_requests() -> u32 {
    3
}

pub fn default_session_timeout() -> u64 {
    3600
}

pub fn default_initial_delay_ms() -> u64 {
    100
}

pub fn default_max_delay_ms() -> u64 {
    30000
}

pub fn default_pool_size() -> u32 {
    10
}

pub fn default_jwt_algorithm() -> String {
    "HS256".to_string()
}

pub fn default_jwt_expiration() -> u64 {
    86400 // 24 hours
}

pub fn default_api_key_header() -> String {
    "Authorization".to_string()
}

pub fn default_cors_methods() -> Vec<String> {
    vec![
        "GET".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "DELETE".to_string(),
    ]
}

pub fn default_cors_headers() -> Vec<String> {
    vec!["Content-Type".to_string(), "Authorization".to_string()]
}

pub fn default_cors_max_age() -> u64 {
    3600
}

pub fn default_metrics_endpoint() -> String {
    "/metrics".to_string()
}

pub fn default_metrics_interval() -> u64 {
    15
}

pub fn default_sampling_rate() -> f64 {
    0.1
}

pub fn default_log_level() -> String {
    "info".to_string()
}

pub fn default_log_format() -> LogFormat {
    LogFormat::Json
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Boolean Default Tests ====================

    #[test]
    fn test_default_true() {
        assert!(default_true());
    }

    // ==================== Numeric Default Tests ====================

    #[test]
    fn test_default_weight() {
        let weight = default_weight();
        assert!((weight - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_default_timeout_seconds() {
        assert_eq!(default_timeout_seconds(), 30);
    }

    #[test]
    fn test_default_max_retries() {
        assert_eq!(default_max_retries(), 3);
    }

    // ==================== Health Check Default Tests ====================

    #[test]
    fn test_default_health_check_interval() {
        assert_eq!(default_health_check_interval(), 30);
    }

    #[test]
    fn test_default_health_check_timeout() {
        assert_eq!(default_health_check_timeout(), 5);
    }

    #[test]
    fn test_default_health_threshold() {
        assert_eq!(default_health_threshold(), 2);
    }

    #[test]
    fn test_default_unhealthy_threshold() {
        assert_eq!(default_unhealthy_threshold(), 3);
    }

    // ==================== Circuit Breaker Default Tests ====================

    #[test]
    fn test_default_failure_threshold() {
        assert_eq!(default_failure_threshold(), 5);
    }

    #[test]
    fn test_default_recovery_timeout() {
        assert_eq!(default_recovery_timeout(), 60);
    }

    #[test]
    fn test_default_half_open_requests() {
        assert_eq!(default_half_open_requests(), 3);
    }

    // ==================== Session Default Tests ====================

    #[test]
    fn test_default_session_timeout() {
        assert_eq!(default_session_timeout(), 3600);
    }

    // ==================== Retry Default Tests ====================

    #[test]
    fn test_default_initial_delay_ms() {
        assert_eq!(default_initial_delay_ms(), 100);
    }

    #[test]
    fn test_default_max_delay_ms() {
        assert_eq!(default_max_delay_ms(), 30000);
    }

    // ==================== Pool Default Tests ====================

    #[test]
    fn test_default_pool_size() {
        assert_eq!(default_pool_size(), 10);
    }

    // ==================== JWT Default Tests ====================

    #[test]
    fn test_default_jwt_algorithm() {
        assert_eq!(default_jwt_algorithm(), "HS256");
    }

    #[test]
    fn test_default_jwt_expiration() {
        assert_eq!(default_jwt_expiration(), 86400);
    }

    // ==================== API Key Default Tests ====================

    #[test]
    fn test_default_api_key_header() {
        assert_eq!(default_api_key_header(), "Authorization");
    }

    // ==================== CORS Default Tests ====================

    #[test]
    fn test_default_cors_methods() {
        let methods = default_cors_methods();
        assert_eq!(methods.len(), 4);
        assert!(methods.contains(&"GET".to_string()));
        assert!(methods.contains(&"POST".to_string()));
        assert!(methods.contains(&"PUT".to_string()));
        assert!(methods.contains(&"DELETE".to_string()));
    }

    #[test]
    fn test_default_cors_headers() {
        let headers = default_cors_headers();
        assert_eq!(headers.len(), 2);
        assert!(headers.contains(&"Content-Type".to_string()));
        assert!(headers.contains(&"Authorization".to_string()));
    }

    #[test]
    fn test_default_cors_max_age() {
        assert_eq!(default_cors_max_age(), 3600);
    }

    // ==================== Metrics Default Tests ====================

    #[test]
    fn test_default_metrics_endpoint() {
        assert_eq!(default_metrics_endpoint(), "/metrics");
    }

    #[test]
    fn test_default_metrics_interval() {
        assert_eq!(default_metrics_interval(), 15);
    }

    // ==================== Sampling Default Tests ====================

    #[test]
    fn test_default_sampling_rate() {
        let rate = default_sampling_rate();
        assert!((rate - 0.1).abs() < f64::EPSILON);
    }

    // ==================== Logging Default Tests ====================

    #[test]
    fn test_default_log_level() {
        assert_eq!(default_log_level(), "info");
    }

    #[test]
    fn test_default_log_format() {
        let format = default_log_format();
        assert!(matches!(format, LogFormat::Json));
    }

    // ==================== Value Range Tests ====================

    #[test]
    fn test_timeout_is_reasonable() {
        let timeout = default_timeout_seconds();
        assert!(timeout > 0);
        assert!(timeout <= 300);
    }

    #[test]
    fn test_retries_is_reasonable() {
        let retries = default_max_retries();
        assert!(retries > 0);
        assert!(retries <= 10);
    }

    #[test]
    fn test_sampling_rate_in_range() {
        let rate = default_sampling_rate();
        assert!(rate >= 0.0);
        assert!(rate <= 1.0);
    }

    #[test]
    fn test_weight_is_positive() {
        let weight = default_weight();
        assert!(weight > 0.0);
    }

    #[test]
    fn test_pool_size_is_reasonable() {
        let size = default_pool_size();
        assert!(size > 0);
        assert!(size <= 100);
    }

    // ==================== String Format Tests ====================

    #[test]
    fn test_metrics_endpoint_starts_with_slash() {
        let endpoint = default_metrics_endpoint();
        assert!(endpoint.starts_with('/'));
    }

    #[test]
    fn test_jwt_algorithm_is_valid() {
        let algo = default_jwt_algorithm();
        let valid_algos = ["HS256", "HS384", "HS512", "RS256", "RS384", "RS512"];
        assert!(valid_algos.contains(&algo.as_str()));
    }

    #[test]
    fn test_log_level_is_valid() {
        let level = default_log_level();
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        assert!(valid_levels.contains(&level.as_str()));
    }
}
