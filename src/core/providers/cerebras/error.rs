//! Cerebras Error Handling
//!
//! Error handling for Cerebras AI API

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Cerebras Error Mapper
#[derive(Debug)]
pub struct CerebrasErrorMapper;

impl ErrorMapper<ProviderError> for CerebrasErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("cerebras", "Invalid API key"),
            403 => ProviderError::authentication("cerebras", "Permission denied"),
            404 => ProviderError::model_not_found("cerebras", "Model not found"),
            429 => {
                let retry_after = crate::core::providers::shared::parse_retry_after_from_body(response_body);
                ProviderError::rate_limit("cerebras", retry_after)
            }
            500..=599 => ProviderError::api_error("cerebras", status_code, response_body),
            _ => ProviderError::api_error("cerebras", status_code, response_body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cerebras_error_mapper_401() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_403() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_404() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_429() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_429_no_rate_limit() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(429, "too many requests");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_500() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_503() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_cerebras_error_mapper_unknown() {
        let mapper = CerebrasErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_parse_retry_after_with_rate_limit() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("rate limit exceeded");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_with_rate_limit_underscore() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("rate_limit exceeded");
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
