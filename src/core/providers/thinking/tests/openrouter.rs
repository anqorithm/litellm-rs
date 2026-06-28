use super::*;

// ============================================================================
// OpenRouter Thinking Tests
// ============================================================================

#[test]
fn test_openrouter_thinking_detection() {
    assert!(openrouter_thinking::supports_thinking("openai/o1-preview"));
    assert!(openrouter_thinking::supports_thinking("o1-mini"));
    assert!(openrouter_thinking::supports_thinking(
        "anthropic/claude-3-opus"
    ));
    assert!(openrouter_thinking::supports_thinking("claude-3-sonnet"));
    assert!(openrouter_thinking::supports_thinking(
        "deepseek/deepseek-r1"
    ));
    assert!(openrouter_thinking::supports_thinking("deepseek-reasoner"));
    assert!(openrouter_thinking::supports_thinking(
        "google/gemini-thinking"
    ));
    assert!(openrouter_thinking::supports_thinking(
        "gemini-2.0-flash-thinking"
    ));
    assert!(!openrouter_thinking::supports_thinking("gpt-4"));
}

#[test]
fn test_openrouter_provider_detection() {
    assert_eq!(
        openrouter_thinking::detect_provider("openai/o1-preview"),
        "openai"
    );
    assert_eq!(openrouter_thinking::detect_provider("o1-mini"), "openai");
    assert_eq!(openrouter_thinking::detect_provider("o3-mini"), "openai");
    assert_eq!(
        openrouter_thinking::detect_provider("anthropic/claude-3-opus"),
        "anthropic"
    );
    assert_eq!(
        openrouter_thinking::detect_provider("claude-3-5-sonnet"),
        "anthropic"
    );
    assert_eq!(
        openrouter_thinking::detect_provider("deepseek/deepseek-r1"),
        "deepseek"
    );
    assert_eq!(
        openrouter_thinking::detect_provider("google/gemini-thinking"),
        "gemini"
    );
    assert_eq!(openrouter_thinking::detect_provider("gemini-pro"), "gemini");
    assert_eq!(
        openrouter_thinking::detect_provider("unknown-model"),
        "unknown"
    );
}

#[test]
fn test_openrouter_capabilities_openai() {
    let caps = openrouter_thinking::capabilities("openai/o1-preview");
    assert!(caps.supports_thinking);
    assert!(!caps.supports_streaming_thinking);
}

#[test]
fn test_openrouter_capabilities_anthropic() {
    let caps = openrouter_thinking::capabilities("anthropic/claude-3-opus");
    assert!(caps.supports_thinking);
    assert!(caps.supports_streaming_thinking);
}

#[test]
fn test_openrouter_capabilities_deepseek() {
    let caps = openrouter_thinking::capabilities("deepseek/deepseek-r1");
    assert!(caps.supports_thinking);
    assert!(caps.thinking_always_on);
}

#[test]
fn test_openrouter_capabilities_gemini() {
    let caps = openrouter_thinking::capabilities("google/gemini-thinking");
    assert!(caps.supports_thinking);
}

#[test]
fn test_openrouter_capabilities_unknown() {
    let caps = openrouter_thinking::capabilities("unknown-model");
    assert!(!caps.supports_thinking);
}

#[test]
fn test_openrouter_transform_config_openai() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(10000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = openrouter_thinking::transform_config(&config, "openai/o1-preview").unwrap();
    assert!(result.get("max_reasoning_tokens").is_some());
    assert!(result.get("reasoning_effort").is_some());
}

#[test]
fn test_openrouter_transform_config_anthropic() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(50000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = openrouter_thinking::transform_config(&config, "anthropic/claude-3-opus").unwrap();
    assert!(result.get("thinking").is_some());
}

#[test]
fn test_openrouter_transform_config_deepseek() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(5000),
        effort: Some(ThinkingEffort::Medium),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = openrouter_thinking::transform_config(&config, "deepseek/deepseek-r1").unwrap();
    assert!(result.get("reasoning_effort").is_some());
}

#[test]
fn test_openrouter_transform_config_gemini() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(15000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    let result = openrouter_thinking::transform_config(&config, "google/gemini-thinking").unwrap();
    assert!(result.get("enableThinking").is_some());
}

#[test]
fn test_openrouter_transform_config_unknown() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(10000),
        effort: Some(ThinkingEffort::High),
        include_thinking: true,
        extra_params: Default::default(),
    };

    // Unknown models use the OpenRouter-native reasoning.effort format
    let Ok(result) = openrouter_thinking::transform_config(&config, "unknown-model") else {
        panic!("transform_config must succeed for unknown-model");
    };
    assert_eq!(
        result
            .get("reasoning")
            .and_then(|r| r.get("effort"))
            .and_then(|v| v.as_str()),
        Some("high"),
        "reasoning.effort should be 'high' for High effort"
    );
    // When effort is set, max_tokens must NOT be emitted; they are mutually exclusive on OpenRouter.
    assert!(
        result
            .get("reasoning")
            .and_then(|r| r.get("max_tokens"))
            .is_none(),
        "reasoning.max_tokens must not be set when effort is also set"
    );
}

