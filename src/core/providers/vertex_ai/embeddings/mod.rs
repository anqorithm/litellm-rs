//! Vertex AI Embeddings Module

use crate::ProviderError;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::core::types::{
    embedding::EmbeddingInput,
    embedding::EmbeddingRequest,
    responses::{EmbeddingData, EmbeddingResponse},
};

/// Vertex AI embedding models
#[derive(Debug, Clone)]
pub enum VertexEmbeddingModel {
    /// Text Embedding 004 - Latest multilingual model
    TextEmbedding004,
    /// Text Embedding Preview - Latest preview model
    TextEmbeddingPreview0409,
    /// Multilingual Embedding 002 - Multilingual support
    TextMultilingualEmbedding002,
    /// Multimodal Embedding - Text and image embeddings
    MultimodalEmbedding,
    /// Legacy Gecko models
    TextEmbeddingGecko,
    TextEmbeddingGecko003,
    TextEmbeddingGeckoMultilingual,
    /// Custom model
    Custom(String),
}

impl VertexEmbeddingModel {
    /// Get the model ID for API calls
    pub fn model_id(&self) -> String {
        match self {
            Self::TextEmbedding004 => "text-embedding-004".to_string(),
            Self::TextEmbeddingPreview0409 => "text-embedding-preview-0409".to_string(),
            Self::TextMultilingualEmbedding002 => "text-multilingual-embedding-002".to_string(),
            Self::MultimodalEmbedding => "multimodalembedding".to_string(),
            Self::TextEmbeddingGecko => "textembedding-gecko".to_string(),
            Self::TextEmbeddingGecko003 => "textembedding-gecko@003".to_string(),
            Self::TextEmbeddingGeckoMultilingual => "textembedding-gecko-multilingual".to_string(),
            Self::Custom(id) => id.clone(),
        }
    }

    /// Get maximum input length
    pub fn max_input_length(&self) -> usize {
        match self {
            Self::TextEmbedding004 => 3072,
            Self::TextEmbeddingPreview0409 => 3072,
            Self::TextMultilingualEmbedding002 => 2048,
            Self::MultimodalEmbedding => 2048,
            Self::TextEmbeddingGecko
            | Self::TextEmbeddingGecko003
            | Self::TextEmbeddingGeckoMultilingual => 3072,
            Self::Custom(_) => 2048, // Default
        }
    }

    /// Get embedding dimensions
    pub fn dimensions(&self) -> usize {
        match self {
            Self::TextEmbedding004 => 768,
            Self::TextEmbeddingPreview0409 => 768,
            Self::TextMultilingualEmbedding002 => 768,
            Self::MultimodalEmbedding => 1408,
            Self::TextEmbeddingGecko
            | Self::TextEmbeddingGecko003
            | Self::TextEmbeddingGeckoMultilingual => 768,
            Self::Custom(_) => 768, // Default
        }
    }

    /// Check if model supports images
    pub fn supports_images(&self) -> bool {
        matches!(self, Self::MultimodalEmbedding)
    }

    /// Check if model supports batch processing
    pub fn supports_batch(&self) -> bool {
        matches!(
            self,
            Self::TextEmbedding004
                | Self::TextEmbeddingPreview0409
                | Self::TextMultilingualEmbedding002
        )
    }
}

/// Parse embedding model string
pub fn parse_embedding_model(model: &str) -> VertexEmbeddingModel {
    match model {
        "text-embedding-004" => VertexEmbeddingModel::TextEmbedding004,
        "text-embedding-preview-0409" => VertexEmbeddingModel::TextEmbeddingPreview0409,
        "text-multilingual-embedding-002" => VertexEmbeddingModel::TextMultilingualEmbedding002,
        "multimodalembedding" => VertexEmbeddingModel::MultimodalEmbedding,
        "textembedding-gecko" => VertexEmbeddingModel::TextEmbeddingGecko,
        "textembedding-gecko@003" => VertexEmbeddingModel::TextEmbeddingGecko003,
        "textembedding-gecko-multilingual" => VertexEmbeddingModel::TextEmbeddingGeckoMultilingual,
        _ => VertexEmbeddingModel::Custom(model.to_string()),
    }
}

