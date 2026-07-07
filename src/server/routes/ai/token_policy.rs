use crate::core::models::openai::requests::ChatCompletionRequest;
use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::chat::ChatRequest;
use crate::core::types::context::{RequestContext, SharedRequestContext};
use crate::utils::error::gateway_error::GatewayError;
use actix_web::HttpRequest;
use std::sync::Arc;

pub(super) fn attach_api_key_token_limit(
    req: &HttpRequest,
    context: &mut RequestContext,
) -> Result<(), GatewayError> {
    if let Some(limit) = super::context::api_key_max_tokens_per_request(req)? {
        context.set_api_key_max_tokens_per_request(limit);
    }
    Ok(())
}

pub(super) fn shared_request_context_with_api_key_token_limit(
    req: &HttpRequest,
) -> Result<SharedRequestContext, GatewayError> {
    let context = super::context::get_shared_request_context(req).map_err(|error| {
        GatewayError::internal(format!("Failed to extract request context: {error}"))
    })?;

    let Some(limit) = super::context::api_key_max_tokens_per_request(req)? else {
        return Ok(context);
    };

    if context.api_key_max_tokens_per_request() == Some(limit) {
        return Ok(context);
    }

    let mut context_with_limit = context.as_ref().clone();
    context_with_limit.set_api_key_max_tokens_per_request(limit);
    Ok(Arc::new(context_with_limit))
}

pub(super) fn requested_chat_output_token_limit(request: &ChatCompletionRequest) -> Option<u32> {
    requested_output_token_limit(request.max_tokens, request.max_completion_tokens)
}

pub(super) fn requested_output_token_limit(
    max_tokens: Option<u32>,
    max_completion_tokens: Option<u32>,
) -> Option<u32> {
    max_tokens.into_iter().chain(max_completion_tokens).max()
}

pub(super) fn apply_api_key_output_token_limit(
    max_tokens_per_request: Option<u32>,
    provider: &str,
    model: &str,
    request: &mut ChatRequest,
) -> Result<(), ProviderError> {
    let Some(limit) = max_tokens_per_request else {
        return Ok(());
    };

    if let Some(requested) =
        requested_output_token_limit(request.max_tokens, request.max_completion_tokens)
        && requested > limit
    {
        return Err(token_policy_error(requested, limit));
    }

    if request.max_tokens.is_none() {
        request.max_tokens = request.max_completion_tokens.or(Some(limit));
    }

    if let Some(effective) = provider_effective_output_cap(provider, model, request)
        && effective > limit
    {
        return Err(token_policy_error(effective, limit));
    }

    Ok(())
}

pub(super) fn prepare_chat_request_for_provider(
    max_tokens_per_request: Option<u32>,
    provider: &str,
    model: &str,
    mut core_request: ChatRequest,
) -> Result<ChatRequest, ProviderError> {
    core_request.model = model.to_string();
    apply_api_key_output_token_limit(max_tokens_per_request, provider, model, &mut core_request)?;
    Ok(core_request)
}

fn provider_effective_output_cap(
    provider: &str,
    model: &str,
    request: &ChatRequest,
) -> Option<u32> {
    let provider = crate::core::pricing::normalize_pricing_provider(provider);
    match provider.as_str() {
        "openai" | "azure" | "azure_ai" | "openai_like" | "openrouter" | "xai" | "groq"
        | "deepseek" | "moonshot" | "minimax" | "zhipuai" | "xiaomi_mimo" | "amazon_nova"
        | "baseten" | "huggingface" | "zai" | "together_ai" | "fireworks_ai" | "aiml" => {
            request.max_completion_tokens.or(request.max_tokens)
        }
        "anthropic" => Some(request.max_tokens.unwrap_or(4096)),
        "bedrock" => bedrock_effective_output_cap(model, request),
        "cohere" | "replicate" => request.max_tokens.or(request.max_completion_tokens),
        _ => request.max_tokens,
    }
}

fn bedrock_effective_output_cap(model: &str, request: &ChatRequest) -> Option<u32> {
    use crate::core::providers::bedrock::BedrockApiType;

    let Ok(config) = crate::core::providers::bedrock::get_model_config_for_model_id(model) else {
        return request.max_tokens;
    };

    match config.api_type {
        BedrockApiType::Converse | BedrockApiType::ConverseStream => {
            request.max_completion_tokens.or(request.max_tokens)
        }
        BedrockApiType::Invoke | BedrockApiType::InvokeStream => request.max_tokens,
    }
}

