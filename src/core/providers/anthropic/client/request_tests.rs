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
fn issue_761_anthropic_transform_tools_rejects_sanitized_name_collisions() {
    let error = anthropic_client()
        .transform_tools(&[tool("weather.lookup"), tool("weather/lookup")])
        .expect_err("colliding sanitized names must fail closed");

    assert!(error.to_string().contains("collides"));
}
