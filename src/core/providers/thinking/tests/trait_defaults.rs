use super::*;

// ============================================================================
// ThinkingProvider Trait Default Methods Tests
// ============================================================================

struct MockThinkingProvider;

impl ThinkingProvider for MockThinkingProvider {
    fn supports_thinking(&self, _model: &str) -> bool {
        true
    }

    fn thinking_capabilities(&self, _model: &str) -> ThinkingCapabilities {
        ThinkingCapabilities {
            supports_thinking: true,
            supports_streaming_thinking: true,
            max_thinking_tokens: Some(5000),
            supported_efforts: vec![ThinkingEffort::Medium, ThinkingEffort::High],
            thinking_models: vec!["test-model".to_string()],
            can_return_thinking: true,
            thinking_always_on: false,
        }
    }

    fn transform_thinking_config(
        &self,
        _config: &ThinkingConfig,
        _model: &str,
    ) -> Result<Value, ProviderError> {
        Ok(serde_json::json!({}))
    }

    fn extract_thinking(&self, _response: &Value) -> Option<ThinkingContent> {
        None
    }

    fn extract_thinking_usage(&self, _response: &Value) -> Option<ThinkingUsage> {
        None
    }
}

#[test]
fn test_thinking_provider_default_effort() {
    let provider = MockThinkingProvider;
    assert_eq!(provider.default_thinking_effort(), ThinkingEffort::Medium);
}

#[test]
fn test_thinking_provider_max_thinking_tokens() {
    let provider = MockThinkingProvider;
    assert_eq!(provider.max_thinking_tokens("test-model"), Some(5000));
}

#[test]
fn test_thinking_provider_supports_streaming_thinking() {
    let provider = MockThinkingProvider;
    assert!(provider.supports_streaming_thinking("test-model"));
}
