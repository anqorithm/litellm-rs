//! Runway ML Error Handling
//!
//! Error mapping for Runway ML API responses.

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

const PROVIDER_NAME: &str = "runwayml";

/// Runway ML error mapper
#[derive(Debug)]
pub struct RunwayMLErrorMapper;

impl ErrorMapper<ProviderError> for RunwayMLErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            400 => {
                // Check for specific error types
                if response_body.contains("invalid_prompt") || response_body.contains("prompt") {
                    ProviderError::invalid_request(PROVIDER_NAME, response_body)
                } else if response_body.contains("content_policy")
                    || response_body.contains("safety")
                    || response_body.contains("moderation")
                {
                    ProviderError::content_filtered(
                        PROVIDER_NAME,
                        "Content was filtered by Runway ML safety systems",
                        None,
                        Some(false),
                    )
                } else {
                    ProviderError::invalid_request(PROVIDER_NAME, response_body)
                }
            }
            401 => ProviderError::authentication(PROVIDER_NAME, "Invalid API key"),
            403 => ProviderError::authentication(
                PROVIDER_NAME,
                "Access denied or insufficient permissions",
            ),
            404 => {
                if response_body.contains("task") {
                    ProviderError::api_error(PROVIDER_NAME, status_code, "Task not found")
                } else {
                    ProviderError::model_not_found(PROVIDER_NAME, "Model or endpoint not found")
                }
            }
            422 => ProviderError::invalid_request(
                PROVIDER_NAME,
                format!("Validation error: {}", response_body),
            ),
            429 => {
                let retry_after = crate::core::providers::shared::parse_retry_after_from_body(response_body);
                ProviderError::rate_limit(PROVIDER_NAME, retry_after)
            }
            500..=599 => ProviderError::provider_unavailable(
                PROVIDER_NAME,
                format!("Server error: {}", response_body),
            ),
            _ => ProviderError::api_error(PROVIDER_NAME, status_code, response_body),
        }
    }
}

// Note: Task-specific error handling is done directly using ProviderError factory methods
// in provider.rs. See poll_task() for usage of:
// - ProviderError::api_error(PROVIDER_NAME, 500, ...) for task failures
// - ProviderError::cancelled(PROVIDER_NAME, "video_generation", ...) for cancellations
// - ProviderError::timeout(PROVIDER_NAME, ...) for timeouts

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runwayml_error_mapper_400() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(400, "Invalid request parameters");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_400_content_filtered() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(400, "content_policy violation detected");
        assert!(matches!(err, ProviderError::ContentFiltered { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_401() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_403() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_404() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_404_task() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(404, "task not found");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_422() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(422, "Validation failed");
        assert!(matches!(err, ProviderError::InvalidRequest { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_429() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_500() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_503() {
        let mapper = RunwayMLErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ProviderUnavailable { .. }));
    }

    #[test]
    fn test_runwayml_error_mapper_unknown() {
        let mapper = RunwayMLErrorMapper;
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

    #[test]
    fn test_task_error_failed_via_provider_error() {
        // Test that task failure errors are created correctly using ProviderError factory
        let err = ProviderError::api_error(PROVIDER_NAME, 500, "Task failed: Generation failed");
        assert!(matches!(err, ProviderError::ApiError { status: 500, .. }));
    }

    #[test]
    fn test_task_error_canceled_via_provider_error() {
        // Test that task cancellation errors are created correctly using ProviderError factory
        let err = ProviderError::cancelled(
            PROVIDER_NAME,
            "video_generation",
            Some("Task canceled: User canceled".to_string()),
        );
        assert!(matches!(err, ProviderError::Cancelled { .. }));
    }

    #[test]
    fn test_task_error_timeout_via_provider_error() {
        // Test that task timeout errors are created correctly using ProviderError factory
        let err = ProviderError::timeout(PROVIDER_NAME, "Task timeout: Max retries exceeded");
        assert!(matches!(err, ProviderError::Timeout { .. }));
    }
}
