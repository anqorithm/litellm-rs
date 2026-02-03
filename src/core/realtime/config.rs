//! Realtime API configuration

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::events::{AudioFormat, SessionConfig, TurnDetection, Voice};

/// Realtime API configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RealtimeConfig {
    /// Model to use (e.g., "gpt-4o-realtime-preview")
    pub model: String,

    /// API key
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// WebSocket URL
    #[serde(default = "default_ws_url")]
    pub ws_url: String,

    /// Voice for audio output
    #[serde(default)]
    pub voice: Voice,

    /// System instructions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,

    /// Input audio format
    #[serde(default)]
    pub input_audio_format: AudioFormat,

    /// Output audio format
    #[serde(default)]
    pub output_audio_format: AudioFormat,

    /// Turn detection configuration
    #[serde(skip_serializing_if = "Option::is_none")]
    pub turn_detection: Option<TurnDetection>,

    /// Temperature for responses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f64>,

    /// Maximum output tokens
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_seconds: u64,

    /// Ping interval in seconds
    #[serde(default = "default_ping_interval")]
    pub ping_interval_seconds: u64,

    /// Additional headers
    #[serde(default)]
    pub headers: HashMap<String, String>,

    /// Enable input audio transcription
    #[serde(default)]
    pub transcribe_input: bool,

    /// Transcription model
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_model: Option<String>,
}

fn default_ws_url() -> String {
    "wss://api.openai.com/v1/realtime".to_string()
}

fn default_timeout() -> u64 {
    30
}

fn default_ping_interval() -> u64 {
    30
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            model: "gpt-4o-realtime-preview".to_string(),
            api_key: None,
            ws_url: default_ws_url(),
            voice: Voice::default(),
            instructions: None,
            input_audio_format: AudioFormat::default(),
            output_audio_format: AudioFormat::default(),
            turn_detection: None,
            temperature: None,
            max_output_tokens: None,
            timeout_seconds: default_timeout(),
            ping_interval_seconds: default_ping_interval(),
            headers: HashMap::new(),
            transcribe_input: false,
            transcription_model: None,
        }
    }
}

impl RealtimeConfig {
    /// Create a new configuration with the specified model
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            ..Default::default()
        }
    }

    /// Create from environment variables
    pub fn from_env() -> Option<Self> {
        let api_key = std::env::var("OPENAI_API_KEY").ok()?;
        Some(Self::default().api_key(api_key))
    }

    /// Set the API key
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Set the WebSocket URL
    pub fn ws_url(mut self, url: impl Into<String>) -> Self {
        self.ws_url = url.into();
        self
    }

    /// Set the voice
    pub fn voice(mut self, voice: Voice) -> Self {
        self.voice = voice;
        self
    }

    /// Set system instructions
    pub fn instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Set input audio format
    pub fn input_audio_format(mut self, format: AudioFormat) -> Self {
        self.input_audio_format = format;
        self
    }

    /// Set output audio format
    pub fn output_audio_format(mut self, format: AudioFormat) -> Self {
        self.output_audio_format = format;
        self
    }

    /// Set turn detection
    pub fn turn_detection(mut self, detection: TurnDetection) -> Self {
        self.turn_detection = Some(detection);
        self
    }

    /// Disable turn detection
    pub fn no_turn_detection(mut self) -> Self {
        self.turn_detection = Some(TurnDetection::None);
        self
    }

    /// Set temperature
    pub fn temperature(mut self, temp: f64) -> Self {
        self.temperature = Some(temp);
        self
    }

    /// Set max output tokens
    pub fn max_output_tokens(mut self, tokens: u32) -> Self {
        self.max_output_tokens = Some(tokens);
        self
    }

    /// Set connection timeout
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Add a custom header
    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Enable input audio transcription
    pub fn with_transcription(mut self) -> Self {
        self.transcribe_input = true;
        self
    }

    /// Set transcription model
    pub fn transcription_model(mut self, model: impl Into<String>) -> Self {
        self.transcription_model = Some(model.into());
        self.transcribe_input = true;
        self
    }

    /// Convert to session config
    pub fn to_session_config(&self) -> SessionConfig {
        use super::events::InputAudioTranscription;

        SessionConfig {
            modalities: Some(vec!["text".to_string(), "audio".to_string()]),
            instructions: self.instructions.clone(),
            voice: Some(self.voice),
            input_audio_format: Some(self.input_audio_format),
            output_audio_format: Some(self.output_audio_format),
            input_audio_transcription: if self.transcribe_input {
                Some(InputAudioTranscription {
                    model: self.transcription_model.clone().unwrap_or_else(|| "whisper-1".to_string()),
                })
            } else {
                None
            },
            turn_detection: self.turn_detection.clone(),
            tools: None,
            tool_choice: None,
            temperature: self.temperature,
            max_response_output_tokens: self.max_output_tokens.map(super::events::MaxTokens::Number),
        }
    }

    /// Get the full WebSocket URL with model parameter
    pub fn get_ws_url(&self) -> String {
        format!("{}?model={}", self.ws_url, self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RealtimeConfig::default();
        assert_eq!(config.model, "gpt-4o-realtime-preview");
        assert_eq!(config.voice, Voice::Alloy);
        assert!(config.api_key.is_none());
    }

    #[test]
    fn test_config_builder() {
        let config = RealtimeConfig::new("gpt-4o-realtime-preview")
            .api_key("sk-test")
            .voice(Voice::Nova)
            .instructions("You are a helpful assistant")
            .temperature(0.7)
            .max_output_tokens(1000)
            .with_transcription();

        assert_eq!(config.model, "gpt-4o-realtime-preview");
        assert_eq!(config.api_key, Some("sk-test".to_string()));
        assert_eq!(config.voice, Voice::Nova);
        assert_eq!(config.instructions, Some("You are a helpful assistant".to_string()));
        assert_eq!(config.temperature, Some(0.7));
        assert_eq!(config.max_output_tokens, Some(1000));
        assert!(config.transcribe_input);
    }

    #[test]
    fn test_get_ws_url() {
        let config = RealtimeConfig::new("gpt-4o-realtime-preview");
        let url = config.get_ws_url();
        assert!(url.contains("model=gpt-4o-realtime-preview"));
    }

    #[test]
    fn test_to_session_config() {
        let config = RealtimeConfig::new("gpt-4o-realtime-preview")
            .voice(Voice::Echo)
            .instructions("Test instructions")
            .temperature(0.5);

        let session = config.to_session_config();
        assert_eq!(session.voice, Some(Voice::Echo));
        assert_eq!(session.instructions, Some("Test instructions".to_string()));
        assert_eq!(session.temperature, Some(0.5));
    }

    #[test]
    fn test_turn_detection() {
        let config = RealtimeConfig::default()
            .turn_detection(TurnDetection::ServerVad {
                threshold: Some(0.5),
                prefix_padding_ms: Some(300),
                silence_duration_ms: Some(500),
            });

        assert!(config.turn_detection.is_some());

        let config = RealtimeConfig::default().no_turn_detection();
        assert!(matches!(config.turn_detection, Some(TurnDetection::None)));
    }
}
