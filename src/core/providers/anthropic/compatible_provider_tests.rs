use super::{AnthropicConfig, AnthropicProvider};
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{chat::ChatRequest, context::RequestContext};

fn compatible_provider() -> AnthropicProvider {
    let config = AnthropicConfig::new_test("test-key")
        .with_base_url("https://token-plan-sgp.xiaomimimo.com/anthropic")
        .with_allow_unknown_models(true)
        .with_configured_models(vec!["mimo-v2.5".to_string()]);
    AnthropicProvider::new(config).unwrap_or_else(|err| panic!("provider should build: {err}"))
}

#[tokio::test]
async fn compatible_models_allow_empty_tools_without_forwarding_tools() {
    let provider = compatible_provider();
    let mut request = ChatRequest::new("mimo-v2.5").add_user_message("Hello");
    request.tools = Some(vec![]);

    let transformed = provider
        .transform_request(request, RequestContext::new())
        .await
        .unwrap_or_else(|err| panic!("empty tools should not declare tool support: {err}"));

    assert!(transformed.get("tools").is_none());
}

#[tokio::test]
async fn compatible_models_reject_legacy_functions() {
    let provider = compatible_provider();
    let mut request = ChatRequest::new("mimo-v2.5").add_user_message("Hello");
    request.functions = Some(vec![serde_json::json!({"name": "lookup"})]);

    let err = match provider
        .transform_request(request, RequestContext::new())
        .await
    {
        Ok(_) => panic!("compatible models must reject legacy function definitions"),
        Err(err) => err,
    };

    assert!(format!("{err}").contains("tool calling support"));
}
