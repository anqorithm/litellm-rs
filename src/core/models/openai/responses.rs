//! Response types for OpenAI-compatible API
//!
//! This module defines response structures for chat completions, text completions,
//! embeddings, image generation, and model listings, including streaming variants.

use serde::{Deserialize, Serialize};

use super::audio::AudioDelta;
#[cfg(test)]
use super::messages::MessageContent;
use super::messages::{ChatMessage, MessageRole};
use super::tools::{FunctionCallDelta, ToolCallDelta};
use crate::core::types::thinking::ThinkingUsage;

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Choices
    pub choices: Vec<ChatChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Chat choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,
    /// Message
    pub message: ChatMessage,
    /// Logprobs
    pub logprobs: Option<Logprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Chat completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChoice {
    /// Choice index
    pub index: u32,
    /// Message content
    pub message: ChatMessage,
    /// Finish reason
    pub finish_reason: Option<String>,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
}

/// Logprobs information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Logprobs {
    /// Content logprobs
    pub content: Option<Vec<ContentLogprob>>,
}

/// Content logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentLogprob {
    /// Token
    pub token: String,
    /// Log probability
    pub logprob: f64,
    /// Bytes
    pub bytes: Option<Vec<u8>>,
    /// Top logprobs
    pub top_logprobs: Option<Vec<TopLogprob>>,
}

/// Top logprob
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopLogprob {
    /// Token
    pub token: String,
    /// Log probability
    pub logprob: f64,
    /// Bytes
    pub bytes: Option<Vec<u8>>,
}

/// Usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Usage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Completion tokens
    pub completion_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
    /// Prompt tokens details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_tokens_details: Option<PromptTokensDetails>,
    /// Completion tokens details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<CompletionTokensDetails>,
    /// Thinking/reasoning usage statistics
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_usage: Option<ThinkingUsage>,
}

/// Prompt tokens details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTokensDetails {
    /// Cached tokens
    pub cached_tokens: Option<u32>,
    /// Tokens written into provider-side prompt cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_tokens: Option<u32>,
    /// Tokens read from provider-side prompt cache
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

/// Completion tokens details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionTokensDetails {
    /// Reasoning tokens
    pub reasoning_tokens: Option<u32>,
    /// Audio tokens
    pub audio_tokens: Option<u32>,
}

/// Chat completion chunk (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// System fingerprint
    pub system_fingerprint: Option<String>,
    /// Choices
    pub choices: Vec<ChatChoiceDelta>,
    /// Usage statistics (only in final chunk)
    pub usage: Option<Usage>,
}

/// Chat choice delta (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoiceDelta {
    /// Choice index
    pub index: u32,
    /// Delta message
    pub delta: ChatMessageDelta,
    /// Logprobs
    pub logprobs: Option<Logprobs>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Chat message delta (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessageDelta {
    /// Message role (only in first chunk)
    pub role: Option<MessageRole>,
    /// Content delta
    pub content: Option<String>,
    /// Function call delta (legacy)
    pub function_call: Option<FunctionCallDelta>,
    /// Tool calls delta
    pub tool_calls: Option<Vec<ToolCallDelta>>,
    /// Audio delta
    pub audio: Option<AudioDelta>,
}

/// Text completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionResponse {
    /// Response ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Model used
    pub model: String,
    /// Completion choices
    pub choices: Vec<CompletionChoice>,
    /// Usage statistics
    pub usage: Option<Usage>,
}

/// Completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionChoice {
    /// Generated text
    pub text: String,
    /// Choice index
    pub index: u32,
    /// Log probabilities
    pub logprobs: Option<serde_json::Value>,
    /// Finish reason
    pub finish_reason: Option<String>,
}

/// Embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingResponse {
    /// Object type
    pub object: String,
    /// Embedding data
    pub data: Vec<EmbeddingObject>,
    /// Model used
    pub model: String,
    /// Usage statistics
    pub usage: EmbeddingUsage,
}

/// Embedding object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingObject {
    /// Object type
    pub object: String,
    /// Embedding vector
    pub embedding: Vec<f64>,
    /// Index
    pub index: u32,
}

/// Embedding usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmbeddingUsage {
    /// Prompt tokens
    pub prompt_tokens: u32,
    /// Total tokens
    pub total_tokens: u32,
}

/// Image generation response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageGenerationResponse {
    /// Creation timestamp
    pub created: u64,
    /// Generated images
    pub data: Vec<ImageObject>,
}

