//! Anthropic SSE Transformer
//!
//! Handles Anthropic's event-based SSE format with message_start, content_block_delta,
//! message_delta, and message_stop events.

use serde_json::Value;

use crate::core::providers::unified_provider::ProviderError;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice, FinishReason};

use super::SSETransformer;

/// Anthropic SSE Transformer
#[derive(Debug, Clone)]
pub struct AnthropicTransformer {
    model: String,
}

impl AnthropicTransformer {
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
        }
    }

    fn parse_anthropic_finish_reason(reason: &str) -> FinishReason {
        match reason {
            "end_turn" => FinishReason::Stop,
            "max_tokens" => FinishReason::Length,
            "tool_use" => FinishReason::ToolCalls,
            _ => FinishReason::Stop,
        }
    }
}

impl SSETransformer for AnthropicTransformer {
    fn provider_name(&self) -> &'static str {
        "anthropic"
    }

    fn transform_chunk(&self, data: &str) -> Result<Option<ChatChunk>, ProviderError> {
        let json: Value = serde_json::from_str(data).map_err(|e| {
            ProviderError::response_parsing(
                "anthropic",
                format!("Failed to parse Anthropic SSE: {}", e),
            )
        })?;

        let event_type = json.get("type").and_then(|v| v.as_str()).unwrap_or("");

        let created = chrono::Utc::now().timestamp();

        match event_type {
            "message_start" => {
                let message_id = json
                    .get("message")
                    .and_then(|m| m.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("anthropic-stream")
                    .to_string();

                Ok(Some(ChatChunk {
                    id: message_id,
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: self.model.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: Some(crate::core::types::message::MessageRole::Assistant),
                            content: None,
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                    system_fingerprint: None,
                }))
            }
            "content_block_delta" => {
                let text = json
                    .get("delta")
                    .and_then(|d| d.get("text"))
                    .and_then(|t| t.as_str())
                    .unwrap_or("");

                Ok(Some(ChatChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: self.model.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: Some(text.to_string()),
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason: None,
                        logprobs: None,
                    }],
                    usage: None,
                    system_fingerprint: None,
                }))
            }
            "message_delta" => {
                let finish_reason = json
                    .get("delta")
                    .and_then(|d| d.get("stop_reason"))
                    .and_then(|r| r.as_str())
                    .map(Self::parse_anthropic_finish_reason);

                let usage = json.get("usage").map(|u| {
                    let input = u.get("input_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    let output =
                        u.get("output_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32;
                    crate::core::types::responses::Usage {
                        prompt_tokens: input,
                        completion_tokens: output,
                        total_tokens: input + output,
                        completion_tokens_details: None,
                        prompt_tokens_details: None,
                        thinking_usage: None,
                    }
                });

                Ok(Some(ChatChunk {
                    id: String::new(),
                    object: "chat.completion.chunk".to_string(),
                    created,
                    model: self.model.clone(),
                    choices: vec![ChatStreamChoice {
                        index: 0,
                        delta: ChatDelta {
                            role: None,
                            content: None,
                            thinking: None,
                            tool_calls: None,
                            function_call: None,
                        },
                        finish_reason,
                        logprobs: None,
                    }],
                    usage,
                    system_fingerprint: None,
                }))
            }
            "message_stop" => Ok(Some(ChatChunk {
                id: String::new(),
                object: "chat.completion.chunk".to_string(),
                created,
                model: self.model.clone(),
                choices: vec![],
                usage: None,
                system_fingerprint: None,
            })),
            "error" => {
                let msg = json
                    .get("error")
                    .and_then(|e| e.get("message"))
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown streaming error");
                Err(ProviderError::streaming_error(
                    "anthropic",
                    "chat",
                    None,
                    None,
                    msg.to_string(),
                ))
            }
            // content_block_start, content_block_stop, ping — skip
            _ => Ok(None),
        }
    }
}
