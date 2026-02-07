//! Cohere Embeddings Handler
//!
//! Handles embedding requests for Cohere embed models.
//! Supports text and image embeddings with various input types.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::config::CohereConfig;
use super::error::CohereError;
use crate::core::types::{EmbeddingInput, EmbeddingRequest};
use crate::core::types::responses::{EmbeddingData, EmbeddingResponse, Usage};

/// Text and image inputs for Cohere embeddings
type ExtractedInputs = (Option<Vec<String>>, Option<Vec<String>>);

/// Cohere embedding input types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum CohereEmbeddingInputType {
    /// For embedding documents to be stored in a vector database
    #[default]
    SearchDocument,
    /// For embedding search queries
    SearchQuery,
    /// For classification tasks
    Classification,
    /// For clustering tasks
    Clustering,
    /// For image embeddings
    Image,
}

impl std::fmt::Display for CohereEmbeddingInputType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SearchDocument => write!(f, "search_document"),
            Self::SearchQuery => write!(f, "search_query"),
            Self::Classification => write!(f, "classification"),
            Self::Clustering => write!(f, "clustering"),
            Self::Image => write!(f, "image"),
        }
    }
}

/// Cohere embedding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddingRequest {
    /// Model to use
    pub model: String,

    /// Input texts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub texts: Option<Vec<String>>,

    /// Input images (base64 encoded)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub images: Option<Vec<String>>,

    /// Input type
    pub input_type: String,

    /// Embedding types to return (float, int8, uint8, binary, ubinary)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_types: Option<Vec<String>>,

    /// Truncate input if too long
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncate: Option<String>,

    /// Output dimension (for dimensionality reduction)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimension: Option<u32>,
}

/// Cohere embedding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddingResponse {
    /// Response ID
    pub id: String,

    /// Embeddings organized by type
    pub embeddings: CohereEmbeddings,

    /// Input texts
    #[serde(default)]
    pub texts: Vec<String>,

    /// Metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<CohereEmbeddingMeta>,
}

/// Cohere embeddings by type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddings {
    /// Float embeddings
    #[serde(default)]
    pub float: Option<Vec<Vec<f32>>>,

    /// Int8 embeddings
    #[serde(default)]
    pub int8: Option<Vec<Vec<i8>>>,

    /// Uint8 embeddings
    #[serde(default)]
    pub uint8: Option<Vec<Vec<u8>>>,

    /// Binary embeddings
    #[serde(default)]
    pub binary: Option<Vec<Vec<i8>>>,

    /// Unsigned binary embeddings
    #[serde(default)]
    pub ubinary: Option<Vec<Vec<u8>>>,
}

/// Cohere embedding metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereEmbeddingMeta {
    /// API version
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_version: Option<Value>,

    /// Billed units
    #[serde(skip_serializing_if = "Option::is_none")]
    pub billed_units: Option<CohereBilledUnits>,

    /// Warnings
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// Cohere billed units
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CohereBilledUnits {
    /// Input tokens
    #[serde(default)]
    pub input_tokens: u32,

    /// Images
    #[serde(default)]
    pub images: Option<u32>,
}

/// Embedding handler utilities
pub struct CohereEmbeddingHandler;

impl CohereEmbeddingHandler {
    /// Transform EmbeddingRequest to Cohere format
    pub fn transform_request(
        request: &EmbeddingRequest,
        config: &CohereConfig,
    ) -> Result<Value, CohereError> {
        let (texts, images) = Self::extract_inputs(&request.input)?;
        let is_image_request = images.is_some();

        let input_type = if is_image_request {
            "image".to_string()
        } else {
            request
                .task_type
                .clone()
                .unwrap_or_else(|| config.default_embedding_input_type.clone())
        };

        let mut body = json!({
            "model": request.model,
            "input_type": input_type,
        });

        if let Some(texts) = texts {
            body["texts"] = json!(texts);
        }

        if let Some(images) = images {
            body["images"] = json!(images);
        }

        // Map encoding_format to embedding_types
        if let Some(encoding_format) = &request.encoding_format {
            body["embedding_types"] = json!([encoding_format]);
        }

        // Map dimensions to output_dimension
        if let Some(dimensions) = request.dimensions {
            body["output_dimension"] = json!(dimensions);
        }

        Ok(body)
    }

    /// Extract inputs from EmbeddingInput
    fn extract_inputs(input: &EmbeddingInput) -> Result<ExtractedInputs, CohereError> {
        match input {
            EmbeddingInput::Text(text) => Ok((Some(vec![text.clone()]), None)),
            EmbeddingInput::Array(arr) => {
                // Check if inputs are base64 encoded images
                let is_image = arr
                    .first()
                    .map(|s| Self::is_base64_image(s))
                    .unwrap_or(false);

                if is_image {
                    Ok((None, Some(arr.clone())))
                } else {
                    Ok((Some(arr.clone()), None))
                }
            }
        }
    }

    /// Check if a string is a base64 encoded image
    fn is_base64_image(s: &str) -> bool {
        // Simple heuristic: check for data URI or long base64 string
        s.starts_with("data:image")
            || (s.len() > 1000
                && s.chars()
                    .all(|c| c.is_alphanumeric() || c == '+' || c == '/' || c == '='))
    }

    /// Transform Cohere response to standard EmbeddingResponse
    pub fn transform_response(
        response_json: Value,
        model: &str,
        input_count: usize,
    ) -> Result<EmbeddingResponse, CohereError> {
        let embeddings = response_json.get("embeddings").ok_or_else(|| {
            super::error::cohere_response_parsing("Missing embeddings in response")
        })?;

        // Get the first available embedding type
        let embedding_vectors = Self::extract_embeddings(embeddings)?;

        let mut data = Vec::new();
        for (index, embedding) in embedding_vectors.into_iter().enumerate() {
            data.push(EmbeddingData {
                object: "embedding".to_string(),
                index: index as u32,
                embedding,
            });
        }

        // Calculate usage
        let usage = Self::extract_usage(&response_json, input_count);

        Ok(EmbeddingResponse {
            object: "list".to_string(),
            data,
            model: model.to_string(),
            usage: Some(usage),
            embeddings: None,
        })
    }

