//! Azure AI Chat Completion Example
//!
//! This example demonstrates how to use the Azure AI provider for chat completions.
//!
//! Before running this example, set the following environment variables:
//! - AZURE_AI_API_KEY: Your Azure AI API key
//! - AZURE_AI_API_BASE: Your Azure AI endpoint (e.g., https://your-resource.cognitiveservices.azure.com)
//!
//! Run with:
//! ```bash
//! AZURE_AI_API_KEY=your_key AZURE_AI_API_BASE=your_endpoint cargo run --example azure_ai_chat
//! ```

use litellm_rs::core::providers::azure_ai::{AzureAIConfig, AzureAIProvider};
use litellm_rs::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use litellm_rs::core::types::RequestContext;
use litellm_rs::core::types::{ChatMessage, ChatRequest, MessageContent, MessageRole};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Azure AI Chat Completion Example");
    println!("====================================\n");

    // Create Azure AI configuration from environment variables
    let config = AzureAIConfig::from_env();

    // Check if API key and base URL are set
    if config.base.api_key.is_none() {
        eprintln!("❌ Error: AZURE_AI_API_KEY environment variable is not set");
        eprintln!("Please set it with: export AZURE_AI_API_KEY=your_api_key");
        return Ok(());
    }

    if config.base.api_base.is_none() {
        eprintln!("❌ Error: AZURE_AI_API_BASE environment variable is not set");
        eprintln!(
            "Please set it with: export AZURE_AI_API_BASE=https://your-resource.cognitiveservices.azure.com"
        );
        return Ok(());
    }

    println!("✅ Configuration loaded successfully");
    println!("   API Base: {}", config.base.api_base.as_ref().unwrap());
    println!();

    // Create the Azure AI provider
    let provider = match AzureAIProvider::new(config) {
        Ok(p) => {
            println!("✅ Azure AI Provider created successfully\n");
            p
        }
        Err(e) => {
            eprintln!("❌ Failed to create provider: {}", e);
            return Ok(());
        }
    };

    // Example 1: Simple chat completion
    println!("📝 Example 1: Simple Chat Completion");
    println!("-------------------------------------");

    let request = ChatRequest {
        model: "gpt-4o".to_string(), // Or use your deployed model name
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a helpful assistant that speaks concisely.".to_string(),
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "What is Rust programming language in one sentence?".to_string(),
                )),
                ..Default::default()
            },
        ],
        temperature: Some(0.7),
        max_tokens: Some(100),
        stream: false,
        ..Default::default()
    };

    println!("🔄 Sending request to Azure AI...\n");

    let context = RequestContext::default();

    match provider.chat_completion(request.clone(), context).await {
        Ok(response) => {
            println!("✅ Response received!");
            println!("   Model: {}", response.model);
            println!("   Response ID: {}", response.id);

            if let Some(content) = response.first_content() {
                println!("\n💬 Assistant: {}", content);
            }

            if let Some(usage) = &response.usage {
                println!("\n📊 Token Usage:");
                println!("   Prompt tokens: {}", usage.prompt_tokens);
                println!("   Completion tokens: {}", usage.completion_tokens);
                println!("   Total tokens: {}", usage.total_tokens);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }

    println!("\n");

    // Example 2: Streaming chat completion
    println!("📝 Example 2: Streaming Chat Completion");
    println!("---------------------------------------");

    let mut streaming_request = request.clone();
    streaming_request.stream = true;
    streaming_request.messages = vec![ChatMessage {
        role: MessageRole::User,
        content: Some(MessageContent::Text(
            "Count from 1 to 5 slowly.".to_string(),
        )),
        ..Default::default()
    }];

    println!("🔄 Starting streaming request...\n");

    let context = RequestContext::default();

    match provider
        .chat_completion_stream(streaming_request, context)
        .await
    {
        Ok(mut stream) => {
            use futures::StreamExt;

            print!("💬 Assistant: ");
            while let Some(chunk_result) = stream.next().await {
                match chunk_result {
                    Ok(chunk) => {
                        // Print each chunk as it arrives
                        for choice in &chunk.choices {
                            if let Some(content) = &choice.delta.content {
                                print!("{}", content);
                                // Flush to ensure immediate output
                                use std::io::{self, Write};
                                io::stdout().flush().unwrap();
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("\n❌ Stream error: {}", e);
                        break;
                    }
                }
            }
            println!("\n\n✅ Streaming completed!");
        }
        Err(e) => {
            eprintln!("❌ Failed to start streaming: {}", e);
        }
    }

    println!("\n");

    // Example 3: Multi-turn conversation
    println!("📝 Example 3: Multi-turn Conversation");
    println!("-------------------------------------");

    let conversation_request = ChatRequest {
        model: "gpt-4o".to_string(),
        messages: vec![
            ChatMessage {
                role: MessageRole::System,
                content: Some(MessageContent::Text(
                    "You are a Rust programming expert.".to_string()
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "What is ownership in Rust?".to_string()
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text(
                    "Ownership is Rust's memory management system where each value has a single owner, and when the owner goes out of scope, the value is dropped.".to_string()
                )),
                ..Default::default()
            },
            ChatMessage {
                role: MessageRole::User,
                content: Some(MessageContent::Text(
                    "Can you give me a simple code example?".to_string()
                )),
                ..Default::default()
            },
        ],
        temperature: Some(0.5),
        max_tokens: Some(200),
        stream: false,
        ..Default::default()
    };

    println!("🔄 Continuing conversation...\n");

    let context = RequestContext::default();

    match provider
        .chat_completion(conversation_request, context)
        .await
    {
        Ok(response) => {
            if let Some(content) = response.first_content() {
                println!("💬 Assistant: {}", content);
            }
        }
        Err(e) => {
            eprintln!("❌ Error: {}", e);
        }
    }

    println!("\n✨ All examples completed!");

    Ok(())
}
