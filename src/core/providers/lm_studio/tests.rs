//! Unit tests for LM Studio provider

use super::*;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::model::ProviderCapability;
use crate::core::types::{chat::ChatMessage, chat::ChatRequest, message::MessageContent, message::MessageRole};
use crate::core::types::responses::FinishReason;

#[test]
fn test_lm_studio_provider_name() {
    // Test that the provider name is correct
    // Note: We can't instantiate the provider without async, so test config instead
    let config = LMStudioConfig::default();
    assert!(config.validate().is_ok());
}

#[test]
fn test_lm_studio_config_creation() {
    let config = LMStudioConfig {
        api_key: Some("test-key".to_string()),
        api_base: Some("http://localhost:1234".to_string()),
        timeout: 60,
        max_retries: 2,
        debug: true,
    };

    assert_eq!(config.api_key, Some("test-key".to_string()));
    assert_eq!(config.api_base, Some("http://localhost:1234".to_string()));
    assert_eq!(config.timeout, 60);
    assert_eq!(config.max_retries, 2);
    assert!(config.debug);
}

#[test]
fn test_lm_studio_config_default_values() {
    let config = LMStudioConfig::default();

    assert!(config.api_key.is_none());
    assert!(config.api_base.is_none());
    assert_eq!(config.timeout, 120);
    assert_eq!(config.max_retries, 3);
    assert!(!config.debug);
}

#[test]
fn test_lm_studio_config_validation() {
    // Valid config
    let config = LMStudioConfig::default();
    assert!(config.validate().is_ok());

    // Invalid: zero timeout
    let config = LMStudioConfig {
        timeout: 0,
        ..Default::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn test_lm_studio_config_endpoints() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint().unwrap(),
        "http://localhost:1234/v1/chat/completions"
    );
    assert_eq!(
        config.get_embeddings_endpoint().unwrap(),
        "http://localhost:1234/v1/embeddings"
    );
    assert_eq!(
        config.get_models_endpoint().unwrap(),
        "http://localhost:1234/v1/models"
    );
}

#[test]
fn test_lm_studio_config_endpoints_with_trailing_slash() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234/".to_string()),
        ..Default::default()
    };

    assert_eq!(
        config.get_chat_endpoint().unwrap(),
        "http://localhost:1234/v1/chat/completions"
    );
}

#[test]
fn test_lm_studio_config_endpoints_no_base() {
    let config = LMStudioConfig::default();

    assert!(config.get_chat_endpoint().is_err());
    assert!(config.get_embeddings_endpoint().is_err());
    assert!(config.get_models_endpoint().is_err());
}

#[test]
fn test_lm_studio_config_get_api_key() {
    // Test with explicit key
    let config = LMStudioConfig {
        api_key: Some("my-key".to_string()),
        ..Default::default()
    };
    assert_eq!(config.get_api_key(), "my-key");

    // Test without key (returns fake-api-key)
    let config = LMStudioConfig::default();
    assert_eq!(config.get_api_key(), "fake-api-key");
}

#[test]
fn test_lm_studio_capabilities() {
    // Test that capabilities constant is properly defined
    assert!(LM_STUDIO_CAPABILITIES.contains(&ProviderCapability::ChatCompletion));
    assert!(LM_STUDIO_CAPABILITIES.contains(&ProviderCapability::ChatCompletionStream));
    assert!(LM_STUDIO_CAPABILITIES.contains(&ProviderCapability::Embeddings));
    assert!(LM_STUDIO_CAPABILITIES.contains(&ProviderCapability::ToolCalling));
}

#[tokio::test]
async fn test_lm_studio_provider_creation() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.name(), "lm_studio");
    assert_eq!(provider.capabilities(), LM_STUDIO_CAPABILITIES);
}

#[tokio::test]
async fn test_lm_studio_provider_with_base_url() {
    let provider = LMStudioProvider::with_base_url("http://192.168.1.100:1234").await;
    assert!(provider.is_ok());

    let provider = provider.unwrap();
    assert_eq!(provider.config.api_base, Some("http://192.168.1.100:1234".to_string()));
}

#[tokio::test]
async fn test_lm_studio_build_chat_request() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let request = ChatRequest {
        model: "lm_studio/llama-3".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("You are a helpful assistant.".to_string())),
                thinking: None,
                tool_calls: None,
                function_call: None,
                name: None,
                tool_call_id: None,
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello!".to_string())),
                thinking: None,
                tool_calls: None,
                function_call: None,
                name: None,
                tool_call_id: None,
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
        ..Default::default()
    };

    let body = provider.build_chat_request(&request, false).unwrap();

    assert_eq!(body["model"], "llama-3");
    assert_eq!(body["stream"], false);
    assert_eq!(body["temperature"], 0.7);
    assert_eq!(body["max_tokens"], 100);

    let messages = body["messages"].as_array().unwrap();
    assert_eq!(messages.len(), 2);
    assert_eq!(messages[0]["role"], "system");
    assert_eq!(messages[0]["content"], "You are a helpful assistant.");
    assert_eq!(messages[1]["role"], "user");
    assert_eq!(messages[1]["content"], "Hello!");
}

