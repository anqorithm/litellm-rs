//! DeepSeek Error Handling
//!
//! Error handling

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Error
#[derive(Debug)]
pub struct DeepSeekErrorMapper;

impl ErrorMapper<ProviderError> for DeepSeekErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("deepseek", "Invalid API key"),
            403 => ProviderError::authentication("deepseek", "Permission denied"),
            404 => ProviderError::model_not_found("deepseek", "Model not found"),
            429 => {
                // Response
                let retry_after = crate::core::providers::shared::parse_retry_after_from_body(response_body);
                ProviderError::rate_limit("deepseek", retry_after)
            }
            500..=599 => ProviderError::api_error("deepseek", status_code, response_body),
            _ => ProviderError::api_error("deepseek", status_code, response_body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deepseek_error_mapper_401() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_403() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_404() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_429() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_429_no_rate_limit() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(429, "too many requests");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_500() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_503() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_unknown() {
        let mapper = DeepSeekErrorMapper;
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
    fn test_parse_retry_after_empty() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("");
        assert_eq!(result, None);
    }
}
