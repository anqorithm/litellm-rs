//! Tests for audio module

#[cfg(test)]
use super::transcription::parse_model_string;
use super::types::{format_to_content_type, supported_audio_formats};

#[test]
fn test_parse_model_string() {
    assert_eq!(
        parse_model_string("groq/whisper-large-v3"),
        ("groq", "whisper-large-v3")
    );
    assert_eq!(
        parse_model_string("openai/whisper-1"),
        ("openai", "whisper-1")
    );
    assert_eq!(
        parse_model_string("whisper-large-v3"),
        ("groq", "whisper-large-v3")
    );
    assert_eq!(parse_model_string("tts-1"), ("openai", "tts-1"));
}

#[test]
fn test_format_to_content_type() {
    assert_eq!(format_to_content_type("mp3"), "audio/mpeg");
    assert_eq!(format_to_content_type("opus"), "audio/opus");
    assert_eq!(format_to_content_type("wav"), "audio/wav");
}

#[test]
fn test_supported_formats() {
    let formats = supported_audio_formats();
    assert!(formats.contains(&"mp3"));
    assert!(formats.contains(&"wav"));
    assert!(formats.contains(&"webm"));
}
