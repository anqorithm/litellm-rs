//! OpenAI Provider Additional API Methods
//!
//! Additional API endpoints beyond core chat completion:
//! - Embeddings
//! - Image generation, editing, and variations
//! - Audio transcription
//! - Text completion (legacy)
//! - Fine-tuning
//! - Vector stores
//! - Realtime sessions
//! - Advanced chat features

use serde_json::Value;

use crate::core::providers::base::HttpMethod;
use crate::core::types::embedding::EmbeddingRequest;
use crate::core::types::responses::EmbeddingResponse;

use super::advanced_chat::{AdvancedChatRequest, AdvancedChatUtils};
use super::client::OpenAIProvider;
use super::completions::validate_completion_request;
use super::config::OpenAIFeature;
use super::error::OpenAIError;
use super::fine_tuning::{OpenAIFineTuningRequest, OpenAIFineTuningUtils};
use super::image_edit::{OpenAIImageEditRequest, OpenAIImageEditUtils};
use super::image_variations::{OpenAIImageVariationsRequest, OpenAIImageVariationsUtils};
use super::realtime::{OpenAIRealtimeUtils, RealtimeSessionConfig};
use super::vector_stores::{OpenAIVectorStoreRequest, OpenAIVectorStoreUtils};

/// Additional OpenAI-specific API methods
impl OpenAIProvider {
    /// Generate embeddings
    pub async fn embeddings(
        &self,
        request: EmbeddingRequest,
    ) -> Result<EmbeddingResponse, OpenAIError> {
        // Like Python LiteLLM, we don't validate models locally
        // OpenAI API will handle invalid models

        // Transform to OpenAI format
        let openai_request = serde_json::json!({
            "input": request.input,
            "model": request.model,
            "encoding_format": request.encoding_format,
            "dimensions": request.dimensions,
            "user": request.user
        });

        // Execute request using high-performance connection pool
        let url = format!("{}/embeddings", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(openai_request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_json: Value =
            serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
                provider: "openai",
                message: e.to_string(),
            })?;

        // Transform response
        serde_json::from_value(response_json).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Generate images (DALL-E)
    pub async fn generate_images(
        &self,
        prompt: String,
        model: Option<String>,
        n: Option<u32>,
        size: Option<String>,
        quality: Option<String>,
        style: Option<String>,
    ) -> Result<Value, OpenAIError> {
        let model = model.unwrap_or_else(|| "dall-e-3".to_string());

        // Validate image generation capability
        if !self
            .config
            .is_feature_enabled(OpenAIFeature::ImageGeneration)
        {
            return Err(OpenAIError::NotSupported {
                provider: "openai",
                feature: "Image generation is disabled in configuration".to_string(),
            });
        }

        let request = serde_json::json!({
            "prompt": prompt,
            "model": model,
            "n": n,
            "size": size,
            "quality": quality,
            "style": style
        });

        let url = format!("{}/images/generations", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Audio transcription (Whisper)
    pub async fn transcribe_audio(
        &self,
        _file: Vec<u8>,
        model: Option<String>,
        language: Option<String>,
        response_format: Option<String>,
    ) -> Result<Value, OpenAIError> {
        if !self
            .config
            .is_feature_enabled(OpenAIFeature::AudioTranscription)
        {
            return Err(OpenAIError::NotSupported {
                provider: "openai",
                feature: "Audio transcription is disabled in configuration".to_string(),
            });
        }

        // This would need multipart form handling - simplified for now
        let request = serde_json::json!({
            "model": model.unwrap_or_else(|| "whisper-1".to_string()),
            "language": language,
            "response_format": response_format
        });

        // In a real implementation, this would handle file upload
        let url = format!("{}/audio/transcriptions", self.config.get_api_base());

        let headers = self.get_request_headers();
        let body = Some(request);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    // ==================== NEW FUNCTIONALITY METHODS ====================

    /// Text completion (legacy)
    pub async fn text_completion(
        &self,
        request: super::completions::OpenAICompletionRequest,
    ) -> Result<super::completions::OpenAICompletionResponse, OpenAIError> {
        // Validate request
        validate_completion_request(&request).map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Execute request
        let url = format!("{}/completions", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create fine-tuning job
    pub async fn create_fine_tuning_job(
        &self,
        request: OpenAIFineTuningRequest,
    ) -> Result<super::fine_tuning::OpenAIFineTuningJob, OpenAIError> {
        // Validate request
        OpenAIFineTuningUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request
        let url = format!("{}/fine_tuning/jobs", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// List fine-tuning jobs
    pub async fn list_fine_tuning_jobs(
        &self,
        after: Option<String>,
        limit: Option<u32>,
    ) -> Result<Value, OpenAIError> {
        let mut query_params = Vec::new();
        if let Some(after) = after {
            query_params.push(format!("after={}", after));
        }
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }

        let endpoint = if query_params.is_empty() {
            "fine_tuning/jobs".to_string()
        } else {
            format!("fine_tuning/jobs?{}", query_params.join("&"))
        };

        let url = format!("{}/{}", self.config.get_api_base(), endpoint);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(api_key) = &self.config.base.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Edit image
    pub async fn edit_image(
        &self,
        request: OpenAIImageEditRequest,
    ) -> Result<super::image_edit::OpenAIImageEditResponse, OpenAIError> {
        // Validate request
        OpenAIImageEditUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request (would need multipart form data in real implementation)
        let url = format!("{}/images/edits", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create image variations
    pub async fn create_image_variations(
        &self,
        request: OpenAIImageVariationsRequest,
    ) -> Result<super::image_variations::OpenAIImageVariationsResponse, OpenAIError> {
        // Validate request
        OpenAIImageVariationsUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request (would need multipart form data in real implementation)
        let url = format!("{}/images/variations", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create vector store
    pub async fn create_vector_store(
        &self,
        request: OpenAIVectorStoreRequest,
    ) -> Result<super::vector_stores::OpenAIVectorStore, OpenAIError> {
        // Validate request
        OpenAIVectorStoreUtils::validate_request(&request).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Execute request
        let url = format!("{}/vector_stores", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// List vector stores
    pub async fn list_vector_stores(
        &self,
        limit: Option<u32>,
        order: Option<String>,
        after: Option<String>,
        before: Option<String>,
    ) -> Result<Value, OpenAIError> {
        let mut query_params = Vec::new();
        if let Some(limit) = limit {
            query_params.push(format!("limit={}", limit));
        }
        if let Some(order) = order {
            query_params.push(format!("order={}", order));
        }
        if let Some(after) = after {
            query_params.push(format!("after={}", after));
        }
        if let Some(before) = before {
            query_params.push(format!("before={}", before));
        }

        let endpoint = if query_params.is_empty() {
            "vector_stores".to_string()
        } else {
            format!("vector_stores?{}", query_params.join("&"))
        };

        let url = format!("{}/{}", self.config.get_api_base(), endpoint);
        let client = reqwest::Client::new();
        let mut req = client.get(&url);

        if let Some(api_key) = &self.config.base.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Create real-time session
    pub async fn create_realtime_session(
        &self,
        config: RealtimeSessionConfig,
    ) -> Result<Value, OpenAIError> {
        // Validate configuration
        OpenAIRealtimeUtils::validate_session_config(&config).map_err(|e| {
            OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            }
        })?;

        // Real-time API uses WebSocket, this is a simplified version
        // In practice, this would establish a WebSocket connection
        Ok(serde_json::json!({
            "session_id": "session_123",
            "status": "connected",
            "config": config
        }))
    }

    /// Advanced chat completion with structured outputs and reasoning
    pub async fn advanced_chat_completion(
        &self,
        request: AdvancedChatRequest,
    ) -> Result<super::advanced_chat::AdvancedChatResponse, OpenAIError> {
        // Validate advanced request
        AdvancedChatUtils::validate_request(&request).map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })?;

        // Execute request
        let url = format!("{}/chat/completions", self.config.get_api_base());
        let request_value =
            serde_json::to_value(request).map_err(|e| OpenAIError::InvalidRequest {
                provider: "openai",
                message: e.to_string(),
            })?;

        let headers = self.get_request_headers();
        let body = Some(request_value);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, body)
            .await
            .map_err(|e| OpenAIError::Network {
                provider: "openai",
                message: e.to_string(),
            })?;

        let response_bytes = response.bytes().await.map_err(|e| OpenAIError::Network {
            provider: "openai",
            message: e.to_string(),
        })?;

        serde_json::from_slice(&response_bytes).map_err(|e| OpenAIError::ResponseParsing {
            provider: "openai",
            message: e.to_string(),
        })
    }

    /// Get model capabilities for advanced features
    pub fn get_advanced_model_capabilities(
        &self,
        model: &str,
    ) -> super::advanced_chat::ModelCapabilities {
        AdvancedChatUtils::get_model_capabilities(model)
    }

    /// Estimate cost for advanced features
    pub fn estimate_advanced_cost(
        &self,
        model: &str,
        input_tokens: u32,
        output_tokens: u32,
        reasoning_tokens: Option<u32>,
    ) -> Result<f64, OpenAIError> {
        AdvancedChatUtils::estimate_advanced_cost(
            model,
            input_tokens,
            output_tokens,
            reasoning_tokens,
        )
        .map_err(|e| OpenAIError::InvalidRequest {
            provider: "openai",
            message: e.to_string(),
        })
    }
}
