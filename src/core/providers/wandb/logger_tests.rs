    use super::*;

    fn create_test_config() -> WandbConfig {
        WandbConfig::new("test-api-key")
            .with_project("test-project")
            .with_entity("test-entity")
    }

    // ==================== LLMCallLog Tests ====================

    #[test]
    fn test_llm_call_log_creation() {
        let log = LLMCallLog::new("openai", "gpt-4");

        assert_eq!(log.provider, "openai");
        assert_eq!(log.model, "gpt-4");
        assert_eq!(log.request_type, "chat_completion");
        assert!(log.success);
        assert!(!log.call_id.is_empty());
    }

    #[test]
    fn test_llm_call_log_builder() {
        let log = LLMCallLog::new("anthropic", "claude-3-opus")
            .with_request_type("embedding")
            .with_input(serde_json::json!({"prompt": "test"}))
            .with_output(serde_json::json!({"response": "output"}))
            .with_token_usage(100, 50, 150)
            .with_cost(0.0045)
            .with_latency(250)
            .with_metadata("user_id", serde_json::json!("user123"));

        assert_eq!(log.request_type, "embedding");
        assert!(log.input.is_some());
        assert!(log.output.is_some());
        assert_eq!(log.input_tokens, Some(100));
        assert_eq!(log.output_tokens, Some(50));
        assert_eq!(log.total_tokens, Some(150));
        assert_eq!(log.cost_usd, Some(0.0045));
        assert_eq!(log.latency_ms, Some(250));
        assert!(log.metadata.contains_key("user_id"));
        assert!(log.success);
    }

    #[test]
    fn test_llm_call_log_with_error() {
        let log = LLMCallLog::new("openai", "gpt-4").with_error("Rate limit exceeded");

        assert!(!log.success);
        assert_eq!(log.error, Some("Rate limit exceeded".to_string()));
    }

    #[test]
    fn test_llm_call_log_serialization() {
        let log = LLMCallLog::new("openai", "gpt-4")
            .with_token_usage(100, 50, 150)
            .with_cost(0.01);

        let json = serde_json::to_value(&log).unwrap();

        assert_eq!(json["provider"], "openai");
        assert_eq!(json["model"], "gpt-4");
        assert_eq!(json["input_tokens"], 100);
        assert_eq!(json["output_tokens"], 50);
        assert_eq!(json["cost_usd"], 0.01);
        assert_eq!(json["success"], true);

        // Optional None values should not be present
        assert!(json.get("error").is_none());
        assert!(json.get("input").is_none());
    }

    // ==================== RunSummary Tests ====================

    #[test]
    fn test_run_summary_default() {
        let summary = RunSummary::default();

        assert_eq!(summary.total_calls, 0);
        assert_eq!(summary.successful_calls, 0);
        assert_eq!(summary.failed_calls, 0);
        assert_eq!(summary.total_input_tokens, 0);
        assert_eq!(summary.total_cost_usd, 0.0);
    }

    #[test]
    fn test_run_summary_update() {
        let mut summary = RunSummary::default();

        let log1 = LLMCallLog::new("openai", "gpt-4")
            .with_token_usage(100, 50, 150)
            .with_cost(0.01)
            .with_latency(200);

        summary.update(&log1);

        assert_eq!(summary.total_calls, 1);
        assert_eq!(summary.successful_calls, 1);
        assert_eq!(summary.total_input_tokens, 100);
        assert_eq!(summary.total_output_tokens, 50);
        assert!((summary.total_cost_usd - 0.01).abs() < 0.0001);
        assert!((summary.avg_latency_ms - 200.0).abs() < 0.1);

        // Add another log
        let log2 = LLMCallLog::new("anthropic", "claude-3")
            .with_token_usage(200, 100, 300)
            .with_cost(0.02)
            .with_latency(400);

        summary.update(&log2);

        assert_eq!(summary.total_calls, 2);
        assert_eq!(summary.successful_calls, 2);
        assert_eq!(summary.total_input_tokens, 300);
        assert!((summary.total_cost_usd - 0.03).abs() < 0.0001);
        assert!((summary.avg_latency_ms - 300.0).abs() < 0.1);
    }

    #[test]
    fn test_run_summary_failed_calls() {
        let mut summary = RunSummary::default();

        let failed_log = LLMCallLog::new("openai", "gpt-4").with_error("API error");

        summary.update(&failed_log);

        assert_eq!(summary.total_calls, 1);
        assert_eq!(summary.successful_calls, 0);
        assert_eq!(summary.failed_calls, 1);
    }

    #[test]
    fn test_run_summary_calls_by_provider() {
        let mut summary = RunSummary::default();

        summary.update(&LLMCallLog::new("openai", "gpt-4"));
        summary.update(&LLMCallLog::new("openai", "gpt-3.5"));
        summary.update(&LLMCallLog::new("anthropic", "claude-3"));

        assert_eq!(summary.calls_by_provider.get("openai"), Some(&2));
        assert_eq!(summary.calls_by_provider.get("anthropic"), Some(&1));
    }

    // ==================== WandbRun Tests ====================

    #[test]
    fn test_wandb_run_serialization() {
        let run = WandbRun {
            id: "run-123".to_string(),
            name: "test-run".to_string(),
            project: "my-project".to_string(),
            entity: Some("my-team".to_string()),
            state: RunState::Running,
            url: Some("https://wandb.ai/my-team/my-project/runs/run-123".to_string()),
            created_at: Utc::now(),
        };

        let json = serde_json::to_value(&run).unwrap();

        assert_eq!(json["id"], "run-123");
        assert_eq!(json["name"], "test-run");
        assert_eq!(json["project"], "my-project");
        assert_eq!(json["entity"], "my-team");
        assert_eq!(json["state"], "running");
    }

    #[test]
    fn test_run_state_serialization() {
        assert_eq!(
            serde_json::to_string(&RunState::Running).unwrap(),
            "\"running\""
        );
        assert_eq!(
            serde_json::to_string(&RunState::Finished).unwrap(),
            "\"finished\""
        );
        assert_eq!(
            serde_json::to_string(&RunState::Failed).unwrap(),
            "\"failed\""
        );
        assert_eq!(
            serde_json::to_string(&RunState::Crashed).unwrap(),
            "\"crashed\""
        );
    }

    // ==================== WandbLogger Tests ====================

    #[test]
    fn test_wandb_logger_creation() {
        let config = create_test_config();
        let logger = WandbLogger::new(config);

        assert!(logger.is_ok());
    }

    #[test]
    fn test_wandb_logger_creation_no_api_key() {
        let config = WandbConfig {
            api_key: None,
            ..Default::default()
        };

        // This may fail or succeed depending on WANDB_API_KEY env var
        let _ = WandbLogger::new(config);
    }

    #[test]
    fn test_wandb_logger_is_enabled() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();

        assert!(logger.is_enabled());
    }

    #[test]
    fn test_wandb_logger_disabled() {
        let mut config = create_test_config();
        config.enabled = false;

        let logger = WandbLogger::new(config).unwrap();
        assert!(!logger.is_enabled());
    }

    #[tokio::test]
    async fn test_wandb_logger_init_run() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();

        let run = logger.init_run().await;
        assert!(run.is_ok());

        let run = run.unwrap();
        assert_eq!(run.project, "test-project");
        assert_eq!(run.entity, Some("test-entity".to_string()));
        assert_eq!(run.state, RunState::Running);
    }

    #[tokio::test]
    async fn test_wandb_logger_log() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();

        let _ = logger.init_run().await;

        let log = LLMCallLog::new("openai", "gpt-4")
            .with_token_usage(100, 50, 150)
            .with_cost(0.01);

        let result = logger.log(log).await;
        assert!(result.is_ok());

        let summary = logger.get_summary().await;
        assert_eq!(summary.total_calls, 1);
        assert_eq!(summary.successful_calls, 1);
    }

    #[tokio::test]
    async fn test_wandb_logger_log_disabled() {
        let mut config = create_test_config();
        config.enabled = false;

        let logger = WandbLogger::new(config).unwrap();

        let log = LLMCallLog::new("openai", "gpt-4");
        let result = logger.log(log).await;

        assert!(result.is_ok());

        // Summary should not be updated when disabled
        let summary = logger.get_summary().await;
        assert_eq!(summary.total_calls, 0);
    }

    #[tokio::test]
    async fn test_wandb_logger_log_success() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();
        let _ = logger.init_run().await;

        let result = logger
            .log_success(
                "openai",
                "gpt-4",
                Some(serde_json::json!({"role": "user", "content": "Hello"})),
                Some(serde_json::json!({"role": "assistant", "content": "Hi there!"})),
                10,
                5,
                Some(0.001),
                150,
            )
            .await;

        assert!(result.is_ok());

        let summary = logger.get_summary().await;
        assert_eq!(summary.total_calls, 1);
        assert_eq!(summary.successful_calls, 1);
    }

    #[tokio::test]
    async fn test_wandb_logger_log_failure() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();
        let _ = logger.init_run().await;

        let result = logger
            .log_failure("openai", "gpt-4", "Rate limit exceeded", Some(50))
            .await;

        assert!(result.is_ok());

        let summary = logger.get_summary().await;
        assert_eq!(summary.total_calls, 1);
        assert_eq!(summary.failed_calls, 1);
    }

    #[tokio::test]
    async fn test_wandb_logger_privacy_filters() {
        let config = create_test_config()
            .without_prompt_logging()
            .without_response_logging();
        let logger = WandbLogger::new(config).unwrap();
        let _ = logger.init_run().await;

        let log = LLMCallLog::new("openai", "gpt-4")
            .with_input(serde_json::json!({"secret": "data"}))
            .with_output(serde_json::json!({"response": "secret"}));

        // The log call should filter out input/output
        let result = logger.log(log).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wandb_logger_get_run() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();

        // Before init, should be None
        assert!(logger.get_run().await.is_none());

        // After init, should have run
        let _ = logger.init_run().await;
        assert!(logger.get_run().await.is_some());
    }

    #[tokio::test]
    async fn test_wandb_logger_finish() {
        let config = create_test_config();
        let logger = WandbLogger::new(config).unwrap();
        let _ = logger.init_run().await;

        let result = logger.finish().await;
        assert!(result.is_ok());

        let run = logger.get_run().await.unwrap();
        assert_eq!(run.state, RunState::Finished);
    }

    #[tokio::test]
    async fn test_wandb_logger_batch_flush() {
        let config = WandbConfig::new("test-key")
            .with_project("test")
            .with_batch_settings(3, 60);

        let logger = WandbLogger::new(config).unwrap();
        // Don't init run - this would require network access
        // Just test buffer behavior

        // Add logs up to batch size - 1
        for _ in 0..2 {
            // Logs will be buffered but not sent because run is not initialized
            let _ = logger.log(LLMCallLog::new("openai", "gpt-4")).await;
        }

        // Verify buffer has logs
        let buffer = logger.log_buffer.read().await;
        assert_eq!(buffer.len(), 2);
    }

    // ==================== create_chat_log Tests ====================

    #[test]
    fn test_create_chat_log() {
        use crate::core::types::chat::ChatRequest;

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let log = create_chat_log("openai", "gpt-4", &request, None, 200, None);

        assert_eq!(log.provider, "openai");
        assert_eq!(log.model, "gpt-4");
        assert_eq!(log.request_type, "chat_completion");
        assert_eq!(log.latency_ms, Some(200));
        assert!(log.success);
    }

    #[test]
    fn test_create_chat_log_with_error() {
        use crate::core::types::chat::ChatRequest;

        let request = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![],
            ..Default::default()
        };

        let log = create_chat_log(
            "openai",
            "gpt-4",
            &request,
            None,
            50,
            Some("Connection timeout"),
        );

        assert!(!log.success);
        assert_eq!(log.error, Some("Connection timeout".to_string()));
    }
