//! Example of using xAI Provider

use litellm_rs::core::providers::xai::{XAIConfig, XAIProvider};
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use litellm_rs::core::types::{
    chat::ChatMessage, chat::ChatRequest, context::RequestContext, message::MessageContent,
    message::MessageRole,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Create xAI configuration
    let config = XAIConfig {
        api_key: std::env::var("XAI_API_KEY").ok(),
        enable_web_search: true, // Enable web search capability
        ..Default::default()
    };

    // Create provider
    let provider = match XAIProvider::new(config).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to create xAI provider: {}", e);
            eprintln!("Make sure XAI_API_KEY environment variable is set");
            return Err(e.into());
        }
    };

    println!("xAI Provider initialized successfully!");
    println!("Available models:");
    for model in provider.models() {
        println!(
            "  - {} ({}): {} tokens context",
            model.id, model.name, model.max_context_length
        );
    }

    // Test chat completion
    println!("\n=== Testing Chat Completion with Grok-2-Mini ===");

    let request = ChatRequest {
        model: "grok-2-mini".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant that can access web information when needed."
                        .to_string(),
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "What is xAI and what are Grok models? Keep it brief.".to_string(),
                )),
                ..Default::default()
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(150),
        stream: false,
        ..Default::default()
    };

    let context = RequestContext::default();

    match provider.chat_completion(request.clone(), context).await {
        Ok(response) => {
            println!("\nResponse:");
            if let Some(choice) = response.choices.first() {
                if let Some(ref content) = choice.message.content {
                    println!("{}", content);
                }
            }
            if let Some(usage) = response.usage {
                println!("\nUsage:");
                println!("  Input tokens: {}", usage.prompt_tokens);
                println!("  Output tokens: {}", usage.completion_tokens);
                println!("  Total tokens: {}", usage.total_tokens);

                // Check for reasoning tokens
                if let Some(ref details) = usage.completion_tokens_details {
                    if let Some(reasoning) = details.reasoning_tokens {
                        println!("  Reasoning tokens: {}", reasoning);
                    }
                }
            }
        }
        Err(e) => {
            println!("Chat completion failed: {}", e);
        }
    }

    // Test with reasoning model (if API key supports it)
    println!("\n=== Testing Reasoning with Grok-2 ===");

    let reasoning_request = ChatRequest {
        model: "grok-2".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "Solve this step by step: If a train travels 120 miles in 2 hours, and then 180 miles in 3 hours, what is its average speed for the entire journey?".to_string()
                )),
                ..Default::default()
            },
        ],
        temperature: Some(0.3),
        max_tokens: Some(300),
        stream: false,
        ..Default::default()
    };

    match provider
        .chat_completion(reasoning_request, RequestContext::default())
        .await
    {
        Ok(response) => {
            println!("\nReasoning Response:");
            if let Some(choice) = response.choices.first() {
                if let Some(ref content) = choice.message.content {
                    println!("{}", content);
                }
            }
            if let Some(usage) = response.usage {
                println!("\nUsage with reasoning:");
                println!("  Total tokens: {}", usage.total_tokens);
                if let Some(ref details) = usage.completion_tokens_details {
                    if let Some(reasoning) = details.reasoning_tokens {
                        println!("  Reasoning tokens: {}", reasoning);
                    }
                }
            }
        }
        Err(e) => {
            println!("Reasoning request failed: {}", e);
        }
    }

    // Test model info and cost calculation
    println!("\n=== Testing Model Info and Cost Calculation ===");

    match provider.calculate_cost("grok-2", 1000, 500).await {
        Ok(cost) => println!(
            "Cost for 1000 input + 500 output tokens on grok-2: ${:.4}",
            cost
        ),
        Err(e) => println!("Cost calculation failed: {}", e),
    };

    match provider.calculate_cost("grok-2-mini", 1000, 500).await {
        Ok(cost) => println!(
            "Cost for 1000 input + 500 output tokens on grok-2-mini: ${:.4}",
            cost
        ),
        Err(e) => println!("Cost calculation failed: {}", e),
    };

    Ok(())
}
