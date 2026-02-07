//! Cost Calculation for Bedrock Models
//!
//! Provides accurate pricing information and cost calculation
//! for all supported Bedrock models.

use std::collections::HashMap;
use std::sync::LazyLock;

/// Model pricing information
#[derive(Debug, Clone)]
pub struct ModelPricing {
    pub input_cost_per_1k: f64,
    pub output_cost_per_1k: f64,
    pub currency: &'static str,
}

/// Comprehensive pricing database for all Bedrock models
static MODEL_PRICING: LazyLock<HashMap<&'static str, ModelPricing>> = LazyLock::new(|| {
    let mut pricing = HashMap::new();

    // Claude models
    pricing.insert(
        "anthropic.claude-opus-4-6-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.025,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-opus-4-6-v1",
        ModelPricing {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.025,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-opus-4-6",
        ModelPricing {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.025,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-opus-4-5-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.025,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-opus-4-5",
        ModelPricing {
            input_cost_per_1k: 0.005,
            output_cost_per_1k: 0.025,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-sonnet-4-5-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-sonnet-4-5",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-sonnet-4-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-sonnet-4",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );

    pricing.insert(
        "anthropic.claude-3-opus-20240229",
        ModelPricing {
            input_cost_per_1k: 0.015,
            output_cost_per_1k: 0.075,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-3-sonnet-20240229",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-3-haiku-20240307",
        ModelPricing {
            input_cost_per_1k: 0.00025,
            output_cost_per_1k: 0.00125,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-3-5-sonnet-20241022",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-3-5-sonnet-20241022-v2:0",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-3-5-haiku-20241022",
        ModelPricing {
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.005,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-v2:1",
        ModelPricing {
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-v2",
        ModelPricing {
            input_cost_per_1k: 0.008,
            output_cost_per_1k: 0.024,
            currency: "USD",
        },
    );
    pricing.insert(
        "anthropic.claude-instant-v1",
        ModelPricing {
            input_cost_per_1k: 0.00163,
            output_cost_per_1k: 0.00551,
            currency: "USD",
        },
    );

    // Titan models
    pricing.insert(
        "amazon.titan-text-express-v1",
        ModelPricing {
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        },
    );
    pricing.insert(
        "amazon.titan-text-lite-v1",
        ModelPricing {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
            currency: "USD",
        },
    );
    pricing.insert(
        "amazon.titan-text-premier-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
            currency: "USD",
        },
    );

    // Nova models
    pricing.insert(
        "amazon.nova-micro-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.000035,
            output_cost_per_1k: 0.00014,
            currency: "USD",
        },
    );
    pricing.insert(
        "amazon.nova-lite-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00006,
            output_cost_per_1k: 0.00024,
            currency: "USD",
        },
    );
    pricing.insert(
        "amazon.nova-pro-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0008,
            output_cost_per_1k: 0.0032,
            currency: "USD",
        },
    );

    // AI21 models
    pricing.insert(
        "ai21.jamba-1-5-large-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.008,
            currency: "USD",
        },
    );
    pricing.insert(
        "ai21.jamba-1-5-mini-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0002,
            output_cost_per_1k: 0.0004,
            currency: "USD",
        },
    );
    pricing.insert(
        "ai21.jamba-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0007,
            currency: "USD",
        },
    );

    // Cohere models
    pricing.insert(
        "cohere.command-r-plus-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.003,
            output_cost_per_1k: 0.015,
            currency: "USD",
        },
    );
    pricing.insert(
        "cohere.command-r-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0005,
            output_cost_per_1k: 0.0015,
            currency: "USD",
        },
    );
    pricing.insert(
        "cohere.command-text-v14",
        ModelPricing {
            input_cost_per_1k: 0.0015,
            output_cost_per_1k: 0.002,
            currency: "USD",
        },
    );
    pricing.insert(
        "cohere.command-light-text-v14",
        ModelPricing {
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        },
    );

    // Mistral models
    pricing.insert(
        "mistral.mistral-7b-instruct-v0:2",
        ModelPricing {
            input_cost_per_1k: 0.00015,
            output_cost_per_1k: 0.0002,
            currency: "USD",
        },
    );
    pricing.insert(
        "mistral.mixtral-8x7b-instruct-v0:1",
        ModelPricing {
            input_cost_per_1k: 0.00045,
            output_cost_per_1k: 0.0007,
            currency: "USD",
        },
    );
    pricing.insert(
        "mistral.mistral-large-2402-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.004,
            output_cost_per_1k: 0.012,
            currency: "USD",
        },
    );
    pricing.insert(
        "mistral.mistral-large-2407-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.002,
            output_cost_per_1k: 0.006,
            currency: "USD",
        },
    );
    pricing.insert(
        "mistral.mistral-small-2402-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.001,
            output_cost_per_1k: 0.003,
            currency: "USD",
        },
    );

    // Meta Llama models
    pricing.insert(
        "meta.llama3-2-1b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00001,
            output_cost_per_1k: 0.00001,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-2-3b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.000015,
            output_cost_per_1k: 0.000015,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-2-11b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.000032,
            output_cost_per_1k: 0.000032,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-2-90b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00072,
            output_cost_per_1k: 0.00072,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-1-8b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00022,
            output_cost_per_1k: 0.00022,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-1-70b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00099,
            output_cost_per_1k: 0.00099,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-1-405b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00532,
            output_cost_per_1k: 0.016,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-8b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.0003,
            output_cost_per_1k: 0.0006,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama3-70b-instruct-v1:0",
        ModelPricing {
            input_cost_per_1k: 0.00265,
            output_cost_per_1k: 0.0035,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama2-13b-chat-v1",
        ModelPricing {
            input_cost_per_1k: 0.00075,
            output_cost_per_1k: 0.001,
            currency: "USD",
        },
    );
    pricing.insert(
        "meta.llama2-70b-chat-v1",
        ModelPricing {
            input_cost_per_1k: 0.00195,
            output_cost_per_1k: 0.00256,
            currency: "USD",
        },
    );

    pricing
});

