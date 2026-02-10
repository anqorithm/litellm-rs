//! Volcengine Error Handling
//!
//! Error mapping for ByteDance's Volcengine AI platform

use crate::core::providers::unified_provider::ProviderError;
use crate::core::traits::error_mapper::trait_def::ErrorMapper;

/// Volcengine error mapper
#[derive(Debug)]
pub struct VolcengineErrorMapper;

impl ErrorMapper<ProviderError> for VolcengineErrorMapper {
    fn map_http_error(&self, status_code: u16, response_body: &str) -> ProviderError {
        match status_code {
            401 => ProviderError::authentication("volcengine", "Invalid API key"),
            403 => ProviderError::authentication("volcengine", "Permission denied"),
            404 => ProviderError::model_not_found("volcengine", "Model not found"),
            429 => {
                let retry_after = crate::core::providers::shared::parse_retry_after_from_body(response_body);
                ProviderError::rate_limit("volcengine", retry_after)
            }
            500..=599 => ProviderError::api_error("volcengine", status_code, response_body),
            _ => ProviderError::api_error("volcengine", status_code, response_body),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volcengine_error_mapper_401() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(401, "Unauthorized");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_403() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(403, "Forbidden");
        assert!(matches!(err, ProviderError::Authentication { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_404() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(404, "Not found");
        assert!(matches!(err, ProviderError::ModelNotFound { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_429() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(429, "rate limit exceeded");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_429_chinese() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(429, "请求频率过高");
        assert!(matches!(err, ProviderError::RateLimit { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_500() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_503() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(503, "Service unavailable");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_volcengine_error_mapper_unknown() {
        let mapper = VolcengineErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_parse_retry_after_with_rate_limit() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("rate limit exceeded");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_chinese() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("请求频率过高");
        assert_eq!(result, Some(60));
    }

    #[test]
    fn test_parse_retry_after_without_rate_limit() {
        let result = crate::core::providers::shared::parse_retry_after_from_body("other error");
        assert_eq!(result, None);
    }
}