#[test]
fn test_openrouter_transform_config_unknown_budget_only() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: Some(8000),
        effort: None,
        include_thinking: true,
        extra_params: Default::default(),
    };

    let Ok(result) = openrouter_thinking::transform_config(&config, "unknown-model") else {
        panic!("transform_config must succeed for unknown-model");
    };
    assert_eq!(
        result
            .get("reasoning")
            .and_then(|r| r.get("max_tokens"))
            .and_then(|v| v.as_u64()),
        Some(8000),
        "reasoning.max_tokens should equal budget_tokens when effort is None"
    );
    assert!(
        result
            .get("reasoning")
            .and_then(|r| r.get("effort"))
            .is_none(),
        "reasoning.effort must not be set when effort is None"
    );
}

#[test]
fn test_openrouter_transform_config_unknown_no_effort() {
    let config = ThinkingConfig {
        enabled: true,
        budget_tokens: None,
        effort: None,
        include_thinking: true,
        extra_params: Default::default(),
    };

    // No effort specified: no reasoning object emitted
    let Ok(result) = openrouter_thinking::transform_config(&config, "unknown-model") else {
        panic!("transform_config must succeed for unknown-model");
    };
    assert!(
        result.as_object().is_some_and(|m| m.is_empty()),
        "no reasoning key when effort is None"
    );
}

#[test]
fn test_openrouter_extract_thinking_openai() {
    let response = serde_json::json!({
        "choices": [{
            "message": {
                "reasoning": "OpenAI reasoning content"
            }
        }]
    });

    let thinking = openrouter_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
}

#[test]
fn test_openrouter_extract_thinking_deepseek() {
    let response = serde_json::json!({
        "choices": [{
            "message": {
                "reasoning_content": "DeepSeek reasoning content"
            }
        }]
    });

    let thinking = openrouter_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
}

#[test]
fn test_openrouter_extract_thinking_anthropic() {
    let response = serde_json::json!({
        "content": [
            {
                "type": "thinking",
                "thinking": "Anthropic thinking content"
            }
        ]
    });

    let thinking = openrouter_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
}

#[test]
fn test_openrouter_extract_thinking_gemini() {
    let response = serde_json::json!({
        "candidates": [{
            "content": {
                "thoughts": "Gemini thoughts content"
            }
        }]
    });

    let thinking = openrouter_thinking::extract_thinking(&response);
    assert!(thinking.is_some());
}

#[test]
fn test_openrouter_extract_thinking_none() {
    let response = serde_json::json!({
        "choices": [{
            "message": {
                "content": "Regular response"
            }
        }]
    });

    let thinking = openrouter_thinking::extract_thinking(&response);
    assert!(thinking.is_none());
}

#[test]
fn test_openrouter_extract_usage_openai() {
    let response = serde_json::json!({
        "usage": {
            "reasoning_tokens": 500
        }
    });

    let usage = openrouter_thinking::extract_usage(&response);
    assert!(usage.is_some());
    let usage = usage.unwrap();
    assert_eq!(usage.thinking_tokens, Some(500));
    assert_eq!(usage.provider, Some("openrouter".to_string()));
}

#[test]
fn test_openrouter_extract_usage_deepseek() {
    let response = serde_json::json!({
        "usage": {
            "reasoning_tokens": 800
        }
    });

    let usage = openrouter_thinking::extract_usage(&response);
    assert!(usage.is_some());
    assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
}

#[test]
fn test_openrouter_extract_usage_anthropic() {
    let response = serde_json::json!({
        "usage": {
            "thinking_tokens": 600
        }
    });

    let usage = openrouter_thinking::extract_usage(&response);
    assert!(usage.is_some());
    assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
}

#[test]
fn test_openrouter_extract_usage_gemini() {
    let response = serde_json::json!({
        "usageMetadata": {
            "thinkingTokenCount": 1000
        }
    });

    let usage = openrouter_thinking::extract_usage(&response);
    assert!(usage.is_some());
    assert_eq!(usage.unwrap().provider, Some("openrouter".to_string()));
}

#[test]
fn test_openrouter_extract_usage_none() {
    let response = serde_json::json!({
        "usage": {
            "total_tokens": 100
        }
    });

    let usage = openrouter_thinking::extract_usage(&response);
    assert!(usage.is_none());
}
