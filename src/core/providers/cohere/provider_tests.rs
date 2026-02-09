use super::*;
use crate::core::types::{chat::ChatMessage, message::MessageContent, message::MessageRole};

fn create_test_config() -> CohereConfig {
    CohereConfig::new("test_api_key")
}

#[tokio::test]
async fn test_provider_creation() {
    let provider = CohereProvider::new(create_test_config()).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "cohere");
}

#[tokio::test]
async fn test_provider_with_api_key() {
    let provider = CohereProvider::with_api_key("test_key").await;
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_provider_creation_no_api_key() {
    let config = CohereConfig::default();
    let provider = CohereProvider::new(config).await;
    assert!(provider.is_err());
}

#[tokio::test]
async fn test_provider_capabilities() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let caps = provider.capabilities();

    assert!(caps.contains(&ProviderCapability::ChatCompletion));
    assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
    assert!(caps.contains(&ProviderCapability::Embeddings));
    assert!(caps.contains(&ProviderCapability::ToolCalling));
}

#[tokio::test]
async fn test_provider_models() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let models = provider.models();

    assert!(!models.is_empty());
    assert!(models.iter().any(|m| m.id == "command-r-plus"));
    assert!(models.iter().any(|m| m.id == "command-r"));
    assert!(models.iter().any(|m| m.id == "embed-english-v3.0"));
    assert!(models.iter().any(|m| m.id == "rerank-english-v3.0"));
}

#[tokio::test]
async fn test_is_embedding_model() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();

    assert!(provider.is_embedding_model("embed-english-v3.0"));
    assert!(provider.is_embedding_model("embed-multilingual-v3.0"));
    assert!(!provider.is_embedding_model("command-r-plus"));
}

#[tokio::test]
async fn test_is_rerank_model() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();

    assert!(provider.is_rerank_model("rerank-english-v3.0"));
    assert!(provider.is_rerank_model("rerank-multilingual-v3.0"));
    assert!(!provider.is_rerank_model("command-r"));
}

#[tokio::test]
async fn test_get_supported_openai_params_chat() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let params = provider.get_supported_openai_params("command-r-plus");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"tools"));
}

#[tokio::test]
async fn test_get_supported_openai_params_embed() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let params = provider.get_supported_openai_params("embed-english-v3.0");

    assert!(params.contains(&"encoding_format"));
    assert!(params.contains(&"dimensions"));
}

#[tokio::test]
async fn test_calculate_cost() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();

    let cost = provider
        .calculate_cost("command-r-plus", 1000, 500)
        .await
        .unwrap();

    // command-r-plus: $0.003 input, $0.015 output per 1k
    // (1000/1000 * 0.003) + (500/1000 * 0.015) = 0.003 + 0.0075 = 0.0105
    assert!((cost - 0.0105).abs() < 0.0001);
}

#[tokio::test]
async fn test_calculate_cost_unknown_model() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();

    let result = provider.calculate_cost("unknown-model", 1000, 500).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_transform_request() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();

    let request = ChatRequest {
        model: "command-r-plus".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        temperature: Some(0.7),
        max_tokens: Some(100),
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let transformed = result.unwrap();
    assert_eq!(transformed["model"], "command-r-plus");
    assert!((transformed["temperature"].as_f64().unwrap() - 0.7).abs() < 0.001);
    assert_eq!(transformed["max_tokens"], 100);
}

#[tokio::test]
async fn test_provider_clone() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let cloned = provider.clone();

    assert_eq!(provider.name(), cloned.name());
    assert_eq!(provider.models().len(), cloned.models().len());
}

#[tokio::test]
async fn test_error_mapper() {
    let provider = CohereProvider::new(create_test_config()).await.unwrap();
    let mapper = provider.get_error_mapper();

    let error = mapper.map_http_error(401, "Unauthorized");
    assert_eq!(error.provider(), "cohere");

    let error = mapper.map_http_error(429, "Rate limited");
    assert_eq!(error.provider(), "cohere");
}
