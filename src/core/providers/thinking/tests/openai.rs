use super::*;

// ============================================================================
// OpenAI Thinking Tests
// ============================================================================

#[test]
fn test_openai_thinking_detection() {
    assert!(openai_thinking::supports_thinking("o1"));
    assert!(openai_thinking::supports_thinking("o1-preview"));
    assert!(openai_thinking::supports_thinking("o3-mini"));
    assert!(openai_thinking::supports_thinking("O1-PREVIEW")); // Case insensitive
    assert!(openai_thinking::supports_thinking("o4"));
    assert!(openai_thinking::supports_thinking("openai/o1-preview")); // With prefix
    assert!(!openai_thinking::supports_thinking("gpt-4"));
    assert!(!openai_thinking::supports_thinking("gpt-4o"));
}

#[test]
fn test_openai_capabilities() {
    let caps = openai_thinking::capabilities("o1-preview");
    assert!(caps.supports_thinking);
    assert!(!caps.supports_streaming_thinking);
    assert_eq!(caps.max_thinking_tokens, Some(20_000));
    assert_eq!(caps.supported_efforts.len(), 3);
    assert!(caps.can_return_thinking);
    assert!(!caps.thinking_always_on);
}

#[test]
fn test_openai_capabilities_non_thinking_model() {
    let caps = openai_thinking::capabilities("gpt-4");
    assert!(!caps.supports_thinking);
}

#[test]
fn test_openai_config_transform() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(10000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = openai_thinking::transform_config(&config, "o1").unwrap();
    assert!(result.get("max_reasoning_tokens").is_some());
    assert_eq!(
        result.get("max_reasoning_tokens").unwrap().as_u64(),
        Some(10000)
    );
    assert!(result.get("include_reasoning").is_some());
    assert_eq!(
        result.get("reasoning_effort").unwrap().as_str(),
        Some("high")
    );
}

#[test]
fn test_openai_config_transform_budget_capping() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(50000), // Over the 20k limit
        effort: Some(ThinkingEffort::Medium),
        include_thinking: false,
        extra_params: Default::default(),
    };

    let result = openai_thinking::transform_config(&config, "o1").unwrap();
    assert_eq!(
        result.get("max_reasoning_tokens").unwrap().as_u64(),
        Some(20000)
    );
    assert_eq!(
        result.get("reasoning_effort").unwrap().as_str(),
        Some("medium")
    );
    assert!(result.get("include_reasoning").is_none());
}

#[test]
fn test_openai_config_transform_minimal() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: None,
        effort: None,
        include_thinking: false,
        extra_params: Default::default(),
    };

    let result = openai_thinking::transform_config(&config, "o1").unwrap();
    assert!(result.get("max_reasoning_tokens").is_none());
    assert!(result.get("reasoning_effort").is_none());
    assert!(result.get("include_reasoning").is_none());
}

#[test]
fn test_openai_config_transform_low_effort() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: None,
        effort: Some(ThinkingEffort::Low),
        include_thinking: false,
        extra_params: Default::default(),
    };

    let result = openai_thinking::transform_config(&config, "o1").unwrap();
    assert_eq!(
        result.get("reasoning_effort").unwrap().as_str(),
        Some("low")
    );
}

#[test]
fn test_openai_thinking_extraction() {
    let openai_response = serde_json::json!({
        "choices": [{
            "message": {
                "content": "The answer is 42.",
                "reasoning": "Let me think about this step by step..."
            }
        }],
        "usage": {
            "reasoning_tokens": 150
        }
    });

    let thinking = openai_thinking::extract_thinking(&openai_response);
    assert!(thinking.is_some());
    if let Some(ThinkingContent::Text { text, .. }) = thinking {
        assert!(text.contains("step by step"));
    }
}

#[test]
fn test_openai_thinking_extraction_missing() {
    let response = serde_json::json!({
        "choices": [{
            "message": {
                "content": "The answer is 42."
            }
        }]
    });

    let thinking = openai_thinking::extract_thinking(&response);
    assert!(thinking.is_none());
}

#[test]
fn test_openai_usage_extraction() {
    let openai_response = serde_json::json!({
        "usage": {
            "reasoning_tokens": 150
        }
    });

    let usage = openai_thinking::extract_usage(&openai_response);
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.thinking_tokens, Some(150));
    assert_eq!(usage.provider, Some("openai".to_string()));
}

#[test]
fn test_openai_usage_extraction_missing() {
    let response = serde_json::json!({
        "usage": {
            "total_tokens": 100
        }
    });

    let usage = openai_thinking::extract_usage(&response);
    assert!(usage.is_none());
}