/// Task types for embedding generation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum TaskType {
    /// For retrieval queries
    #[serde(rename = "RETRIEVAL_QUERY")]
    RetrievalQuery,
    /// For retrieval documents
    #[serde(rename = "RETRIEVAL_DOCUMENT")]
    #[default]
    RetrievalDocument,
    /// For semantic similarity
    #[serde(rename = "SEMANTIC_SIMILARITY")]
    SemanticSimilarity,
    /// For classification tasks
    #[serde(rename = "CLASSIFICATION")]
    Classification,
    /// For clustering tasks
    #[serde(rename = "CLUSTERING")]
    Clustering,
    /// For question answering
    #[serde(rename = "QUESTION_ANSWERING")]
    QuestionAnswering,
    /// For fact verification
    #[serde(rename = "FACT_VERIFICATION")]
    FactVerification,
}

/// Embedding instance for Vertex AI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingInstance {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_type: Option<TaskType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Multimodal embedding instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultimodalEmbeddingInstance {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<ImageData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<VideoData>,
}

/// Image data for multimodal embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bytes_base64_encoded: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcs_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
}

/// Video data for multimodal embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gcs_uri: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_offset_sec: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_offset_sec: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval_sec: Option<f32>,
}

/// Embedding parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingParameters {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_truncate: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_dimensionality: Option<i32>,
}

/// Embedding handler
pub struct EmbeddingHandler {
    model: VertexEmbeddingModel,
}

impl EmbeddingHandler {
    /// Create new embedding handler
    pub fn new(model: VertexEmbeddingModel) -> Self {
        Self { model }
    }

    /// Transform embedding request to Vertex AI format
    pub fn transform_request(&self, request: &EmbeddingRequest) -> Result<Value, ProviderError> {
        // Convert EmbeddingInput to Vec<String>
        let input_strings = match &request.input {
            EmbeddingInput::Text(text) => vec![text.clone()],
            EmbeddingInput::Array(texts) => texts.clone(),
        };

        let instances = if self.model.supports_images() {
            // For multimodal embeddings
            self.create_multimodal_instances(&input_strings)?
        } else {
            // For text embeddings
            self.create_text_instances(&input_strings, request.task_type.as_deref())?
        };

        let mut body = json!({
            "instances": instances
        });

        // Add parameters if specified
        let parameters = EmbeddingParameters {
            auto_truncate: Some(true),
            output_dimensionality: request.dimensions.map(|d| d as i32),
        };

        body["parameters"] = serde_json::to_value(parameters)?;

        Ok(body)
    }

    /// Create text embedding instances
    fn create_text_instances(
        &self,
        inputs: &[String],
        task_type: Option<&str>,
    ) -> Result<Vec<Value>, ProviderError> {
        let task = task_type
            .and_then(|t| self.parse_task_type(t))
            .unwrap_or_default();

        let instances = inputs
            .iter()
            .map(|content| {
                let instance = EmbeddingInstance {
                    content: content.clone(),
                    task_type: Some(task.clone()),
                    title: None,
                };
                serde_json::to_value(instance).unwrap_or_default()
            })
            .collect();

        Ok(instances)
    }

