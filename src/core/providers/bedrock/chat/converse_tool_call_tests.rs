use super::*;
use crate::core::types::{chat::ChatMessage, message::MessageRole};

#[test]
fn assistant_tool_call_preserves_valid_arguments_json() {
    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::Assistant,
            content: None,
            tool_calls: Some(vec![crate::core::types::tools::ToolCall {
                id: "tool-123".to_string(),
                tool_type: "function".to_string(),
                function: crate::core::types::tools::FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: r#"{"city":"Paris","units":["metric"]}"#.to_string(),
                },
            }]),
            ..Default::default()
        }],
        ..Default::default()
    };

    let Ok(converse) = transform_to_converse(&request) else {
        panic!("valid tool-call arguments JSON should transform");
    };
    let Ok(json) = serde_json::to_value(&converse) else {
        panic!("Converse request should serialize");
    };
    let input = &json["messages"][0]["content"][0]["toolUse"]["input"];

    assert_eq!(input["city"], "Paris");
    assert_eq!(input["units"][0], "metric");
}

#[test]
fn assistant_tool_call_rejects_invalid_arguments_json() {
    let request = ChatRequest {
        model: "anthropic.claude-3-sonnet".to_string(),
        messages: vec![ChatMessage {
            role: MessageRole::Assistant,
            content: None,
            tool_calls: Some(vec![crate::core::types::tools::ToolCall {
                id: "tool-123".to_string(),
                tool_type: "function".to_string(),
                function: crate::core::types::tools::FunctionCall {
                    name: "get_weather".to_string(),
                    arguments: "{city:Paris}".to_string(),
                },
            }]),
            ..Default::default()
        }],
        ..Default::default()
    };

    let err = match transform_to_converse(&request) {
        Ok(_) => panic!("invalid tool-call arguments JSON should be rejected"),
        Err(err) => err,
    };
    let message = format!("{err}");

    assert!(message.contains("Invalid Bedrock assistant tool-call arguments JSON"));
    assert!(message.contains("tool-123"));
    assert!(message.contains("get_weather"));
}
