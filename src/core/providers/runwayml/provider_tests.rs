use super::*;
use crate::utils::test_env;

#[test]
fn test_provider_creation_without_api_key() {
    let config = RunwayMLConfig::default();
    let result = RunwayMLProvider::new(config);
    assert!(result.is_err());
}

#[test]
fn test_provider_creation_with_api_key() {
    let config = RunwayMLConfig::new("test-api-key");
    let result = RunwayMLProvider::new(config);
    assert!(result.is_ok());
}

#[test]
fn test_provider_name() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();
    assert_eq!(provider.name(), PROVIDER_NAME);
}

#[test]
fn test_provider_capabilities() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();
    let capabilities = provider.capabilities();
    assert!(capabilities.contains(&ProviderCapability::ImageGeneration));
}

#[test]
fn test_provider_models() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();
    let models = provider.models();
    assert!(!models.is_empty());
}

#[test]
fn test_get_request_headers() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();
    let headers = provider.get_request_headers();

    assert!(headers.iter().any(|h| h.0 == "Authorization"));
    assert!(headers.iter().any(|h| h.0 == "Content-Type"));
}

#[test]
fn test_transform_image_to_video_request() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();

    let request = ImageGenerationRequest {
        prompt: "A beautiful sunset over the ocean".to_string(),
        model: Some("gen3a_turbo".to_string()),
        n: Some(1),
        size: Some("1792x1024".to_string()),
        quality: None,
        response_format: None,
        style: None,
        user: None,
    };

    let task_request = provider.transform_image_to_video_request(&request);

    assert_eq!(task_request.model, "gen3a_turbo");
    assert_eq!(
        task_request.prompt_text,
        Some("A beautiful sunset over the ocean".to_string())
    );
    assert_eq!(task_request.ratio, Some("16:9".to_string()));
    assert_eq!(task_request.duration, Some(5));
}

#[test]
fn test_transform_video_to_image_response() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();

    let video_response = VideoGenerationResponse {
        task_id: "task-123".to_string(),
        video_urls: vec!["https://example.com/video.mp4".to_string()],
        duration_seconds: 5,
    };

    let response = provider.transform_video_to_image_response(video_response);

    assert_eq!(response.data.len(), 1);
    assert!(response.data[0].url.is_some());
}

#[test]
fn test_supported_openai_params() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();
    let params = provider.get_supported_openai_params("gen3a_turbo");

    assert!(params.contains(&"prompt"));
    assert!(params.contains(&"size"));
}

#[tokio::test]
async fn test_chat_completion_not_supported() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();

    let request = ChatRequest {
        model: "gen3a_turbo".to_string(),
        messages: vec![],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.chat_completion(request, context).await;

    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        ProviderError::NotSupported { .. }
    ));
}

#[test]
fn test_health_check_with_api_key() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();

    let rt = tokio::runtime::Runtime::new().unwrap();
    let health = rt.block_on(provider.health_check());
    assert_eq!(health, HealthStatus::Healthy);
}

#[test]
fn test_from_env_missing_api_key() {
    // Clear any existing env var
    let _guard = test_env::lock();
    test_env::remove_var("RUNWAYML_API_KEY");

    let result = RunwayMLProvider::from_env();
    assert!(result.is_err());
}

#[test]
fn test_create_task_request_serialization() {
    let request = CreateTaskRequest {
        model: "gen3a_turbo".to_string(),
        prompt_text: Some("A cat playing piano".to_string()),
        prompt_image: None,
        duration: Some(5),
        ratio: Some("16:9".to_string()),
        seed: None,
        watermark: Some(false),
    };

    let json = serde_json::to_value(&request).unwrap();
    assert_eq!(json["model"], "gen3a_turbo");
    assert_eq!(json["promptText"], "A cat playing piano");
    assert_eq!(json["duration"], 5);
    assert_eq!(json["ratio"], "16:9");
}

#[test]
fn test_task_status_deserialization() {
    let json =
        r#"{"id":"task-123","status":"SUCCEEDED","output":["https://example.com/video.mp4"]}"#;
    let task: TaskResponse = serde_json::from_str(json).unwrap();
    assert_eq!(task.status, TaskStatus::Succeeded);
    assert_eq!(task.output.unwrap().len(), 1);
}

#[tokio::test]
async fn test_map_openai_params() {
    let config = RunwayMLConfig::new("test-api-key");
    let provider = RunwayMLProvider::new(config).unwrap();

    let mut params = HashMap::new();
    params.insert("size".to_string(), serde_json::json!("1792x1024"));
    params.insert("n".to_string(), serde_json::json!(1));

    let mapped = provider
        .map_openai_params(params, "gen3a_turbo")
        .await
        .unwrap();

    assert!(mapped.contains_key("ratio"));
    assert_eq!(mapped.get("ratio").unwrap(), "16:9");
}