    /// Create multimodal embedding instances
    fn create_multimodal_instances(&self, inputs: &[String]) -> Result<Vec<Value>, ProviderError> {
        let instances = inputs
            .iter()
            .map(|content| {
                // Check if input is a data URL or GCS URI
                if content.starts_with("data:image/") {
                    // Base64 image
                    let parts: Vec<&str> = content.splitn(2, ',').collect();
                    if parts.len() == 2 {
                        let mime_type = parts[0]
                            .strip_prefix("data:")
                            .and_then(|s| s.strip_suffix(";base64"))
                            .unwrap_or("image/jpeg")
                            .to_string();

                        MultimodalEmbeddingInstance {
                            text: None,
                            image: Some(ImageData {
                                bytes_base64_encoded: Some(parts[1].to_string()),
                                gcs_uri: None,
                                mime_type: Some(mime_type),
                            }),
                            video: None,
                        }
                    } else {
                        MultimodalEmbeddingInstance {
                            text: Some(content.clone()),
                            image: None,
                            video: None,
                        }
                    }
                } else if content.starts_with("gs://") {
                    // GCS URI - could be image or video
                    if content.contains(".mp4")
                        || content.contains(".avi")
                        || content.contains(".mov")
                    {
                        MultimodalEmbeddingInstance {
                            text: None,
                            image: None,
                            video: Some(VideoData {
                                gcs_uri: Some(content.clone()),
                                start_offset_sec: None,
                                end_offset_sec: None,
                                interval_sec: None,
                            }),
                        }
                    } else {
                        MultimodalEmbeddingInstance {
                            text: None,
                            image: Some(ImageData {
                                bytes_base64_encoded: None,
                                gcs_uri: Some(content.clone()),
                                mime_type: None,
                            }),
                            video: None,
                        }
                    }
                } else {
                    // Regular text
                    MultimodalEmbeddingInstance {
                        text: Some(content.clone()),
                        image: None,
                        video: None,
                    }
                }
            })
            .map(|instance| serde_json::to_value(instance).unwrap_or_default())
            .collect();

        Ok(instances)
    }

    /// Parse task type from string
    fn parse_task_type(&self, task_type: &str) -> Option<TaskType> {
        match task_type.to_uppercase().as_str() {
            "RETRIEVAL_QUERY" => Some(TaskType::RetrievalQuery),
            "RETRIEVAL_DOCUMENT" => Some(TaskType::RetrievalDocument),
            "SEMANTIC_SIMILARITY" => Some(TaskType::SemanticSimilarity),
            "CLASSIFICATION" => Some(TaskType::Classification),
            "CLUSTERING" => Some(TaskType::Clustering),
            "QUESTION_ANSWERING" => Some(TaskType::QuestionAnswering),
            "FACT_VERIFICATION" => Some(TaskType::FactVerification),
            _ => None,
        }
    }

    /// Transform Vertex AI response to standard format
    pub fn transform_response(&self, response: Value) -> Result<EmbeddingResponse, ProviderError> {
        let predictions = response["predictions"].as_array().ok_or_else(|| {
            ProviderError::response_parsing(
                "vertex_ai",
                "Missing predictions in embedding response",
            )
        })?;

        let mut embeddings = Vec::new();

        for prediction in predictions {
            let values =
                if let Some(embedding_values) = prediction["embeddings"]["values"].as_array() {
                    // Standard embedding format
                    embedding_values
                        .iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect()
                } else if let Some(values) = prediction["values"].as_array() {
                    // Alternative format
                    values
                        .iter()
                        .filter_map(|v| v.as_f64().map(|f| f as f32))
                        .collect()
                } else {
                    return Err(ProviderError::response_parsing(
                        "vertex_ai",
                        "Missing embedding values",
                    ));
                };

            embeddings.push(values);
        }

        let embedding_data: Vec<EmbeddingData> = embeddings
            .into_iter()
            .enumerate()
            .map(|(index, embedding)| EmbeddingData {
                object: "embedding".to_string(),
                embedding,
                index: index as u32,
            })
            .collect();

        Ok(EmbeddingResponse {
            object: "list".to_string(),
            data: embedding_data.clone(),
            embeddings: Some(embedding_data),
            model: self.model.model_id(),
            usage: None, // Vertex AI doesn't return token usage for embeddings
        })
    }
}

/// Batch embedding handler for processing large numbers of texts
pub struct BatchEmbeddingHandler {
    model: VertexEmbeddingModel,
    batch_size: usize,
}

