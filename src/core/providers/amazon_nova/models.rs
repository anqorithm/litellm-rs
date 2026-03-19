//! Amazon Nova Model Registry
//!
//! Model definitions and registry for Amazon Nova multimodal models

use std::collections::HashMap;

/// Amazon Nova model definition
#[derive(Debug, Clone)]
pub struct AmazonNovaModel {
    /// Model ID (e.g., "amazon.nova-pro-v1:0")
    pub id: String,
    /// Display name
    pub name: String,
    /// Model description
    pub description: String,
    /// Maximum context length
    pub context_length: u32,
    /// Maximum output tokens
    pub max_output_tokens: u32,
    /// Input cost per 1K tokens in USD
    pub input_cost_per_1k: f64,
    /// Output cost per 1K tokens in USD
    pub output_cost_per_1k: f64,
    /// Whether the model supports vision (image input)
    pub supports_vision: bool,
    /// Whether the model supports tool calling
    pub supports_tools: bool,
    /// Whether the model supports reasoning mode
    pub supports_reasoning: bool,
    /// Whether the model supports streaming
    pub supports_streaming: bool,
}

impl AmazonNovaModel {
    /// Create a new Amazon Nova model definition
    pub fn new(
        id: &str,
        name: &str,
        description: &str,
        context_length: u32,
        max_output_tokens: u32,
    ) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            context_length,
            max_output_tokens,
            input_cost_per_1k: 0.0,
            output_cost_per_1k: 0.0,
            supports_vision: false,
            supports_tools: true,
            supports_reasoning: false,
            supports_streaming: true,
        }
    }

    /// Set pricing
    pub fn with_pricing(mut self, input_cost: f64, output_cost: f64) -> Self {
        self.input_cost_per_1k = input_cost;
        self.output_cost_per_1k = output_cost;
        self
    }

    /// Enable vision support
    pub fn with_vision(mut self) -> Self {
        self.supports_vision = true;
        self
    }

    /// Enable reasoning support
    pub fn with_reasoning(mut self) -> Self {
        self.supports_reasoning = true;
        self
    }

    /// Disable tool support
    pub fn without_tools(mut self) -> Self {
        self.supports_tools = false;
        self
    }
}

/// Amazon Nova model registry
#[derive(Debug, Clone)]
pub struct AmazonNovaModelRegistry {
    models: HashMap<String, AmazonNovaModel>,
}

impl Default for AmazonNovaModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl AmazonNovaModelRegistry {
    /// Create a new model registry with default models
    pub fn new() -> Self {
        let mut models = HashMap::new();

        // Nova Pro - High capability model
        models.insert(
            "amazon.nova-pro-v1:0".to_string(),
            AmazonNovaModel::new(
                "amazon.nova-pro-v1:0",
                "Amazon Nova Pro",
                "High-capability multimodal model for complex tasks",
                300000,
                5000,
            )
            .with_pricing(0.0008, 0.0032)
            .with_vision()
            .with_reasoning(),
        );

        // Nova Lite - Cost-effective model
        models.insert(
            "amazon.nova-lite-v1:0".to_string(),
            AmazonNovaModel::new(
                "amazon.nova-lite-v1:0",
                "Amazon Nova Lite",
                "Cost-effective multimodal model for everyday tasks",
                300000,
                5000,
            )
            .with_pricing(0.00006, 0.00024)
            .with_vision(),
        );

        // Nova Micro - Fast text-only model
        models.insert(
            "amazon.nova-micro-v1:0".to_string(),
            AmazonNovaModel::new(
                "amazon.nova-micro-v1:0",
                "Amazon Nova Micro",
                "Fast text-only model optimized for speed",
                128000,
                5000,
            )
            .with_pricing(0.000035, 0.00014),
        );

        // Nova Premier - Most capable model (upcoming)
        models.insert(
            "amazon.nova-premier-v1:0".to_string(),
            AmazonNovaModel::new(
                "amazon.nova-premier-v1:0",
                "Amazon Nova Premier",
                "Most capable model for complex reasoning and multimodal tasks",
                1000000,
                10000,
            )
            .with_pricing(0.0025, 0.0125)
            .with_vision()
            .with_reasoning(),
        );

        // Also register simplified names (keys were inserted above, so expect is safe)
        let nova_pro = models
            .get("amazon.nova-pro-v1:0")
            .expect("nova-pro-v1:0 was just inserted")
            .clone();
        let nova_lite = models
            .get("amazon.nova-lite-v1:0")
            .expect("nova-lite-v1:0 was just inserted")
            .clone();
        let nova_micro = models
            .get("amazon.nova-micro-v1:0")
            .expect("nova-micro-v1:0 was just inserted")
            .clone();
        let nova_premier = models
            .get("amazon.nova-premier-v1:0")
            .expect("nova-premier-v1:0 was just inserted")
            .clone();
        models.insert("nova-pro".to_string(), nova_pro);
        models.insert("nova-lite".to_string(), nova_lite);
        models.insert("nova-micro".to_string(), nova_micro);
        models.insert("nova-premier".to_string(), nova_premier);

        Self { models }
    }

    /// Get model by ID
    pub fn get(&self, model_id: &str) -> Option<&AmazonNovaModel> {
        self.models.get(model_id)
    }

