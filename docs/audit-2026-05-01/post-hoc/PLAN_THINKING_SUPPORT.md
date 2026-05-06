# Unified Thinking/Reasoning Support Implementation Plan

> **ARCHIVED — 2026-05-07.** This plan was already mostly delivered through the audit-remediation campaign (see [`closeout-2026-05-02.md`](../closeout-2026-05-02.md)) and earlier provider work. Phase 1 types (`src/core/types/thinking.rs`, `ChatMessage.thinking`, `ChatRequest.thinking`), Phase 2 OpenAI / Anthropic / Gemini transforms, and Phase 4 cost are landed. DeepSeek and OpenRouter inherit thinking via `OpenAILikeProvider` + `reasoning_content`. The unfinished `trait ThinkingTransformer` abstraction was reviewed as YAGNI and intentionally not built. Preserved for cross-reference.

## Overview

This plan adds comprehensive thinking/reasoning support across all AI providers in litellm-rs. The goal is to create a unified abstraction that handles:
- OpenAI o1/o3/o4 reasoning
- Anthropic Claude extended thinking
- DeepSeek R1/Reasoner
- Gemini 2.0 Flash Thinking / 3.0 Deep Think
- OpenRouter passthrough

## Current State Analysis

| Provider | Request Config | Response Extract | Token Track | Cost Track |
|----------|----------------|------------------|-------------|------------|
| OpenAI | ✓ Partial | ✓ Partial | ✓ Yes | Partial |
| Anthropic | ✗ Minimal | ✗ Missing | ✗ Missing | ✗ Missing |
| DeepSeek | ✗ Missing | ✗ Missing | ✗ Missing | ✗ Missing |
| Gemini | ✗ Missing | ✗ Missing | ✗ Missing | ✗ Missing |
| OpenRouter | ✗ Passthrough | ✗ Missing | ✗ Missing | ✗ Missing |

## Implementation Steps

### Phase 1: Core Types (Foundation)

#### Step 1.1: Create unified thinking types
**File**: `src/core/types/thinking.rs` (NEW)

```rust
/// Unified thinking content - provider agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ThinkingContent {
    /// Text-based thinking (most providers)
    Text {
        text: String,
        /// Optional signature for verification
        signature: Option<String>,
    },
    /// Structured thinking blocks (Anthropic style)
    Block {
        thinking: String,
        /// Block type identifier
        block_type: Option<String>,
    },
    /// Redacted thinking (when provider hides details)
    Redacted {
        token_count: Option<u32>,
    },
}

/// Unified thinking request configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingConfig {
    /// Enable thinking mode
    #[serde(default)]
    pub enabled: bool,

    /// Maximum thinking tokens budget
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u32>,

    /// Thinking effort level (normalized across providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ThinkingEffort>,

    /// Include thinking content in response
    #[serde(default = "default_include_thinking")]
    pub include_thinking: bool,
}

/// Thinking effort levels (provider-agnostic)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingEffort {
    /// Minimal thinking - fast responses
    Low,
    /// Balanced thinking (default)
    Medium,
    /// Deep thinking - thorough reasoning
    High,
}

/// Thinking usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingUsage {
    /// Tokens used for thinking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_tokens: Option<u32>,

    /// Budget that was allocated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget_tokens: Option<u32>,

    /// Cost for thinking (USD)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_cost: Option<f64>,
}

/// Provider-specific thinking capabilities
#[derive(Debug, Clone)]
pub struct ThinkingCapabilities {
    pub supports_thinking: bool,
    pub supports_streaming_thinking: bool,
    pub max_thinking_tokens: Option<u32>,
    pub supported_efforts: Vec<ThinkingEffort>,
    pub thinking_models: Vec<String>,
}
```

#### Step 1.2: Update ChatMessage
**File**: `src/core/types/chat.rs`

Add thinking field:
```rust
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: Option<MessageContent>,

    // NEW: Thinking content
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingContent>,

    // ... existing fields
}
```

#### Step 1.3: Update ChatDelta for streaming
**File**: `src/core/types/responses/delta.rs`

```rust
pub struct ChatDelta {
    pub role: Option<MessageRole>,
    pub content: Option<String>,

    // NEW: Thinking delta
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<String>,

    // ... existing fields
}
```

#### Step 1.4: Update Usage types
**File**: `src/core/types/responses/usage.rs`

```rust
pub struct Usage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,

    // NEW: Thinking usage
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_usage: Option<ThinkingUsage>,

    // ... existing fields
}
```

#### Step 1.5: Update ChatRequest
**File**: `src/core/types/requests/chat.rs`

```rust
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,

    // NEW: Thinking configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<ThinkingConfig>,

    // ... existing fields
}
```

### Phase 2: Provider Transformers

#### Step 2.1: Create thinking transformer trait
**File**: `src/core/providers/thinking.rs` (NEW)

