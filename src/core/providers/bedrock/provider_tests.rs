//! Unit tests for BedrockProvider
//!
//! Tests for provider creation, capabilities, message conversion,
//! request/response transformation, and cost calculation.

use super::client::BedrockClient;
use super::config::BedrockConfig;
use super::provider::{BEDROCK_CAPABILITIES, BedrockProvider};
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::ContentPart;
use crate::core::types::model::ProviderCapability;
use crate::core::types::{ChatMessage, MessageContent, MessageRole};
use std::collections::HashMap;

fn create_test_config() -> BedrockConfig {
    BedrockConfig {
        aws_access_key_id: "AKIAIOSFODNN7EXAMPLE".to_string(),
        aws_secret_access_key: "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY".to_string(),
        aws_session_token: None,
        aws_region: "us-east-1".to_string(),
        timeout_seconds: 30,
        max_retries: 3,
    }
}

fn create_test_provider() -> BedrockProvider {
    let config = create_test_config();
    BedrockProvider::new_for_test(BedrockClient::new(config).unwrap(), vec![])
}

// ==================== Provider Creation Tests ====================

#[tokio::test]
async fn test_bedrock_provider_creation() {
    let config = BedrockConfig {
        aws_access_key_id: "AKIATEST123456789012".to_string(),
        aws_secret_access_key: "test_secret".to_string(),
        aws_session_token: None,
        aws_region: "us-east-1".to_string(),
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = BedrockProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "bedrock");
    assert!(
        provider
            .capabilities()
            .contains(&ProviderCapability::ChatCompletion)
    );
}

#[tokio::test]
async fn test_bedrock_provider_creation_with_session_token() {
    let config = BedrockConfig {
        aws_access_key_id: "AKIATEST123456789012".to_string(),
        aws_secret_access_key: "test_secret".to_string(),
        aws_session_token: Some("session_token".to_string()),
        aws_region: "us-west-2".to_string(),
        timeout_seconds: 60,
        max_retries: 5,
    };

    let provider = BedrockProvider::new(config).await;
    assert!(provider.is_ok());
}

#[tokio::test]
async fn test_bedrock_provider_creation_invalid_region() {
    let config = BedrockConfig {
        aws_access_key_id: "AKIATEST123456789012".to_string(),
        aws_secret_access_key: "test_secret".to_string(),
        aws_session_token: None,
        aws_region: "invalid-region-xyz".to_string(),
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = BedrockProvider::new(config).await;
    assert!(provider.is_err());
}

#[tokio::test]
async fn test_bedrock_provider_creation_empty_credentials() {
    let config = BedrockConfig {
        aws_access_key_id: "".to_string(),
        aws_secret_access_key: "test_secret".to_string(),
        aws_session_token: None,
        aws_region: "us-east-1".to_string(),
        timeout_seconds: 30,
        max_retries: 3,
    };

    let provider = BedrockProvider::new(config).await;
    assert!(provider.is_err());
}

// ==================== Provider Capabilities Tests ====================

#[test]
fn test_provider_name() {
    let provider = create_test_provider();
    assert_eq!(provider.name(), "bedrock");
}

#[test]
fn test_provider_capabilities() {
    let provider = create_test_provider();
    let caps = provider.capabilities();

    assert!(caps.contains(&ProviderCapability::ChatCompletion));
    assert!(caps.contains(&ProviderCapability::ChatCompletionStream));
    assert!(caps.contains(&ProviderCapability::FunctionCalling));
    assert!(caps.contains(&ProviderCapability::Embeddings));
}

#[test]
fn test_provider_supported_openai_params() {
    let provider = create_test_provider();
    let params = provider.get_supported_openai_params("anthropic.claude-3-sonnet-20240229");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"stop"));
    assert!(params.contains(&"tools"));
    assert!(params.contains(&"tool_choice"));
}

#[test]
fn test_provider_models_empty_initially() {
    let provider = create_test_provider();
    assert!(provider.models().is_empty());
}

// ==================== Embedding Model Detection Tests ====================