#[tokio::test]
async fn test_lm_studio_build_chat_request_with_tools() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let request = ChatRequest {
        model: "lm_studio/llama-3".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::User,
            content: Some(MessageContent::Text("What's the weather?".to_string())),
            thinking: None,
            tool_calls: None,
            function_call: None,
            name: None,
            tool_call_id: None,
        }],
        tools: Some(vec![crate::core::types::tools::Tool {
            tool_type: crate::core::types::tools::ToolType::Function,
            function: crate::core::types::tools::FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get the current weather".to_string()),
                parameters: Some(serde_json::json!({
                    "type": "object",
                    "properties": {
                        "location": {"type": "string"}
                    }
                })),
            },
        }]),
        stream: false,
        ..Default::default()
    };

    let body = provider.build_chat_request(&request, false).unwrap();

    assert!(body.get("tools").is_some());
    let tools = body["tools"].as_array().unwrap();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0]["type"], "function");
    assert_eq!(tools[0]["function"]["name"], "get_weather");
}

#[tokio::test]
async fn test_lm_studio_parse_chat_response() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let response_json = serde_json::json!({
        "id": "chatcmpl-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "llama-3",
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

    let response = provider.parse_chat_response(response_json, "llama-3").unwrap();

    assert_eq!(response.id, "chatcmpl-123");
    assert_eq!(response.model, "lm_studio/llama-3");
    assert_eq!(response.choices.len(), 1);
    assert_eq!(response.choices[0].message.role, MessageRole::Assistant);

    if let Some(MessageContent::Text(content)) = &response.choices[0].message.content {
        assert_eq!(content, "Hello! How can I help you?");
    } else {
        panic!("Expected text content");
    }

    let usage = response.usage.unwrap();
    assert_eq!(usage.prompt_tokens, 10);
    assert_eq!(usage.completion_tokens, 8);
    assert_eq!(usage.total_tokens, 18);
}

#[tokio::test]
async fn test_lm_studio_parse_chat_response_with_tool_calls() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let response_json = serde_json::json!({
        "id": "chatcmpl-456",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "llama-3",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_abc123",
                    "type": "function",
                    "function": {
                        "name": "get_weather",
                        "arguments": "{\"location\": \"San Francisco\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {
            "prompt_tokens": 15,
            "completion_tokens": 10,
            "total_tokens": 25
        }
    });

    let response = provider.parse_chat_response(response_json, "llama-3").unwrap();

    assert_eq!(response.choices.len(), 1);
    let tool_calls = response.choices[0].message.tool_calls.as_ref().unwrap();
    assert_eq!(tool_calls.len(), 1);
    assert_eq!(tool_calls[0].id, "call_abc123");
    assert_eq!(tool_calls[0].function.name, "get_weather");
    assert_eq!(
        tool_calls[0].function.arguments,
        "{\"location\": \"San Francisco\"}"
    );
    assert_eq!(response.choices[0].finish_reason, Some(FinishReason::ToolCalls));
}

#[tokio::test]
async fn test_lm_studio_map_openai_params() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let mut params = std::collections::HashMap::new();
    params.insert("temperature".to_string(), serde_json::json!(0.7));
    params.insert("max_tokens".to_string(), serde_json::json!(100));

    let mapped = provider.map_openai_params(params.clone(), "llama-3").await.unwrap();

    // LM Studio uses OpenAI-compatible API, so params should pass through unchanged
    assert_eq!(mapped["temperature"], serde_json::json!(0.7));
    assert_eq!(mapped["max_tokens"], serde_json::json!(100));
}

#[tokio::test]
async fn test_lm_studio_calculate_cost() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    // LM Studio is free, cost should always be 0
    let cost = provider.calculate_cost("llama-3", 1000, 500).await.unwrap();
    assert_eq!(cost, 0.0);
}

#[tokio::test]
async fn test_lm_studio_get_supported_params() {
    let config = LMStudioConfig {
        api_base: Some("http://localhost:1234".to_string()),
        ..Default::default()
    };

    let provider = LMStudioProvider::new(config).await.unwrap();

    let params = provider.get_supported_openai_params("llama-3");

    assert!(params.contains(&"temperature"));
    assert!(params.contains(&"max_tokens"));
    assert!(params.contains(&"top_p"));
    assert!(params.contains(&"stream"));
    assert!(params.contains(&"tools"));
    assert!(params.contains(&"response_format"));
}