```rust
/// Trait for providers that support thinking
pub trait ThinkingProvider {
    /// Check if model supports thinking
    fn supports_thinking(&self, model: &str) -> bool;

    /// Get thinking capabilities
    fn thinking_capabilities(&self, model: &str) -> ThinkingCapabilities;

    /// Transform thinking config to provider format
    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        model: &str,
    ) -> Result<serde_json::Value, ProviderError>;

    /// Extract thinking from response
    fn extract_thinking(
        &self,
        response: &serde_json::Value,
    ) -> Option<ThinkingContent>;

    /// Extract thinking usage
    fn extract_thinking_usage(
        &self,
        response: &serde_json::Value,
    ) -> Option<ThinkingUsage>;
}
```

#### Step 2.2: OpenAI thinking implementation
**File**: `src/core/providers/openai/thinking.rs` (NEW)

```rust
impl ThinkingProvider for OpenAIProvider {
    fn supports_thinking(&self, model: &str) -> bool {
        // o1, o3, o4, gpt-5.1-thinking series
        model.starts_with("o1") ||
        model.starts_with("o3") ||
        model.starts_with("o4") ||
        model.contains("thinking")
    }

    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        model: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut params = serde_json::Map::new();

        if let Some(budget) = config.budget_tokens {
            // OpenAI max is 20,000
            let capped = budget.min(20_000);
            params.insert("max_reasoning_tokens".into(), capped.into());
        }

        if config.include_thinking {
            params.insert("include_reasoning".into(), true.into());
        }

        Ok(serde_json::Value::Object(params))
    }

    fn extract_thinking(&self, response: &serde_json::Value) -> Option<ThinkingContent> {
        response
            .pointer("/choices/0/message/reasoning")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
    }
}
```

#### Step 2.3: Anthropic thinking implementation
**File**: `src/core/providers/anthropic/thinking.rs` (NEW)

```rust
impl ThinkingProvider for AnthropicProvider {
    fn supports_thinking(&self, model: &str) -> bool {
        // Claude 3.5+ models with thinking
        model.contains("claude-3") || model.contains("claude-4")
    }

    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        _model: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut params = serde_json::Map::new();

        // Anthropic uses "thinking" block in request
        if config.enabled {
            let mut thinking = serde_json::Map::new();
            thinking.insert("type".into(), "enabled".into());

            if let Some(budget) = config.budget_tokens {
                thinking.insert("budget_tokens".into(), budget.into());
            }

            params.insert("thinking".into(), thinking.into());
        }

        Ok(serde_json::Value::Object(params))
    }

    fn extract_thinking(&self, response: &serde_json::Value) -> Option<ThinkingContent> {
        // Anthropic returns thinking in content blocks
        response
            .pointer("/content")
            .and_then(|v| v.as_array())
            .and_then(|blocks| {
                blocks.iter().find_map(|block| {
                    if block.get("type")?.as_str()? == "thinking" {
                        Some(ThinkingContent::Block {
                            thinking: block.get("thinking")?.as_str()?.to_string(),
                            block_type: Some("thinking".to_string()),
                        })
                    } else {
                        None
                    }
                })
            })
    }
}
```

#### Step 2.4: DeepSeek thinking implementation
**File**: `src/core/providers/deepseek/thinking.rs` (NEW)

```rust
impl ThinkingProvider for DeepSeekProvider {
    fn supports_thinking(&self, model: &str) -> bool {
        model.contains("reasoner") || model.contains("r1")
    }

    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        _model: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut params = serde_json::Map::new();

        // DeepSeek uses reasoning_effort
        if let Some(effort) = &config.effort {
            let effort_str = match effort {
                ThinkingEffort::Low => "low",
                ThinkingEffort::Medium => "medium",
                ThinkingEffort::High => "high",
            };
            params.insert("reasoning_effort".into(), effort_str.into());
        }

        Ok(serde_json::Value::Object(params))
    }

    fn extract_thinking(&self, response: &serde_json::Value) -> Option<ThinkingContent> {
        response
            .pointer("/choices/0/message/reasoning_content")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
    }
}
```

#### Step 2.5: Gemini thinking implementation
**File**: `src/core/providers/gemini/thinking.rs` (NEW)

```rust
impl ThinkingProvider for GeminiProvider {
    fn supports_thinking(&self, model: &str) -> bool {
        model.contains("thinking") || model.contains("deep-think")
    }

    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        _model: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        let mut params = serde_json::Map::new();

        // Gemini thinking config (API may vary)
        if config.enabled {
            params.insert("enableThinking".into(), true.into());

            if let Some(budget) = config.budget_tokens {
                params.insert("thinkingBudget".into(), budget.into());
            }
        }

        Ok(serde_json::Value::Object(params))
    }

    fn extract_thinking(&self, response: &serde_json::Value) -> Option<ThinkingContent> {
        // Gemini may return thinking in thoughts field
        response
            .pointer("/candidates/0/content/thoughts")
            .and_then(|v| v.as_str())
            .map(|text| ThinkingContent::Text {
                text: text.to_string(),
                signature: None,
            })
    }
}
```

#### Step 2.6: OpenRouter passthrough
**File**: `src/core/providers/openrouter/thinking.rs` (NEW)