    /// Extract embeddings from the response
    fn extract_embeddings(embeddings: &Value) -> Result<Vec<Vec<f32>>, CohereError> {
        // Try float first (most common)
        if let Some(float_embeddings) = embeddings.get("float") {
            if let Some(arr) = float_embeddings.as_array() {
                return arr
                    .iter()
                    .map(|emb| {
                        emb.as_array()
                            .map(|v| {
                                v.iter()
                                    .filter_map(|n| n.as_f64().map(|f| f as f32))
                                    .collect()
                            })
                            .ok_or_else(|| {
                                super::error::cohere_response_parsing("Invalid embedding format")
                            })
                    })
                    .collect();
            }
        }

        // Fallback: try to parse embeddings directly as a nested array
        if let Some(arr) = embeddings.as_array() {
            if let Some(first) = arr.first() {
                if first.is_array() {
                    return arr
                        .iter()
                        .map(|emb| {
                            emb.as_array()
                                .map(|v| {
                                    v.iter()
                                        .filter_map(|n| n.as_f64().map(|f| f as f32))
                                        .collect()
                                })
                                .ok_or_else(|| {
                                    super::error::cohere_response_parsing(
                                        "Invalid embedding format",
                                    )
                                })
                        })
                        .collect();
                }
            }
        }

        Err(super::error::cohere_response_parsing(
            "No valid embeddings found in response",
        ))
    }

    /// Extract usage from response
    fn extract_usage(response_json: &Value, input_count: usize) -> Usage {
        let mut prompt_tokens = 0u32;

        if let Some(meta) = response_json.get("meta") {
            if let Some(billed_units) = meta.get("billed_units") {
                if let Some(input_tokens) =
                    billed_units.get("input_tokens").and_then(|v| v.as_u64())
                {
                    prompt_tokens = input_tokens as u32;
                }
                if let Some(images) = billed_units.get("images").and_then(|v| v.as_u64()) {
                    prompt_tokens += images as u32;
                }
            }
        }

        // If no usage info, estimate based on input count
        if prompt_tokens == 0 {
            prompt_tokens = (input_count * 100) as u32; // Rough estimate
        }

        Usage {
            prompt_tokens,
            completion_tokens: 0,
            total_tokens: prompt_tokens,
            prompt_tokens_details: None,
            completion_tokens_details: None,
            thinking_usage: None,
        }
    }

    /// Get supported OpenAI parameters for embeddings
    pub fn get_supported_params() -> &'static [&'static str] {
        &["encoding_format", "dimensions"]
    }

    /// Get default dimensions for a model
    pub fn get_default_dimensions(model: &str) -> Option<u32> {
        match model {
            m if m.contains("embed-english-v3") => Some(1024),
            m if m.contains("embed-multilingual-v3") => Some(1024),
            m if m.contains("embed-english-v2") => Some(4096),
            m if m.contains("embed-multilingual-v2") => Some(768),
            _ => None,
        }
    }

    /// Get maximum input length for a model
    pub fn get_max_input_length(model: &str) -> u32 {
        match model {
            m if m.contains("embed") && m.contains("v3") => 512,
            m if m.contains("embed") && m.contains("v2") => 512,
            _ => 512,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_type_display() {
        assert_eq!(
            CohereEmbeddingInputType::SearchDocument.to_string(),
            "search_document"
        );
        assert_eq!(
            CohereEmbeddingInputType::SearchQuery.to_string(),
            "search_query"
        );
        assert_eq!(
            CohereEmbeddingInputType::Classification.to_string(),
            "classification"
        );
    }

    #[test]
    fn test_extract_inputs_text() {
        let input = EmbeddingInput::Text("Hello world".to_string());
        let (texts, images) = CohereEmbeddingHandler::extract_inputs(&input).unwrap();

        assert!(texts.is_some());
        assert!(images.is_none());
        assert_eq!(texts.unwrap()[0], "Hello world");
    }

    #[test]
    fn test_extract_inputs_array() {
        let input = EmbeddingInput::Array(vec!["Hello".to_string(), "World".to_string()]);
        let (texts, images) = CohereEmbeddingHandler::extract_inputs(&input).unwrap();

        assert!(texts.is_some());
        assert!(images.is_none());
        assert_eq!(texts.unwrap().len(), 2);
    }

    #[test]
    fn test_extract_embeddings() {
        let embeddings = json!({
            "float": [[0.1, 0.2, 0.3], [0.4, 0.5, 0.6]]
        });

        let result = CohereEmbeddingHandler::extract_embeddings(&embeddings).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].len(), 3);
    }

    #[test]
    fn test_get_default_dimensions() {
        assert_eq!(
            CohereEmbeddingHandler::get_default_dimensions("embed-english-v3.0"),
            Some(1024)
        );
        assert_eq!(
            CohereEmbeddingHandler::get_default_dimensions("embed-multilingual-v3.0"),
            Some(1024)
        );
        assert_eq!(
            CohereEmbeddingHandler::get_default_dimensions("embed-english-v2.0"),
            Some(4096)
        );
    }

    #[test]
    fn test_supported_params() {
        let params = CohereEmbeddingHandler::get_supported_params();
        assert!(params.contains(&"encoding_format"));
        assert!(params.contains(&"dimensions"));
    }
}
