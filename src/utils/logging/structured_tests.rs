use super::*;

// ==================== LogContext Tests ====================

#[test]
fn test_log_context_new() {
    let context = LogContext::new();
    assert!(context.request_id.is_none());
    assert!(context.user_id.is_none());
    assert!(context.organization_id.is_none());
    assert!(context.api_key.is_none());
    assert!(context.model.is_none());
    assert!(context.provider.is_none());
    assert!(context.extra.is_empty());
}

#[test]
fn test_log_context_default() {
    let context = LogContext::default();
    assert!(context.request_id.is_none());
    assert!(context.extra.is_empty());
}

#[test]
fn test_log_context_with_request_id() {
    let request_id = RequestId::new();
    let context = LogContext::new().with_request_id(request_id.clone());
    assert!(context.request_id.is_some());
    assert_eq!(context.request_id.unwrap().as_str(), request_id.as_str());
}

#[test]
fn test_log_context_with_user_id() {
    let user_id = UserId::new();
    let context = LogContext::new().with_user_id(user_id);
    assert!(context.user_id.is_some());
}

#[test]
fn test_log_context_with_organization_id() {
    let org_id = Uuid::new_v4();
    let context = LogContext::new().with_organization_id(org_id);
    assert_eq!(context.organization_id, Some(org_id));
}

#[test]
fn test_log_context_with_model() {
    let model = ModelName::new("gpt-4".to_string());
    let context = LogContext::new().with_model(model.clone());
    assert!(context.model.is_some());
    assert_eq!(context.model.unwrap().as_str(), "gpt-4");
}

#[test]
fn test_log_context_with_provider() {
    let context = LogContext::new().with_provider("openai".to_string());
    assert_eq!(context.provider, Some("openai".to_string()));
}

#[test]
fn test_log_context_with_field() {
    let context = LogContext::new()
        .with_field("test_field", "test_value")
        .with_field("number_field", 42);

    assert!(context.extra.contains_key("test_field"));
    assert!(context.extra.contains_key("number_field"));
}

#[test]
fn test_log_context_with_multiple_fields() {
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_user_id(UserId::new())
        .with_provider("anthropic".to_string())
        .with_field("custom", "value");

    assert!(context.request_id.is_some());
    assert!(context.user_id.is_some());
    assert_eq!(context.provider, Some("anthropic".to_string()));
    assert!(context.extra.contains_key("custom"));
}

#[test]
fn test_log_context_context_fields_empty() {
    let context = LogContext::new();
    let fields = context.context_fields();
    assert!(fields.is_empty());
}

#[test]
fn test_log_context_context_fields_with_request_id() {
    let context = LogContext::new().with_request_id(RequestId::new());
    let fields = context.context_fields();
    assert!(fields.contains("request_id="));
}

#[test]
fn test_log_context_context_fields_with_model() {
    let model = ModelName::new("claude-3".to_string());
    let context = LogContext::new().with_model(model);
    let fields = context.context_fields();
    assert!(fields.contains("model=claude-3"));
}

#[test]
fn test_log_context_context_fields_with_provider() {
    let context = LogContext::new().with_provider("azure".to_string());
    let fields = context.context_fields();
    assert!(fields.contains("provider=azure"));
}

#[test]
fn test_log_context_context_fields_multiple() {
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_model(ModelName::new("gpt-4".to_string()))
        .with_provider("openai".to_string());

    let fields = context.context_fields();
    assert!(fields.contains("request_id="));
    assert!(fields.contains("model=gpt-4"));
    assert!(fields.contains("provider=openai"));
    assert!(fields.contains(", ")); // Fields should be comma-separated
}

#[test]
fn test_log_context_serialization() {
    let context = LogContext::new()
        .with_provider("test".to_string())
        .with_field("key", "value");

    let json = serde_json::to_string(&context).unwrap();
    assert!(json.contains("provider"));
    assert!(json.contains("test"));
}

// ==================== PerformanceMetrics Tests ====================

