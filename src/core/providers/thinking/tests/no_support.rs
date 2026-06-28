use super::*;

// ============================================================================
// NoThinkingSupport Tests
// ============================================================================

#[test]
fn test_no_thinking_support() {
    let no_support = NoThinkingSupport;
    assert!(!no_support.supports_thinking("any-model"));
    assert!(
        no_support
            .extract_thinking(&serde_json::json!({}))
            .is_none()
    );
}

#[test]
fn test_no_thinking_support_capabilities() {
    let no_support = NoThinkingSupport;
    let caps = no_support.thinking_capabilities("any-model");
    assert!(!caps.supports_thinking);
    assert!(!caps.supports_streaming_thinking);
    assert!(!caps.can_return_thinking);
}

#[test]
fn test_no_thinking_support_transform_config() {
    let no_support = NoThinkingSupport;
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(1000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };
    let result = no_support
        .transform_thinking_config(&config, "model")
        .unwrap();
    assert!(result.as_object().unwrap().is_empty());
}

#[test]
fn test_no_thinking_support_extract_usage() {
    let no_support = NoThinkingSupport;
    let response = serde_json::json!({
        "usage": {
            "thinking_tokens": 100
        }
    });
    assert!(no_support.extract_thinking_usage(&response).is_none());
}
