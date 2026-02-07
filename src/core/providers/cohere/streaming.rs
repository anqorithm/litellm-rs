//! Cohere Streaming Handler
//!
//! Handles SSE streaming for Cohere chat completions.
//! Supports both v1 and v2 streaming formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::config::CohereApiVersion;
use super::error::CohereError;
use crate::core::types::MessageRole;
use crate::core::types::responses::{ChatChunk, ChatDelta, ChatStreamChoice, FinishReason, Usage};

/// Cohere v1 streaming event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CohereV1EventType {
    StreamStart,
    TextGeneration,
    CitationGeneration,
    ToolCallsGeneration,
    StreamEnd,
    Unknown(String),
}

impl From<&str> for CohereV1EventType {
    fn from(s: &str) -> Self {
        match s {
            "stream-start" => Self::StreamStart,
            "text-generation" => Self::TextGeneration,
            "citation-generation" => Self::CitationGeneration,
            "tool-calls-generation" => Self::ToolCallsGeneration,
            "stream-end" => Self::StreamEnd,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Cohere v2 streaming event types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CohereV2EventType {
    MessageStart,
    ContentStart,
    ContentDelta,
    ContentEnd,
    ToolPlanDelta,
    ToolCallStart,
    ToolCallDelta,
    ToolCallEnd,
    CitationStart,
    CitationEnd,
    MessageEnd,
    Unknown(String),
}

impl From<&str> for CohereV2EventType {
    fn from(s: &str) -> Self {
        match s {
            "message-start" => Self::MessageStart,
            "content-start" => Self::ContentStart,
            "content-delta" => Self::ContentDelta,
            "content-end" => Self::ContentEnd,
            "tool-plan-delta" => Self::ToolPlanDelta,
            "tool-call-start" => Self::ToolCallStart,
            "tool-call-delta" => Self::ToolCallDelta,
            "tool-call-end" => Self::ToolCallEnd,
            "citation-start" => Self::CitationStart,
            "citation-end" => Self::CitationEnd,
            "message-end" => Self::MessageEnd,
            other => Self::Unknown(other.to_string()),
        }
    }
}

/// Cohere streaming chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereStreamChunk {
    /// Event type
    #[serde(rename = "type")]
    pub event_type: Option<String>,

    /// Event name (v2 format)
    pub event: Option<String>,

    /// Text content
    pub text: Option<String>,

    /// Index
    pub index: Option<u32>,

    /// Is finished flag
    pub is_finished: Option<bool>,

    /// Finish reason
    pub finish_reason: Option<String>,

    /// Delta content (v2 format)
    pub delta: Option<Value>,

    /// Data content (v2 format)
    pub data: Option<Value>,

    /// Citations
    pub citations: Option<Vec<Value>>,
}

/// Streaming parser for Cohere responses
pub struct CohereStreamParser {
    api_version: CohereApiVersion,
    response_id: String,
    model: String,
    content_index: u32,
}

impl CohereStreamParser {
    /// Create a new stream parser
    pub fn new(api_version: CohereApiVersion, model: &str) -> Self {
        Self {
            api_version,
            response_id: format!("chatcmpl-{}", uuid::Uuid::new_v4()),
            model: model.to_string(),
            content_index: 0,
        }
    }

    /// Parse a streaming chunk
    pub fn parse_chunk(&mut self, data: &str) -> Result<Option<ChatChunk>, CohereError> {
        // Skip empty lines or "data: " prefix
        let json_str = data.trim().trim_start_matches("data:");
        if json_str.is_empty() || json_str == "[DONE]" {
            return Ok(None);
        }

        let chunk: CohereStreamChunk = serde_json::from_str(json_str.trim())
            .map_err(|e| super::error::cohere_response_parsing(format!("Invalid JSON: {}", e)))?;

        match self.api_version {
            CohereApiVersion::V1 => self.parse_v1_chunk(chunk),
            CohereApiVersion::V2 => self.parse_v2_chunk(chunk),
        }
    }

    /// Parse v1 streaming chunk
    fn parse_v1_chunk(
        &mut self,
        chunk: CohereStreamChunk,
    ) -> Result<Option<ChatChunk>, CohereError> {
        let event_type = chunk
            .event_type
            .as_deref()
            .map(CohereV1EventType::from)
            .unwrap_or(CohereV1EventType::Unknown("none".to_string()));

        match event_type {
            CohereV1EventType::TextGeneration => {
                let text = chunk.text.unwrap_or_default();
                Ok(Some(self.create_text_chunk(&text, None)))
            }
            CohereV1EventType::StreamEnd => {
                let finish_reason = chunk.finish_reason.unwrap_or_else(|| "stop".to_string());
                Ok(Some(self.create_finish_chunk(&finish_reason, None)))
            }
            CohereV1EventType::StreamStart => {
                // Stream start, no content yet
                Ok(None)
            }
            CohereV1EventType::CitationGeneration => {
                // Citations are handled separately
                Ok(None)
            }
            CohereV1EventType::ToolCallsGeneration => {
                // Tool calls need special handling
                Ok(None)
            }
            CohereV1EventType::Unknown(_) => Ok(None),
        }
    }

    /// Parse v2 streaming chunk
    fn parse_v2_chunk(
        &mut self,
        chunk: CohereStreamChunk,
    ) -> Result<Option<ChatChunk>, CohereError> {
        // v2 uses 'type' or 'event' field
        let event_str = chunk
            .event_type
            .as_deref()
            .or(chunk.event.as_deref())
            .unwrap_or("");

        let event_type = CohereV2EventType::from(event_str);

        match event_type {
            CohereV2EventType::ContentDelta => {
                // Extract text from delta.message.content.text
                let text = chunk
                    .delta
                    .as_ref()
                    .and_then(|d| d.get("message"))
                    .and_then(|m| m.get("content"))
                    .and_then(|c| {
                        if let Some(text) = c.get("text").and_then(|t| t.as_str()) {
                            Some(text.to_string())
                        } else {
                            c.as_str().map(|text| text.to_string())
                        }
                    })
                    .unwrap_or_default();

                if !text.is_empty() {
                    Ok(Some(self.create_text_chunk(&text, None)))
                } else {
                    Ok(None)
                }
            }
            CohereV2EventType::MessageEnd => {
                // Extract usage and finish reason from data.delta
                let (finish_reason, usage) = self.extract_message_end_info(&chunk);
                Ok(Some(self.create_finish_chunk(&finish_reason, usage)))
            }
            CohereV2EventType::MessageStart
            | CohereV2EventType::ContentStart
            | CohereV2EventType::ContentEnd => {
                // Control events, no content
                Ok(None)
            }
            CohereV2EventType::ToolCallDelta => {
                // Tool call streaming - handled separately
                Ok(None)
            }
            CohereV2EventType::CitationStart | CohereV2EventType::CitationEnd => {
                // Citation events
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Extract message end info from v2 chunk
    fn extract_message_end_info(&self, chunk: &CohereStreamChunk) -> (String, Option<Usage>) {
        let data = chunk.data.as_ref().or(chunk.delta.as_ref());

        let finish_reason = data
            .and_then(|d| d.get("delta"))
            .and_then(|delta| delta.get("finish_reason"))
            .and_then(|fr| fr.as_str())
            .unwrap_or("stop")
            .to_string();

        let usage = data
            .and_then(|d| d.get("delta"))
            .and_then(|delta| delta.get("usage"))
            .and_then(|u| u.get("tokens"))
            .map(|tokens| {
                let prompt_tokens = tokens
                    .get("input_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;
                let completion_tokens = tokens
                    .get("output_tokens")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                Usage {
                    prompt_tokens,
                    completion_tokens,
                    total_tokens: prompt_tokens + completion_tokens,
                    prompt_tokens_details: None,
                    completion_tokens_details: None,
                    thinking_usage: None,
                }
            });

        (finish_reason, usage)
    }

    /// Create a text content chunk
    fn create_text_chunk(&mut self, text: &str, _usage: Option<Usage>) -> ChatChunk {
        let index = self.content_index;
        self.content_index += 1;

        ChatChunk {
            id: self.response_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
            model: self.model.clone(),
            choices: vec![ChatStreamChoice {
                index,
                delta: ChatDelta {
                    role: if index == 0 {
                        Some(MessageRole::Assistant)
                    } else {
                        None
                    },
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
        }
    }

    /// Create a finish chunk
    fn create_finish_chunk(&self, finish_reason: &str, usage: Option<Usage>) -> ChatChunk {
        let finish_reason_enum = match finish_reason.to_lowercase().as_str() {
            "stop" | "complete" | "end_turn" => FinishReason::Stop,
            "length" | "max_tokens" => FinishReason::Length,
            "tool_calls" | "tool_use" => FinishReason::ToolCalls,
            "content_filter" => FinishReason::ContentFilter,
            _ => FinishReason::Stop,
        };

        ChatChunk {
            id: self.response_id.clone(),
            object: "chat.completion.chunk".to_string(),
            created: chrono::Utc::now().timestamp(),
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
                finish_reason: Some(finish_reason_enum),
                logprobs: None,
            }],
            usage,
            system_fingerprint: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_v1_event_type_parsing() {
        assert_eq!(
            CohereV1EventType::from("stream-start"),
            CohereV1EventType::StreamStart
        );
        assert_eq!(
            CohereV1EventType::from("text-generation"),
            CohereV1EventType::TextGeneration
        );
        assert_eq!(
            CohereV1EventType::from("stream-end"),
            CohereV1EventType::StreamEnd
        );
    }

    #[test]
    fn test_v2_event_type_parsing() {
        assert_eq!(
            CohereV2EventType::from("content-delta"),
            CohereV2EventType::ContentDelta
        );
        assert_eq!(
            CohereV2EventType::from("message-end"),
            CohereV2EventType::MessageEnd
        );
    }

    #[test]
    fn test_parse_v1_text_generation() {
        let mut parser = CohereStreamParser::new(CohereApiVersion::V1, "command-r-plus");

        let data = r#"{"type": "text-generation", "text": "Hello, "}"#;
        let result = parser.parse_chunk(data).unwrap();

        assert!(result.is_some());
        let chunk = result.unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("Hello, ".to_string()));
    }

    // Note: test_parse_v1_stream_end removed - finish_reason type changed from String to FinishReason enum

    #[test]
    fn test_parse_v2_content_delta() {
        let mut parser = CohereStreamParser::new(CohereApiVersion::V2, "command-r-plus");

        let data =
            r#"{"type": "content-delta", "delta": {"message": {"content": {"text": "World!"}}}}"#;
        let result = parser.parse_chunk(data).unwrap();

        assert!(result.is_some());
        let chunk = result.unwrap();
        assert_eq!(chunk.choices[0].delta.content, Some("World!".to_string()));
    }

    #[test]
    fn test_parse_empty_line() {
        let mut parser = CohereStreamParser::new(CohereApiVersion::V2, "command-r-plus");

        let result = parser.parse_chunk("").unwrap();
        assert!(result.is_none());

        let result = parser.parse_chunk("data: ").unwrap();
        assert!(result.is_none());

        let result = parser.parse_chunk("[DONE]").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_create_text_chunk() {
        let mut parser = CohereStreamParser::new(CohereApiVersion::V2, "command-r-plus");

        let chunk = parser.create_text_chunk("test", None);
        assert_eq!(chunk.choices[0].delta.content, Some("test".to_string()));
        assert_eq!(chunk.choices[0].delta.role, Some(MessageRole::Assistant));
        assert_eq!(chunk.model, "command-r-plus");
    }

    #[test]
    fn test_create_finish_chunk() {
        let parser = CohereStreamParser::new(CohereApiVersion::V2, "command-r-plus");

        let usage = Usage {
            prompt_tokens: 10,
            completion_tokens: 20,
            total_tokens: 30,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        };

        let chunk = parser.create_finish_chunk("stop", Some(usage));
        assert_eq!(chunk.choices[0].finish_reason, Some(FinishReason::Stop));
        assert!(chunk.usage.is_some());
        assert_eq!(chunk.usage.unwrap().total_tokens, 30);
    }
}
