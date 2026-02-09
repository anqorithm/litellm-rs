//! SDK data types

use serde::{Deserialize, Serialize};

/// Message role
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    /// System message
    System,
    /// User message
    User,
    /// Assistant message
    Assistant,
    /// Tool message
    Tool,
}

/// Message content type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Content {
    /// Plain text content
    Text(String),
    /// Multimodal content
    Multimodal(Vec<ContentPart>),
}

/// Content part
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ContentPart {
    /// Text content
    #[serde(rename = "text")]
    Text {
        /// Text string
        text: String,
    },
    /// Image content
    #[serde(rename = "image_url")]
    Image {
        /// Image URL information
        image_url: ImageUrl,
    },
    /// Audio content
    #[serde(rename = "audio")]
    Audio {
        /// Audio data
        audio: AudioData,
    },
}

/// Image URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUrl {
    /// Image URL or base64 data
    pub url: String,
    /// Image detail level
    pub detail: Option<String>,
}

/// Audio data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioData {
    /// Audio data or URL
    pub data: String,
    /// Audio format
    pub format: Option<String>,
}

/// Chat message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// Message role
    pub role: Role,
    /// Message content
    pub content: Option<Content>,
    /// Message name
    pub name: Option<String>,
    /// Tool calls
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Tool call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// Call ID
    pub id: String,
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function call
    pub function: Function,
}

/// Function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    /// Function name
    pub name: String,
    /// Function description
    pub description: Option<String>,
    /// Function parameter schema
    pub parameters: serde_json::Value,
    /// Function parameters (used for calls)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// Tool type
    #[serde(rename = "type")]
    pub tool_type: String,
    /// Function definition
    pub function: Function,
}

/// Tool choice
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// Don't use tools
    None,
    /// Auto selection
    Auto,
    /// Must use tools
    Required,
    /// Specific function
    Function {
        /// Function name
        name: String,
    },
}

/// Chat request
#[derive(Debug, Clone)]
pub struct ChatRequest {
    /// Model name
    pub model: String,
    /// Message list
    pub messages: Vec<Message>,
    /// Request options
    pub options: ChatOptions,
}

/// Chat options
#[derive(Debug, Clone, Default)]
pub struct ChatOptions {
    /// Temperature parameter
    pub temperature: Option<f32>,
    /// Maximum token count
    pub max_tokens: Option<u32>,
    /// Top-p parameter
    pub top_p: Option<f32>,
    /// Frequency penalty
    pub frequency_penalty: Option<f32>,
    /// Presence penalty
    pub presence_penalty: Option<f32>,
    /// Stop sequences
    pub stop: Option<Vec<String>>,
    /// Stream response
    pub stream: bool,
    /// Tool list
    pub tools: Option<Vec<Tool>>,
    /// Tool choice
    pub tool_choice: Option<ToolChoice>,
}

/// Chat response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatResponse {
    /// Response ID
    pub id: String,
    /// Model name
    pub model: String,
    /// Choice list
    pub choices: Vec<ChatChoice>,
    /// Usage statistics
    pub usage: Usage,
    /// Creation timestamp
    pub created: u64,
}

/// Chat choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,
    /// Message content
    pub message: Message,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Chat chunk (streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChunk {
    /// Response ID
    pub id: String,
    /// Model name
    pub model: String,
    /// Choice list
    pub choices: Vec<ChunkChoice>,
}

/// Streaming choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkChoice {
    /// Choice index
    pub index: u32,
    /// Delta message
    pub delta: MessageDelta,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Delta message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageDelta {
    /// Message role
    pub role: Option<Role>,
    /// Message content
    pub content: Option<String>,
    /// Tool calls
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Usage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Usage {
    /// Prompt token count
    pub prompt_tokens: u32,
    /// Completion token count
    pub completion_tokens: u32,
    /// Total token count
    pub total_tokens: u32,
}

/// Cost information
#[derive(Debug, Clone)]
pub struct Cost {
    /// Cost amount
    pub amount: f64,
    /// Currency type
    pub currency: String,
    /// Cost breakdown
    pub breakdown: CostBreakdown,
}

/// Cost breakdown
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    /// Input cost
    pub input_cost: f64,
    /// Output cost
    pub output_cost: f64,
    /// Total cost
    pub total_cost: f64,
}

// ==================== Unit Tests ====================


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