fn token_policy_error(requested: u32, limit: u32) -> ProviderError {
    ProviderError::authentication(
        "api_key",
        format!("requested token limit {requested} exceeds API key max_tokens_per_request {limit}"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::models::{ApiKey, Metadata, UsageStats};
    use crate::core::types::context::{RequestContext, SharedRequestContext};
    use actix_web::HttpMessage;
    use std::sync::Arc;

    fn api_key_with_max_tokens(max_tokens_per_request: u32) -> ApiKey {
        let mut metadata = Metadata::new();
        metadata.set_extra(
            "__core_keys",
            serde_json::json!({
                "permissions": {
                    "max_tokens_per_request": max_tokens_per_request
                }
            }),
        );

        ApiKey {
            metadata,
            name: "token-policy-test-key".to_string(),
            key_hash: "hash".to_string(),
            key_prefix: "sk-test".to_string(),
            user_id: None,
            team_id: None,
            permissions: vec![],
            rate_limits: None,
            expires_at: None,
            is_active: true,
            last_used_at: None,
            usage_stats: UsageStats::default(),
        }
    }

    #[test]
    fn shared_context_without_token_policy_reuses_extension_handle() {
        let context = Arc::new(RequestContext::new().with_header("x-large", "kept-by-ref"));
        let req = actix_web::test::TestRequest::default().to_http_request();
        req.extensions_mut()
            .insert::<SharedRequestContext>(Arc::clone(&context));

        let extracted = shared_request_context_with_api_key_token_limit(&req)
            .expect("shared context should be extracted");

        assert!(Arc::ptr_eq(&context, &extracted));
        assert_eq!(
            extracted.headers.get("x-large").map(String::as_str),
            Some("kept-by-ref")
        );
    }

    #[test]
    fn shared_context_with_token_policy_materializes_updated_context_once() {
        let original = Arc::new(RequestContext::new().with_header("x-large", "kept"));
        let req = actix_web::test::TestRequest::default().to_http_request();
        req.extensions_mut()
            .insert::<SharedRequestContext>(Arc::clone(&original));
        req.extensions_mut().insert(api_key_with_max_tokens(128));

        let extracted = shared_request_context_with_api_key_token_limit(&req)
            .expect("token-limited context should be extracted");

        assert!(!Arc::ptr_eq(&original, &extracted));
        assert_eq!(original.api_key_max_tokens_per_request(), None);
        assert_eq!(extracted.api_key_max_tokens_per_request(), Some(128));
        assert_eq!(
            extracted.headers.get("x-large").map(String::as_str),
            Some("kept")
        );
    }

    #[test]
    fn requested_output_token_limit_uses_largest_supplied_cap() {
        assert_eq!(requested_output_token_limit(Some(100), Some(10)), Some(100));
        assert_eq!(requested_output_token_limit(None, Some(10)), Some(10));
    }

    #[test]
    fn rejects_bypass_when_legacy_max_tokens_exceeds_limit() {
        let mut request = ChatRequest {
            max_tokens: Some(100),
            max_completion_tokens: Some(10),
            ..Default::default()
        };

        assert!(
            apply_api_key_output_token_limit(Some(20), "anthropic", "claude-3-haiku", &mut request)
                .is_err()
        );
    }

    #[test]
    fn fills_provider_effective_cap_when_only_max_completion_tokens_is_set() {
        let mut request = ChatRequest {
            max_completion_tokens: Some(10),
            ..Default::default()
        };

        apply_api_key_output_token_limit(Some(20), "anthropic", "claude-3-haiku", &mut request)
            .expect("max_completion_tokens should cap max_tokens-only providers");

        assert_eq!(request.max_tokens, Some(10));
    }

    #[test]
    fn issue_760_alias_providers_honor_max_completion_tokens() {
        let request = ChatRequest {
            max_completion_tokens: Some(10),
            ..Default::default()
        };

        for provider in [
            "zai",
            "together",
            "together_ai",
            "fireworks",
            "fireworks_ai",
            "aiml_api",
            "aiml",
        ] {
            assert_eq!(
                provider_effective_output_cap(provider, "model", &request),
                Some(10),
                "{provider} should honor max_completion_tokens"
            );
        }
    }

    #[test]
    fn caps_provider_default_when_request_omits_token_limit() {
        let mut request = ChatRequest::default();

        apply_api_key_output_token_limit(Some(20), "anthropic", "claude-3-haiku", &mut request)
            .expect("missing token cap should be filled from key limit");

        assert_eq!(request.max_tokens, Some(20));
    }
}
