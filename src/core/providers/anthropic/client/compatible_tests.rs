use super::*;
use crate::core::providers::anthropic::config::AnthropicConfig;
use crate::core::types::{
    content::{ContentPart, ImageSource, ImageUrl},
    message::{MessageContent, MessageRole},
};

#[test]
fn configured_unknown_model_serializes_image_url_parts() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()])
        .with_configured_multimodal_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let request = ChatRequest::new("mimo-v2.5").add_message(
        MessageRole::User,
        MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Describe this image".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "data:image/png;base64,ZmFrZQ==".to_string(),
                    detail: None,
                },
            },
        ]),
    );

    let result = match client.transform_chat_request(&request) {
        Ok(result) => result,
        Err(err) => panic!("configured multimodal model should serialize image input: {err}"),
    };

    assert_eq!(result["model"], "mimo-v2.5");
    assert_eq!(result["messages"][0]["content"][1]["type"], "image");
}

#[test]
fn configured_unknown_model_serializes_image_parts() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()])
        .with_configured_multimodal_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let request = ChatRequest::new("mimo-v2.5").add_message(
        MessageRole::User,
        MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Describe this image".to_string(),
            },
            ContentPart::Image {
                source: ImageSource {
                    media_type: "image/png".to_string(),
                    data: "ZmFrZQ==".to_string(),
                },
                detail: None,
                image_url: None,
            },
        ]),
    );

    let result = match client.transform_chat_request(&request) {
        Ok(result) => result,
        Err(err) => panic!("configured multimodal model should serialize image input: {err}"),
    };

    assert_eq!(result["messages"][0]["content"][1]["type"], "image");
    assert_eq!(
        result["messages"][0]["content"][1]["source"]["data"],
        "ZmFrZQ=="
    );
}

#[test]
fn text_only_configured_unknown_model_rejects_image_input() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5-pro".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let request = ChatRequest::new("mimo-v2.5-pro").add_message(
        MessageRole::User,
        MessageContent::Parts(vec![
            ContentPart::Text {
                text: "Describe this image".to_string(),
            },
            ContentPart::ImageUrl {
                image_url: ImageUrl {
                    url: "data:image/png;base64,ZmFrZQ==".to_string(),
                    detail: None,
                },
            },
        ]),
    );

    let err = match client.transform_chat_request(&request) {
        Ok(_) => panic!("text-only compatible model must reject image input"),
        Err(err) => err,
    };

    assert!(format!("{err}").contains("does not support image input"));
}

#[test]
fn compatible_allow_list_rejects_registry_model_ids_not_configured() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let request = ChatRequest::new("claude-3-haiku-20240307").add_user_message("Hello");

    let err = match client.transform_chat_request(&request) {
        Ok(_) => panic!("compatible allow-list must reject unlisted registry model IDs"),
        Err(err) => err,
    };

    assert!(format!("{err}").contains("Unsupported model: claude-3-haiku-20240307"));
}

#[test]
fn compatible_models_allow_empty_tools_without_forwarding_tools() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let mut request = ChatRequest::new("mimo-v2.5").add_user_message("Hello");
    request.tools = Some(vec![]);

    let transformed = client
        .transform_chat_request(&request)
        .unwrap_or_else(|err| panic!("empty tools should not declare tool support: {err}"));

    assert!(transformed.get("tools").is_none());
}

#[test]
fn compatible_models_reject_legacy_functions() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let mut request = ChatRequest::new("mimo-v2.5").add_user_message("Hello");
    request.functions = Some(vec![serde_json::json!({"name": "lookup"})]);

    let err = match client.transform_chat_request(&request) {
        Ok(_) => panic!("compatible models must reject legacy function definitions"),
        Err(err) => err,
    };

    assert!(format!("{err}").contains("tool calling support"));
}

#[test]
fn compatible_models_reject_tool_role_messages() {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()]);
    let client = match AnthropicClient::new(config) {
        Ok(client) => client,
        Err(err) => panic!("client should build: {err}"),
    };
    let request = ChatRequest::new("mimo-v2.5").add_message(
        MessageRole::Tool,
        MessageContent::Text("tool result".to_string()),
    );

    let err = match client.transform_chat_request(&request) {
        Ok(_) => panic!("compatible models must reject tool-role messages"),
        Err(err) => err,
    };

    assert!(format!("{err}").contains("only supports text and image content"));
}