#[test]
fn test_performance_metrics_new() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(100));
    assert_eq!(metrics.duration_ms, 100);
    assert!(metrics.memory_bytes.is_none());
    assert!(metrics.db_queries.is_none());
    assert!(metrics.cache_hits.is_none());
    assert!(metrics.cache_misses.is_none());
    assert!(metrics.tokens_used.is_none());
    assert!(metrics.cost_usd.is_none());
}

#[test]
fn test_performance_metrics_with_memory() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(50)).with_memory(2048);
    assert_eq!(metrics.memory_bytes, Some(2048));
}

#[test]
fn test_performance_metrics_with_db_queries() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(50)).with_db_queries(10);
    assert_eq!(metrics.db_queries, Some(10));
}

#[test]
fn test_performance_metrics_with_cache_stats() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(50)).with_cache_stats(100, 20);
    assert_eq!(metrics.cache_hits, Some(100));
    assert_eq!(metrics.cache_misses, Some(20));
}

#[test]
fn test_performance_metrics_with_tokens() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(50)).with_tokens(1500);
    assert_eq!(metrics.tokens_used, Some(1500));
}

#[test]
fn test_performance_metrics_with_cost() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(50)).with_cost(0.05);
    assert_eq!(metrics.cost_usd, Some(0.05));
}

#[test]
fn test_performance_metrics_full() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(100))
        .with_memory(1024)
        .with_db_queries(5)
        .with_cache_stats(10, 2)
        .with_tokens(150)
        .with_cost(0.001);

    assert_eq!(metrics.duration_ms, 100);
    assert_eq!(metrics.memory_bytes, Some(1024));
    assert_eq!(metrics.db_queries, Some(5));
    assert_eq!(metrics.cache_hits, Some(10));
    assert_eq!(metrics.cache_misses, Some(2));
    assert_eq!(metrics.tokens_used, Some(150));
    assert_eq!(metrics.cost_usd, Some(0.001));
}

#[test]
fn test_performance_metrics_zero_duration() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(0));
    assert_eq!(metrics.duration_ms, 0);
}

#[test]
fn test_performance_metrics_large_values() {
    let metrics = PerformanceMetrics::new(Duration::from_secs(3600))
        .with_memory(1024 * 1024 * 1024)
        .with_tokens(1_000_000);

    assert_eq!(metrics.duration_ms, 3_600_000);
    assert_eq!(metrics.memory_bytes, Some(1024 * 1024 * 1024));
    assert_eq!(metrics.tokens_used, Some(1_000_000));
}

#[test]
fn test_performance_metrics_clone() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(100))
        .with_memory(512)
        .with_cost(0.01);

    let cloned = metrics.clone();
    assert_eq!(metrics.duration_ms, cloned.duration_ms);
    assert_eq!(metrics.memory_bytes, cloned.memory_bytes);
    assert_eq!(metrics.cost_usd, cloned.cost_usd);
}

#[test]
fn test_performance_metrics_serialization() {
    let metrics = PerformanceMetrics::new(Duration::from_millis(100))
        .with_memory(1024)
        .with_tokens(500);

    let json = serde_json::to_string(&metrics).unwrap();
    assert!(json.contains("duration_ms"));
    assert!(json.contains("100"));
    assert!(json.contains("memory_bytes"));
    assert!(json.contains("tokens_used"));
}

// ==================== StructuredLogger Tests ====================

#[test]
fn test_structured_logger_new() {
    let context = LogContext::new();
    let _logger = StructuredLogger::new(context);
    // Just verify it doesn't panic
}

#[test]
fn test_structured_logger_with_context() {
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_provider("test".to_string());
    let logger = StructuredLogger::new(context);

    // These would normally log to the configured tracing subscriber
    logger.info("Test info message");
    logger.debug("Test debug message");
}

#[test]
fn test_structured_logger_info() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.info("Test info message");
}

#[test]
fn test_structured_logger_warn() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.warn("Test warning message");
}

#[test]
fn test_structured_logger_error_without_error() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.error("Test error message", None);
}

