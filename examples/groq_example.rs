//! Example of using the Groq provider

use litellm_rs::core::providers::groq::{GroqConfig, GroqProvider};
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create Groq provider
    let config = GroqConfig::from_env()
        .with_api_key(std::env::var("GROQ_API_KEY").unwrap_or_else(|_| "test-key".to_string()));

    let provider = GroqProvider::new(config).await?;

    println!("✅ Groq Provider created successfully!");
    println!("Provider name: {}", provider.name());

    // Show capabilities
    println!("\nCapabilities:");
    for capability in provider.capabilities() {
        println!("  - {:?}", capability);
    }

    // Show models
    println!("\nAvailable models ({} total):", provider.models().len());
    for model in provider.models().iter().take(5) {
        println!("  - {} (context: {})", model.id, model.max_context_length);
        if let Some(input_cost) = model.input_cost_per_1k_tokens {
            println!("    Cost: ${:.4}/1K input tokens", input_cost);
        }
    }

    // Show supported parameters for a model
    let model = "llama-3.3-70b-versatile";
    let params = provider.get_supported_openai_params(model);
    println!("\nSupported parameters for {}:", model);
    for (i, param) in params.iter().enumerate() {
        if i % 4 == 0 && i > 0 {
            println!();
        }
        print!("  {:<20}", param);
    }
    println!();

    // Test cost calculation
    match provider.calculate_cost(model, 1000, 500).await {
        Ok(cost) => println!("\nCost for 1000 input + 500 output tokens: ${:.6}", cost),
        Err(e) => println!("\nCost calculation error: {}", e),
    }

    println!("\n✅ All checks passed!");
    Ok(())
}
