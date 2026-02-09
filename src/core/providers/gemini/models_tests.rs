    use super::*;

    #[test]
    fn test_model_registry() {
        let registry = get_gemini_registry();

        // Test Gemini 2.0 Flash
        let flash_spec = registry.get_model_spec("gemini-2.0-flash-exp").unwrap();
        assert_eq!(flash_spec.family, GeminiModelFamily::Gemini20Flash);
        assert!(
            flash_spec
                .features
                .contains(&ModelFeature::MultimodalSupport)
        );
        assert!(
            flash_spec
                .features
                .contains(&ModelFeature::VideoUnderstanding)
        );

        // Test pricing
        assert_eq!(flash_spec.pricing.input_price, 0.01);
        assert_eq!(flash_spec.pricing.output_price, 0.04);
    }

    #[test]
    fn test_model_family_detection() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.0-flash-exp"),
            Some(GeminiModelFamily::Gemini20Flash)
        );

        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.5-pro-latest"),
            Some(GeminiModelFamily::Gemini15Pro)
        );

        assert_eq!(GeminiModelRegistry::from_model_name("unknown-model"), None);
    }

    #[test]
    fn test_cost_calculation() {
        let cost = CostCalculator::calculate_cost("gemini-1.5-flash", 1000, 500);
        assert!(cost.is_some());

        let cost_value = cost.unwrap();
        // Expected: (1000/1M * $0.075) + (500/1M * $0.30) = $0.000075 + $0.00015 = $0.000225
        assert!((cost_value - 0.000225).abs() < 0.000001);
    }

    #[test]
    fn test_feature_support() {
        let registry = get_gemini_registry();

        // Gemini 2.0 Flash supports video understanding
        assert!(
            registry.supports_feature("gemini-2.0-flash-exp", &ModelFeature::VideoUnderstanding)
        );

        // Gemini 1.0 Pro does not support multimodal
        assert!(!registry.supports_feature("gemini-1.0-pro", &ModelFeature::VideoUnderstanding));
    }

    #[test]
    fn test_registry_default() {
        let registry = GeminiModelRegistry::default();
        assert!(!registry.models.is_empty());
    }

    #[test]
    fn test_list_models() {
        let registry = get_gemini_registry();
        let models = registry.list_models();
        assert!(!models.is_empty());
        assert!(models.len() >= 10); // We have at least 10 models registered
    }

    #[test]
    fn test_get_model_family() {
        let registry = get_gemini_registry();
        let family = registry.get_model_family("gemini-1.5-pro");
        assert!(family.is_some());
        assert_eq!(*family.unwrap(), GeminiModelFamily::Gemini15Pro);

        let family_unknown = registry.get_model_family("unknown-model");
        assert!(family_unknown.is_none());
    }

    #[test]
    fn test_get_model_pricing() {
        let registry = get_gemini_registry();
        let pricing = registry.get_model_pricing("gemini-1.5-flash");
        assert!(pricing.is_some());
        let pricing_value = pricing.unwrap();
        assert_eq!(pricing_value.input_price, 0.075);
        assert_eq!(pricing_value.output_price, 0.30);
        assert!(pricing_value.cached_input_price.is_some());
    }

    #[test]
    fn test_get_model_limits() {
        let registry = get_gemini_registry();
        let limits = registry.get_model_limits("gemini-1.5-pro");
        assert!(limits.is_some());
        let limits_value = limits.unwrap();
        assert_eq!(limits_value.max_context_length, 2_000_000);
        assert_eq!(limits_value.max_output_tokens, 8192);
    }

    #[test]
    fn test_model_family_detection_gemini_3() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-3-pro"),
            Some(GeminiModelFamily::Gemini3Pro)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-3-pro-deep-think"),
            Some(GeminiModelFamily::Gemini3ProDeepThink)
        );
    }

    #[test]
    fn test_model_family_detection_gemini_25() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.5-pro"),
            Some(GeminiModelFamily::Gemini25Pro)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.5-flash"),
            Some(GeminiModelFamily::Gemini25Flash)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.5-flash-lite"),
            Some(GeminiModelFamily::Gemini25FlashLite)
        );
    }

    #[test]
    fn test_model_family_detection_gemini_20() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-2.0-flash-thinking-exp"),
            Some(GeminiModelFamily::Gemini20FlashThinking)
        );
    }

    #[test]
    fn test_model_family_detection_gemini_15() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.5-flash-8b"),
            Some(GeminiModelFamily::Gemini15Flash8B)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.5-flash"),
            Some(GeminiModelFamily::Gemini15Flash)
        );
    }

    #[test]
    fn test_model_family_detection_gemini_10() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.0-pro"),
            Some(GeminiModelFamily::Gemini10Pro)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-1.0-pro-vision"),
            Some(GeminiModelFamily::Gemini10ProVision)
        );
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-pro"),
            Some(GeminiModelFamily::Gemini10Pro)
        );
    }

    #[test]
    fn test_model_family_detection_experimental() {
        assert_eq!(
            GeminiModelRegistry::from_model_name("gemini-exp-something"),
            Some(GeminiModelFamily::GeminiExperimental)
        );
    }

    #[test]
    fn test_cost_calculation_unknown_model() {
        let cost = CostCalculator::calculate_cost("unknown-model", 1000, 500);
        assert!(cost.is_none());
    }

    #[test]
    fn test_multimodal_cost_calculation() {
        let cost = CostCalculator::calculate_multimodal_cost(
            "gemini-1.5-flash",
            1000,
            500,
            Some(200),
            Some(5),
            None,
            None,
        );
        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // Should include cached tokens discount and image cost
        assert!(cost_value > 0.0);
    }

    #[test]
    fn test_multimodal_cost_with_video_and_audio() {
        let cost = CostCalculator::calculate_multimodal_cost(
            "gemini-2.0-flash-exp",
            1000,
            500,
            None,
            Some(5),
            Some(60),
            Some(120),
        );
        assert!(cost.is_some());
        let cost_value = cost.unwrap();
        // Should include image, video, and audio costs
        assert!(cost_value > 0.0);
    }

    #[test]
    fn test_estimate_tokens() {
        let tokens = CostCalculator::estimate_tokens("Hello, world!");
        // "Hello, world!" is 13 characters, ~4 tokens (13/4 = 3.25, ceil = 4)
        assert!((3..=5).contains(&tokens));
    }

    #[test]
    fn test_feature_support_unknown_model() {
        let registry = get_gemini_registry();
        assert!(!registry.supports_feature("unknown-model", &ModelFeature::MultimodalSupport));
    }

    #[test]
    fn test_gemini_15_pro_features() {
        let registry = get_gemini_registry();
        let spec = registry.get_model_spec("gemini-1.5-pro").unwrap();

        assert!(spec.features.contains(&ModelFeature::ToolCalling));
        assert!(spec.features.contains(&ModelFeature::FunctionCalling));
        assert!(spec.features.contains(&ModelFeature::StreamingSupport));
        assert!(spec.features.contains(&ModelFeature::ContextCaching));
        assert!(spec.features.contains(&ModelFeature::SystemInstructions));
        assert!(spec.features.contains(&ModelFeature::BatchProcessing));
        assert!(spec.features.contains(&ModelFeature::JsonMode));
        assert!(spec.features.contains(&ModelFeature::CodeExecution));
        assert!(spec.features.contains(&ModelFeature::SearchGrounding));
        assert!(spec.features.contains(&ModelFeature::VideoUnderstanding));
        assert!(spec.features.contains(&ModelFeature::AudioUnderstanding));
    }

    #[test]
    fn test_gemini_10_pro_limited_features() {
        let registry = get_gemini_registry();
        let spec = registry.get_model_spec("gemini-1.0-pro").unwrap();

        // Gemini 1.0 Pro should not have multimodal support
        assert!(!spec.features.contains(&ModelFeature::MultimodalSupport));
        assert!(!spec.features.contains(&ModelFeature::VideoUnderstanding));
        assert!(!spec.features.contains(&ModelFeature::AudioUnderstanding));

        // But should have basic features
        assert!(spec.features.contains(&ModelFeature::ToolCalling));
        assert!(spec.features.contains(&ModelFeature::StreamingSupport));
    }

    #[test]
    fn test_model_info_structure() {
        let registry = get_gemini_registry();
        let spec = registry.get_model_spec("gemini-2.5-flash").unwrap();

        assert_eq!(spec.model_info.id, "gemini-2.5-flash");
        assert_eq!(spec.model_info.provider, "gemini");
        assert!(spec.model_info.supports_streaming);
        assert!(spec.model_info.supports_tools);
        assert!(spec.model_info.supports_multimodal);
    }
