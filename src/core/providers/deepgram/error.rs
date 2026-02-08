//! Deepgram-specific error types and error mapping
//!
//! Handles error conversion from Deepgram API responses to unified provider errors.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Error mapper for Deepgram provider
pub struct DeepgramErrorMapper;

impl ErrorMapper<ProviderError> for DeepgramErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        let message = if response_body.is_empty() {
            format!("HTTP error {}", status_code)
        } else {
            response_body.to_string()
        };

        match status_code {
            400 => ProviderError::invalid_request("deepgram", message),
            401 => ProviderError::authentication("deepgram", "Invalid API key"),
            402 => ProviderError::quota_exceeded("deepgram", "Usage quota exceeded"),
            403 => ProviderError::authentication("deepgram", "Access forbidden"),
            404 => ProviderError::model_not_found("deepgram", "Model not found"),
            429 => ProviderError::rate_limit("deepgram", Some(60)),
            500 => ProviderError::api_error("deepgram", 500, "Internal server error"),
            502 => ProviderError::api_error("deepgram", 502, "Bad gateway"),
            503 => ProviderError::api_error("deepgram", 503, "Service unavailable"),
            _ => ProviderError::api_error("deepgram", status_code, message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepgram_error_mapper_http_errors() {
        let mapper = DeepgramErrorMapper;

        let err = mapper.map_http_error(400, "bad request");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));

        let err = mapper.map_http_error(401, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(402, "");
        assert!(matches!(err, ProviderError::QuotaExceeded { .. }));

        let err = mapper.map_http_error(403, "");
        assert!(matches!(err, ProviderError::Authentication { .. }));

        let err = mapper.map_http_error(404, "");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));

        let err = mapper.map_http_error(429, "");
        assert!(matches!(err, ProviderError::RateLimit { .. }));

        let err = mapper.map_http_error(500, "");
        assert!(matches!(err, ProviderError::ApiError { .. }));

        let err = mapper.map_http_error(503, "");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_deepgram_error_mapper_unknown_status() {
        let mapper = DeepgramErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }
}