#[test]
fn test_embedding_model_detection() {
    let provider = create_test_provider();

    assert!(provider.is_embedding_model("amazon.titan-embed-text-v1"));
    assert!(provider.is_embedding_model("cohere.embed-english-v3"));
    assert!(provider.is_embedding_model("my-embed-model"));
    assert!(!provider.is_embedding_model("anthropic.claude-3-sonnet"));
    assert!(!provider.is_embedding_model("amazon.titan-text-express-v1"));
}

// ==================== Messages to Prompt Conversion Tests ====================

#[test]
fn test_messages_to_prompt_simple_user_message() {
    let provider = create_test_provider();

    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text("Hello, how are you?".to_string())),
        ..Default::default()
    }];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("Human: Hello, how are you?"));
    assert!(prompt.ends_with("Assistant:"));
}

#[test]
fn test_messages_to_prompt_system_message() {
    let provider = create_test_provider();

    let messages = vec![
        ChatMessage {
            role: MessageRole::System,
            content: Some(MessageContent::Text(
                "You are a helpful assistant.".to_string(),
            )),
            ..Default::default()
        },
        ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        },
    ];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("System: You are a helpful assistant."));
    assert!(prompt.contains("Human: Hello"));
}

#[test]
fn test_messages_to_prompt_assistant_message() {
    let provider = create_test_provider();

    let messages = vec![
        ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        },
        ChatMessage {
            role: MessageRole::Assistant,
            content: Some(MessageContent::Text("Hi there!".to_string())),
            ..Default::default()
        },
        ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("How are you?".to_string())),
            ..Default::default()
        },
    ];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("Human: Hello"));
    assert!(prompt.contains("Assistant: Hi there!"));
    assert!(prompt.contains("Human: How are you?"));
}

#[test]
fn test_messages_to_prompt_tool_message() {
    let provider = create_test_provider();

    let messages = vec![ChatMessage {
        role: MessageRole::Tool,
        content: Some(MessageContent::Text("Tool output".to_string())),
        ..Default::default()
    }];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("Tool: Tool output"));
}

#[test]
fn test_messages_to_prompt_function_message() {
    let provider = create_test_provider();

    let messages = vec![ChatMessage {
        role: MessageRole::Function,
        content: Some(MessageContent::Text("Function result".to_string())),
        ..Default::default()
    }];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("Tool: Function result"));
}

#[test]
fn test_messages_to_prompt_with_content_parts() {
    let provider = create_test_provider();

    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Hello".to_string(),
            },
            ContentPart::Text {
                text: "World".to_string(),
            },
        ])),
        ..Default::default()
    }];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.contains("Human: Hello World"));
}

#[test]
fn test_messages_to_prompt_none_content() {
    let provider = create_test_provider();

    let messages = vec![ChatMessage {
        role: MessageRole::User,
        content: None,
        ..Default::default()
    }];

    let prompt = provider.messages_to_prompt(&messages).unwrap();
    assert!(prompt.ends_with("Assistant:"));
}

// ==================== OpenAI Params Mapping Tests ====================

#[tokio::test]
async fn test_map_openai_params_max_tokens() {
    let provider = create_test_provider();

    let mut params = HashMap::new();
    params.insert(
        "max_tokens".to_string(),
        serde_json::Value::Number(100.into()),
    );

    let mapped = provider
        .map_openai_params(params, "anthropic.claude-3-sonnet-20240229")
        .await
        .unwrap();

    assert!(mapped.contains_key("max_tokens_to_sample"));
    assert_eq!(
        mapped.get("max_tokens_to_sample").unwrap(),
        &serde_json::Value::Number(100.into())
    );
}

#[tokio::test]
async fn test_map_openai_params_temperature() {
    let provider = create_test_provider();

    let mut params = HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.7));

    let mapped = provider
        .map_openai_params(params, "anthropic.claude-3-sonnet-20240229")
        .await
        .unwrap();

    assert!(mapped.contains_key("temperature"));
}

