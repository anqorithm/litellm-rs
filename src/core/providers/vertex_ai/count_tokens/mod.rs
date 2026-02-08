//! Vertex AI Count Tokens Module

use crate::ProviderError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::types::chat::ChatMessage;

/// Count tokens request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountTokensRequest {
    pub contents: Vec<Content>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_config: Option<GenerationConfig>,
}

/// Content for token counting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Content {
    pub role: String,
    pub parts: Vec<Part>,
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Part {
    Text { text: String },
    InlineData { inline_data: InlineData },
    FileData { file_data: FileData },
}

/// Inline data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InlineData {
    pub mime_type: String,
    pub data: String,
}

/// File data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileData {
    pub mime_type: String,
    pub file_uri: String,
}

/// Generation config for counting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<i32>,
}

/// Count tokens response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountTokensResponse {
    pub total_tokens: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_billable_characters: Option<i32>,
}

/// Token counter handler
pub struct TokenCountHandler;

impl TokenCountHandler {
    /// Create count tokens request from chat messages
    pub fn create_request(messages: &[ChatMessage]) -> Result<CountTokensRequest, ProviderError> {
        let contents = messages
            .iter()
            .map(|msg| {
                let role = match msg.role.to_string().to_lowercase().as_str() {
                    "system" => "user",
                    "user" => "user",
                    "assistant" => "model",
                    _ => "user",
                };

                let parts = if let Some(ref content) = msg.content {
                    vec![Part::Text {
                        text: content.to_string(),
                    }]
                } else {
                    vec![]
                };

                Content {
                    role: role.to_string(),
                    parts,
                }
            })
            .collect();

        Ok(CountTokensRequest {
            contents,
            generation_config: None,
        })
    }

    /// Parse response to get token count
    pub fn parse_response(response: Value) -> Result<usize, ProviderError> {
        response["totalTokens"]
            .as_i64()
            .or_else(|| response["total_tokens"].as_i64())
            .map(|v| v as usize)
            .ok_or_else(|| {
                ProviderError::response_parsing("vertex_ai", "Missing token count in response")
            })
    }

    /// Estimate tokens for text (rough approximation)
    pub fn estimate_tokens(text: &str) -> usize {
        // Rough estimation: ~4 characters per token for English
        // More sophisticated tokenization would require the actual tokenizer
        let char_count = text.chars().count();
        char_count.div_ceil(4) // Round up division
    }

    /// Estimate tokens for messages
    pub fn estimate_message_tokens(messages: &[ChatMessage]) -> usize {
        let mut total = 0;

        for message in messages {
            // Role overhead
            total += 4;

            // Content tokens
            if let Some(ref content) = message.content {
                total += Self::estimate_tokens(&content.to_string());
            }

            // Message formatting overhead
            total += 3;
        }

        // Request formatting overhead
        total += 10;

        total
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::types::{message::MessageContent, message::MessageRole};

    #[test]
    fn test_estimate_tokens() {
        assert_eq!(TokenCountHandler::estimate_tokens("hello world"), 3);
        assert_eq!(TokenCountHandler::estimate_tokens(""), 0);
        assert_eq!(TokenCountHandler::estimate_tokens("a"), 1);
        assert_eq!(TokenCountHandler::estimate_tokens("abcd"), 1);
        assert_eq!(TokenCountHandler::estimate_tokens("abcde"), 2);
    }

    #[test]
    fn test_create_request() {
        let messages = vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text("You are helpful".to_string())),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text("Hello".to_string())),
                ..Default::default()
            },
        ];

        let request = TokenCountHandler::create_request(&messages).unwrap();
        assert_eq!(request.contents.len(), 2);
        assert_eq!(request.contents[0].role, "user");
        assert_eq!(request.contents[1].role, "user");
    }
}
