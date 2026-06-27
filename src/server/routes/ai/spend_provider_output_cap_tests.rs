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
