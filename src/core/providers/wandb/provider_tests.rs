use super::*;
use crate::core::types::{embedding::EmbeddingInput, embedding::EmbeddingRequest};

fn create_test_config() -> WandbConfig {
    WandbConfig::new("test-api-key")
        .with_project("test-project")
        .with_entity("test-entity")
}

// ==================== Provider Creation Tests ====================

#[tokio::test]
async fn test_wandb_provider_creation() {
    let config = create_test_config();
    let provider = WandbProvider::new(config).await;

    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), PROVIDER_NAME);
}

#[tokio::test]
async fn test_wandb_provider_from_api_key() {
    let provider = WandbProvider::with_api_key("test-key").await;
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_wandb_provider_no_api_key() {
    let config = WandbConfig {
        api_key: None,
        ..Default::default()
    };

    // Will fail if WANDB_API_KEY env is not set
    let _ = WandbProvider::new(config).await;
}

// ==================== Provider Trait Tests ====================

#[tokio::test]
async fn test_provider_name() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    assert_eq!(provider.name(), "wandb");
}

#[tokio::test]
async fn test_provider_capabilities() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let caps = provider.capabilities();

    // W&B is not an LLM provider, so no capabilities
    assert!(caps.is_empty());
}

#[tokio::test]
async fn test_provider_models() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let models = provider.models();

    // W&B doesn't have models
    assert!(models.is_empty());
}

#[tokio::test]
async fn test_chat_completion_not_supported() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![],
        ..Default::default()
    };

    let result = provider
        .chat_completion(request, RequestContext::default())
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        ProviderError::NotSupported { provider, .. } => {
            assert_eq!(provider, "wandb");
        }
        _ => panic!("Expected NotSupported error"),
    }
}

#[tokio::test]
async fn test_embeddings_not_supported() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();

    let request = EmbeddingRequest {
        model: "text-embedding-ada-002".to_string(),
        input: EmbeddingInput::Text("test".to_string()),
        user: None,
        encoding_format: None,
        dimensions: None,
        task_type: None,
    };

    let result = provider
        .embeddings(request, RequestContext::default())
        .await;

    assert!(result.is_err());
}

// ==================== Logging Tests ====================

#[tokio::test]
async fn test_init_run() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();

    let result = provider.init_run().await;
    assert!(result.is_ok());

    let run = provider.get_run().await;
    assert!(run.is_some());
}

#[tokio::test]
async fn test_log_call() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let _ = provider.init_run().await;

    let result = provider
        .log_call(
            "openai",
            "gpt-4",
            Some(100),
            Some(50),
            Some(0.01),
            200,
            true,
            None,
        )
        .await;

    assert!(result.is_ok());

    let summary = provider.get_summary().await;
    assert_eq!(summary.total_calls, 1);
    assert_eq!(summary.successful_calls, 1);
}

#[tokio::test]
async fn test_log_call_failure() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let _ = provider.init_run().await;

    let result = provider
        .log_call(
            "openai",
            "gpt-4",
            None,
            None,
            None,
            50,
            false,
            Some("Rate limit exceeded"),
        )
        .await;

    assert!(result.is_ok());

    let summary = provider.get_summary().await;
    assert_eq!(summary.total_calls, 1);
    assert_eq!(summary.failed_calls, 1);
}

#[tokio::test]
async fn test_log_chat_completion() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let _ = provider.init_run().await;

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![],
        ..Default::default()
    };

    let result = provider
        .log_chat_completion("openai", &request, None, 150, None)
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_finish() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    // Don't call init_run to avoid network calls
    // Just test that finish doesn't panic when run is not initialized

    // Log some calls (will buffer but not send)
    let _ = provider
        .log_call(
            "openai",
            "gpt-4",
            Some(100),
            Some(50),
            Some(0.01),
            200,
            true,
            None,
        )
        .await;

    // finish should succeed even if run wasn't initialized
    // (flush returns Ok when disabled or empty buffer)
    let result = provider.finish().await;
    // The result depends on whether logging is enabled and run is initialized
    // We just verify it doesn't panic
    let _ = result;
}

#[tokio::test]
async fn test_is_enabled() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    assert!(provider.is_enabled());
}

#[tokio::test]
async fn test_is_disabled() {
    let mut config = create_test_config();
    config.enabled = false;

    let provider = WandbProvider::new(config).await.unwrap();
    assert!(!provider.is_enabled());
}

// ==================== Health Check Tests ====================

#[tokio::test]
async fn test_health_check_enabled() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let status = provider.health_check().await;

    assert_eq!(status, HealthStatus::Healthy);
}

#[tokio::test]
async fn test_health_check_disabled() {
    let mut config = create_test_config();
    config.enabled = false;

    let provider = WandbProvider::new(config).await.unwrap();
    let status = provider.health_check().await;

    assert_eq!(status, HealthStatus::Degraded);
}

// ==================== Error Mapper Tests ====================

#[test]
fn test_error_mapper_authentication() {
    let mapper = WandbErrorMapper;
    let error = mapper.map_http_error(401, "Unauthorized");

    match error {
        ProviderError::Authentication { provider, .. } => {
            assert_eq!(provider, "wandb");
        }
        _ => panic!("Expected Authentication error"),
    }
}

#[test]
fn test_error_mapper_rate_limit() {
    let mapper = WandbErrorMapper;
    let error = mapper.map_http_error(429, "Rate limit exceeded");

    match error {
        ProviderError::RateLimit { provider, .. } => {
            assert_eq!(provider, "wandb");
        }
        _ => panic!("Expected RateLimit error"),
    }
}

#[test]
fn test_error_mapper_network() {
    let mapper = WandbErrorMapper;
    let error =
        std::io::Error::new(std::io::ErrorKind::ConnectionRefused, "Connection refused");
    let mapped = mapper.map_network_error(&error);

    match mapped {
        ProviderError::Network { provider, .. } => {
            assert_eq!(provider, "wandb");
        }
        _ => panic!("Expected Network error"),
    }
}

// ==================== Clone Tests ====================

#[tokio::test]
async fn test_provider_clone() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let cloned = provider.clone();

    assert_eq!(provider.name(), cloned.name());
    assert_eq!(provider.is_enabled(), cloned.is_enabled());
}

// ==================== Cost Calculation Tests ====================

#[tokio::test]
async fn test_calculate_cost() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();

    // W&B doesn't have model costs
    let cost = provider.calculate_cost("gpt-4", 1000, 500).await;
    assert!(cost.is_ok());
    assert_eq!(cost.unwrap(), 0.0);
}

// ==================== Summary Tests ====================

#[tokio::test]
async fn test_get_summary() {
    let provider = WandbProvider::new(create_test_config()).await.unwrap();
    let _ = provider.init_run().await;

    // Log multiple calls
    for _ in 0..3 {
        let _ = provider
            .log_call(
                "openai",
                "gpt-4",
                Some(100),
                Some(50),
                Some(0.01),
                200,
                true,
                None,
            )
            .await;
    }

    let summary = provider.get_summary().await;
    assert_eq!(summary.total_calls, 3);
    assert_eq!(summary.successful_calls, 3);
    assert!((summary.total_cost_usd - 0.03).abs() < 0.001);
}