#[tokio::test]
async fn test_map_openai_params_unsupported_ignored() {
    let provider = create_test_provider();

    let mut params = HashMap::new();
    params.insert(
        "unsupported_param".to_string(),
        serde_json::Value::String("value".to_string()),
    );

    let mapped = provider
        .map_openai_params(params, "anthropic.claude-3-sonnet-20240229")
        .await
        .unwrap();

    assert!(!mapped.contains_key("unsupported_param"));
}

#[tokio::test]
async fn test_map_openai_params_multiple() {
    let provider = create_test_provider();

    let mut params = HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.5));
    params.insert("top_p".to_string(), serde_json::json!(0.9));
    params.insert("stream".to_string(), serde_json::Value::Bool(true));
    params.insert("stop".to_string(), serde_json::json!(["END"]));

    let mapped = provider
        .map_openai_params(params, "anthropic.claude-3-sonnet-20240229")
        .await
        .unwrap();

    assert!(mapped.contains_key("temperature"));
    assert!(mapped.contains_key("top_p"));
    assert!(mapped.contains_key("stream"));
    assert!(mapped.contains_key("stop"));
}

// ==================== Transform Request Tests ====================

#[tokio::test]
async fn test_transform_request_claude() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet-20240229".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        max_tokens: Some(1000),
        temperature: Some(0.7),
        top_p: Some(0.9),
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("messages").is_some());
    assert_eq!(body.get("max_tokens").unwrap(), 1000);
    assert!(body.get("anthropic_version").is_some());
}

#[tokio::test]
async fn test_transform_request_titan() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "amazon.titan-text-express-v1".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        max_tokens: Some(500),
        temperature: Some(0.5),
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("inputText").is_some());
    assert!(body.get("textGenerationConfig").is_some());
}

#[tokio::test]
async fn test_transform_request_nova() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "amazon.nova-pro-v1:0".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        max_tokens: Some(2000),
        temperature: Some(0.8),
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("messages").is_some());
    assert_eq!(body.get("max_tokens").unwrap(), 2000);
}

#[tokio::test]
async fn test_transform_request_llama() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "meta.llama3-70b-instruct-v1:0".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        max_tokens: Some(1500),
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_request_mistral() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "mistral.mistral-large-2407-v1:0".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("prompt").is_some());
}

#[tokio::test]
async fn test_transform_request_ai21() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "ai21.jamba-1-5-large-v1:0".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("prompt").is_some());
    assert!(body.get("maxTokens").is_some());
}

#[tokio::test]
async fn test_transform_request_cohere() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "cohere.command-r-plus-v1:0".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_ok());
    let body = result.unwrap();
    assert!(body.get("prompt").is_some());
}

#[tokio::test]
async fn test_transform_request_embedding_model_error() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "amazon.titan-embed-text-v1".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transform_request_unknown_model() {
    use crate::core::types::ChatRequest;
    use crate::core::types::RequestContext;

    let provider = create_test_provider();

    let request = ChatRequest {
        model: "unknown.model-v1".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("Hello".to_string())),
            ..Default::default()
        }],
        ..Default::default()
    };

    let context = RequestContext::default();
    let result = provider.transform_request(request, context).await;

    assert!(result.is_err());
}

// ==================== Transform Response Tests ====================

#[tokio::test]
async fn test_transform_response_claude() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "content": [{"text": "Hello! I'm doing well."}],
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(
            &response_bytes,
            "anthropic.claude-3-sonnet-20240229",
            "test-request-id",
        )
        .await;

    assert!(result.is_ok());
    let chat_response = result.unwrap();
    assert_eq!(chat_response.model, "anthropic.claude-3-sonnet-20240229");
    assert!(!chat_response.choices.is_empty());
}