/// Cost calculator for Bedrock models
pub struct CostCalculator;

impl CostCalculator {
    /// Calculate cost for a specific model and token usage
    pub fn calculate_cost(model_id: &str, input_tokens: u32, output_tokens: u32) -> Option<f64> {
        MODEL_PRICING.get(model_id).map(|pricing| {
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;
            input_cost + output_cost
        })
    }

    /// Get pricing information for a model
    pub fn get_model_pricing(model_id: &str) -> Option<&'static ModelPricing> {
        MODEL_PRICING.get(model_id)
    }

    /// Get all available models with pricing
    pub fn get_all_models() -> Vec<&'static str> {
        MODEL_PRICING.keys().copied().collect()
    }

    /// Calculate cost with breakdown
    pub fn calculate_detailed_cost(
        model_id: &str,
        input_tokens: u32,
        output_tokens: u32,
    ) -> Option<CostBreakdown> {
        MODEL_PRICING.get(model_id).map(|pricing| {
            let input_cost = (input_tokens as f64 / 1000.0) * pricing.input_cost_per_1k;
            let output_cost = (output_tokens as f64 / 1000.0) * pricing.output_cost_per_1k;

            CostBreakdown {
                input_tokens,
                output_tokens,
                input_cost,
                output_cost,
                total_cost: input_cost + output_cost,
                currency: pricing.currency,
            }
        })
    }
}

/// Detailed cost breakdown
#[derive(Debug, Clone)]
pub struct CostBreakdown {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub input_cost: f64,
    pub output_cost: f64,
    pub total_cost: f64,
    pub currency: &'static str,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Cost Calculation Tests ====================

