//! OpenRouter request/response transformer
//!
//! OpenRouter uses OpenAI-compatible API, but needs to process additional parameters

use super::error::OpenRouterError;
use crate::ProviderError;
use crate::core::providers::openai::models as openai_models;
use crate::core::providers::openai::transformer::OpenAIRequestTransformer;
use crate::core::types::{
    chat::ChatRequest,
    responses::{ChatChunk, ChatResponse},
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OpenRouter specific parameters
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OpenRouterExtraParams {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub models: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
}

/// Error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenRouterErrorModel {
    pub message: String,
    pub code: i64,
    #[serde(rename = "type")]
    pub error_type: Option<String>,
}

/// Request transformer
/// Inherits from OpenAI transformer, because OpenRouter is OpenAI compatible
pub struct OpenRouterRequestTransformer;

impl OpenRouterRequestTransformer {
    /// Transform request
    /// OpenRouter uses OpenAI format, but supports additional parameters
    pub fn transform_request(
        request: ChatRequest,
        extra_params: Option<OpenRouterExtraParams>,
    ) -> Result<openai_models::OpenAIChatRequest, OpenRouterError> {
        // Transform to OpenAI request
        let openai_request = OpenAIRequestTransformer::transform(request)
            .map_err(|e| ProviderError::invalid_request("openrouter", e.to_string()))?;

        // If there are OpenRouter specific params, add to extra_body
        if let Some(extra) = extra_params {
            let mut extra_body = HashMap::new();

            if let Some(transforms) = extra.transforms {
                extra_body.insert("transforms".to_string(), serde_json::to_value(transforms)?);
            }

            if let Some(models) = extra.models {
                extra_body.insert("models".to_string(), serde_json::to_value(models)?);
            }

            if let Some(route) = extra.route {
                extra_body.insert("route".to_string(), serde_json::to_value(route)?);
            }

            if let Some(provider) = extra.provider {
                extra_body.insert("provider".to_string(), serde_json::to_value(provider)?);
            }

            // OpenRouter's extra_body parameters will be passed through OpenAI client
            // Additional request processing could be done here
        }

        Ok(openai_request)
    }

    /// Check if should keep cache control
    /// Model specific logic
    pub fn should_keep_cache_control(model: &str) -> bool {
        model.to_lowercase().contains("claude")
    }
}

/// Response transformer
pub struct OpenRouterResponseTransformer;

impl OpenRouterResponseTransformer {
    /// Transform response
    pub fn transform_response(
        response: openai_models::OpenAIChatResponse,
    ) -> Result<ChatResponse, OpenRouterError> {
        // Delegate to OpenAI transformer
        crate::core::providers::openai::transformer::OpenAIResponseTransformer::transform(response)
            .map_err(|e| {
                ProviderError::transformation_error(
                    "openrouter",
                    "openai",
                    "openrouter",
                    e.to_string(),
                )
            })
    }

    /// Transform stream chunk
    pub fn transform_stream_chunk(
        chunk: openai_models::OpenAIStreamChunk,
    ) -> Result<ChatChunk, OpenRouterError> {
        // Check for errors in chunk
        if let Some(error) = Self::check_error_in_chunk(&chunk) {
            return Err(error);
        }

        // OpenRouter may include reasoning field in delta
        // Handle OpenRouter-specific delta fields if needed

        // Delegate to OpenAI transformer
        crate::core::providers::openai::transformer::OpenAIResponseTransformer::transform_stream_chunk(chunk)
            .map_err(|e| ProviderError::transformation_error("openrouter", "openai", "openrouter", e.to_string()))
    }

    /// Check error in chunk
    fn check_error_in_chunk(_chunk: &openai_models::OpenAIStreamChunk) -> Option<OpenRouterError> {
        // OpenRouter specific error checking could be implemented here
        // Handle any special error patterns
        None // No error found
    }

    /// Parse error response
    pub fn parse_error(error_body: &str, status_code: u16) -> OpenRouterError {
        if let Ok(error_model) = serde_json::from_str::<OpenRouterErrorModel>(error_body) {
            let message = format!(
                "OpenRouter Error: {} (Code: {})",
                error_model.message, error_model.code
            );

            match error_model.code {
                401 => ProviderError::authentication("openrouter", message),
                429 => ProviderError::rate_limit_simple("openrouter", message),
                400 => ProviderError::invalid_request("openrouter", message),
                404 => ProviderError::model_not_found("openrouter", error_model.message),
                _ => ProviderError::api_error("openrouter", status_code, message),
            }
        } else {
            ProviderError::api_error("openrouter", status_code, error_body.to_string())
        }
    }
}

