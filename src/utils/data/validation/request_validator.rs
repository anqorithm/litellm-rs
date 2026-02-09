//! Request validation utilities

use crate::core::models::openai::MessageContent;
use crate::utils::error::error::{GatewayError, Result};
use regex::Regex;

/// Request validation utilities
pub struct RequestValidator;

impl RequestValidator {
    /// Validate chat completion request
    pub fn validate_chat_completion_request(
        model: &str,
        messages: &[crate::core::models::openai::ChatMessage],
        max_tokens: Option<u32>,
        temperature: Option<f32>,
    ) -> Result<()> {
        // Validate model
        Self::validate_model_name(model)?;

        // Validate messages
        if messages.is_empty() {
            return Err(GatewayError::Validation(
                "Messages cannot be empty".to_string(),
            ));
        }

        for (i, message) in messages.iter().enumerate() {
            Self::validate_chat_message(message, i)?;
        }

        // Validate max_tokens
        if let Some(max_tokens) = max_tokens {
            if max_tokens == 0 {
                return Err(GatewayError::Validation(
                    "max_tokens must be greater than 0".to_string(),
                ));
            }
            if max_tokens > 100000 {
                return Err(GatewayError::Validation(
                    "max_tokens cannot exceed 100000".to_string(),
                ));
            }
        }

        // Validate temperature
        if let Some(temperature) = temperature {
            if !(0.0..=2.0).contains(&temperature) {
                return Err(GatewayError::Validation(
                    "temperature must be between 0.0 and 2.0".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate chat message
    fn validate_chat_message(
        message: &crate::core::models::openai::ChatMessage,
        index: usize,
    ) -> Result<()> {
        use crate::core::models::openai::MessageRole;

        // Validate role
        match message.role {
            MessageRole::System | MessageRole::User | MessageRole::Assistant => {
                // These roles should have content
                if message.content.is_none() {
                    return Err(GatewayError::Validation(format!(
                        "Message at index {} with role {:?} must have content",
                        index, message.role
                    )));
                }
            }
            MessageRole::Function => {
                // Function messages should have name and content
                if message.name.is_none() {
                    return Err(GatewayError::Validation(format!(
                        "Function message at index {} must have a name",
                        index
                    )));
                }
                if message.content.is_none() {
                    return Err(GatewayError::Validation(format!(
                        "Function message at index {} must have content",
                        index
                    )));
                }
            }
            MessageRole::Tool => {
                // Tool messages should have tool_call_id and content
                if message.tool_call_id.is_none() {
                    return Err(GatewayError::Validation(format!(
                        "Tool message at index {} must have tool_call_id",
                        index
                    )));
                }
                if message.content.is_none() {
                    return Err(GatewayError::Validation(format!(
                        "Tool message at index {} must have content",
                        index
                    )));
                }
            }
        }

        // Validate content if present
        if let Some(content) = &message.content {
            Self::validate_message_content(content, index)?;
        }

        // Validate name if present
        if let Some(name) = &message.name {
            Self::validate_function_name(name)?;
        }

        Ok(())
    }

    /// Validate message content
    fn validate_message_content(
        content: &crate::core::models::openai::MessageContent,
        index: usize,
    ) -> Result<()> {
        match content {
            MessageContent::Text(text) => {
                if text.trim().is_empty() {
                    return Err(GatewayError::Validation(format!(
                        "Text content at message index {} cannot be empty",
                        index
                    )));
                }
                if text.len() > 1_000_000 {
                    return Err(GatewayError::Validation(format!(
                        "Text content at message index {} is too long (max 1M characters)",
                        index
                    )));
                }
            }
            MessageContent::Parts(parts) => {
                if parts.is_empty() {
                    return Err(GatewayError::Validation(format!(
                        "Content parts at message index {} cannot be empty",
                        index
                    )));
                }
                for (part_index, part) in parts.iter().enumerate() {
                    Self::validate_content_part(part, index, part_index)?;
                }
            }
        }

        Ok(())
    }

    /// Validate content part
    fn validate_content_part(
        part: &crate::core::models::openai::ContentPart,
        message_index: usize,
        part_index: usize,
    ) -> Result<()> {
        use crate::core::models::openai::ContentPart;

        match part {
            ContentPart::Text { text } => {
                if text.trim().is_empty() {
                    return Err(GatewayError::Validation(format!(
                        "Text part at message {} part {} cannot be empty",
                        message_index, part_index
                    )));
                }
            }
            ContentPart::ImageUrl { image_url } => {
                Self::validate_image_url(&image_url.url)?;
                if let Some(detail) = &image_url.detail {
                    if !["low", "high", "auto"].contains(&detail.as_str()) {
                        return Err(GatewayError::Validation(
                            "Image detail must be 'low', 'high', or 'auto'".to_string(),
                        ));
                    }
                }
            }
            ContentPart::Audio { audio } => {
                Self::validate_audio_data(&audio.data)?;
                Self::validate_audio_format(&audio.format)?;
            }
        }

        Ok(())
    }

    /// Validate model name
    fn validate_model_name(model: &str) -> Result<()> {
        if model.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Model name cannot be empty".to_string(),
            ));
        }

        // Check for valid characters
        let model_regex = Regex::new(r"^[a-zA-Z0-9._/-]+$")
            .map_err(|e| GatewayError::Internal(format!("Regex error: {}", e)))?;

        if !model_regex.is_match(model) {
            return Err(GatewayError::Validation(
                "Model name contains invalid characters".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate function name
    fn validate_function_name(name: &str) -> Result<()> {
        if name.trim().is_empty() {
            return Err(GatewayError::Validation(
                "Function name cannot be empty".to_string(),
            ));
        }

        // Function names should follow identifier rules
        let name_regex = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")
            .map_err(|e| GatewayError::Internal(format!("Regex error: {}", e)))?;

        if !name_regex.is_match(name) {
            return Err(GatewayError::Validation(
                "Function name must be a valid identifier".to_string(),
            ));
        }

        Ok(())
    }

    /// Validate image URL
    fn validate_image_url(url: &str) -> Result<()> {
        if url.starts_with("data:image/") {
            // Base64 encoded image
            Self::validate_base64_image(url)?;
        } else {
            // Regular URL
            url::Url::parse(url)
                .map_err(|e| GatewayError::Validation(format!("Invalid image URL: {}", e)))?;
        }
        Ok(())
    }

    /// Validate base64 image data
    fn validate_base64_image(data_url: &str) -> Result<()> {
        if !data_url.starts_with("data:image/") {
            return Err(GatewayError::Validation(
                "Invalid image data URL format".to_string(),
            ));
        }

        let parts: Vec<&str> = data_url.splitn(2, ',').collect();
        if parts.len() != 2 {
            return Err(GatewayError::Validation(
                "Invalid image data URL format".to_string(),
            ));
        }

        // Validate base64 data
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, parts[1])
            .map_err(|e| GatewayError::Validation(format!("Invalid base64 image data: {}", e)))?;

        Ok(())
    }

    /// Validate audio data
    fn validate_audio_data(data: &str) -> Result<()> {
        // Validate base64 encoded audio data
        base64::Engine::decode(&base64::engine::general_purpose::STANDARD, data)
            .map_err(|e| GatewayError::Validation(format!("Invalid base64 audio data: {}", e)))?;
        Ok(())
    }

    /// Validate audio format
    fn validate_audio_format(format: &str) -> Result<()> {
        let valid_formats = ["mp3", "wav", "flac", "m4a", "ogg", "webm"];
        if !valid_formats.contains(&format) {
            return Err(GatewayError::Validation(format!(
                "Invalid audio format: {}. Supported formats: {:?}",
                format, valid_formats
            )));
        }
        Ok(())
    }
}

// ==================== Unit Tests ====================

#[cfg(test)]
#[path = "request_validator_tests.rs"]
mod tests;