#[test]
fn test_structured_logger_error_with_error() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    let error = std::io::Error::other("test error");
    logger.error("Test error message", Some(&error));
}

#[test]
fn test_structured_logger_debug() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.debug("Test debug message");
}

#[test]
fn test_structured_logger_performance() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    let metrics = PerformanceMetrics::new(Duration::from_millis(50));
    logger.performance("test_op", metrics);
}

#[test]
fn test_structured_logger_api_request() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.api_request(
        "POST",
        "/v1/chat/completions",
        200,
        Duration::from_millis(150),
    );
}

#[test]
fn test_structured_logger_database_operation() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.database_operation("SELECT", "users", Duration::from_millis(5), Some(10));
}

#[test]
fn test_structured_logger_database_operation_no_rows() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.database_operation("DELETE", "sessions", Duration::from_millis(2), None);
}

#[test]
fn test_structured_logger_cache_operation_hit() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.cache_operation("GET", "user:123", true, Duration::from_micros(100));
}

#[test]
fn test_structured_logger_cache_operation_miss() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.cache_operation("GET", "user:456", false, Duration::from_micros(50));
}

#[test]
fn test_structured_logger_provider_interaction() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.provider_interaction(
        "openai",
        "gpt-4",
        Some(1500),
        Some(0.045),
        Duration::from_millis(2000),
    );
}

#[test]
fn test_structured_logger_provider_interaction_no_optionals() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    logger.provider_interaction(
        "anthropic",
        "claude-3",
        None,
        None,
        Duration::from_millis(500),
    );
}

// ==================== Timer Tests ====================

#[test]
fn test_timer_start_and_stop() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    let timer = Timer::start("test_operation".to_string(), logger);

    // Simulate some work
    std::thread::sleep(Duration::from_millis(1));

    timer.stop();
}

#[test]
fn test_timer_immediate_stop() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    let timer = Timer::start("instant_operation".to_string(), logger);
    timer.stop();
}

#[test]
fn test_timer_stop_with_metrics() {
    let context = LogContext::new();
    let logger = StructuredLogger::new(context);
    let timer = Timer::start("metered_operation".to_string(), logger);

    std::thread::sleep(Duration::from_millis(5));

    let metrics = PerformanceMetrics::new(Duration::from_millis(5))
        .with_tokens(100)
        .with_cost(0.001);

    timer.stop_with_metrics(metrics);
}

#[test]
fn test_timer_with_context() {
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_provider("openai".to_string());
    let logger = StructuredLogger::new(context);
    let timer = Timer::start("contextualized_operation".to_string(), logger);
    timer.stop();
}

// ==================== Integration Tests ====================

#[test]
fn test_full_logging_workflow() {
    // Create context
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_user_id(UserId::new())
        .with_provider("openai".to_string())
        .with_model(ModelName::new("gpt-4".to_string()));

    // Create logger
    let logger = StructuredLogger::new(context);

    // Log various events
    logger.info("Starting request processing");
    logger.cache_operation("GET", "cache:key", false, Duration::from_micros(50));
    logger.database_operation("SELECT", "users", Duration::from_millis(5), Some(1));
    logger.provider_interaction(
        "openai",
        "gpt-4",
        Some(1000),
        Some(0.03),
        Duration::from_millis(1500),
    );
    logger.api_request(
        "POST",
        "/v1/chat/completions",
        200,
        Duration::from_millis(1600),
    );
}

#[test]
fn test_timer_with_full_metrics() {
    let context = LogContext::new()
        .with_request_id(RequestId::new())
        .with_field("operation_type", "completion");

    let logger = StructuredLogger::new(context);
    let timer = Timer::start("full_completion_request".to_string(), logger);

    std::thread::sleep(Duration::from_millis(10));

    let metrics = PerformanceMetrics::new(Duration::from_millis(10))
        .with_memory(1024 * 100)
        .with_db_queries(3)
        .with_cache_stats(2, 1)
        .with_tokens(500)
        .with_cost(0.015);

    timer.stop_with_metrics(metrics);
}