    #[test]
    fn test_cost_calculation() {
        // Test Claude Opus pricing
        let cost = CostCalculator::calculate_cost(
            "anthropic.claude-3-opus-20240229",
            1000, // 1k input tokens
            500,  // 500 output tokens
        )
        .unwrap();

        // Expected: (1000/1000 * 0.015) + (500/1000 * 0.075) = 0.015 + 0.0375 = 0.0525
        assert!((cost - 0.0525).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_claude_opus_46() {
        let cost =
            CostCalculator::calculate_cost("anthropic.claude-opus-4-6-v1:0", 1000, 500).unwrap();
        // Expected: (1 * 0.005) + (0.5 * 0.025) = 0.0175
        assert!((cost - 0.0175).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_claude_sonnet() {
        let cost = CostCalculator::calculate_cost("anthropic.claude-3-sonnet-20240229", 1000, 1000)
            .unwrap();
        // Expected: (1 * 0.003) + (1 * 0.015) = 0.018
        assert!((cost - 0.018).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_claude_haiku() {
        let cost = CostCalculator::calculate_cost("anthropic.claude-3-haiku-20240307", 10000, 5000)
            .unwrap();
        // Expected: (10 * 0.00025) + (5 * 0.00125) = 0.0025 + 0.00625 = 0.00875
        assert!((cost - 0.00875).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_titan() {
        let cost =
            CostCalculator::calculate_cost("amazon.titan-text-express-v1", 5000, 2000).unwrap();
        // Expected: (5 * 0.0002) + (2 * 0.0006) = 0.001 + 0.0012 = 0.0022
        assert!((cost - 0.0022).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_nova_micro() {
        let cost = CostCalculator::calculate_cost("amazon.nova-micro-v1:0", 100000, 50000).unwrap();
        // Expected: (100 * 0.000035) + (50 * 0.00014) = 0.0035 + 0.007 = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_mistral() {
        let cost =
            CostCalculator::calculate_cost("mistral.mistral-large-2407-v1:0", 2000, 1000).unwrap();
        // Expected: (2 * 0.002) + (1 * 0.006) = 0.004 + 0.006 = 0.01
        assert!((cost - 0.01).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_llama() {
        let cost =
            CostCalculator::calculate_cost("meta.llama3-70b-instruct-v1:0", 3000, 2000).unwrap();
        // Expected: (3 * 0.00265) + (2 * 0.0035) = 0.00795 + 0.007 = 0.01495
        assert!((cost - 0.01495).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_cohere() {
        let cost = CostCalculator::calculate_cost("cohere.command-r-plus-v1:0", 1000, 500).unwrap();
        // Expected: (1 * 0.003) + (0.5 * 0.015) = 0.003 + 0.0075 = 0.0105
        assert!((cost - 0.0105).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_ai21() {
        let cost = CostCalculator::calculate_cost("ai21.jamba-1-5-large-v1:0", 4000, 2000).unwrap();
        // Expected: (4 * 0.002) + (2 * 0.008) = 0.008 + 0.016 = 0.024
        assert!((cost - 0.024).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_zero_tokens() {
        let cost =
            CostCalculator::calculate_cost("anthropic.claude-3-opus-20240229", 0, 0).unwrap();
        assert!((cost - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_only_input() {
        let cost =
            CostCalculator::calculate_cost("anthropic.claude-3-opus-20240229", 1000, 0).unwrap();
        // Expected: (1 * 0.015) + 0 = 0.015
        assert!((cost - 0.015).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_only_output() {
        let cost =
            CostCalculator::calculate_cost("anthropic.claude-3-opus-20240229", 0, 1000).unwrap();
        // Expected: 0 + (1 * 0.075) = 0.075
        assert!((cost - 0.075).abs() < 0.0001);
    }

    #[test]
    fn test_cost_calculation_large_tokens() {
        let cost =
            CostCalculator::calculate_cost("anthropic.claude-3-haiku-20240307", 1_000_000, 500_000)
                .unwrap();
        // Expected: (1000 * 0.00025) + (500 * 0.00125) = 0.25 + 0.625 = 0.875
        assert!((cost - 0.875).abs() < 0.001);
    }

    // ==================== Model Pricing Lookup Tests ====================

    #[test]
    fn test_model_pricing_lookup() {
        let pricing =
            CostCalculator::get_model_pricing("anthropic.claude-3-opus-20240229").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.015);
        assert_eq!(pricing.output_cost_per_1k, 0.075);
        assert_eq!(pricing.currency, "USD");
    }

    #[test]
    fn test_model_pricing_lookup_sonnet() {
        let pricing =
            CostCalculator::get_model_pricing("anthropic.claude-3-sonnet-20240229").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.003);
        assert_eq!(pricing.output_cost_per_1k, 0.015);
    }

    #[test]
    fn test_model_pricing_lookup_haiku() {
        let pricing =
            CostCalculator::get_model_pricing("anthropic.claude-3-haiku-20240307").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.00025);
        assert_eq!(pricing.output_cost_per_1k, 0.00125);
    }

    #[test]
    fn test_model_pricing_lookup_claude_35_sonnet() {
        let pricing =
            CostCalculator::get_model_pricing("anthropic.claude-3-5-sonnet-20241022").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.003);
        assert_eq!(pricing.output_cost_per_1k, 0.015);
    }

    #[test]
    fn test_model_pricing_lookup_titan() {
        let pricing = CostCalculator::get_model_pricing("amazon.titan-text-express-v1").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.0002);
        assert_eq!(pricing.output_cost_per_1k, 0.0006);
    }

    #[test]
    fn test_model_pricing_lookup_nova() {
        let pricing = CostCalculator::get_model_pricing("amazon.nova-pro-v1:0").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.0008);
        assert_eq!(pricing.output_cost_per_1k, 0.0032);
    }

    #[test]
    fn test_model_pricing_lookup_mistral() {
        let pricing =
            CostCalculator::get_model_pricing("mistral.mixtral-8x7b-instruct-v0:1").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.00045);
        assert_eq!(pricing.output_cost_per_1k, 0.0007);
    }

    #[test]
    fn test_model_pricing_lookup_llama() {
        let pricing =
            CostCalculator::get_model_pricing("meta.llama3-1-405b-instruct-v1:0").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.00532);
        assert_eq!(pricing.output_cost_per_1k, 0.016);
    }

    #[test]
    fn test_model_pricing_lookup_cohere() {
        let pricing = CostCalculator::get_model_pricing("cohere.command-r-v1:0").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.0005);
        assert_eq!(pricing.output_cost_per_1k, 0.0015);
    }

    #[test]
    fn test_model_pricing_lookup_ai21() {
        let pricing = CostCalculator::get_model_pricing("ai21.jamba-instruct-v1:0").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.0005);
        assert_eq!(pricing.output_cost_per_1k, 0.0007);
    }

    #[test]
    fn test_model_pricing_lookup_unknown() {
        let pricing = CostCalculator::get_model_pricing("unknown-model");
        assert!(pricing.is_none());
    }

    // ==================== Detailed Cost Breakdown Tests ====================

    #[test]
    fn test_detailed_cost_breakdown() {
        let breakdown =
            CostCalculator::calculate_detailed_cost("amazon.titan-text-express-v1", 2000, 1000)
                .unwrap();

        assert_eq!(breakdown.input_tokens, 2000);
        assert_eq!(breakdown.output_tokens, 1000);
        assert_eq!(breakdown.currency, "USD");
        assert!(breakdown.total_cost > 0.0);
    }

    #[test]
    fn test_detailed_cost_breakdown_claude() {
        let breakdown =
            CostCalculator::calculate_detailed_cost("anthropic.claude-3-opus-20240229", 1000, 500)
                .unwrap();

        assert_eq!(breakdown.input_tokens, 1000);
        assert_eq!(breakdown.output_tokens, 500);
        assert!((breakdown.input_cost - 0.015).abs() < 0.0001);
        assert!((breakdown.output_cost - 0.0375).abs() < 0.0001);
        assert!((breakdown.total_cost - 0.0525).abs() < 0.0001);
        assert_eq!(breakdown.currency, "USD");
    }

    #[test]
    fn test_detailed_cost_breakdown_zero_tokens() {
        let breakdown =
            CostCalculator::calculate_detailed_cost("anthropic.claude-3-haiku-20240307", 0, 0)
                .unwrap();

        assert_eq!(breakdown.input_tokens, 0);
        assert_eq!(breakdown.output_tokens, 0);
        assert!((breakdown.input_cost - 0.0).abs() < 0.0001);
        assert!((breakdown.output_cost - 0.0).abs() < 0.0001);
        assert!((breakdown.total_cost - 0.0).abs() < 0.0001);
    }

    #[test]
    fn test_detailed_cost_breakdown_unknown_model() {
        let breakdown = CostCalculator::calculate_detailed_cost("unknown-model", 1000, 500);
        assert!(breakdown.is_none());
    }

    #[test]
    fn test_detailed_cost_sum() {
        let breakdown =
            CostCalculator::calculate_detailed_cost("mistral.mistral-large-2407-v1:0", 5000, 3000)
                .unwrap();

        // Verify total equals input + output
        let expected_total = breakdown.input_cost + breakdown.output_cost;
        assert!((breakdown.total_cost - expected_total).abs() < 0.0001);
    }

    // ==================== Unknown Model Tests ====================

    #[test]
    fn test_unknown_model() {
        let cost = CostCalculator::calculate_cost("unknown-model", 1000, 500);
        assert!(cost.is_none());
    }

    #[test]
    fn test_empty_model_id() {
        let cost = CostCalculator::calculate_cost("", 1000, 500);
        assert!(cost.is_none());
    }

    #[test]
    fn test_partial_model_id() {
        let cost = CostCalculator::calculate_cost("anthropic.claude", 1000, 500);
        assert!(cost.is_none());
    }

    // ==================== All Models List Tests ====================

    #[test]
    fn test_all_models_list() {
        let models = CostCalculator::get_all_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"anthropic.claude-3-opus-20240229"));
        assert!(models.contains(&"amazon.titan-text-express-v1"));
    }

    #[test]
    fn test_all_models_contains_claude() {
        let models = CostCalculator::get_all_models();
        let claude_count = models
            .iter()
            .filter(|m| m.starts_with("anthropic."))
            .count();
        assert!(claude_count >= 8);
    }

    #[test]
    fn test_all_models_contains_titan() {
        let models = CostCalculator::get_all_models();
        let titan_count = models
            .iter()
            .filter(|m| m.starts_with("amazon.titan"))
            .count();
        assert!(titan_count >= 3);
    }

    #[test]
    fn test_all_models_contains_nova() {
        let models = CostCalculator::get_all_models();
        let nova_count = models
            .iter()
            .filter(|m| m.starts_with("amazon.nova"))
            .count();
        assert!(nova_count >= 3);
    }

    #[test]
    fn test_all_models_contains_mistral() {
        let models = CostCalculator::get_all_models();
        let mistral_count = models.iter().filter(|m| m.starts_with("mistral.")).count();
        assert!(mistral_count >= 5);
    }

    #[test]
    fn test_all_models_contains_llama() {
        let models = CostCalculator::get_all_models();
        let llama_count = models
            .iter()
            .filter(|m| m.starts_with("meta.llama"))
            .count();
        assert!(llama_count >= 10);
    }

    #[test]
    fn test_all_models_contains_cohere() {
        let models = CostCalculator::get_all_models();
        let cohere_count = models.iter().filter(|m| m.starts_with("cohere.")).count();
        assert!(cohere_count >= 4);
    }

    #[test]
    fn test_all_models_contains_ai21() {
        let models = CostCalculator::get_all_models();
        let ai21_count = models.iter().filter(|m| m.starts_with("ai21.")).count();
        assert!(ai21_count >= 3);
    }

    #[test]
    fn test_all_models_total_count() {
        let models = CostCalculator::get_all_models();
        // Should have at least 30 models
        assert!(models.len() >= 30);
    }

    // ==================== ModelPricing Struct Tests ====================

    #[test]
    fn test_model_pricing_debug() {
        let pricing = ModelPricing {
            input_cost_per_1k: 0.01,
            output_cost_per_1k: 0.02,
            currency: "USD",
        };
        let debug = format!("{:?}", pricing);
        assert!(debug.contains("ModelPricing"));
        assert!(debug.contains("0.01"));
        assert!(debug.contains("0.02"));
    }

    #[test]
    fn test_model_pricing_clone() {
        let pricing = ModelPricing {
            input_cost_per_1k: 0.01,
            output_cost_per_1k: 0.02,
            currency: "USD",
        };
        let cloned = pricing.clone();
        assert_eq!(cloned.input_cost_per_1k, 0.01);
        assert_eq!(cloned.output_cost_per_1k, 0.02);
        assert_eq!(cloned.currency, "USD");
    }

    // ==================== CostBreakdown Struct Tests ====================

    #[test]
    fn test_cost_breakdown_debug() {
        let breakdown = CostBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            input_cost: 0.015,
            output_cost: 0.0375,
            total_cost: 0.0525,
            currency: "USD",
        };
        let debug = format!("{:?}", breakdown);
        assert!(debug.contains("CostBreakdown"));
        assert!(debug.contains("1000"));
        assert!(debug.contains("500"));
    }

    #[test]
    fn test_cost_breakdown_clone() {
        let breakdown = CostBreakdown {
            input_tokens: 1000,
            output_tokens: 500,
            input_cost: 0.015,
            output_cost: 0.0375,
            total_cost: 0.0525,
            currency: "USD",
        };
        let cloned = breakdown.clone();
        assert_eq!(cloned.input_tokens, 1000);
        assert_eq!(cloned.output_tokens, 500);
        assert_eq!(cloned.total_cost, 0.0525);
    }

    // ==================== Legacy Model Tests ====================

    #[test]
    fn test_claude_v2_pricing() {
        let pricing = CostCalculator::get_model_pricing("anthropic.claude-v2").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.008);
        assert_eq!(pricing.output_cost_per_1k, 0.024);
    }

    #[test]
    fn test_claude_instant_pricing() {
        let pricing = CostCalculator::get_model_pricing("anthropic.claude-instant-v1").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.00163);
        assert_eq!(pricing.output_cost_per_1k, 0.00551);
    }

    #[test]
    fn test_llama2_pricing() {
        let pricing = CostCalculator::get_model_pricing("meta.llama2-70b-chat-v1").unwrap();
        assert_eq!(pricing.input_cost_per_1k, 0.00195);
        assert_eq!(pricing.output_cost_per_1k, 0.00256);
    }
}
