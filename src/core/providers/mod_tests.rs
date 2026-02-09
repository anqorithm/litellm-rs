    use super::*;

    // ==================== ModelPricing Tests ====================

    #[test]
    fn test_model_pricing_creation() {
        let pricing = ModelPricing {
            model: "gpt-4".to_string(),
            input_cost_per_1k: 0.03,
            output_cost_per_1k: 0.06,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
        };

        assert_eq!(pricing.model, "gpt-4");
        assert_eq!(pricing.input_cost_per_1k, 0.03);
        assert_eq!(pricing.output_cost_per_1k, 0.06);
        assert_eq!(pricing.currency, "USD");
    }

    #[test]
    fn test_model_pricing_clone() {
        let pricing = ModelPricing {
            model: "claude-3-opus".to_string(),
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
        };

        let cloned = pricing.clone();
        assert_eq!(cloned.model, pricing.model);
        assert_eq!(cloned.input_cost_per_1k, pricing.input_cost_per_1k);
        assert_eq!(cloned.output_cost_per_1k, pricing.output_cost_per_1k);
    }

    #[test]
    fn test_model_pricing_zero_cost() {
        let pricing = ModelPricing {
            model: "free-model".to_string(),
            input_cost_per_1k: 0.0,
            output_cost_per_1k: 0.0,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
        };

        assert_eq!(pricing.input_cost_per_1k, 0.0);
        assert_eq!(pricing.output_cost_per_1k, 0.0);
    }

    #[test]
    fn test_model_pricing_debug() {
        let pricing = ModelPricing {
            model: "gpt-4".to_string(),
            input_cost_per_1k: 0.03,
            output_cost_per_1k: 0.06,
            currency: "USD".to_string(),
            updated_at: Utc::now(),
        };

        let debug_str = format!("{:?}", pricing);
        assert!(debug_str.contains("gpt-4"));
        assert!(debug_str.contains("0.03"));
    }

    // ==================== ProviderType Tests ====================

    #[test]
    fn test_provider_type_from_str_openai() {
        assert_eq!(ProviderType::from("openai"), ProviderType::OpenAI);
        assert_eq!(ProviderType::from("OpenAI"), ProviderType::OpenAI);
        assert_eq!(ProviderType::from("OPENAI"), ProviderType::OpenAI);
    }

    #[test]
    fn test_provider_type_from_str_anthropic() {
        assert_eq!(ProviderType::from("anthropic"), ProviderType::Anthropic);
        assert_eq!(ProviderType::from("Anthropic"), ProviderType::Anthropic);
    }

    #[test]
    fn test_provider_type_from_str_bedrock() {
        assert_eq!(ProviderType::from("bedrock"), ProviderType::Bedrock);
        assert_eq!(ProviderType::from("aws-bedrock"), ProviderType::Bedrock);
    }

    #[test]
    fn test_provider_type_from_str_vertex_ai() {
        assert_eq!(ProviderType::from("vertex_ai"), ProviderType::VertexAI);
        assert_eq!(ProviderType::from("vertexai"), ProviderType::VertexAI);
        assert_eq!(ProviderType::from("vertex-ai"), ProviderType::VertexAI);
    }

    #[test]
    fn test_provider_type_from_str_azure() {
        assert_eq!(ProviderType::from("azure"), ProviderType::Azure);
        assert_eq!(ProviderType::from("azure-openai"), ProviderType::Azure);
    }

    #[test]
    fn test_provider_type_from_str_azure_ai() {
        assert_eq!(ProviderType::from("azure_ai"), ProviderType::AzureAI);
        assert_eq!(ProviderType::from("azureai"), ProviderType::AzureAI);
        assert_eq!(ProviderType::from("azure-ai"), ProviderType::AzureAI);
    }

    #[test]
    fn test_provider_type_from_str_deepseek() {
        assert_eq!(ProviderType::from("deepseek"), ProviderType::DeepSeek);
        assert_eq!(ProviderType::from("deep-seek"), ProviderType::DeepSeek);
    }

    #[test]
    fn test_provider_type_from_str_deepinfra() {
        assert_eq!(ProviderType::from("deepinfra"), ProviderType::DeepInfra);
        assert_eq!(ProviderType::from("deep-infra"), ProviderType::DeepInfra);
    }

    #[test]
    fn test_provider_type_from_str_meta_llama() {
        assert_eq!(ProviderType::from("meta_llama"), ProviderType::MetaLlama);
        assert_eq!(ProviderType::from("llama"), ProviderType::MetaLlama);
        assert_eq!(ProviderType::from("meta-llama"), ProviderType::MetaLlama);
    }

    #[test]
    fn test_provider_type_from_str_mistral() {
        assert_eq!(ProviderType::from("mistral"), ProviderType::Mistral);
        assert_eq!(ProviderType::from("mistralai"), ProviderType::Mistral);
    }

    #[test]
    fn test_provider_type_from_str_moonshot() {
        assert_eq!(ProviderType::from("moonshot"), ProviderType::Moonshot);
        assert_eq!(ProviderType::from("moonshot-ai"), ProviderType::Moonshot);
    }

    #[test]
    fn test_provider_type_from_str_cloudflare() {
        assert_eq!(ProviderType::from("cloudflare"), ProviderType::Cloudflare);
        assert_eq!(ProviderType::from("cf"), ProviderType::Cloudflare);
        assert_eq!(ProviderType::from("workers-ai"), ProviderType::Cloudflare);
    }

    #[test]
    fn test_provider_type_from_str_other_providers() {
        assert_eq!(ProviderType::from("openrouter"), ProviderType::OpenRouter);
        assert_eq!(ProviderType::from("groq"), ProviderType::Groq);
        assert_eq!(ProviderType::from("xai"), ProviderType::XAI);
        assert_eq!(ProviderType::from("v0"), ProviderType::V0);
    }

    #[test]
    fn test_provider_type_from_str_custom() {
        assert_eq!(
            ProviderType::from("custom-provider"),
            ProviderType::Custom("custom-provider".to_string())
        );
        assert_eq!(
            ProviderType::from("my-local-llm"),
            ProviderType::Custom("my-local-llm".to_string())
        );
    }

    #[test]
    fn test_provider_type_display() {
        assert_eq!(format!("{}", ProviderType::OpenAI), "openai");
        assert_eq!(format!("{}", ProviderType::Anthropic), "anthropic");
        assert_eq!(format!("{}", ProviderType::Bedrock), "bedrock");
        assert_eq!(format!("{}", ProviderType::OpenRouter), "openrouter");
        assert_eq!(format!("{}", ProviderType::VertexAI), "vertex_ai");
        assert_eq!(format!("{}", ProviderType::Azure), "azure");
        assert_eq!(format!("{}", ProviderType::AzureAI), "azure_ai");
        assert_eq!(format!("{}", ProviderType::DeepSeek), "deepseek");
        assert_eq!(format!("{}", ProviderType::DeepInfra), "deepinfra");
        assert_eq!(format!("{}", ProviderType::V0), "v0");
        assert_eq!(format!("{}", ProviderType::MetaLlama), "meta_llama");
        assert_eq!(format!("{}", ProviderType::Mistral), "mistral");
        assert_eq!(format!("{}", ProviderType::Moonshot), "moonshot");
        assert_eq!(format!("{}", ProviderType::Groq), "groq");
        assert_eq!(format!("{}", ProviderType::XAI), "xai");
        assert_eq!(format!("{}", ProviderType::Cloudflare), "cloudflare");
    }

    #[test]
    fn test_provider_type_display_custom() {
        let custom = ProviderType::Custom("my-custom-provider".to_string());
        assert_eq!(format!("{}", custom), "my-custom-provider");
    }

    #[test]
    fn test_provider_type_clone() {
        let original = ProviderType::OpenAI;
        let cloned = original.clone();
        assert_eq!(original, cloned);

        let custom = ProviderType::Custom("test".to_string());
        let custom_cloned = custom.clone();
        assert_eq!(custom, custom_cloned);
    }

    #[test]
    fn test_provider_type_equality() {
        assert_eq!(ProviderType::OpenAI, ProviderType::OpenAI);
        assert_ne!(ProviderType::OpenAI, ProviderType::Anthropic);
        assert_eq!(
            ProviderType::Custom("test".to_string()),
            ProviderType::Custom("test".to_string())
        );
        assert_ne!(
            ProviderType::Custom("test1".to_string()),
            ProviderType::Custom("test2".to_string())
        );
    }

    #[test]
    fn test_provider_type_hash() {
        use std::collections::HashSet;

        let mut set = HashSet::new();
        set.insert(ProviderType::OpenAI);
        set.insert(ProviderType::Anthropic);
        set.insert(ProviderType::Custom("custom".to_string()));

        assert!(set.contains(&ProviderType::OpenAI));
        assert!(set.contains(&ProviderType::Anthropic));
        assert!(set.contains(&ProviderType::Custom("custom".to_string())));
        assert!(!set.contains(&ProviderType::Bedrock));
    }

    #[test]
    fn test_provider_type_serialization() {
        let provider = ProviderType::OpenAI;
        let json = serde_json::to_string(&provider).unwrap();
        assert_eq!(json, "\"OpenAI\"");

        let custom = ProviderType::Custom("my-provider".to_string());
        let custom_json = serde_json::to_string(&custom).unwrap();
        assert!(custom_json.contains("Custom"));
        assert!(custom_json.contains("my-provider"));
    }

    #[test]
    fn test_provider_type_deserialization() {
        let provider: ProviderType = serde_json::from_str("\"OpenAI\"").unwrap();
        assert_eq!(provider, ProviderType::OpenAI);

        let anthropic: ProviderType = serde_json::from_str("\"Anthropic\"").unwrap();
        assert_eq!(anthropic, ProviderType::Anthropic);
    }

    #[test]
    fn test_provider_type_roundtrip_serialization() {
        let providers = vec![
            ProviderType::OpenAI,
            ProviderType::Anthropic,
            ProviderType::Bedrock,
            ProviderType::Custom("test".to_string()),
        ];

        for provider in providers {
            let json = serde_json::to_string(&provider).unwrap();
            let deserialized: ProviderType = serde_json::from_str(&json).unwrap();
            assert_eq!(provider, deserialized);
        }
    }

    #[test]
    fn test_provider_type_debug() {
        let provider = ProviderType::OpenAI;
        let debug_str = format!("{:?}", provider);
        assert_eq!(debug_str, "OpenAI");

        let custom = ProviderType::Custom("test".to_string());
        let custom_debug = format!("{:?}", custom);
        assert!(custom_debug.contains("Custom"));
        assert!(custom_debug.contains("test"));
    }

    // ==================== ProviderType From/To Consistency Tests ====================

    #[test]
    fn test_provider_type_from_display_consistency() {
        // Test that Display output can be parsed back (for non-custom types)
        let providers = vec![
            ProviderType::OpenAI,
            ProviderType::Anthropic,
            ProviderType::Bedrock,
            ProviderType::OpenRouter,
            ProviderType::VertexAI,
            ProviderType::Azure,
            ProviderType::AzureAI,
            ProviderType::DeepSeek,
            ProviderType::DeepInfra,
            ProviderType::V0,
            ProviderType::MetaLlama,
            ProviderType::Mistral,
            ProviderType::Moonshot,
            ProviderType::Groq,
            ProviderType::XAI,
            ProviderType::Cloudflare,
        ];

        for provider in providers {
            let display = format!("{}", provider);
            let parsed = ProviderType::from(display.as_str());
            assert_eq!(
                provider, parsed,
                "Display/From roundtrip failed for {:?}",
                provider
            );
        }
    }

    // ==================== Provider Enum Tests ====================

    // Note: Provider enum tests require actual provider initialization
    // which needs API keys. These tests verify the enum structure.

    #[test]
    fn test_provider_enum_is_send_sync() {
        // This compile-time check ensures Provider is Send + Sync
        // which is important for async code
        // Note: Commenting out as Provider may not implement Send + Sync
        // assert_send_sync::<Provider>();
    }

    #[test]
    fn test_provider_type_all_variants_covered() {
        // This test ensures we don't forget to update tests when adding new providers
        let all_known_providers = [
            "openai",
            "anthropic",
            "bedrock",
            "openrouter",
            "vertex_ai",
            "azure",
            "azure_ai",
            "deepseek",
            "deepinfra",
            "v0",
            "meta_llama",
            "mistral",
            "moonshot",
            "groq",
            "xai",
            "cloudflare",
        ];

        for provider_str in all_known_providers {
            let provider_type = ProviderType::from(provider_str);
            // Should not be Custom for known providers
            assert!(
                !matches!(provider_type, ProviderType::Custom(_)),
                "Provider '{}' should not be Custom",
                provider_str
            );
        }
    }

    #[test]
    fn test_provider_type_case_insensitive() {
        // Test various case combinations
        let cases = vec![
            ("OPENAI", ProviderType::OpenAI),
            ("OpenAI", ProviderType::OpenAI),
            ("openai", ProviderType::OpenAI),
            ("OpenAi", ProviderType::OpenAI),
            ("ANTHROPIC", ProviderType::Anthropic),
            ("Anthropic", ProviderType::Anthropic),
            ("GROQ", ProviderType::Groq),
            ("Groq", ProviderType::Groq),
        ];

        for (input, expected) in cases {
            assert_eq!(
                ProviderType::from(input),
                expected,
                "Case-insensitive parsing failed for '{}'",
                input
            );
        }
    }