/// Create OpenRouter headers
pub fn create_openrouter_headers(
    api_key: &str,
    http_referer: Option<&str>,
    x_title: Option<&str>,
) -> HashMap<String, String> {
    // Pre-allocate capacity for known headers: Authorization, Content-Type, User-Agent + optional
    let mut headers = HashMap::with_capacity(5);

    // Authorization header
    headers.insert("Authorization".to_string(), format!("Bearer {}", api_key));

    // Content type
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    // OpenRouter specific HTTP headers
    if let Some(referer) = http_referer {
        headers.insert("HTTP-Referer".to_string(), referer.to_string());
    }

    if let Some(title) = x_title {
        headers.insert("X-Title".to_string(), title.to_string());
    }

    // User agent
    headers.insert(
        "User-Agent".to_string(),
        "LiteLLM-RS/0.1.0 (OpenRouter)".to_string(),
    );

    headers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openrouter_extra_params_default() {
        let params = OpenRouterExtraParams::default();
        assert!(params.transforms.is_none());
        assert!(params.models.is_none());
        assert!(params.route.is_none());
        assert!(params.provider.is_none());
    }

    #[test]
    fn test_openrouter_extra_params_with_values() {
        let params = OpenRouterExtraParams {
            transforms: Some(vec!["middle-out".to_string()]),
            models: Some(vec!["gpt-4".to_string(), "claude-3".to_string()]),
            route: Some("fallback".to_string()),
            provider: Some("openai".to_string()),
        };

        assert_eq!(params.transforms.as_ref().unwrap().len(), 1);
        assert_eq!(params.models.as_ref().unwrap().len(), 2);
        assert_eq!(params.route, Some("fallback".to_string()));
        assert_eq!(params.provider, Some("openai".to_string()));
    }

    #[test]
    fn test_openrouter_extra_params_serialization() {
        let params = OpenRouterExtraParams {
            transforms: Some(vec!["middle-out".to_string()]),
            models: None,
            route: Some("fallback".to_string()),
            provider: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["transforms"], serde_json::json!(["middle-out"]));
        assert_eq!(json["route"], "fallback");
        assert!(json.get("models").is_none());
        assert!(json.get("provider").is_none());
    }

    #[test]
    fn test_openrouter_error_model() {
        let json = r#"{
            "message": "Rate limit exceeded",
            "code": 429,
            "type": "rate_limit_error"
        }"#;

        let error: OpenRouterErrorModel = serde_json::from_str(json).unwrap();
        assert_eq!(error.message, "Rate limit exceeded");
        assert_eq!(error.code, 429);
        assert_eq!(error.error_type, Some("rate_limit_error".to_string()));
    }

    #[test]
    fn test_should_keep_cache_control() {
        assert!(OpenRouterRequestTransformer::should_keep_cache_control(
            "anthropic/claude-3-opus"
        ));
        assert!(OpenRouterRequestTransformer::should_keep_cache_control(
            "CLAUDE-3-sonnet"
        ));
        assert!(!OpenRouterRequestTransformer::should_keep_cache_control(
            "openai/gpt-4"
        ));
        assert!(!OpenRouterRequestTransformer::should_keep_cache_control(
            "meta-llama/llama-3"
        ));
    }

    #[test]
    fn test_create_openrouter_headers_basic() {
        let headers = create_openrouter_headers("sk-test-key", None, None);

        assert_eq!(headers.get("Authorization").unwrap(), "Bearer sk-test-key");
        assert_eq!(headers.get("Content-Type").unwrap(), "application/json");
        assert!(headers.get("User-Agent").unwrap().contains("LiteLLM-RS"));
        assert!(!headers.contains_key("HTTP-Referer"));
        assert!(!headers.contains_key("X-Title"));
    }

    #[test]
    fn test_create_openrouter_headers_with_referer() {
        let headers = create_openrouter_headers("sk-test-key", Some("https://myapp.com"), None);

        assert_eq!(headers.get("HTTP-Referer").unwrap(), "https://myapp.com");
    }

    #[test]
    fn test_create_openrouter_headers_with_title() {
        let headers = create_openrouter_headers("sk-test-key", None, Some("My App"));

        assert_eq!(headers.get("X-Title").unwrap(), "My App");
    }

    #[test]
    fn test_create_openrouter_headers_full() {
        let headers =
            create_openrouter_headers("sk-test-key", Some("https://myapp.com"), Some("My App"));

        assert_eq!(headers.get("Authorization").unwrap(), "Bearer sk-test-key");
        assert_eq!(headers.get("HTTP-Referer").unwrap(), "https://myapp.com");
        assert_eq!(headers.get("X-Title").unwrap(), "My App");
        assert_eq!(headers.len(), 5);
    }

    #[test]
    fn test_parse_error_with_valid_json() {
        let error_body = r#"{"message": "Rate limit exceeded", "code": 429}"#;
        let error = OpenRouterResponseTransformer::parse_error(error_body, 429);
        assert!(matches!(error, OpenRouterError::RateLimit { .. }));
    }

    #[test]
    fn test_parse_error_auth() {
        let error_body = r#"{"message": "Invalid API key", "code": 401}"#;
        let error = OpenRouterResponseTransformer::parse_error(error_body, 401);
        assert!(matches!(error, OpenRouterError::Authentication { .. }));
    }

    #[test]
    fn test_parse_error_invalid_request() {
        let error_body = r#"{"message": "Invalid parameters", "code": 400}"#;
        let error = OpenRouterResponseTransformer::parse_error(error_body, 400);
        assert!(matches!(error, OpenRouterError::InvalidRequest { .. }));
    }

    #[test]
    fn test_parse_error_model_not_found() {
        let error_body = r#"{"message": "Model gpt-5 not found", "code": 404}"#;
        let error = OpenRouterResponseTransformer::parse_error(error_body, 404);
        assert!(matches!(error, OpenRouterError::ModelNotFound { .. }));
    }

    #[test]
    fn test_parse_error_invalid_json() {
        let error_body = "invalid json";
        let error = OpenRouterResponseTransformer::parse_error(error_body, 500);
        assert!(matches!(error, OpenRouterError::ApiError { .. }));
    }
}