/// Image object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageObject {
    /// Image URL
    pub url: Option<String>,
    /// Base64 encoded image
    pub b64_json: Option<String>,
}

/// Model information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    /// Model ID
    pub id: String,
    /// Object type
    pub object: String,
    /// Creation timestamp
    pub created: u64,
    /// Owner
    pub owned_by: String,
}

/// Model list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelListResponse {
    /// Object type
    pub object: String,
    /// List of models
    pub data: Vec<Model>,
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Usage Tests ====================

    #[test]
    fn test_usage_default() {
        let usage = Usage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.completion_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
        assert!(usage.prompt_tokens_details.is_none());
        assert!(usage.completion_tokens_details.is_none());
    }

    #[test]
    fn test_usage_with_values() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        };

        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
        assert_eq!(usage.total_tokens, 150);
    }

    #[test]
    fn test_usage_with_details() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            prompt_tokens_details: Some(PromptTokensDetails {
                cached_tokens: Some(20),
                cache_creation_tokens: None,
                cache_read_tokens: Some(20),
                audio_tokens: None,
            }),
            completion_tokens_details: Some(CompletionTokensDetails {
                reasoning_tokens: Some(10),
                audio_tokens: Some(5),
            }),
            thinking_usage: None,
        };

        assert!(usage.prompt_tokens_details.is_some());
        assert_eq!(
            usage.prompt_tokens_details.as_ref().unwrap().cached_tokens,
            Some(20)
        );
        assert_eq!(
            usage
                .completion_tokens_details
                .as_ref()
                .unwrap()
                .reasoning_tokens,
            Some(10)
        );
    }

    #[test]
    fn test_usage_serialize() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        };

        let json = serde_json::to_string(&usage).unwrap();
        assert!(json.contains("\"prompt_tokens\":100"));
        assert!(json.contains("\"completion_tokens\":50"));
        assert!(json.contains("\"total_tokens\":150"));
        assert!(!json.contains("prompt_tokens_details"));
        assert!(!json.contains("completion_tokens_details"));
        assert!(!json.contains("thinking_usage"));
    }

    #[test]
    fn test_usage_serialize_thinking_usage() {
        let usage = Usage {
            prompt_tokens: 100,
            completion_tokens: 50,
            total_tokens: 150,
            prompt_tokens_details: None,
            completion_tokens_details: Some(CompletionTokensDetails {
                reasoning_tokens: Some(25),
                audio_tokens: None,
            }),
            thinking_usage: Some(ThinkingUsage::new(25).with_budget(1000)),
        };

        let json = serde_json::to_value(&usage).unwrap();
        assert_eq!(json["thinking_usage"]["thinking_tokens"], 25);
        assert_eq!(json["thinking_usage"]["budget_tokens"], 1000);
        assert_eq!(json["completion_tokens_details"]["reasoning_tokens"], 25);
    }

    #[test]
    fn test_usage_deserialize() {
        let json = r#"{"prompt_tokens":100,"completion_tokens":50,"total_tokens":150}"#;
        let usage: Usage = serde_json::from_str(json).unwrap();
        assert_eq!(usage.prompt_tokens, 100);
        assert_eq!(usage.completion_tokens, 50);
    }

    // ==================== PromptTokensDetails Tests ====================

    #[test]
    fn test_prompt_tokens_details_full() {
        let details = PromptTokensDetails {
            cached_tokens: Some(50),
            cache_creation_tokens: Some(5),
            cache_read_tokens: Some(45),
            audio_tokens: Some(10),
        };

        assert_eq!(details.cached_tokens, Some(50));
        assert_eq!(details.cache_creation_tokens, Some(5));
        assert_eq!(details.cache_read_tokens, Some(45));
        assert_eq!(details.audio_tokens, Some(10));
    }

    #[test]
    fn test_prompt_tokens_details_partial() {
        let details = PromptTokensDetails {
            cached_tokens: Some(30),
            cache_creation_tokens: None,
            cache_read_tokens: Some(30),
            audio_tokens: None,
        };

        assert_eq!(details.cached_tokens, Some(30));
        assert!(details.audio_tokens.is_none());
    }

    // ==================== CompletionTokensDetails Tests ====================

    #[test]
    fn test_completion_tokens_details_full() {
        let details = CompletionTokensDetails {
            reasoning_tokens: Some(25),
            audio_tokens: Some(15),
        };

        assert_eq!(details.reasoning_tokens, Some(25));
        assert_eq!(details.audio_tokens, Some(15));
    }

    #[test]
    fn test_completion_tokens_details_partial() {
        let details = CompletionTokensDetails {
            reasoning_tokens: None,
            audio_tokens: Some(20),
        };

        assert!(details.reasoning_tokens.is_none());
        assert_eq!(details.audio_tokens, Some(20));
    }

    // ==================== ChatCompletionResponse Tests ====================

    #[test]
    fn test_chat_completion_response_creation() {
        let response = ChatCompletionResponse {
            id: "chatcmpl-123".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            system_fingerprint: Some("fp_abc123".to_string()),
            choices: vec![],
            usage: Some(Usage::default()),
        };

        assert_eq!(response.id, "chatcmpl-123");
        assert_eq!(response.object, "chat.completion");
        assert_eq!(response.model, "gpt-4");
        assert!(response.system_fingerprint.is_some());
    }

    #[test]
    fn test_chat_completion_response_serialize() {
        let response = ChatCompletionResponse {
            id: "chatcmpl-456".to_string(),
            object: "chat.completion".to_string(),
            created: 1677652288,
            model: "gpt-3.5-turbo".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("chatcmpl-456"));
        assert!(json.contains("chat.completion"));
        assert!(json.contains("gpt-3.5-turbo"));
    }

    // ==================== ChatChoice Tests ====================

    #[test]
    fn test_chat_choice_creation() {
        let choice = ChatChoice {
            index: 0,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text("Hello!".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            logprobs: None,
            finish_reason: Some("stop".to_string()),
        };

        assert_eq!(choice.index, 0);
        assert_eq!(choice.finish_reason, Some("stop".to_string()));
    }

    #[test]
    fn test_chat_choice_serialize() {
        let choice = ChatChoice {
            index: 1,
            message: ChatMessage {
                role: MessageRole::Assistant,
                content: Some(MessageContent::Text("Test response".to_string())),
                name: None,
                function_call: None,
                tool_calls: None,
                tool_call_id: None,
                audio: None,
            },
            logprobs: None,
            finish_reason: Some("length".to_string()),
        };

        let json = serde_json::to_string(&choice).unwrap();
        assert!(json.contains("\"index\":1"));
        assert!(json.contains("length"));
    }

    // ==================== Logprobs Tests ====================

    #[test]
    fn test_logprobs_empty() {
        let logprobs = Logprobs { content: None };
        assert!(logprobs.content.is_none());
    }

    #[test]
    fn test_logprobs_with_content() {
        let logprobs = Logprobs {
            content: Some(vec![ContentLogprob {
                token: "hello".to_string(),
                logprob: -0.5,
                bytes: Some(vec![104, 101, 108, 108, 111]),
                top_logprobs: None,
            }]),
        };

        assert!(logprobs.content.is_some());
        assert_eq!(logprobs.content.as_ref().unwrap().len(), 1);
    }

    // ==================== ContentLogprob Tests ====================

    #[test]
    fn test_content_logprob() {
        let logprob = ContentLogprob {
            token: "world".to_string(),
            logprob: -1.2,
            bytes: Some(vec![119, 111, 114, 108, 100]),
            top_logprobs: Some(vec![TopLogprob {
                token: "world".to_string(),
                logprob: -1.2,
                bytes: None,
            }]),
        };

        assert_eq!(logprob.token, "world");
        assert!((logprob.logprob - (-1.2)).abs() < f64::EPSILON);
        assert!(logprob.top_logprobs.is_some());
    }

    // ==================== TopLogprob Tests ====================

    #[test]
    fn test_top_logprob() {
        let top = TopLogprob {
            token: "test".to_string(),
            logprob: -0.8,
            bytes: Some(vec![116, 101, 115, 116]),
        };

        assert_eq!(top.token, "test");
        assert!((top.logprob - (-0.8)).abs() < f64::EPSILON);
    }

    // ==================== ChatCompletionChunk Tests ====================

    #[test]
    fn test_chat_completion_chunk_creation() {
        let chunk = ChatCompletionChunk {
            id: "chatcmpl-stream-123".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            system_fingerprint: Some("fp_stream".to_string()),
            choices: vec![],
            usage: None,
        };

        assert_eq!(chunk.id, "chatcmpl-stream-123");
        assert_eq!(chunk.object, "chat.completion.chunk");
    }

    #[test]
    fn test_chat_completion_chunk_with_usage() {
        let chunk = ChatCompletionChunk {
            id: "chatcmpl-final".to_string(),
            object: "chat.completion.chunk".to_string(),
            created: 1677652288,
            model: "gpt-4".to_string(),
            system_fingerprint: None,
            choices: vec![],
            usage: Some(Usage {
                prompt_tokens: 10,
                completion_tokens: 20,
                total_tokens: 30,
                prompt_tokens_details: None,
                completion_tokens_details: None,
                thinking_usage: None,
            }),
        };

        assert!(chunk.usage.is_some());
        assert_eq!(chunk.usage.as_ref().unwrap().total_tokens, 30);
    }

    // ==================== ChatChoiceDelta Tests ====================

    #[test]
    fn test_chat_choice_delta() {
        let delta = ChatChoiceDelta {
            index: 0,
            delta: ChatMessageDelta {
                role: Some(MessageRole::Assistant),
                content: Some("Hello".to_string()),
                function_call: None,
                tool_calls: None,
                audio: None,
            },
            logprobs: None,
            finish_reason: None,
        };

        assert_eq!(delta.index, 0);
        assert!(delta.delta.role.is_some());
        assert!(delta.finish_reason.is_none());
    }

    #[test]
    fn test_chat_choice_delta_final() {
        let delta = ChatChoiceDelta {
            index: 0,
            delta: ChatMessageDelta {
                role: None,
                content: None,
                function_call: None,
                tool_calls: None,
                audio: None,
            },
            logprobs: None,
            finish_reason: Some("stop".to_string()),
        };

        assert_eq!(delta.finish_reason, Some("stop".to_string()));
    }

    // ==================== ChatMessageDelta Tests ====================

    #[test]
    fn test_chat_message_delta_first_chunk() {
        let delta = ChatMessageDelta {
            role: Some(MessageRole::Assistant),
            content: None,
            function_call: None,
            tool_calls: None,
            audio: None,
        };

        assert!(delta.role.is_some());
        assert!(delta.content.is_none());
    }

    #[test]
    fn test_chat_message_delta_content_chunk() {
        let delta = ChatMessageDelta {
            role: None,
            content: Some("partial content".to_string()),
            function_call: None,
            tool_calls: None,
            audio: None,
        };

        assert!(delta.role.is_none());
        assert_eq!(delta.content, Some("partial content".to_string()));
    }

    // ==================== CompletionResponse Tests ====================

    #[test]
    fn test_completion_response_creation() {
        let response = CompletionResponse {
            id: "cmpl-123".to_string(),
            object: "text_completion".to_string(),
            created: 1677652288,
            model: "text-davinci-003".to_string(),
            choices: vec![],
            usage: Some(Usage::default()),
        };

        assert_eq!(response.id, "cmpl-123");
        assert_eq!(response.object, "text_completion");
    }

    #[test]
    fn test_completion_response_serialize() {
        let response = CompletionResponse {
            id: "cmpl-456".to_string(),
            object: "text_completion".to_string(),
            created: 1677652288,
            model: "gpt-3.5-turbo-instruct".to_string(),
            choices: vec![CompletionChoice {
                text: "Generated text".to_string(),
                index: 0,
                logprobs: None,
                finish_reason: Some("stop".to_string()),
            }],
            usage: None,
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Generated text"));
        assert!(json.contains("text_completion"));
    }

    // ==================== CompletionChoice Tests ====================

    #[test]
    fn test_completion_choice() {
        let choice = CompletionChoice {
            text: "Hello, world!".to_string(),
            index: 0,
            logprobs: None,
            finish_reason: Some("stop".to_string()),
        };

        assert_eq!(choice.text, "Hello, world!");
        assert_eq!(choice.index, 0);
        assert_eq!(choice.finish_reason, Some("stop".to_string()));
    }

    // ==================== EmbeddingResponse Tests ====================

    #[test]
    fn test_embedding_response_creation() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![EmbeddingObject {
                object: "embedding".to_string(),
                embedding: vec![0.1, 0.2, 0.3],
                index: 0,
            }],
            model: "text-embedding-ada-002".to_string(),
            usage: EmbeddingUsage::default(),
        };

        assert_eq!(response.object, "list");
        assert_eq!(response.data.len(), 1);
        assert_eq!(response.data[0].embedding.len(), 3);
    }

    #[test]
    fn test_embedding_response_serialize() {
        let response = EmbeddingResponse {
            object: "list".to_string(),
            data: vec![],
            model: "text-embedding-3-small".to_string(),
            usage: EmbeddingUsage {
                prompt_tokens: 10,
                total_tokens: 10,
            },
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("text-embedding-3-small"));
    }

    // ==================== EmbeddingObject Tests ====================

    #[test]
    fn test_embedding_object() {
        let obj = EmbeddingObject {
            object: "embedding".to_string(),
            embedding: vec![0.5, -0.3, 0.8, 0.1],
            index: 2,
        };

        assert_eq!(obj.object, "embedding");
        assert_eq!(obj.embedding.len(), 4);
        assert_eq!(obj.index, 2);
    }

    // ==================== EmbeddingUsage Tests ====================

    #[test]
    fn test_embedding_usage_default() {
        let usage = EmbeddingUsage::default();
        assert_eq!(usage.prompt_tokens, 0);
        assert_eq!(usage.total_tokens, 0);
    }

    #[test]
    fn test_embedding_usage_with_values() {
        let usage = EmbeddingUsage {
            prompt_tokens: 50,
            total_tokens: 50,
        };

        assert_eq!(usage.prompt_tokens, 50);
        assert_eq!(usage.total_tokens, 50);
    }

    // ==================== ImageGenerationResponse Tests ====================

    #[test]
    fn test_image_generation_response() {
        let response = ImageGenerationResponse {
            created: 1677652288,
            data: vec![ImageObject {
                url: Some("https://example.com/image.png".to_string()),
                b64_json: None,
            }],
        };

        assert_eq!(response.created, 1677652288);
        assert_eq!(response.data.len(), 1);
        assert!(response.data[0].url.is_some());
    }

    #[test]
    fn test_image_generation_response_b64() {
        let response = ImageGenerationResponse {
            created: 1677652288,
            data: vec![ImageObject {
                url: None,
                b64_json: Some("base64encodeddata".to_string()),
            }],
        };

        assert!(response.data[0].url.is_none());
        assert!(response.data[0].b64_json.is_some());
    }

    // ==================== ImageObject Tests ====================

    #[test]
    fn test_image_object_url() {
        let obj = ImageObject {
            url: Some("https://cdn.example.com/img.jpg".to_string()),
            b64_json: None,
        };

        assert_eq!(obj.url, Some("https://cdn.example.com/img.jpg".to_string()));
        assert!(obj.b64_json.is_none());
    }

    #[test]
    fn test_image_object_b64() {
        let obj = ImageObject {
            url: None,
            b64_json: Some("SGVsbG8gV29ybGQ=".to_string()),
        };

        assert!(obj.url.is_none());
        assert!(obj.b64_json.is_some());
    }

    // ==================== Model Tests ====================

    #[test]
    fn test_model_creation() {
        let model = Model {
            id: "gpt-4".to_string(),
            object: "model".to_string(),
            created: 1687882411,
            owned_by: "openai".to_string(),
        };

        assert_eq!(model.id, "gpt-4");
        assert_eq!(model.object, "model");
        assert_eq!(model.owned_by, "openai");
    }

    #[test]
    fn test_model_serialize() {
        let model = Model {
            id: "claude-3-opus".to_string(),
            object: "model".to_string(),
            created: 1709596800,
            owned_by: "anthropic".to_string(),
        };

        let json = serde_json::to_string(&model).unwrap();
        assert!(json.contains("claude-3-opus"));
        assert!(json.contains("anthropic"));
    }

    // ==================== ModelListResponse Tests ====================

    #[test]
    fn test_model_list_response() {
        let response = ModelListResponse {
            object: "list".to_string(),
            data: vec![
                Model {
                    id: "gpt-4".to_string(),
                    object: "model".to_string(),
                    created: 1687882411,
                    owned_by: "openai".to_string(),
                },
                Model {
                    id: "gpt-3.5-turbo".to_string(),
                    object: "model".to_string(),
                    created: 1677610602,
                    owned_by: "openai".to_string(),
                },
            ],
        };

        assert_eq!(response.object, "list");
        assert_eq!(response.data.len(), 2);
    }

    #[test]
    fn test_model_list_response_empty() {
        let response = ModelListResponse {
            object: "list".to_string(),
            data: vec![],
        };

        assert!(response.data.is_empty());
    }

    #[test]
    fn test_model_list_response_serialize() {
        let response = ModelListResponse {
            object: "list".to_string(),
            data: vec![Model {
                id: "test-model".to_string(),
                object: "model".to_string(),
                created: 1700000000,
                owned_by: "test-org".to_string(),
            }],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("test-model"));
        assert!(json.contains("\"object\":\"list\""));
    }
}