impl BatchEmbeddingHandler {
    /// Create new batch embedding handler
    pub fn new(model: VertexEmbeddingModel, batch_size: usize) -> Self {
        Self { model, batch_size }
    }

    /// Process embeddings in batches
    pub async fn process_batch(
        &self,
        inputs: Vec<String>,
        _task_type: Option<String>,
    ) -> Result<Vec<Vec<f32>>, ProviderError> {
        if !self.model.supports_batch() {
            return Err(ProviderError::not_supported(
                "vertex_ai",
                format!(
                    "Model {} does not support batch processing",
                    self.model.model_id()
                ),
            ));
        }

        let mut all_embeddings = Vec::new();

        // Process in batches
        for chunk in inputs.chunks(self.batch_size) {
            let request = EmbeddingRequest {
                model: self.model.model_id(),
                input: crate::core::types::embedding::EmbeddingInput::Array(chunk.to_vec()),
                encoding_format: None,
                dimensions: None,
                user: None,
                task_type: Some("RETRIEVAL_DOCUMENT".to_string()), // Default
            };

            let handler = EmbeddingHandler::new(self.model.clone());
            let _vertex_request = handler.transform_request(&request)?;

            // TODO: Make actual API call
            // For now, return dummy embeddings
            for _ in chunk {
                all_embeddings.push(vec![0.0; self.model.dimensions()]);
            }
        }

        Ok(all_embeddings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ==================== VertexEmbeddingModel Tests ====================

    #[test]
    fn test_model_id_text_embedding_004() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbedding004.model_id(),
            "text-embedding-004"
        );
    }

