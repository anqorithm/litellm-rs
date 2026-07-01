use serde_json::json;

use super::*;
use crate::core::providers::anthropic::config::AnthropicConfig;
use crate::core::types::chat::{ChatMessage, ChatRequest};
use crate::core::types::message::{MessageContent, MessageRole};
use crate::core::types::tools::{
    FunctionCall, FunctionChoice, FunctionDefinition, Tool, ToolCall, ToolChoice, ToolType,
};

fn anthropic_client() -> AnthropicClient {
    AnthropicClient::new(AnthropicConfig::new_test("test-key")).unwrap()
}

fn tool(name: &str) -> Tool {
    Tool {
        tool_type: ToolType::Function,
        function: FunctionDefinition {
            name: name.to_string(),
            description: Some("lookup".to_string()),
            parameters: Some(json!({"type": "object"})),
        },
    }
}

#[test]
fn issue_761_anthropic_transform_tools_sanitizes_invalid_function_names() {
    let result = anthropic_client()
        .transform_tools(&[tool("actions/download-job-logs.for-workflow-run")])
        .unwrap();

    assert_eq!(
        result[0]["name"],
        "actions_download-job-logs_for-workflow-run"
    );
}

#[test]
fn issue_761_anthropic_transform_request_sanitizes_specific_tool_choice() {
    let mut request = ChatRequest::new("claude-3-opus-20240229")
        .add_user_message("weather?")
        .with_tools(vec![tool("weather.lookup")]);
    request.tool_choice = Some(ToolChoice::Specific {
        choice_type: "function".to_string(),
        function: Some(FunctionChoice {
            name: "weather.lookup".to_string(),
        }),
    });

    let transformed = anthropic_client().transform_chat_request(&request).unwrap();

    assert_eq!(transformed["tools"][0]["name"], "weather_lookup");
    assert_eq!(transformed["tool_choice"]["name"], "weather_lookup");
}

#[test]
fn issue_761_anthropic_transform_request_rejects_sanitized_tool_choice_alias() {
    let mut request = ChatRequest::new("claude-3-opus-20240229")
        .add_user_message("weather?")
        .with_tools(vec![tool("weather_lookup")]);
    request.tool_choice = Some(ToolChoice::Specific {
        choice_type: "function".to_string(),
        function: Some(FunctionChoice {
            name: "weather.lookup".to_string(),
        }),
    });

    let result = anthropic_client().transform_chat_request(&request);
    let message = result
        .err()
        .map_or_else(String::new, |error| error.to_string());

    assert!(message.contains("Tool choice"));
    assert!(message.contains("weather.lookup"));
    assert!(message.contains("weather_lookup"));
}

#[test]
fn issue_761_anthropic_transform_request_sanitizes_assistant_tool_call_history() {
    let mut request = ChatRequest::new("claude-3-opus-20240229");
    request.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        content: Some(MessageContent::Text("checking".to_string())),
        tool_calls: Some(vec![ToolCall {
            id: "toolu_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "weather.lookup".to_string(),
                arguments: r#"{"city":"Paris"}"#.to_string(),
            },
        }]),
        ..Default::default()
    });

    let transformed = anthropic_client().transform_chat_request(&request).unwrap();
    let content = transformed["messages"][0]["content"].as_array().unwrap();

    assert_eq!(content[1]["name"], "weather_lookup");
}

#[test]
fn issue_761_anthropic_transform_request_rejects_sanitized_history_alias() {
    let mut request = ChatRequest::new("claude-3-opus-20240229")
        .add_user_message("weather?")
        .with_tools(vec![tool("weather_lookup")]);
    request.messages.push(ChatMessage {
        role: MessageRole::Assistant,
        content: None,
        tool_calls: Some(vec![ToolCall {
            id: "toolu_123".to_string(),
            tool_type: "function".to_string(),
            function: FunctionCall {
                name: "weather.lookup".to_string(),
                arguments: r#"{"city":"Paris"}"#.to_string(),
            },
        }]),
        ..Default::default()
    });

    let result = anthropic_client().transform_chat_request(&request);
    let message = result
        .err()
        .map_or_else(String::new, |error| error.to_string());

    assert!(message.contains("Tool call"));
    assert!(message.contains("weather.lookup"));
    assert!(message.contains("weather_lookup"));
}

#[test]
fn issue_761_anthropic_transform_tools_rejects_sanitized_name_collisions() {
    let error = anthropic_client()
        .transform_tools(&[tool("weather.lookup"), tool("weather/lookup")])
        .expect_err("colliding sanitized names must fail closed");
    let message = error.to_string();

    assert!(message.contains("weather.lookup"));
    assert!(message.contains("weather/lookup"));
    assert!(message.contains("weather_lookup"));
}

#[test]
fn issue_761_anthropic_transform_response_restores_original_tool_names()
-> Result<(), crate::core::providers::unified_provider::ProviderError> {
    let client = anthropic_client();
    let request = ChatRequest::new("claude-3-opus-20240229")
        .add_user_message("weather?")
        .with_tools(vec![tool("weather.lookup")]);
    let tool_name_map = client.anthropic_tool_name_map_for_request(&request)?;
    let response = json!({
        "id": "msg_123",
        "model": "claude-3-opus-20240229",
        "content": [{
            "type": "tool_use",
            "id": "toolu_123",
            "name": "weather_lookup",
            "input": {"city": "Paris"}
        }],
        "stop_reason": "tool_use",
        "usage": {"input_tokens": 10, "output_tokens": 5}
    });

    let result = client.transform_chat_response_with_tool_name_map(response, &tool_name_map)?;
    let name = result.choices[0]
        .message
        .tool_calls
        .as_ref()
        .and_then(|tool_calls| tool_calls.first())
        .map(|tool_call| tool_call.function.name.as_str());

    assert_eq!(name, Some("weather.lookup"));
    Ok(())
}
