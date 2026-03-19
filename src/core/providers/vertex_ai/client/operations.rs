//! Vertex AI Provider operations
//!
//! Contains chat completion, embedding, and token counting implementations.

use serde_json::Value;

use crate::ProviderError;
use crate::core::types::{
    chat::ChatRequest,
    context::RequestContext,
    embedding::EmbeddingRequest,
    responses::{ChatResponse, EmbeddingResponse},
};

use super::{super::error::VertexAIError, VertexAIProvider};

impl VertexAIProvider {
    /// Execute chat completion
    pub async fn chat_completion_internal(
        &self,
        request: ChatRequest,
        _context: RequestContext,
    ) -> Result<ChatResponse, VertexAIError> {
        let model = super::super::parse_vertex_model(&request.model);

        // Transform request based on model type
        let (endpoint, body) = if model.is_gemini() {
            let endpoint = if request.stream {
                "streamGenerateContent"
            } else {
                "generateContent"
            };

            let body = self
                .gemini_transformer
                .transform_chat_request(&request, &model)?;
            (endpoint, body)
        } else if model.is_partner_model() {
            // Partner models use different endpoints
            let endpoint = "predict";
            let body = self
                .partner_transformer
                .transform_chat_request(&request, &model)?;
            (endpoint, body)
        } else {
            return Err(ProviderError::model_not_found("vertex_ai", &request.model));
        };

        let url = self.build_url(&model, endpoint, request.stream);
        let response = self.make_request(&url, body).await?;

        // Parse response
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("vertex_ai", e.to_string()))?;

        // Transform response back to standard format
        if model.is_gemini() {
            self.gemini_transformer
                .transform_chat_response(response_body, &model)
        } else {
            self.partner_transformer
                .transform_chat_response(response_body, &model)
        }
    }

    /// Execute embedding request
    pub async fn embedding_internal(
        &self,
        request: EmbeddingRequest,
        _context: RequestContext,
    ) -> Result<EmbeddingResponse, VertexAIError> {
        // Vertex AI uses specific embedding models
        let model_name = if request.model.contains("embedding") {
            request.model.clone()
        } else {
            "text-embedding-004".to_string() // Default embedding model
        };

        let endpoint = "predict";
        let url = format!(
            "https://{}-aiplatform.googleapis.com/v1/projects/{}/locations/{}/publishers/google/models/{}:{}",
            self.config.location,
            self.config.project_id,
            self.config.location,
            model_name,
            endpoint
        );

        // Build request body
        let instances: Vec<Value> = request
            .input
            .iter()
            .map(|text| {
                serde_json::json!({
                    "content": text,
                    "task_type": "RETRIEVAL_DOCUMENT"
                })
            })
            .collect();

        let body = serde_json::json!({
            "instances": instances
        });

        let response = self.make_request(&url, body).await?;
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("vertex_ai", e.to_string()))?;

        // Parse embeddings from response
        let predictions = response_body["predictions"]
            .as_array()
            .ok_or_else(|| ProviderError::response_parsing("vertex_ai", "Missing predictions"))?;

        let embeddings = predictions
            .iter()
            .enumerate()
            .map(|(index, pred)| {
                let values = pred["embeddings"]["values"]
                    .as_array()
                    .ok_or_else(|| {
                        ProviderError::response_parsing("vertex_ai", "Missing embedding values")
                    })?
                    .iter()
                    .map(|v| v.as_f64().unwrap_or(0.0) as f32)
                    .collect();

                Ok(crate::core::types::responses::EmbeddingData {
                    object: "embedding".to_string(),
                    index: index as u32,
                    embedding: values,
                })
            })
            .collect::<Result<Vec<crate::core::types::responses::EmbeddingData>, VertexAIError>>(
            )?;

        Ok(EmbeddingResponse {
            object: "list".to_string(),
            data: embeddings.clone(),
            model: model_name,
            usage: None, // Vertex AI doesn't return token usage for embeddings
            embeddings: Some(embeddings), // Backward compatibility field
        })
    }

    /// Count tokens for a request
    pub async fn count_tokens(
        &self,
        model: &str,
        messages: &[Value],
    ) -> Result<usize, VertexAIError> {
        let model_obj = super::super::parse_vertex_model(model);
        let endpoint = "countTokens";
        let url = self.build_url(&model_obj, endpoint, false);

        let body = serde_json::json!({
            "contents": messages
        });

        let response = self.make_request(&url, body).await?;
        let response_body: Value = response
            .json()
            .await
            .map_err(|e| ProviderError::response_parsing("vertex_ai", e.to_string()))?;

        response_body["totalTokens"]
            .as_u64()
            .map(|v| v as usize)
            .ok_or_else(|| ProviderError::response_parsing("vertex_ai", "Missing token count"))
    }
}
