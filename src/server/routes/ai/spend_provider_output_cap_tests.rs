use super::*;
use crate::core::models::openai::requests::ChatCompletionRequest;

#[test]
fn providers_that_ignore_max_completion_tokens_use_max_tokens_only() {
    let mut request = ChatCompletionRequest {
        model: "model".to_string(),
        ..Default::default()
    };
    request.max_completion_tokens = Some(10);

    for provider in ["ollama", "sagemaker", "snowflake"] {
        assert_eq!(
            provider_effective_max_output_tokens(provider, "model", &request),
            None,
            "{provider} must not reserve against ignored max_completion_tokens"
        );
    }

    request.max_tokens = Some(100);
    for provider in ["ollama", "sagemaker", "snowflake"] {
        assert_eq!(
            provider_effective_max_output_tokens(provider, "model", &request),
            Some(100),
            "{provider} should reserve against max_tokens"
        );
    }
}

#[test]
fn anthropic_without_max_tokens_reserves_adapter_default_output_cap() {
    let mut request = ChatCompletionRequest {
        model: "claude-sonnet-4-20250514".to_string(),
        ..Default::default()
    };
    request.max_completion_tokens = Some(100_000);

    assert_eq!(
        provider_effective_max_output_tokens("anthropic", &request.model, &request),
        Some(4096),
        "Anthropic sends max_tokens default 4096 when max_tokens is omitted"
    );

    request.max_tokens = Some(512);
    assert_eq!(
        provider_effective_max_output_tokens("anthropic", &request.model, &request),
        Some(512),
        "explicit max_tokens is the effective Anthropic output cap"
    );
}
