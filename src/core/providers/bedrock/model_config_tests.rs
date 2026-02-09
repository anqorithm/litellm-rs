    use super::*;

    #[test]
    fn test_model_config_lookup() {
        let config = get_model_config("anthropic.claude-opus-4-6-v1:0").unwrap();
        assert_eq!(config.family, BedrockModelFamily::Claude);
        assert_eq!(config.api_type, BedrockApiType::Converse);

        let config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(config.family, BedrockModelFamily::Claude);
        assert_eq!(config.api_type, BedrockApiType::Converse);
        assert!(config.supports_streaming);
        assert!(config.supports_function_calling);
        assert!(config.supports_multimodal);

        let sonnet_v2 = get_model_config("anthropic.claude-3-5-sonnet-20241022-v2:0").unwrap();
        assert_eq!(sonnet_v2.family, BedrockModelFamily::Claude);
        assert_eq!(sonnet_v2.api_type, BedrockApiType::Converse);
    }

    #[test]
    fn test_model_capabilities() {
        assert!(model_supports_capability(
            "anthropic.claude-opus-4-6-v1:0",
            "streaming"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-opus-4-6-v1:0",
            "function_calling"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-opus-4-6-v1:0",
            "multimodal"
        ));

        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "streaming"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "function_calling"
        ));
        assert!(model_supports_capability(
            "anthropic.claude-3-opus-20240229",
            "multimodal"
        ));

        assert!(!model_supports_capability(
            "amazon.titan-text-express-v1",
            "function_calling"
        ));
        assert!(!model_supports_capability(
            "amazon.titan-text-express-v1",
            "multimodal"
        ));
    }

    #[test]
    fn test_unknown_model() {
        assert!(get_model_config("unknown-model").is_err());
        assert!(!model_supports_capability("unknown-model", "streaming"));
    }

    #[test]
    fn test_model_families() {
        let claude_config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(claude_config.family, BedrockModelFamily::Claude);

        let titan_config = get_model_config("amazon.titan-text-express-v1").unwrap();
        assert_eq!(titan_config.family, BedrockModelFamily::TitanText);

        let nova_config = get_model_config("amazon.nova-pro-v1:0").unwrap();
        assert_eq!(nova_config.family, BedrockModelFamily::Nova);
    }

    #[test]
    fn test_api_types() {
        let claude_config = get_model_config("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(claude_config.api_type, BedrockApiType::Converse);

        let titan_config = get_model_config("amazon.titan-text-express-v1").unwrap();
        assert_eq!(titan_config.api_type, BedrockApiType::Invoke);
    }
