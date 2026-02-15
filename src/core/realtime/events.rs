//! Realtime API events
//!
//! Event types for WebSocket communication.

use serde::{Deserialize, Serialize};

/// Result type for realtime operations
pub type RealtimeResult<T> = Result<T, RealtimeError>;

/// Realtime API error types
#[derive(Debug, thiserror::Error)]
pub enum RealtimeError {
    #[error("Connection error: {0}")]
    Connection(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Session error: {0}")]
    Session(String),

    #[error("Invalid message: {0}")]
    InvalidMessage(String),

    #[error("Rate limited")]
    RateLimited,

    #[error("Server error: {code} - {message}")]
    Server { code: String, message: String },

    #[error("Timeout")]
    Timeout,

    #[error("Closed")]
    Closed,

    #[error("Realtime error: {0}")]
    Other(String),
}

impl RealtimeError {
    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Authentication(msg.into())
    }

    pub fn session(msg: impl Into<String>) -> Self {
        Self::Session(msg.into())
    }

    pub fn invalid_message(msg: impl Into<String>) -> Self {
        Self::InvalidMessage(msg.into())
    }

    pub fn server(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Server {
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Voice options for audio output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Voice {
    #[default]
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

impl std::fmt::Display for Voice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Alloy => write!(f, "alloy"),
            Self::Echo => write!(f, "echo"),
            Self::Fable => write!(f, "fable"),
            Self::Onyx => write!(f, "onyx"),
            Self::Nova => write!(f, "nova"),
            Self::Shimmer => write!(f, "shimmer"),
        }
    }
}

/// Audio format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AudioFormat {
    #[default]
    Pcm16,
    G711Ulaw,
    G711Alaw,
}

/// Turn detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TurnDetection {
    ServerVad {
        #[serde(skip_serializing_if = "Option::is_none")]
        threshold: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix_padding_ms: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        silence_duration_ms: Option<u32>,
    },
    None,
}

impl Default for TurnDetection {
    fn default() -> Self {
        Self::ServerVad {
            threshold: None,
            prefix_padding_ms: None,
            silence_duration_ms: None,
        }
    }
}

/// Session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_audio_transcription: Option<InputAudioTranscription>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_response_output_tokens: Option<MaxTokens>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            modalities: Some(vec!["text".to_string(), "audio".to_string()]),
            instructions: None,
            voice: Some(Voice::default()),
            input_audio_format: Some(AudioFormat::default()),
            output_audio_format: Some(AudioFormat::default()),
            input_audio_transcription: None,
            turn_detection: Some(TurnDetection::default()),
            tools: None,
            tool_choice: None,
            temperature: None,
            max_response_output_tokens: None,
        }
    }
}

/// Input audio transcription config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InputAudioTranscription {
    pub model: String,
}

/// Max tokens configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MaxTokens {
    Inf(String),
    Number(u32),
}

/// Tool definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Content part in a message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    InputText {
        text: String,
    },
    InputAudio {
        audio: String,
    },
    Text {
        text: String,
    },
    Audio {
        audio: String,
        transcript: Option<String>,
    },
}

/// Response status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    InProgress,
    Completed,
    Cancelled,
    Failed,
    Incomplete,
}

/// Item type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    Message,
    FunctionCall,
    FunctionCallOutput,
}

/// Item role
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemRole {
    User,
    Assistant,
    System,
}

/// Conversation item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Item {
    pub id: String,
    #[serde(rename = "type")]
    pub item_type: ItemType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<ItemRole>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<Vec<ContentPart>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub call_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

/// Client events (sent to server)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    SessionUpdate {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        session: SessionConfig,
    },
    InputAudioBufferAppend {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        audio: String,
    },
    InputAudioBufferCommit {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
    InputAudioBufferClear {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
    ConversationItemCreate {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        previous_item_id: Option<String>,
        item: Item,
    },
    ConversationItemTruncate {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
    },
    ConversationItemDelete {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        item_id: String,
    },
    ResponseCreate {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        response: Option<ResponseConfig>,
    },
    ResponseCancel {
        #[serde(skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
    },
}

/// Response configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResponseConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice: Option<Voice>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_audio_format: Option<AudioFormat>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<MaxTokens>,
}

/// Server events (received from server)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerEvent {
    Error {
        event_id: String,
        error: RealtimeErrorDetail,
    },
    SessionCreated {
        event_id: String,
        session: SessionInfo,
    },
    SessionUpdated {
        event_id: String,
        session: SessionInfo,
    },
    ConversationCreated {
        event_id: String,
        conversation: ConversationInfo,
    },
    InputAudioBufferCommitted {
        event_id: String,
        previous_item_id: Option<String>,
        item_id: String,
    },
    InputAudioBufferCleared {
        event_id: String,
    },
    InputAudioBufferSpeechStarted {
        event_id: String,
        audio_start_ms: u32,
        item_id: String,
    },
    InputAudioBufferSpeechStopped {
        event_id: String,
        audio_end_ms: u32,
        item_id: String,
    },
    ConversationItemCreated {
        event_id: String,
        previous_item_id: Option<String>,
        item: Item,
    },
    ConversationItemInputAudioTranscriptionCompleted {
        event_id: String,
        item_id: String,
        content_index: u32,
        transcript: String,
    },
    ConversationItemInputAudioTranscriptionFailed {
        event_id: String,
        item_id: String,
        content_index: u32,
        error: RealtimeErrorDetail,
    },
    ConversationItemTruncated {
        event_id: String,
        item_id: String,
        content_index: u32,
        audio_end_ms: u32,
    },
    ConversationItemDeleted {
        event_id: String,
        item_id: String,
    },
    ResponseCreated {
        event_id: String,
        response: ResponseInfo,
    },
    ResponseDone {
        event_id: String,
        response: ResponseInfo,
    },
    ResponseOutputItemAdded {
        event_id: String,
        response_id: String,
        output_index: u32,
        item: Item,
    },
    ResponseOutputItemDone {
        event_id: String,
        response_id: String,
        output_index: u32,
        item: Item,
    },
    ResponseContentPartAdded {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: ContentPart,
    },
    ResponseContentPartDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        part: ContentPart,
    },
    ResponseTextDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },
    ResponseTextDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        text: String,
    },
    ResponseAudioTranscriptDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },
    ResponseAudioTranscriptDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        transcript: String,
    },
    ResponseAudioDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
        delta: String,
    },
    ResponseAudioDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        content_index: u32,
    },
    ResponseFunctionCallArgumentsDelta {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        call_id: String,
        delta: String,
    },
    ResponseFunctionCallArgumentsDone {
        event_id: String,
        response_id: String,
        item_id: String,
        output_index: u32,
        call_id: String,
        arguments: String,
    },
    RateLimitsUpdated {
        event_id: String,
        rate_limits: Vec<RateLimit>,
    },
}