    /// Check if model is supported
    pub fn is_supported(&self, model_id: &str) -> bool {
        self.models.contains_key(model_id)
    }

    /// List all supported models (unique, not aliases)
    pub fn list_models(&self) -> Vec<&AmazonNovaModel> {
        // Return only the canonical model IDs
        self.models
            .iter()
            .filter(|(k, _)| k.starts_with("amazon.nova"))
            .map(|(_, v)| v)
            .collect()
    }

    /// Get pricing for a model
    pub fn get_pricing(&self, model_id: &str) -> Option<(f64, f64)> {
        self.models
            .get(model_id)
            .map(|m| (m.input_cost_per_1k, m.output_cost_per_1k))
    }

    /// Calculate cost for a request
    pub fn calculate_cost(&self, model_id: &str, input_tokens: u32, output_tokens: u32) -> f64 {
        if let Some((input_cost, output_cost)) = self.get_pricing(model_id) {
            let input_cost_total = (input_tokens as f64 / 1000.0) * input_cost;
            let output_cost_total = (output_tokens as f64 / 1000.0) * output_cost;
            input_cost_total + output_cost_total
        } else {
            0.0
        }
    }

    /// Register a custom model
    pub fn register(&mut self, model: AmazonNovaModel) {
        self.models.insert(model.id.clone(), model);
    }
}

/// Supported OpenAI parameters for Amazon Nova
pub const SUPPORTED_OPENAI_PARAMS: &[&str] = &[
    "max_tokens",
    "max_completion_tokens",
    "temperature",
    "top_p",
    "stop",
    "stream",
    "stream_options",
    "tools",
    "tool_choice",
    "reasoning_effort",
    "metadata",
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_default() {
        let registry = AmazonNovaModelRegistry::new();
        assert!(registry.is_supported("amazon.nova-pro-v1:0"));
        assert!(registry.is_supported("amazon.nova-lite-v1:0"));
        assert!(registry.is_supported("amazon.nova-micro-v1:0"));
    }

    #[test]
    fn test_model_registry_aliases() {
        let registry = AmazonNovaModelRegistry::new();
        assert!(registry.is_supported("nova-pro"));
        assert!(registry.is_supported("nova-lite"));
        assert!(registry.is_supported("nova-micro"));
    }

    #[test]
    fn test_model_registry_get() {
        let registry = AmazonNovaModelRegistry::new();
        let model = registry.get("amazon.nova-pro-v1:0");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "Amazon Nova Pro");
    }

    #[test]
    fn test_model_capabilities() {
        let registry = AmazonNovaModelRegistry::new();

        // Pro supports vision and reasoning
        let pro = registry.get("amazon.nova-pro-v1:0").unwrap();
        assert!(pro.supports_vision);
        assert!(pro.supports_reasoning);
        assert!(pro.supports_tools);

        // Micro is text-only
        let micro = registry.get("amazon.nova-micro-v1:0").unwrap();
        assert!(!micro.supports_vision);
        assert!(micro.supports_tools);
    }

    #[test]
    fn test_calculate_cost() {
        let registry = AmazonNovaModelRegistry::new();
        let cost = registry.calculate_cost("amazon.nova-pro-v1:0", 1000, 500);
        assert!(cost > 0.0);
    }

    #[test]
    fn test_calculate_cost_unknown_model() {
        let registry = AmazonNovaModelRegistry::new();
        let cost = registry.calculate_cost("unknown-model", 1000, 500);
        assert_eq!(cost, 0.0);
    }

    #[test]
    fn test_list_models() {
        let registry = AmazonNovaModelRegistry::new();
        let models = registry.list_models();
        assert!(!models.is_empty());
        // Should only return canonical names
        assert!(models.iter().all(|m| m.id.starts_with("amazon.nova")));
    }

    #[test]
    fn test_model_builder() {
        let model = AmazonNovaModel::new("test", "Test", "Test model", 100000, 4096)
            .with_pricing(0.001, 0.002)
            .with_vision()
            .with_reasoning();

        assert_eq!(model.input_cost_per_1k, 0.001);
        assert!(model.supports_vision);
        assert!(model.supports_reasoning);
    }

    #[test]
    fn test_register_custom_model() {
        let mut registry = AmazonNovaModelRegistry::new();
        let custom =
            AmazonNovaModel::new("custom-nova", "Custom Nova", "A custom model", 50000, 2000);
        registry.register(custom);

        assert!(registry.is_supported("custom-nova"));
    }

    #[test]
    fn test_get_pricing() {
        let registry = AmazonNovaModelRegistry::new();
        let pricing = registry.get_pricing("amazon.nova-lite-v1:0");
        assert!(pricing.is_some());
        let (input, output) = pricing.unwrap();
        assert!(input > 0.0);
        assert!(output > 0.0);
    }

    #[test]
    fn test_supported_openai_params() {
        assert!(SUPPORTED_OPENAI_PARAMS.contains(&"max_tokens"));
        assert!(SUPPORTED_OPENAI_PARAMS.contains(&"temperature"));
        assert!(SUPPORTED_OPENAI_PARAMS.contains(&"tools"));
        assert!(SUPPORTED_OPENAI_PARAMS.contains(&"reasoning_effort"));
    }
}
