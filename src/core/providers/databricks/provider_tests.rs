use super::*;

#[test]
fn test_databricks_provider_name() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();
    assert_eq!(provider.name(), "databricks");
}

#[test]
fn test_databricks_provider_capabilities() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();
    let caps = provider.capabilities();
    assert!(caps.contains(&ProviderCapability::ChatCompletion));
    assert!(caps.contains(&ProviderCapability::Embeddings));
}

#[test]
fn test_databricks_provider_models() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();
    let models = provider.models();
    assert!(!models.is_empty());
}

#[test]
fn test_get_endpoint_name() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    assert_eq!(
        provider.get_endpoint_name("databricks/dbrx-instruct"),
        "dbrx-instruct"
    );
    assert_eq!(provider.get_endpoint_name("llama-3-70b"), "llama-3-70b");
}

#[test]
fn test_build_endpoint_url() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    let url = provider
        .build_endpoint_url("dbrx-instruct", "chat")
        .unwrap();
    assert!(url.contains("/serving-endpoints/"));
    assert!(url.contains("dbrx-instruct"));
    assert!(url.ends_with("/invocations"));
}

#[test]
fn test_transform_messages() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    let messages = vec![ChatMessage {
        role: crate::core::types::message::MessageRole::User,
        content: Some(MessageContent::Text("Hello".to_string())),
        thinking: None,
        name: None,
        tool_calls: None,
        tool_call_id: None,
        function_call: None,
    }];

    let transformed = provider.transform_messages(&messages, false);
    assert_eq!(transformed.len(), 1);
    assert_eq!(transformed[0]["role"], "user");
    assert_eq!(transformed[0]["content"], "Hello");
}

#[test]
fn test_transform_chat_request() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    let request = ChatRequest {
        model: "dbrx-instruct".to_string(),
        messages: vec![ChatMessage {
            role: crate::core::types::message::MessageRole::User,
            content: Some(MessageContent::Text("Test".to_string())),
            thinking: None,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            function_call: None,
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    let body = provider.transform_chat_request_to_value(&request);
    assert!(body.get("messages").is_some());
    assert!((body["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001);
    assert_eq!(body["max_tokens"], 100);
}

#[test]
fn test_parse_chat_response() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    let response_json = serde_json::json!({
        "id": "chatcmpl-123",
        "created": 1700000000,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": "Hello! How can I help you?"
            },
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 8,
            "total_tokens": 18
        }
    });

    let response = provider
        .parse_chat_response(&response_json, "dbrx-instruct")
        .unwrap();
    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].finish_reason, Some(FinishReason::Stop));
    assert!(response.usage.is_some());
}

#[test]
fn test_health_check() {
    let config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    let provider = DatabricksProvider::new(config).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let health = rt.block_on(provider.health_check());
    assert_eq!(health, HealthStatus::Healthy);
}

#[test]
fn test_health_check_unhealthy() {
    let mut config =
        DatabricksConfig::with_credentials("dapi-test-key", "https://test.databricks.net");
    config.base.api_base = None;

    // This will fail validation, so we construct manually for testing
    let provider = DatabricksProvider {
        config,
        pool_manager: Arc::new(GlobalPoolManager::new().unwrap()),
        supported_models: vec![],
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    let health = rt.block_on(provider.health_check());
    assert_eq!(health, HealthStatus::Unhealthy);
}
