    use super::*;
    use std::thread;
    use std::time::Duration;

    // ==================== ContentFilterLevel Tests ====================

    #[test]
    fn test_content_filter_level_default() {
        let level = ContentFilterLevel::default();
        assert!(matches!(level, ContentFilterLevel::Medium));
    }

    #[test]
    fn test_content_filter_level_variants() {
        let none = ContentFilterLevel::None;
        let low = ContentFilterLevel::Low;
        let medium = ContentFilterLevel::Medium;
        let high = ContentFilterLevel::High;
        let strict = ContentFilterLevel::Strict;

        assert!(matches!(none, ContentFilterLevel::None));
        assert!(matches!(low, ContentFilterLevel::Low));
        assert!(matches!(medium, ContentFilterLevel::Medium));
        assert!(matches!(high, ContentFilterLevel::High));
        assert!(matches!(strict, ContentFilterLevel::Strict));
    }

    #[test]
    fn test_content_filter_level_serialization() {
        let level = ContentFilterLevel::High;
        let json = serde_json::to_string(&level).unwrap();
        assert_eq!(json, "\"High\"");
    }

    #[test]
    fn test_content_filter_level_deserialization() {
        let level: ContentFilterLevel = serde_json::from_str("\"Strict\"").unwrap();
        assert!(matches!(level, ContentFilterLevel::Strict));
    }

    #[test]
    fn test_content_filter_level_clone() {
        let level = ContentFilterLevel::Low;
        let cloned = level.clone();
        assert!(matches!(cloned, ContentFilterLevel::Low));
    }

    // ==================== RoutingStrategy Tests ====================

    #[test]
    fn test_routing_strategy_default() {
        let strategy = RoutingStrategy::default();
        assert!(matches!(strategy, RoutingStrategy::RoundRobin));
    }

    #[test]
    fn test_routing_strategy_variants() {
        let round_robin = RoutingStrategy::RoundRobin;
        let least_latency = RoutingStrategy::LeastLatency;
        let least_busy = RoutingStrategy::LeastBusy;
        let cost_optimized = RoutingStrategy::CostOptimized;
        let health_based = RoutingStrategy::HealthBased;
        let weighted = RoutingStrategy::Weighted;
        let random = RoutingStrategy::Random;

        assert!(matches!(round_robin, RoutingStrategy::RoundRobin));
        assert!(matches!(least_latency, RoutingStrategy::LeastLatency));
        assert!(matches!(least_busy, RoutingStrategy::LeastBusy));
        assert!(matches!(cost_optimized, RoutingStrategy::CostOptimized));
        assert!(matches!(health_based, RoutingStrategy::HealthBased));
        assert!(matches!(weighted, RoutingStrategy::Weighted));
        assert!(matches!(random, RoutingStrategy::Random));
    }

    #[test]
    fn test_routing_strategy_serialization() {
        let strategy = RoutingStrategy::LeastLatency;
        let json = serde_json::to_string(&strategy).unwrap();
        assert_eq!(json, "\"LeastLatency\"");
    }

    #[test]
    fn test_routing_strategy_deserialization() {
        let strategy: RoutingStrategy = serde_json::from_str("\"CostOptimized\"").unwrap();
        assert!(matches!(strategy, RoutingStrategy::CostOptimized));
    }

    // ==================== CacheTier Tests ====================

    #[test]
    fn test_cache_tier_variants() {
        let memory = CacheTier::Memory;
        let redis = CacheTier::Redis;
        let database = CacheTier::Database;
        let object_storage = CacheTier::ObjectStorage;
        let semantic = CacheTier::Semantic;

        assert!(matches!(memory, CacheTier::Memory));
        assert!(matches!(redis, CacheTier::Redis));
        assert!(matches!(database, CacheTier::Database));
        assert!(matches!(object_storage, CacheTier::ObjectStorage));
        assert!(matches!(semantic, CacheTier::Semantic));
    }

    #[test]
    fn test_cache_tier_serialization() {
        let tier = CacheTier::Redis;
        let json = serde_json::to_string(&tier).unwrap();
        assert_eq!(json, "\"Redis\"");
    }

    #[test]
    fn test_cache_tier_deserialization() {
        let tier: CacheTier = serde_json::from_str("\"Semantic\"").unwrap();
        assert!(matches!(tier, CacheTier::Semantic));
    }

    // ==================== ErrorCategory Tests ====================

    #[test]
    fn test_error_category_variants() {
        let auth = ErrorCategory::Authentication;
        let authz = ErrorCategory::Authorization;
        let rate_limit = ErrorCategory::RateLimit;
        let validation = ErrorCategory::Validation;
        let provider = ErrorCategory::Provider;
        let network = ErrorCategory::Network;
        let timeout = ErrorCategory::Timeout;
        let internal = ErrorCategory::Internal;
        let config = ErrorCategory::Configuration;
        let cost = ErrorCategory::Cost;

        assert!(matches!(auth, ErrorCategory::Authentication));
        assert!(matches!(authz, ErrorCategory::Authorization));
        assert!(matches!(rate_limit, ErrorCategory::RateLimit));
        assert!(matches!(validation, ErrorCategory::Validation));
        assert!(matches!(provider, ErrorCategory::Provider));
        assert!(matches!(network, ErrorCategory::Network));
        assert!(matches!(timeout, ErrorCategory::Timeout));
        assert!(matches!(internal, ErrorCategory::Internal));
        assert!(matches!(config, ErrorCategory::Configuration));
        assert!(matches!(cost, ErrorCategory::Cost));
    }

    #[test]
    fn test_error_category_serialization() {
        let category = ErrorCategory::RateLimit;
        let json = serde_json::to_string(&category).unwrap();
        assert_eq!(json, "\"RateLimit\"");
    }

    // ==================== SecurityContext Tests ====================

    #[test]
    fn test_security_context_default() {
        let ctx = SecurityContext::default();

        assert!(!ctx.authenticated);
        assert!(ctx.roles.is_empty());
        assert!(ctx.allowed_models.is_empty());
        assert!(matches!(
            ctx.content_filter_level,
            ContentFilterLevel::Medium
        ));
        assert!(ctx.pii_detection_enabled);
        assert!(ctx.audit_tags.is_empty());
    }

    #[test]
    fn test_security_context_with_values() {
        let ctx = SecurityContext {
            authenticated: true,
            roles: vec!["admin".to_string(), "user".to_string()],
            allowed_models: vec!["gpt-4".to_string(), "claude-3".to_string()],
            content_filter_level: ContentFilterLevel::Strict,
            pii_detection_enabled: false,
            audit_tags: vec!["production".to_string()],
        };

        assert!(ctx.authenticated);
        assert_eq!(ctx.roles.len(), 2);
        assert_eq!(ctx.allowed_models.len(), 2);
        assert!(matches!(
            ctx.content_filter_level,
            ContentFilterLevel::Strict
        ));
        assert!(!ctx.pii_detection_enabled);
        assert_eq!(ctx.audit_tags.len(), 1);
    }

    #[test]
    fn test_security_context_serialization() {
        let ctx = SecurityContext::default();
        let json = serde_json::to_value(&ctx).unwrap();

        assert_eq!(json["authenticated"], false);
        assert!(json["roles"].as_array().unwrap().is_empty());
        assert_eq!(json["content_filter_level"], "Medium");
        assert_eq!(json["pii_detection_enabled"], true);
    }

    #[test]
    fn test_security_context_clone() {
        let ctx = SecurityContext {
            authenticated: true,
            roles: vec!["admin".to_string()],
            ..Default::default()
        };

        let cloned = ctx.clone();
        assert!(cloned.authenticated);
        assert_eq!(cloned.roles, vec!["admin".to_string()]);
    }

    // ==================== RoutingContext Tests ====================

    #[test]
    fn test_routing_context_default() {
        let ctx = RoutingContext::default();

        assert!(ctx.preferred_provider.is_none());
        assert!(matches!(ctx.strategy, RoutingStrategy::RoundRobin));
        assert!(ctx.fallback_providers.is_empty());
        assert!(ctx.region_preference.is_none());
        assert!(!ctx.allow_degraded);
        assert!(ctx.max_latency_ms.is_none());
    }

    #[test]
    fn test_routing_context_with_values() {
        let ctx = RoutingContext {
            preferred_provider: Some(ProviderType::OpenAI),
            strategy: RoutingStrategy::LeastLatency,
            fallback_providers: vec![ProviderType::Anthropic, ProviderType::Azure],
            region_preference: Some("us-east-1".to_string()),
            allow_degraded: true,
            max_latency_ms: Some(500.0),
        };

        assert!(ctx.preferred_provider.is_some());
        assert!(matches!(ctx.strategy, RoutingStrategy::LeastLatency));
        assert_eq!(ctx.fallback_providers.len(), 2);
        assert_eq!(ctx.region_preference, Some("us-east-1".to_string()));
        assert!(ctx.allow_degraded);
        assert_eq!(ctx.max_latency_ms, Some(500.0));
    }

    #[test]
    fn test_routing_context_serialization() {
        let ctx = RoutingContext::default();
        let json = serde_json::to_value(&ctx).unwrap();

        assert!(json["preferred_provider"].is_null());
        assert_eq!(json["strategy"], "RoundRobin");
        assert!(!json["allow_degraded"].as_bool().unwrap());
    }

    // ==================== RateLimitContext Tests ====================

    #[test]
    fn test_rate_limit_context_creation() {
        let ctx = RateLimitContext {
            key: "user_123".to_string(),
            remaining_requests: 95,
            limit: 100,
            reset_time: SystemTime::now(),
            window_seconds: 60,
        };

        assert_eq!(ctx.key, "user_123");
        assert_eq!(ctx.remaining_requests, 95);
        assert_eq!(ctx.limit, 100);
        assert_eq!(ctx.window_seconds, 60);
    }

    #[test]
    fn test_rate_limit_context_serialization() {
        let ctx = RateLimitContext {
            key: "api_key_456".to_string(),
            remaining_requests: 50,
            limit: 1000,
            reset_time: SystemTime::UNIX_EPOCH,
            window_seconds: 3600,
        };

        let json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(json["key"], "api_key_456");
        assert_eq!(json["remaining_requests"], 50);
        assert_eq!(json["limit"], 1000);
        assert_eq!(json["window_seconds"], 3600);
    }

    #[test]
    fn test_rate_limit_context_clone() {
        let ctx = RateLimitContext {
            key: "test".to_string(),
            remaining_requests: 10,
            limit: 20,
            reset_time: SystemTime::now(),
            window_seconds: 30,
        };

        let cloned = ctx.clone();
        assert_eq!(cloned.key, ctx.key);
        assert_eq!(cloned.remaining_requests, ctx.remaining_requests);
    }

    // ==================== CostContext Tests ====================

    #[test]
    fn test_cost_context_creation() {
        let ctx = CostContext {
            budget_key: "team_dev".to_string(),
            remaining_budget: 850.50,
            total_budget: 1000.0,
            currency: "USD".to_string(),
            period_end: SystemTime::now(),
        };

        assert_eq!(ctx.budget_key, "team_dev");
        assert_eq!(ctx.remaining_budget, 850.50);
        assert_eq!(ctx.total_budget, 1000.0);
        assert_eq!(ctx.currency, "USD");
    }

    #[test]
    fn test_cost_context_serialization() {
        let ctx = CostContext {
            budget_key: "user_budget".to_string(),
            remaining_budget: 99.99,
            total_budget: 100.0,
            currency: "EUR".to_string(),
            period_end: SystemTime::UNIX_EPOCH,
        };

        let json = serde_json::to_value(&ctx).unwrap();
        assert_eq!(json["budget_key"], "user_budget");
        assert_eq!(json["currency"], "EUR");
    }

    // ==================== CacheInfo Tests ====================

    #[test]
    fn test_cache_info_hit() {
        let info = CacheInfo {
            cache_key: "abc123".to_string(),
            cache_tier: CacheTier::Memory,
            hit: true,
            ttl_remaining: Some(3600),
            size_bytes: Some(1024),
        };

        assert_eq!(info.cache_key, "abc123");
        assert!(matches!(info.cache_tier, CacheTier::Memory));
        assert!(info.hit);
        assert_eq!(info.ttl_remaining, Some(3600));
        assert_eq!(info.size_bytes, Some(1024));
    }

    #[test]
    fn test_cache_info_miss() {
        let info = CacheInfo {
            cache_key: "xyz789".to_string(),
            cache_tier: CacheTier::Redis,
            hit: false,
            ttl_remaining: None,
            size_bytes: None,
        };

        assert!(!info.hit);
        assert!(info.ttl_remaining.is_none());
        assert!(info.size_bytes.is_none());
    }

    #[test]
    fn test_cache_info_serialization() {
        let info = CacheInfo {
            cache_key: "key123".to_string(),
            cache_tier: CacheTier::Semantic,
            hit: true,
            ttl_remaining: Some(600),
            size_bytes: Some(2048),
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["cache_key"], "key123");
        assert_eq!(json["cache_tier"], "Semantic");
        assert_eq!(json["hit"], true);
    }

    // ==================== RetryInfo Tests ====================

    #[test]
    fn test_retry_info_no_retries() {
        let info = RetryInfo {
            attempts: 1,
            max_attempts: 3,
            providers_tried: vec!["openai".to_string()],
            retry_errors: vec![],
            total_retry_delay_ms: 0.0,
        };

        assert_eq!(info.attempts, 1);
        assert_eq!(info.providers_tried.len(), 1);
        assert!(info.retry_errors.is_empty());
    }

    #[test]
    fn test_retry_info_with_retries() {
        let info = RetryInfo {
            attempts: 3,
            max_attempts: 5,
            providers_tried: vec![
                "openai".to_string(),
                "anthropic".to_string(),
                "azure".to_string(),
            ],
            retry_errors: vec!["rate_limit".to_string(), "timeout".to_string()],
            total_retry_delay_ms: 5500.0,
        };

        assert_eq!(info.attempts, 3);
        assert_eq!(info.providers_tried.len(), 3);
        assert_eq!(info.retry_errors.len(), 2);
        assert_eq!(info.total_retry_delay_ms, 5500.0);
    }

    #[test]
    fn test_retry_info_serialization() {
        let info = RetryInfo {
            attempts: 2,
            max_attempts: 3,
            providers_tried: vec!["openai".to_string()],
            retry_errors: vec!["error".to_string()],
            total_retry_delay_ms: 1000.0,
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["attempts"], 2);
        assert_eq!(json["max_attempts"], 3);
    }

    // ==================== CostInfo Tests ====================

    #[test]
    fn test_cost_info_creation() {
        let mut cost_breakdown = HashMap::new();
        cost_breakdown.insert("input".to_string(), 0.01);
        cost_breakdown.insert("output".to_string(), 0.02);

        let info = CostInfo {
            provider_cost: 0.03,
            currency: "USD".to_string(),
            input_tokens: 100,
            output_tokens: 50,
            cost_breakdown,
            estimated_cost: Some(0.025),
        };

        assert_eq!(info.provider_cost, 0.03);
        assert_eq!(info.currency, "USD");
        assert_eq!(info.input_tokens, 100);
        assert_eq!(info.output_tokens, 50);
        assert_eq!(info.cost_breakdown.len(), 2);
        assert_eq!(info.estimated_cost, Some(0.025));
    }

    #[test]
    fn test_cost_info_serialization() {
        let info = CostInfo {
            provider_cost: 0.05,
            currency: "EUR".to_string(),
            input_tokens: 200,
            output_tokens: 100,
            cost_breakdown: HashMap::new(),
            estimated_cost: None,
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["provider_cost"], 0.05);
        assert_eq!(json["currency"], "EUR");
        assert_eq!(json["input_tokens"], 200);
        assert_eq!(json["output_tokens"], 100);
    }

    // ==================== ResponseMetrics Tests ====================

    #[test]
    fn test_response_metrics_default() {
        let metrics = ResponseMetrics::default();

        assert_eq!(metrics.auth_time_ms, 0.0);
        assert_eq!(metrics.routing_time_ms, 0.0);
        assert_eq!(metrics.transform_request_time_ms, 0.0);
        assert_eq!(metrics.provider_call_time_ms, 0.0);
        assert_eq!(metrics.transform_response_time_ms, 0.0);
        assert_eq!(metrics.cache_time_ms, 0.0);
        assert_eq!(metrics.queue_wait_time_ms, 0.0);
        assert_eq!(metrics.total_time_ms, 0.0);
        assert!(metrics.first_byte_time_ms.is_none());
        assert!(metrics.tokens_per_second.is_none());
    }

    #[test]
    fn test_response_metrics_with_values() {
        let metrics = ResponseMetrics {
            auth_time_ms: 5.0,
            routing_time_ms: 2.0,
            transform_request_time_ms: 3.0,
            provider_call_time_ms: 150.0,
            transform_response_time_ms: 4.0,
            cache_time_ms: 1.0,
            queue_wait_time_ms: 10.0,
            total_time_ms: 175.0,
            first_byte_time_ms: Some(50.0),
            tokens_per_second: Some(100.0),
        };

        assert_eq!(metrics.auth_time_ms, 5.0);
        assert_eq!(metrics.provider_call_time_ms, 150.0);
        assert_eq!(metrics.total_time_ms, 175.0);
        assert_eq!(metrics.first_byte_time_ms, Some(50.0));
        assert_eq!(metrics.tokens_per_second, Some(100.0));
    }

    #[test]
    fn test_response_metrics_serialization() {
        let metrics = ResponseMetrics {
            total_time_ms: 100.0,
            ..Default::default()
        };

        let json = serde_json::to_value(&metrics).unwrap();
        assert_eq!(json["total_time_ms"], 100.0);
        assert_eq!(json["auth_time_ms"], 0.0);
    }

    // ==================== ErrorInfo Tests ====================

    #[test]
    fn test_error_info_creation() {
        let info = ErrorInfo {
            error_code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: "Too many requests".to_string(),
            details: Some("Retry after 60 seconds".to_string()),
            http_status: Some(429),
            provider_error_code: Some("rate_limit".to_string()),
            retryable: true,
            category: ErrorCategory::RateLimit,
        };

        assert_eq!(info.error_code, "RATE_LIMIT_EXCEEDED");
        assert_eq!(info.message, "Too many requests");
        assert_eq!(info.http_status, Some(429));
        assert!(info.retryable);
        assert!(matches!(info.category, ErrorCategory::RateLimit));
    }

    #[test]
    fn test_error_info_non_retryable() {
        let info = ErrorInfo {
            error_code: "AUTH_FAILED".to_string(),
            message: "Invalid API key".to_string(),
            details: None,
            http_status: Some(401),
            provider_error_code: None,
            retryable: false,
            category: ErrorCategory::Authentication,
        };

        assert!(!info.retryable);
        assert!(info.details.is_none());
        assert!(info.provider_error_code.is_none());
    }

    #[test]
    fn test_error_info_serialization() {
        let info = ErrorInfo {
            error_code: "TIMEOUT".to_string(),
            message: "Request timed out".to_string(),
            details: None,
            http_status: Some(504),
            provider_error_code: None,
            retryable: true,
            category: ErrorCategory::Timeout,
        };

        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["error_code"], "TIMEOUT");
        assert_eq!(json["http_status"], 504);
        assert_eq!(json["retryable"], true);
        assert_eq!(json["category"], "Timeout");
    }

    // ==================== RequestContext Tests ====================

    #[test]
    fn test_request_context_new() {
        let ctx = RequestContext::new("req-123".to_string());

        assert_eq!(ctx.request_id, "req-123");
        assert!(ctx.user_id.is_none());
        assert!(ctx.api_key_id.is_none());
        assert!(ctx.config_overrides.is_empty());
        assert!(ctx.headers.is_empty());
        assert!(ctx.query_params.is_empty());
        assert!(ctx.client_ip.is_none());
        assert!(ctx.user_agent.is_none());
        assert!(ctx.rate_limit.is_none());
        assert!(ctx.cost_context.is_none());
        assert!(!ctx.security_context.authenticated);
        assert!(matches!(
            ctx.routing_context.strategy,
            RoutingStrategy::RoundRobin
        ));
        assert!(ctx.metadata.is_empty());
        assert_eq!(ctx.priority, 128);
        assert!(ctx.timeout_ms.is_none());
        assert!(!ctx.debug_mode);
    }

    #[test]
    fn test_request_context_elapsed_ms() {
        let ctx = RequestContext::new("req-456".to_string());

        // Allow some time to pass
        thread::sleep(Duration::from_millis(10));

        let elapsed = ctx.elapsed_ms();
        assert!(elapsed >= 10.0);
    }

    #[test]
    fn test_request_context_is_timed_out_false() {
        let ctx = RequestContext {
            timeout_ms: Some(10000), // 10 seconds
            ..RequestContext::new("req-789".to_string())
        };

        assert!(!ctx.is_timed_out());
    }

    #[test]
    fn test_request_context_is_timed_out_no_timeout() {
        let ctx = RequestContext::new("req-abc".to_string());

        // No timeout set should never timeout
        assert!(!ctx.is_timed_out());
    }

    #[test]
    fn test_request_context_add_metadata() {
        let mut ctx = RequestContext::new("req-def".to_string());

        ctx.add_metadata("key1".to_string(), "value1".to_string());
        ctx.add_metadata("key2".to_string(), "value2".to_string());

        assert_eq!(ctx.metadata.len(), 2);
        assert_eq!(ctx.metadata.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_request_context_get_metadata() {
        let mut ctx = RequestContext::new("req-ghi".to_string());
        ctx.add_metadata("test_key".to_string(), "test_value".to_string());

        assert_eq!(
            ctx.get_metadata("test_key"),
            Some(&"test_value".to_string())
        );
        assert_eq!(ctx.get_metadata("nonexistent"), None);
    }

    #[test]
    fn test_request_context_with_full_config() {
        let mut headers = HashMap::new();
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let ctx = RequestContext {
            request_id: "req-full".to_string(),
            user_id: Some("user_123".to_string()),
            api_key_id: Some("key_456".to_string()),
            timestamp: SystemTime::now(),
            start_time: Instant::now(),
            config_overrides: HashMap::new(),
            headers,
            query_params: HashMap::new(),
            client_ip: Some("192.168.1.1".to_string()),
            user_agent: Some("Mozilla/5.0".to_string()),
            rate_limit: None,
            cost_context: None,
            security_context: SecurityContext {
                authenticated: true,
                roles: vec!["admin".to_string()],
                ..Default::default()
            },
            routing_context: RoutingContext {
                strategy: RoutingStrategy::LeastLatency,
                ..Default::default()
            },
            metadata: HashMap::new(),
            priority: 255,
            timeout_ms: Some(30000),
            debug_mode: true,
        };

        assert_eq!(ctx.user_id, Some("user_123".to_string()));
        assert_eq!(ctx.client_ip, Some("192.168.1.1".to_string()));
        assert!(ctx.security_context.authenticated);
        assert_eq!(ctx.priority, 255);
        assert!(ctx.debug_mode);
    }

    #[test]
    fn test_request_context_clone() {
        let mut ctx = RequestContext::new("req-clone".to_string());
        ctx.add_metadata("key".to_string(), "value".to_string());
        ctx.debug_mode = true;

        let cloned = ctx.clone();
        assert_eq!(cloned.request_id, "req-clone");
        assert_eq!(cloned.get_metadata("key"), Some(&"value".to_string()));
        assert!(cloned.debug_mode);
    }

    // ==================== ResponseContext Tests ====================

    #[test]
    fn test_response_context_from_request() {
        let req_ctx = RequestContext::new("req-response".to_string());

        // Small delay to ensure elapsed time > 0
        thread::sleep(Duration::from_millis(1));

        let resp_ctx =
            ResponseContext::from_request(req_ctx, "openai-1".to_string(), ProviderType::OpenAI);

        assert_eq!(resp_ctx.request_context.request_id, "req-response");
        assert_eq!(resp_ctx.provider_id, "openai-1");
        assert!(matches!(resp_ctx.provider_type, ProviderType::OpenAI));
        assert!(!resp_ctx.from_cache);
        assert!(resp_ctx.cache_info.is_none());
        assert!(resp_ctx.retry_info.is_none());
        assert!(resp_ctx.cost_info.is_none());
        assert!(resp_ctx.warnings.is_empty());
        assert!(resp_ctx.error_info.is_none());
        assert!(resp_ctx.execution_time_ms >= 0.0);
    }

    #[test]
    fn test_response_context_add_warning() {
        let req_ctx = RequestContext::new("req-warn".to_string());
        let mut resp_ctx = ResponseContext::from_request(
            req_ctx,
            "anthropic-1".to_string(),
            ProviderType::Anthropic,
        );

        resp_ctx.add_warning("Deprecated model version".to_string());
        resp_ctx.add_warning("Rate limit approaching".to_string());

        assert_eq!(resp_ctx.warnings.len(), 2);
        assert_eq!(resp_ctx.warnings[0], "Deprecated model version");
        assert_eq!(resp_ctx.warnings[1], "Rate limit approaching");
    }

    #[test]
    fn test_response_context_set_error() {
        let req_ctx = RequestContext::new("req-error".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "azure-1".to_string(), ProviderType::Azure);

        let error_info = ErrorInfo {
            error_code: "PROVIDER_ERROR".to_string(),
            message: "Provider returned error".to_string(),
            details: None,
            http_status: Some(500),
            provider_error_code: None,
            retryable: true,
            category: ErrorCategory::Provider,
        };

        resp_ctx.set_error(error_info);

        assert!(resp_ctx.error_info.is_some());
        assert_eq!(
            resp_ctx.error_info.as_ref().unwrap().error_code,
            "PROVIDER_ERROR"
        );
    }

    #[test]
    fn test_response_context_has_error() {
        let req_ctx = RequestContext::new("req-has-error".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "gemini-1".to_string(), ProviderType::VertexAI);

        assert!(!resp_ctx.has_error());

        resp_ctx.set_error(ErrorInfo {
            error_code: "TEST".to_string(),
            message: "Test error".to_string(),
            details: None,
            http_status: None,
            provider_error_code: None,
            retryable: false,
            category: ErrorCategory::Internal,
        });

        assert!(resp_ctx.has_error());
    }

    #[test]
    fn test_response_context_with_cache() {
        let req_ctx = RequestContext::new("req-cache".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "openai-2".to_string(), ProviderType::OpenAI);

        resp_ctx.from_cache = true;
        resp_ctx.cache_info = Some(CacheInfo {
            cache_key: "cache-key-123".to_string(),
            cache_tier: CacheTier::Redis,
            hit: true,
            ttl_remaining: Some(1800),
            size_bytes: Some(512),
        });

        assert!(resp_ctx.from_cache);
        assert!(resp_ctx.cache_info.is_some());
        assert!(resp_ctx.cache_info.as_ref().unwrap().hit);
    }

    #[test]
    fn test_response_context_with_retries() {
        let req_ctx = RequestContext::new("req-retry".to_string());
        let mut resp_ctx = ResponseContext::from_request(
            req_ctx,
            "anthropic-2".to_string(),
            ProviderType::Anthropic,
        );

        resp_ctx.retry_info = Some(RetryInfo {
            attempts: 2,
            max_attempts: 3,
            providers_tried: vec!["openai".to_string(), "anthropic".to_string()],
            retry_errors: vec!["rate_limit".to_string()],
            total_retry_delay_ms: 2000.0,
        });

        assert!(resp_ctx.retry_info.is_some());
        assert_eq!(resp_ctx.retry_info.as_ref().unwrap().attempts, 2);
    }

    #[test]
    fn test_response_context_with_cost() {
        let req_ctx = RequestContext::new("req-cost".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "openai-3".to_string(), ProviderType::OpenAI);

        resp_ctx.cost_info = Some(CostInfo {
            provider_cost: 0.015,
            currency: "USD".to_string(),
            input_tokens: 500,
            output_tokens: 200,
            cost_breakdown: HashMap::new(),
            estimated_cost: Some(0.012),
        });

        assert!(resp_ctx.cost_info.is_some());
        assert_eq!(resp_ctx.cost_info.as_ref().unwrap().provider_cost, 0.015);
    }

    #[test]
    fn test_response_context_with_metrics() {
        let req_ctx = RequestContext::new("req-metrics".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "gemini-2".to_string(), ProviderType::VertexAI);

        resp_ctx.metrics = ResponseMetrics {
            auth_time_ms: 2.0,
            routing_time_ms: 1.0,
            transform_request_time_ms: 3.0,
            provider_call_time_ms: 120.0,
            transform_response_time_ms: 2.0,
            cache_time_ms: 0.5,
            queue_wait_time_ms: 5.0,
            total_time_ms: 133.5,
            first_byte_time_ms: Some(80.0),
            tokens_per_second: Some(150.0),
        };

        assert_eq!(resp_ctx.metrics.total_time_ms, 133.5);
        assert_eq!(resp_ctx.metrics.tokens_per_second, Some(150.0));
    }

    #[test]
    fn test_response_context_clone() {
        let req_ctx = RequestContext::new("req-clone-resp".to_string());
        let mut resp_ctx =
            ResponseContext::from_request(req_ctx, "provider-1".to_string(), ProviderType::OpenAI);
        resp_ctx.add_warning("Test warning".to_string());

        let cloned = resp_ctx.clone();
        assert_eq!(cloned.provider_id, "provider-1");
        assert_eq!(cloned.warnings.len(), 1);
    }

    // ==================== Debug Format Tests ====================

    #[test]
    fn test_request_context_debug() {
        let ctx = RequestContext::new("debug-test".to_string());
        let debug = format!("{:?}", ctx);
        assert!(debug.contains("RequestContext"));
        assert!(debug.contains("debug-test"));
    }

    #[test]
    fn test_response_context_debug() {
        let req_ctx = RequestContext::new("debug-resp".to_string());
        let resp_ctx = ResponseContext::from_request(
            req_ctx,
            "debug-provider".to_string(),
            ProviderType::OpenAI,
        );
        let debug = format!("{:?}", resp_ctx);
        assert!(debug.contains("ResponseContext"));
        assert!(debug.contains("debug-provider"));
    }

    #[test]
    fn test_error_info_debug() {
        let info = ErrorInfo {
            error_code: "DEBUG_ERROR".to_string(),
            message: "Debug error message".to_string(),
            details: None,
            http_status: Some(400),
            provider_error_code: None,
            retryable: false,
            category: ErrorCategory::Validation,
        };
        let debug = format!("{:?}", info);
        assert!(debug.contains("ErrorInfo"));
        assert!(debug.contains("DEBUG_ERROR"));
    }