```rust
impl ThinkingProvider for OpenRouterProvider {
    fn supports_thinking(&self, model: &str) -> bool {
        // OpenRouter supports thinking models from multiple providers
        model.contains("o1") ||
        model.contains("claude") ||
        model.contains("deepseek") ||
        model.contains("gemini") && model.contains("thinking")
    }

    fn transform_thinking_config(
        &self,
        config: &ThinkingConfig,
        model: &str,
    ) -> Result<serde_json::Value, ProviderError> {
        // Detect underlying provider and delegate
        let provider = self.detect_provider(model);
        provider.transform_thinking_config(config, model)
    }

    fn extract_thinking(&self, response: &serde_json::Value) -> Option<ThinkingContent> {
        // Try multiple extraction patterns
        // OpenAI style
        if let Some(thinking) = response.pointer("/choices/0/message/reasoning") {
            return Some(ThinkingContent::Text {
                text: thinking.as_str()?.to_string(),
                signature: None,
            });
        }

        // DeepSeek style
        if let Some(thinking) = response.pointer("/choices/0/message/reasoning_content") {
            return Some(ThinkingContent::Text {
                text: thinking.as_str()?.to_string(),
                signature: None,
            });
        }

        // Anthropic style
        // ... (check content blocks)

        None
    }
}
```

### Phase 3: Integration

#### Step 3.1: Update completion API
**File**: `src/core/completion/mod.rs`

```rust
/// Completion with thinking support
pub async fn completion_with_thinking(
    model: &str,
    messages: Vec<ChatMessage>,
    thinking: Option<ThinkingConfig>,
    options: Option<CompletionOptions>,
) -> Result<CompletionResponse, LiteLLMError> {
    let mut request = build_request(model, messages, options);
    request.thinking = thinking;

    // ... execute and extract thinking
}
```

#### Step 3.2: Update response conversion
**File**: `src/core/completion/conversion.rs`

Add thinking extraction to response conversion.

#### Step 3.3: Update streaming
**File**: `src/core/completion/stream.rs`

Support streaming thinking deltas.

### Phase 4: Cost & Pricing

#### Step 4.1: Update pricing database
**File**: `src/core/providers/base/pricing.rs`

Add thinking token pricing per provider.

#### Step 4.2: Update cost calculator
**File**: `src/core/cost/calculator.rs`

Include thinking tokens in cost calculation.

## Files Summary

### New Files (7)
1. `src/core/types/thinking.rs` - Core thinking types
2. `src/core/providers/thinking.rs` - ThinkingProvider trait
3. `src/core/providers/openai/thinking.rs` - OpenAI implementation
4. `src/core/providers/anthropic/thinking.rs` - Anthropic implementation
5. `src/core/providers/deepseek/thinking.rs` - DeepSeek implementation
6. `src/core/providers/gemini/thinking.rs` - Gemini implementation
7. `src/core/providers/openrouter/thinking.rs` - OpenRouter passthrough

### Modified Files (10)
1. `src/core/types/mod.rs` - Export thinking types
2. `src/core/types/chat.rs` - Add thinking to ChatMessage
3. `src/core/types/requests/chat.rs` - Add thinking to ChatRequest
4. `src/core/types/responses/delta.rs` - Add thinking to ChatDelta
5. `src/core/types/responses/usage.rs` - Add ThinkingUsage
6. `src/core/providers/mod.rs` - Export ThinkingProvider
7. `src/core/completion/mod.rs` - Add thinking API
8. `src/core/completion/conversion.rs` - Handle thinking in responses
9. `src/core/cost/calculator.rs` - Calculate thinking costs
10. `src/lib.rs` - Export thinking types

## API Usage Example

```rust
use litellm_rs::{completion, user_message, ThinkingConfig, ThinkingEffort};

// Enable thinking with budget
let thinking = ThinkingConfig {
    enabled: true,
    budget_tokens: Some(10000),
    effort: Some(ThinkingEffort::High),
    include_thinking: true,
};

let response = completion(
    "openrouter/deepseek/deepseek-r1",
    vec![user_message("Solve: What is 15% of 240?")],
    Some(CompletionOptions {
        thinking: Some(thinking),
        ..Default::default()
    }),
).await?;

// Access thinking content
if let Some(thinking) = &response.choices[0].message.thinking {
    match thinking {
        ThinkingContent::Text { text, .. } => {
            println!("Thinking: {}", text);
        }
        _ => {}
    }
}

// Access thinking usage
if let Some(usage) = &response.usage {
    if let Some(thinking_usage) = &usage.thinking_usage {
        println!("Thinking tokens: {:?}", thinking_usage.thinking_tokens);
        println!("Thinking cost: ${:.4}", thinking_usage.thinking_cost.unwrap_or(0.0));
    }
}

// Main response
println!("Answer: {:?}", response.choices[0].message.content);
```

## Testing Plan

1. Unit tests for each provider's thinking implementation
2. Integration tests with real API calls (requires API keys)
3. Streaming thinking tests
4. Cost calculation accuracy tests
5. Edge cases: thinking disabled, unsupported models, budget limits

## Timeline Estimate

This is a significant feature addition that touches core types and all provider implementations. Implementation should be done in phases to ensure stability.
