//! DeepSeek Error Handling

crate::define_standard_error_mapper!("deepseek", DeepSeek);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::providers::unified_provider::ProviderError;
    use crate::core::traits::error_mapper::trait_def::ErrorMapper;

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
    fn test_deepseek_error_mapper_500() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(500, "Internal error");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }

    #[test]
    fn test_deepseek_error_mapper_unknown() {
        let mapper = DeepSeekErrorMapper;
        let err = mapper.map_http_error(418, "I'm a teapot");
        assert!(matches!(err, ProviderError::ApiError { .. }));
    }
}
