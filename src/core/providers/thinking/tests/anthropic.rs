use super::*;

// ============================================================================
// Anthropic Thinking Tests
// ============================================================================

#[test]
fn test_anthropic_thinking_detection() {
    assert!(anthropic_thinking::supports_thinking("claude-3-opus"));
    assert!(anthropic_thinking::supports_thinking(
        "claude-3-5-sonnet-20241022"
    ));
    assert!(anthropic_thinking::supports_thinking("Claude-3-Opus")); // Case insensitive
    assert!(anthropic_thinking::supports_thinking("claude-4"));
    assert!(!anthropic_thinking::supports_thinking("claude-2"));
}

#[test]
fn test_anthropic_capabilities() {
    let caps = anthropic_thinking::capabilities("claude-3-opus");
    assert!(caps.supports_thinking);
    assert!(caps.supports_streaming_thinking);
    assert_eq!(caps.max_thinking_tokens, Some(100_000));
    assert_eq!(caps.supported_efforts.len(), 2);
    assert!(caps.can_return_thinking);
    assert!(!caps.thinking_always_on);
}

#[test]
fn test_anthropic_capabilities_non_thinking_model() {
    let caps = anthropic_thinking::capabilities("claude-2");
    assert!(!caps.supports_thinking);
}

#[test]
fn test_anthropic_config_transform_enabled() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(50000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
    let thinking_obj = result.get("thinking").unwrap();
    assert_eq!(thinking_obj.get("type").unwrap().as_str(), Some("enabled"));
    assert_eq!(
        thinking_obj.get("budget_tokens").unwrap().as_u64(),
        Some(50000)
    );
}

#[test]
fn test_anthropic_config_transform_disabled() {
    let config = ThinkingConfig {
        enabled: false,
        budget_tokens: Some(50000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
    assert!(result.get("thinking").is_none());
}

#[test]
fn test_anthropic_config_transform_no_budget() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: None,
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = anthropic_thinking::transform_config(&config, "claude-3-opus").unwrap();
    let thinking_obj = result.get("thinking").unwrap();
    assert_eq!(thinking_obj.get("type").unwrap().as_str(), Some("enabled"));
    assert!(thinking_obj.get("budget_tokens").is_none());
}

#[test]
fn test_anthropic_thinking_extraction() {
    let response = serde_json::json!({
        "content": [
            {
                "type": "thinking",
                "thinking": "Let me analyze this carefully..."
            },
            {
                "type": "text",
                "text": "The answer is 42."
            }
        ]
    });

    let thinking = anthropic_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
    if let Some(ThinkingContent::Block { thinking, .. }) = thinking {
        assert!(thinking.contains("analyze"));
    }
}

#[test]
fn test_anthropic_thinking_extraction_no_thinking_block() {
    let response = serde_json::json!({
        "content": [
            {
                "type": "text",
                "text": "The answer is 42."
            }
        ]
    });

    let thinking = anthropic_thinking::extract_thinking(&response);
    assert!(thinking.is_none());
}

#[test]
fn test_anthropic_usage_extraction() {
    let response = serde_json::json!({
        "usage": {
            "thinking_tokens": 500,
            "thinking_budget_tokens": 100000
        }
    });

    let usage = anthropic_thinking::extract_usage(&response);
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.thinking_tokens, Some(500));
    assert_eq!(usage.budget_tokens, Some(100000));
    assert_eq!(usage.provider, Some("anthropic".to_string()));
}

#[test]
fn test_anthropic_usage_extraction_partial() {
    let response = serde_json::json!({
        "usage": {
            "thinking_tokens": 500
        }
    });

    let usage = anthropic_thinking::extract_usage(&response);
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.thinking_tokens, Some(500));
    assert!(usage.budget_tokens.is_none());
}

#[test]
fn test_anthropic_usage_extraction_missing() {
    let response = serde_json::json!({
        "usage": {
            "total_tokens": 1000
        }
    });

    let usage = anthropic_thinking::extract_usage(&response);
    assert!(usage.is_none());
}
