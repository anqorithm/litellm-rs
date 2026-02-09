use super::*;

#[tokio::test]
async fn test_github_copilot_provider_creation() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "github_copilot");
}

#[tokio::test]
async fn test_github_copilot_provider_capabilities() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();
    let capabilities = provider.capabilities();

    assert!(capabilities.contains(&ProviderCapability::ChatCompletion));
    assert!(capabilities.contains(&ProviderCapability::ChatCompletionStream));
    assert!(capabilities.contains(&ProviderCapability::ToolCalling));
}

#[tokio::test]
async fn test_github_copilot_provider_models() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();
    let models = provider.models();

    assert!(!models.is_empty());

    // Check that we have expected models
    let model_ids: Vec<&str> = models.iter().map(|m| m.id.as_str()).collect();
    assert!(model_ids.contains(&"gpt-4o"));
    assert!(model_ids.contains(&"claude-3.5-sonnet"));
}

#[tokio::test]
async fn test_github_copilot_provider_supported_params() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();

    // Non-reasoning model
    let params = provider.get_supported_openai_params("gpt-4o");
    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"tools"));
    assert!(!params.contains(&"reasoning_effort"));

    // Reasoning model
    let params = provider.get_supported_openai_params("o1-preview");
    assert!(params.contains(&"reasoning_effort"));

    // Claude reasoning model
    let params = provider.get_supported_openai_params("claude-3-7-sonnet");
    assert!(params.contains(&"thinking"));
    assert!(params.contains(&"reasoning_effort"));
}

#[test]
fn test_determine_initiator() {
    let config = GitHubCopilotConfig::default();
    // Create a sync provider for testing
    let authenticator = CopilotAuthenticator::new(&config);
    let provider = GitHubCopilotProvider {
        config,
        authenticator,
        models: vec![],
        cached_api_key: Arc::new(RwLock::new(None)),
        cached_api_base: Arc::new(RwLock::new(None)),
    };

    // User message only
    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(crate::core::types::message::MessageContent::Text(
            "Hello".to_string(),
        )),
        ..Default::default()
    }];
    assert_eq!(provider.determine_initiator(&messages), "user");

    // Assistant message present
    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: Some(crate::core::types::message::MessageContent::Text(
                "Hello".to_string(),
            )),
            ..Default::default()
        },
        ChatMessage {
            role: MessageRole::Assistant,
            content: Some(crate::core::types::message::MessageContent::Text(
                "Hi!".to_string(),
            )),
            ..Default::default()
        },
    ];
    assert_eq!(provider.determine_initiator(&messages), "agent");
}

#[tokio::test]
async fn test_github_copilot_provider_cost_calculation() {
    let config = GitHubCopilotConfig::default();
    let provider = GitHubCopilotProvider::new(config).await.unwrap();

    // Copilot is subscription-based, cost should be 0
    let cost = provider.calculate_cost("gpt-4o", 1000, 500).await;
    assert!(cost.is_ok());
    assert_eq!(cost.unwrap(), 0.0);
}
