//! Stability AI Error Handling
//!
//! Error mapping for Stability AI API responses.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Stability AI error mapper
#[derive(Debug)]
pub struct StabilityErrorMapper;

impl ErrorMapper<ProviderError> for StabilityErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => {
                // Check for specific error types
                if response_body.contains("content_filtered")
                    || response_body.contains("CONTENT_FILTERED")
                {
                    ProviderError::content_filtered(
                        "stability",
                        "Content was filtered by Stability AI safety systems",
                        None,
                        Some(false),
                    )
                } else if response_body.contains("invalid_prompt") {
                    ProviderError::invalid_request("stability", "Invalid prompt provided")
                } else {
                    ProviderError::invalid_request("stability", response_body)
                }
            }
            401 => ProviderError::authentication("stability", "Invalid API key"),
            403 => ProviderError::authentication("stability", "Access denied or API key expired"),
            404 => ProviderError::model_not_found("stability", "Model or endpoint not found"),
            429 => {
                let retry_after = crate::core::providers::shared::parse_retry_after_from_body(response_body);
                ProviderError::rate_limit("stability", retry_after)
            }
            500..=599 => ProviderError::provider_unavailable(
                "stability",
                format!("Server error: {}", response_body),
            ),
            _ => ProviderError::api_error("stability", status_code, response_body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stability_error_mapper_400() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(400, "Invalid request");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_stability_error_mapper_400_content_filtered() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(400, "content_filtered: nudity detected");
        assert!(matches!(err, ProviderError::ContentFiltered { .. }));
    }

    #[test]
    fn test_stability_error_mapper_401() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_stability_error_mapper_403() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_stability_error_mapper_404() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_stability_error_mapper_429() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_stability_error_mapper_500() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_stability_error_mapper_503() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_stability_error_mapper_unknown() {
        let mapper = StabilityErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_parse_retry_after_with_rate_limit() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("rate limit exceeded");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_without_rate_limit() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("other error");
        assert_eq!(result, None);
    }
}