#[tokio::test]
async fn test_transform_response_titan() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "results": [{"outputText": "Hello from Titan!"}],
        "inputTextTokenCount": 5
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(
            &response_bytes,
            "amazon.titan-text-express-v1",
            "test-request-id",
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_response_nova() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "content": [{"text": "Nova response"}],
        "usage": {
            "input_tokens": 15,
            "output_tokens": 25
        }
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(&response_bytes, "amazon.nova-pro-v1:0", "test-request-id")
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_response_mistral() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "outputs": [{"text": "Mistral response"}]
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(
            &response_bytes,
            "mistral.mistral-large-2407-v1:0",
            "test-request-id",
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_response_ai21() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "completions": [{"data": {"text": "AI21 response"}}]
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(
            &response_bytes,
            "ai21.jamba-1-5-large-v1:0",
            "test-request-id",
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_response_cohere() {
    let provider = create_test_provider();

    let response = serde_json::json!({
        "text": "Cohere response"
    });
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(
            &response_bytes,
            "cohere.command-r-plus-v1:0",
            "test-request-id",
        )
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_transform_response_invalid_json() {
    let provider = create_test_provider();

    let response_bytes = b"not valid json";

    let result = provider
        .transform_response(
            response_bytes,
            "anthropic.claude-3-sonnet-20240229",
            "test-request-id",
        )
        .await;

    assert!(result.is_err());
}

#[tokio::test]
async fn test_transform_response_unknown_model() {
    let provider = create_test_provider();

    let response = serde_json::json!({"text": "response"});
    let response_bytes = serde_json::to_vec(&response).unwrap();

    let result = provider
        .transform_response(&response_bytes, "unknown.model-v1", "test-request-id")
        .await;

    assert!(result.is_err());
}

// ==================== Cost Calculation Tests ====================

#[tokio::test]
async fn test_calculate_cost_known_model() {
    let provider = create_test_provider();

    let cost = provider
        .calculate_cost("anthropic.claude-3-opus-20240229", 1000, 500)
        .await;

    assert!(cost.is_ok());
    let cost_value = cost.unwrap();
    assert!(cost_value > 0.0);
}

#[tokio::test]
async fn test_calculate_cost_unknown_model() {
    let provider = create_test_provider();

    let cost = provider.calculate_cost("unknown.model-v1", 1000, 500).await;

    assert!(cost.is_err());
}

#[tokio::test]
async fn test_calculate_cost_zero_tokens() {
    let provider = create_test_provider();

    let cost = provider
        .calculate_cost("anthropic.claude-3-haiku-20240307", 0, 0)
        .await;

    assert!(cost.is_ok());
    assert!((cost.unwrap() - 0.0).abs() < 0.0001);
}

// ==================== Error Mapper Tests ====================

#[test]
fn test_get_error_mapper() {
    let provider = create_test_provider();
    let mapper = provider.get_error_mapper();

    // Test that we can get an error mapper (it's a struct)
    let _ = format!("{:?}", mapper);
}

// ==================== Client Access Tests ====================

#[test]
fn test_agents_client_access() {
    let provider = create_test_provider();
    let _agents_client = provider.agents_client();
    // Just verify we can access the agents client
}

#[test]
fn test_knowledge_bases_client_access() {
    let provider = create_test_provider();
    let _kb_client = provider.knowledge_bases_client();
    // Just verify we can access the knowledge bases client
}

#[test]
fn test_batch_client_access() {
    let provider = create_test_provider();
    let _batch_client = provider.batch_client();
    // Just verify we can access the batch client
}

#[test]
fn test_guardrails_client_access() {
    let provider = create_test_provider();
    let _guardrails_client = provider.guardrails_client();
    // Just verify we can access the guardrails client
}

// ==================== Capabilities Constants Tests ====================

#[test]
fn test_bedrock_capabilities_constant() {
    assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
    assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
    assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::FunctionCalling));
    assert!(BEDROCK_CAPABILITIES.contains(&ProviderCapability::Embeddings));
    assert_eq!(BEDROCK_CAPABILITIES.len(), 4);
}

// ==================== Provider Clone/Debug Tests ====================

#[test]
fn test_provider_clone() {
    let provider = create_test_provider();
    let cloned = provider.clone();

    assert_eq!(provider.name(), cloned.name());
    assert_eq!(provider.capabilities().len(), cloned.capabilities().len());
}

#[test]
fn test_provider_debug() {
    let provider = create_test_provider();
    let debug_str = format!("{:?}", provider);

    assert!(debug_str.contains("BedrockProvider"));
}
