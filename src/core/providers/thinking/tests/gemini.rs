use super::*;

// ============================================================================
// Gemini Thinking Tests
// ============================================================================

#[test]
fn test_gemini_thinking_detection() {
    assert!(gemini_thinking::supports_thinking(
        "gemini-2.0-flash-thinking-exp"
    ));
    assert!(gemini_thinking::supports_thinking("gemini-thinking"));
    assert!(gemini_thinking::supports_thinking("gemini-3.0-deep-think"));
    assert!(gemini_thinking::supports_thinking("Gemini-Thinking")); // Case insensitive
    assert!(gemini_thinking::supports_thinking("gemini-deep-think"));
    assert!(!gemini_thinking::supports_thinking("gemini-pro"));
}

#[test]
fn test_gemini_capabilities() {
    let caps = gemini_thinking::capabilities("gemini-2.0-flash-thinking");
    assert!(caps.supports_thinking);
    assert!(caps.supports_streaming_thinking);
    assert_eq!(caps.max_thinking_tokens, Some(32_000));
    assert_eq!(caps.supported_efforts.len(), 2);
    assert!(caps.can_return_thinking);
    assert!(!caps.thinking_always_on);
}

#[test]
fn test_gemini_capabilities_non_thinking_model() {
    let caps = gemini_thinking::capabilities("gemini-pro");
    assert!(!caps.supports_thinking);
}

#[test]
fn test_gemini_config_transform_enabled() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(10000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
    assert_eq!(result.get("enableThinking").unwrap().as_bool(), Some(true));
    assert_eq!(result.get("thinkingBudget").unwrap().as_u64(), Some(10000));
}

#[test]
fn test_gemini_config_transform_disabled() {
    let config = ThinkingConfig {
        enabled: false,
        budget_tokens: Some(10000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
    assert!(result.get("enableThinking").is_none());
}

#[test]
fn test_gemini_config_transform_no_budget() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: None,
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = gemini_thinking::transform_config(&config, "gemini-thinking").unwrap();
    assert_eq!(result.get("enableThinking").unwrap().as_bool(), Some(true));
    assert!(result.get("thinkingBudget").is_none());
}

#[test]
fn test_gemini_thinking_extraction_thoughts() {
    let response = serde_json::json!({
        "candidates": [{
            "content": {
                "thoughts": "Let me think through this problem..."
            }
        }]
    });

    let thinking = gemini_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
    if let Some(ThinkingContent::Text { text, .. }) = thinking {
        assert!(text.contains("think through"));
    }
}

#[test]
fn test_gemini_thinking_extraction_thinking() {
    let response = serde_json::json!({
        "candidates": [{
            "content": {
                "thinking": "Analyzing the data..."
            }
        }]
    });

    let thinking = gemini_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
    if let Some(ThinkingContent::Text { text, .. }) = thinking {
        assert!(text.contains("Analyzing"));
    }
}

#[test]
fn test_gemini_thinking_extraction_missing() {
    let response = serde_json::json!({
        "candidates": [{
            "content": {
                "text": "The answer is 42."
            }
        }]
    });

    let thinking = gemini_thinking::extract_thinking(&response);
    assert!(thinking.is_none());
}

#[test]
fn test_gemini_usage_extraction() {
    let response = serde_json::json!({
        "usageMetadata": {
            "thinkingTokenCount": 1200
        }
    });

    let usage = gemini_thinking::extract_usage(&response);
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.thinking_tokens, Some(1200));
    assert_eq!(usage.provider, Some("gemini".to_string()));
}

#[test]
fn test_gemini_usage_extraction_missing() {
    let response = serde_json::json!({
        "usageMetadata": {
            "totalTokenCount": 2000
        }
    });

    let usage = gemini_thinking::extract_usage(&response);
    assert!(usage.is_none());
}
