//! Test Groq streaming functionality

use futures::StreamExt;
use litellm_rs::core::providers::groq::streaming::create_fake_stream;
use litellm_rs::core::types::responses::{ChatChoice, ChatResponse};
use litellm_rs::core::types::{ChatMessage, message::MessageContent, message::MessageRole};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a mock response for testing fake streaming
    let response = ChatResponse {
        id: "test-123".to_string(),
        object: "chat.completion".to_string(),
        created: 1234567890,
        model: "llama-3.3-70b-versatile".to_string(),
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text(
                    "Hello! This is a test response from Groq's ultra-fast LPU inference engine."
                        .to_string(),
                )),
                ..Default::default()
            },
            finish_reason: None,
            logprobs: None,
        }],
        usage: None,
        system_fingerprint: None,
    };

    println!("Testing Groq fake streaming...\n");

    // Create fake stream
    let mut stream = create_fake_stream(response).await?;

    let mut chunk_count = 0;
    let mut full_text = String::new();

    // Process stream chunks
    while let Some(chunk_result) = stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                chunk_count += 1;
                if let Some(choice) = chunk.choices.first() {
                    if let Some(content) = &choice.delta.content {
                        print!("{}", content);
                        full_text.push_str(content);
                    }
                    if let Some(role) = &choice.delta.role {
                        println!("[Role: {:?}]", role);
                    }
                    if let Some(finish) = &choice.finish_reason {
                        println!("\n[Finished: {:?}]", finish);
                    }
                }
            }
            Err(e) => {
                eprintln!("Stream error: {}", e);
                break;
            }
        }
    }

    println!("\n\n✅ Streaming test complete!");
    println!("  - Chunks received: {}", chunk_count);
    println!("  - Total characters: {}", full_text.len());

    Ok(())
}