    #[test]
    fn test_model_id_text_embedding_preview() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbeddingPreview0409.model_id(),
            "text-embedding-preview-0409"
        );
    }

    #[test]
    fn test_model_id_multilingual() {
        assert_eq!(
            VertexEmbeddingModel::TextMultilingualEmbedding002.model_id(),
            "text-multilingual-embedding-002"
        );
    }

    #[test]
    fn test_model_id_multimodal() {
        assert_eq!(
            VertexEmbeddingModel::MultimodalEmbedding.model_id(),
            "multimodalembedding"
        );
    }

    #[test]
    fn test_model_id_gecko() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbeddingGecko.model_id(),
            "textembedding-gecko"
        );
    }

    #[test]
    fn test_model_id_gecko_003() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbeddingGecko003.model_id(),
            "textembedding-gecko@003"
        );
    }

    #[test]
    fn test_model_id_gecko_multilingual() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbeddingGeckoMultilingual.model_id(),
            "textembedding-gecko-multilingual"
        );
    }

    #[test]
    fn test_model_id_custom() {
        let custom_model = VertexEmbeddingModel::Custom("my-custom-model".to_string());
        assert_eq!(custom_model.model_id(), "my-custom-model");
    }

    #[test]
    fn test_max_input_length_text_embedding_004() {
        assert_eq!(
            VertexEmbeddingModel::TextEmbedding004.max_input_length(),
            3072
        );
    }

    #[test]
    fn test_max_input_length_multilingual() {
        assert_eq!(
            VertexEmbeddingModel::TextMultilingualEmbedding002.max_input_length(),
            2048
        );
    }

    #[test]
    fn test_max_input_length_custom() {
        assert_eq!(
            VertexEmbeddingModel::Custom("test".to_string()).max_input_length(),
            2048
        );
    }

    #[test]
    fn test_dimensions_text_embedding_004() {
        assert_eq!(VertexEmbeddingModel::TextEmbedding004.dimensions(), 768);
    }

    #[test]
    fn test_dimensions_multimodal() {
        assert_eq!(VertexEmbeddingModel::MultimodalEmbedding.dimensions(), 1408);
    }

    #[test]
    fn test_dimensions_custom() {
        assert_eq!(
            VertexEmbeddingModel::Custom("test".to_string()).dimensions(),
            768
        );
    }

    #[test]
    fn test_supports_images_multimodal() {
        assert!(VertexEmbeddingModel::MultimodalEmbedding.supports_images());
    }

    #[test]
    fn test_supports_images_text() {
        assert!(!VertexEmbeddingModel::TextEmbedding004.supports_images());
    }

    #[test]
    fn test_supports_batch_text_embedding_004() {
        assert!(VertexEmbeddingModel::TextEmbedding004.supports_batch());
    }

    #[test]
    fn test_supports_batch_gecko() {
        assert!(!VertexEmbeddingModel::TextEmbeddingGecko.supports_batch());
    }

    #[test]
    fn test_supports_batch_multimodal() {
        assert!(!VertexEmbeddingModel::MultimodalEmbedding.supports_batch());
    }

    // ==================== parse_embedding_model Tests ====================

    #[test]
    fn test_parse_text_embedding_004() {
        let model = parse_embedding_model("text-embedding-004");
        assert_eq!(model.model_id(), "text-embedding-004");
    }

    #[test]
    fn test_parse_text_embedding_preview() {
        let model = parse_embedding_model("text-embedding-preview-0409");
        assert_eq!(model.model_id(), "text-embedding-preview-0409");
    }

    #[test]
    fn test_parse_multilingual() {
        let model = parse_embedding_model("text-multilingual-embedding-002");
        assert_eq!(model.model_id(), "text-multilingual-embedding-002");
    }

    #[test]
    fn test_parse_multimodal() {
        let model = parse_embedding_model("multimodalembedding");
        assert!(model.supports_images());
    }

    #[test]
    fn test_parse_gecko() {
        let model = parse_embedding_model("textembedding-gecko");
        assert_eq!(model.model_id(), "textembedding-gecko");
    }

    #[test]
    fn test_parse_unknown_model() {
        let model = parse_embedding_model("unknown-model");
        assert_eq!(model.model_id(), "unknown-model");
    }

    // ==================== TaskType Tests ====================

    #[test]
    fn test_task_type_serialization_retrieval_query() {
        let task = TaskType::RetrievalQuery;
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json, "RETRIEVAL_QUERY");
    }

    #[test]
    fn test_task_type_serialization_retrieval_document() {
        let task = TaskType::RetrievalDocument;
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json, "RETRIEVAL_DOCUMENT");
    }

    #[test]
    fn test_task_type_serialization_all() {
        let tasks = vec![
            (TaskType::RetrievalQuery, "RETRIEVAL_QUERY"),
            (TaskType::RetrievalDocument, "RETRIEVAL_DOCUMENT"),
            (TaskType::SemanticSimilarity, "SEMANTIC_SIMILARITY"),
            (TaskType::Classification, "CLASSIFICATION"),
            (TaskType::Clustering, "CLUSTERING"),
            (TaskType::QuestionAnswering, "QUESTION_ANSWERING"),
            (TaskType::FactVerification, "FACT_VERIFICATION"),
        ];

        for (task, expected) in tasks {
            let json = serde_json::to_value(&task).unwrap();
            assert_eq!(json, expected);
        }
    }

    #[test]
    fn test_task_type_deserialization() {
        let json = json!("RETRIEVAL_QUERY");
        let task: TaskType = serde_json::from_value(json).unwrap();
        assert!(matches!(task, TaskType::RetrievalQuery));
    }

    #[test]
    fn test_task_type_default() {
        let task = TaskType::default();
        let json = serde_json::to_value(&task).unwrap();
        assert_eq!(json, "RETRIEVAL_DOCUMENT");
    }

    // ==================== EmbeddingInstance Tests ====================

    #[test]
    fn test_embedding_instance_serialization() {
        let instance = EmbeddingInstance {
            content: "Test content".to_string(),
            task_type: Some(TaskType::RetrievalQuery),
            title: Some("Test title".to_string()),
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["content"], "Test content");
        assert_eq!(json["task_type"], "RETRIEVAL_QUERY");
        assert_eq!(json["title"], "Test title");
    }

    #[test]
    fn test_embedding_instance_minimal() {
        let instance = EmbeddingInstance {
            content: "Test".to_string(),
            task_type: None,
            title: None,
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["content"], "Test");
        assert!(json.get("task_type").is_none());
        assert!(json.get("title").is_none());
    }

    // ==================== MultimodalEmbeddingInstance Tests ====================

    #[test]
    fn test_multimodal_instance_text() {
        let instance = MultimodalEmbeddingInstance {
            text: Some("Text content".to_string()),
            image: None,
            video: None,
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["text"], "Text content");
        assert!(json.get("image").is_none());
        assert!(json.get("video").is_none());
    }

    #[test]
    fn test_multimodal_instance_image_base64() {
        let instance = MultimodalEmbeddingInstance {
            text: None,
            image: Some(ImageData {
                bytes_base64_encoded: Some("base64data".to_string()),
                gcs_uri: None,
                mime_type: Some("image/png".to_string()),
            }),
            video: None,
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert!(json.get("text").is_none());
        assert_eq!(json["image"]["bytes_base64_encoded"], "base64data");
        assert_eq!(json["image"]["mime_type"], "image/png");
    }

    #[test]
    fn test_multimodal_instance_image_gcs() {
        let instance = MultimodalEmbeddingInstance {
            text: None,
            image: Some(ImageData {
                bytes_base64_encoded: None,
                gcs_uri: Some("gs://bucket/image.png".to_string()),
                mime_type: None,
            }),
            video: None,
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["image"]["gcs_uri"], "gs://bucket/image.png");
    }

    #[test]
    fn test_multimodal_instance_video() {
        let instance = MultimodalEmbeddingInstance {
            text: None,
            image: None,
            video: Some(VideoData {
                gcs_uri: Some("gs://bucket/video.mp4".to_string()),
                start_offset_sec: Some(0.0),
                end_offset_sec: Some(10.0),
                interval_sec: Some(1.0),
            }),
        };

        let json = serde_json::to_value(&instance).unwrap();
        assert_eq!(json["video"]["gcs_uri"], "gs://bucket/video.mp4");
        assert_eq!(json["video"]["start_offset_sec"], 0.0);
        assert_eq!(json["video"]["end_offset_sec"], 10.0);
    }

    // ==================== EmbeddingParameters Tests ====================

    #[test]
    fn test_embedding_parameters_full() {
        let params = EmbeddingParameters {
            auto_truncate: Some(true),
            output_dimensionality: Some(256),
        };

        let json = serde_json::to_value(&params).unwrap();
        assert_eq!(json["auto_truncate"], true);
        assert_eq!(json["output_dimensionality"], 256);
    }

    #[test]
    fn test_embedding_parameters_minimal() {
        let params = EmbeddingParameters {
            auto_truncate: None,
            output_dimensionality: None,
        };

        let json = serde_json::to_value(&params).unwrap();
        assert!(json.as_object().unwrap().is_empty());
    }

    // ==================== EmbeddingHandler Tests ====================

    #[test]
    fn test_embedding_handler_new() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        // Handler should be created successfully
        assert_eq!(handler.model.model_id(), "text-embedding-004");
    }

    #[test]
    fn test_embedding_handler_transform_request_single_text() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let request = EmbeddingRequest {
            model: "text-embedding-004".to_string(),
            input: EmbeddingInput::Text("Hello world".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert!(body["instances"].is_array());
        assert_eq!(body["instances"].as_array().unwrap().len(), 1);
    }

    #[test]
    fn test_embedding_handler_transform_request_array() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let request = EmbeddingRequest {
            model: "text-embedding-004".to_string(),
            input: EmbeddingInput::Array(vec!["Hello".to_string(), "World".to_string()]),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert_eq!(body["instances"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_embedding_handler_transform_request_with_dimensions() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let request = EmbeddingRequest {
            model: "text-embedding-004".to_string(),
            input: EmbeddingInput::Text("Test".to_string()),
            encoding_format: None,
            dimensions: Some(256),
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert_eq!(body["parameters"]["output_dimensionality"], 256);
    }

    #[test]
    fn test_embedding_handler_multimodal_text() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::MultimodalEmbedding);
        let request = EmbeddingRequest {
            model: "multimodalembedding".to_string(),
            input: EmbeddingInput::Text("Plain text".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert!(body["instances"][0]["text"].is_string());
    }

    #[test]
    fn test_embedding_handler_multimodal_base64_image() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::MultimodalEmbedding);
        let request = EmbeddingRequest {
            model: "multimodalembedding".to_string(),
            input: EmbeddingInput::Text("data:image/png;base64,iVBORw0KGgo=".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert!(body["instances"][0]["image"].is_object());
        assert_eq!(
            body["instances"][0]["image"]["bytes_base64_encoded"],
            "iVBORw0KGgo="
        );
    }

    #[test]
    fn test_embedding_handler_multimodal_gcs_image() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::MultimodalEmbedding);
        let request = EmbeddingRequest {
            model: "multimodalembedding".to_string(),
            input: EmbeddingInput::Text("gs://my-bucket/image.jpg".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert!(body["instances"][0]["image"].is_object());
        assert_eq!(
            body["instances"][0]["image"]["gcs_uri"],
            "gs://my-bucket/image.jpg"
        );
    }

    #[test]
    fn test_embedding_handler_multimodal_gcs_video() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::MultimodalEmbedding);
        let request = EmbeddingRequest {
            model: "multimodalembedding".to_string(),
            input: EmbeddingInput::Text("gs://my-bucket/video.mp4".to_string()),
            encoding_format: None,
            dimensions: None,
            user: None,
            task_type: None,
        };

        let result = handler.transform_request(&request);
        assert!(result.is_ok());

        let body = result.unwrap();
        assert!(body["instances"][0]["video"].is_object());
        assert_eq!(
            body["instances"][0]["video"]["gcs_uri"],
            "gs://my-bucket/video.mp4"
        );
    }

    #[test]
    fn test_embedding_handler_parse_task_type() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);

        assert!(handler.parse_task_type("RETRIEVAL_QUERY").is_some());
        assert!(handler.parse_task_type("retrieval_query").is_some());
        assert!(handler.parse_task_type("SEMANTIC_SIMILARITY").is_some());
        assert!(handler.parse_task_type("INVALID_TYPE").is_none());
    }

    #[test]
    fn test_embedding_handler_transform_response_standard_format() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let response = json!({
            "predictions": [
                {
                    "embeddings": {
                        "values": [0.1, 0.2, 0.3, 0.4]
                    }
                }
            ]
        });

        let result = handler.transform_response(response);
        assert!(result.is_ok());

        let embedding_response = result.unwrap();
        assert_eq!(embedding_response.object, "list");
        assert_eq!(embedding_response.data.len(), 1);
        assert_eq!(embedding_response.data[0].embedding.len(), 4);
        assert!((embedding_response.data[0].embedding[0] - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_embedding_handler_transform_response_alternative_format() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let response = json!({
            "predictions": [
                {
                    "values": [0.5, 0.6, 0.7]
                }
            ]
        });

        let result = handler.transform_response(response);
        assert!(result.is_ok());

        let embedding_response = result.unwrap();
        assert_eq!(embedding_response.data.len(), 1);
        assert_eq!(embedding_response.data[0].embedding.len(), 3);
    }

    #[test]
    fn test_embedding_handler_transform_response_multiple() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let response = json!({
            "predictions": [
                {"embeddings": {"values": [0.1, 0.2]}},
                {"embeddings": {"values": [0.3, 0.4]}},
                {"embeddings": {"values": [0.5, 0.6]}}
            ]
        });

        let result = handler.transform_response(response);
        assert!(result.is_ok());

        let embedding_response = result.unwrap();
        assert_eq!(embedding_response.data.len(), 3);
        assert_eq!(embedding_response.data[0].index, 0);
        assert_eq!(embedding_response.data[1].index, 1);
        assert_eq!(embedding_response.data[2].index, 2);
    }

    #[test]
    fn test_embedding_handler_transform_response_missing_predictions() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let response = json!({});

        let result = handler.transform_response(response);
        assert!(result.is_err());
    }

    #[test]
    fn test_embedding_handler_transform_response_missing_values() {
        let handler = EmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004);
        let response = json!({
            "predictions": [
                {"embeddings": {}}
            ]
        });

        let result = handler.transform_response(response);
        assert!(result.is_err());
    }

    // ==================== BatchEmbeddingHandler Tests ====================

    #[test]
    fn test_batch_embedding_handler_new() {
        let handler = BatchEmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004, 100);
        assert_eq!(handler.batch_size, 100);
    }

    #[tokio::test]
    async fn test_batch_embedding_handler_unsupported_model() {
        let handler = BatchEmbeddingHandler::new(VertexEmbeddingModel::TextEmbeddingGecko, 100);
        let result = handler.process_batch(vec!["test".to_string()], None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_batch_embedding_handler_process_batch() {
        let handler = BatchEmbeddingHandler::new(VertexEmbeddingModel::TextEmbedding004, 2);
        let inputs = vec![
            "Text 1".to_string(),
            "Text 2".to_string(),
            "Text 3".to_string(),
        ];

        let result = handler.process_batch(inputs, None).await;
        assert!(result.is_ok());

        let embeddings = result.unwrap();
        assert_eq!(embeddings.len(), 3);
        // Each embedding should have correct dimensions
        assert_eq!(embeddings[0].len(), 768);
    }

    // ==================== ImageData Tests ====================

    #[test]
    fn test_image_data_serialization_base64() {
        let image = ImageData {
            bytes_base64_encoded: Some("abc123".to_string()),
            gcs_uri: None,
            mime_type: Some("image/jpeg".to_string()),
        };

        let json = serde_json::to_value(&image).unwrap();
        assert_eq!(json["bytes_base64_encoded"], "abc123");
        assert_eq!(json["mime_type"], "image/jpeg");
        assert!(json.get("gcs_uri").is_none());
    }

    #[test]
    fn test_image_data_serialization_gcs() {
        let image = ImageData {
            bytes_base64_encoded: None,
            gcs_uri: Some("gs://bucket/file.png".to_string()),
            mime_type: None,
        };

        let json = serde_json::to_value(&image).unwrap();
        assert_eq!(json["gcs_uri"], "gs://bucket/file.png");
    }

    // ==================== VideoData Tests ====================

    #[test]
    fn test_video_data_serialization_full() {
        let video = VideoData {
            gcs_uri: Some("gs://bucket/video.mp4".to_string()),
            start_offset_sec: Some(5.0),
            end_offset_sec: Some(15.0),
            interval_sec: Some(2.0),
        };

        let json = serde_json::to_value(&video).unwrap();
        assert_eq!(json["gcs_uri"], "gs://bucket/video.mp4");
        assert_eq!(json["start_offset_sec"], 5.0);
        assert_eq!(json["end_offset_sec"], 15.0);
        assert_eq!(json["interval_sec"], 2.0);
    }

    #[test]
    fn test_video_data_serialization_minimal() {
        let video = VideoData {
            gcs_uri: Some("gs://bucket/video.mp4".to_string()),
            start_offset_sec: None,
            end_offset_sec: None,
            interval_sec: None,
        };

        let json = serde_json::to_value(&video).unwrap();
        assert_eq!(json["gcs_uri"], "gs://bucket/video.mp4");
        assert!(json.get("start_offset_sec").is_none());
    }
}