/// Realtime API error detail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeErrorDetail {
    #[serde(rename = "type")]
    pub error_type: String,
    pub code: Option<String>,
    pub message: String,
    pub param: Option<String>,
    pub event_id: Option<String>,
}

/// Session info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub id: String,
    pub object: String,
    pub model: String,
    #[serde(flatten)]
    pub config: SessionConfig,
}

/// Conversation info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub id: String,
    pub object: String,
}

/// Response info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseInfo {
    pub id: String,
    pub object: String,
    pub status: ResponseStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_details: Option<serde_json::Value>,
    pub output: Vec<Item>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<UsageInfo>,
}

/// Usage info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    pub total_tokens: u32,
    pub input_tokens: u32,
    pub output_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_token_details: Option<TokenDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_token_details: Option<TokenDetails>,
}

/// Token details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDetails {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cached_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_tokens: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio_tokens: Option<u32>,
}

/// Rate limit info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimit {
    pub name: String,
    pub limit: u32,
    pub remaining: u32,
    pub reset_seconds: f64,
}

/// Simplified realtime event for easier handling
#[derive(Debug, Clone)]
pub enum RealtimeEvent {
    SessionCreated {
        session_id: String,
        model: String,
    },
    SessionUpdated {
        session_id: String,
    },
    ResponseStarted {
        response_id: String,
    },
    ResponseText {
        response_id: String,
        item_id: String,
        text: String,
        is_done: bool,
    },
    ResponseAudio {
        response_id: String,
        item_id: String,
        audio: Vec<u8>,
        is_done: bool,
    },
    ResponseDone {
        response_id: String,
        status: ResponseStatus,
    },
    FunctionCall {
        call_id: String,
        name: String,
        arguments: String,
    },
    Error {
        code: String,
        message: String,
    },
    RateLimitUpdated {
        limits: Vec<RateLimit>,
    },
    Raw(Box<ServerEvent>),
}

impl From<ServerEvent> for RealtimeEvent {
    fn from(event: ServerEvent) -> Self {
        match event {
            ServerEvent::SessionCreated { session, .. } => RealtimeEvent::SessionCreated {
                session_id: session.id,
                model: session.model,
            },
            ServerEvent::SessionUpdated { session, .. } => RealtimeEvent::SessionUpdated {
                session_id: session.id,
            },
            ServerEvent::ResponseCreated { response, .. } => RealtimeEvent::ResponseStarted {
                response_id: response.id,
            },
            ServerEvent::ResponseTextDelta {
                response_id,
                item_id,
                delta,
                ..
            } => RealtimeEvent::ResponseText {
                response_id,
                item_id,
                text: delta,
                is_done: false,
            },
            ServerEvent::ResponseTextDone {
                response_id,
                item_id,
                text,
                ..
            } => RealtimeEvent::ResponseText {
                response_id,
                item_id,
                text,
                is_done: true,
            },
            ServerEvent::ResponseDone { response, .. } => RealtimeEvent::ResponseDone {
                response_id: response.id,
                status: response.status,
            },
            ServerEvent::Error { error, .. } => RealtimeEvent::Error {
                code: error.code.unwrap_or_default(),
                message: error.message,
            },
            ServerEvent::RateLimitsUpdated { rate_limits, .. } => RealtimeEvent::RateLimitUpdated {
                limits: rate_limits,
            },
            other => RealtimeEvent::Raw(Box::new(other)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voice_display() {
        assert_eq!(Voice::Alloy.to_string(), "alloy");
        assert_eq!(Voice::Echo.to_string(), "echo");
        assert_eq!(Voice::Nova.to_string(), "nova");
    }

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert!(config.modalities.is_some());
        assert_eq!(config.voice, Some(Voice::Alloy));
    }

    #[test]
    fn test_client_event_serialization() {
        let event = ClientEvent::SessionUpdate {
            event_id: Some("evt-1".to_string()),
            session: SessionConfig::default(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("session_update"));
    }

    #[test]
    fn test_error_constructors() {
        let err = RealtimeError::connection("Failed to connect");
        assert!(matches!(err, RealtimeError::Connection(_)));

        let err = RealtimeError::auth("Invalid token");
        assert!(matches!(err, RealtimeError::Authentication(_)));

        let err = RealtimeError::server("500", "Internal error");
        assert!(matches!(err, RealtimeError::Server { .. }));
    }
}
