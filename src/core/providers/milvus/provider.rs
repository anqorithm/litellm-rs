//! Milvus Vector Database Provider Implementation
//!
//! Implements the LLMProvider trait for Milvus vector database.
//! Milvus is an open-source vector database designed for AI applications,
//! providing high-performance similarity search and vector storage.
//!
//! This provider focuses on embedding-related operations:
//! - Vector insertion
//! - Similarity search
//! - Collection management
//!
//! Reference: <https://milvus.io/docs/restful_api.md>

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::debug;

use super::config::MilvusConfig;
use super::error::MilvusError;
use super::models::{get_available_models, get_model_info};
use crate::core::providers::base::{
    GlobalPoolManager, HeaderPair, HttpMethod, header, header_owned,
};
use crate::core::traits::provider::ProviderConfig as _;
use crate::core::traits::provider::llm_provider::trait_definition::LLMProvider;
use crate::core::types::{model::ModelInfo, model::ProviderCapability};

/// Provider name constant
const PROVIDER_NAME: &str = "milvus";

/// Static capabilities for Milvus provider
/// Milvus is primarily a vector database, so it supports embeddings storage/retrieval
const MILVUS_CAPABILITIES: &[ProviderCapability] = &[ProviderCapability::Embeddings];

/// Milvus REST API endpoints
mod endpoints {
    pub const VECTOR_INSERT: &str = "/v1/vector/insert";
    pub const VECTOR_SEARCH: &str = "/v1/vector/search";
    pub const HEALTH: &str = "/v1/vector/collections";
}

/// Milvus vector insert request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MilvusInsertRequest {
    /// Collection name
    pub collection_name: String,
    /// Database name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_name: Option<String>,
    /// Vector data to insert
    pub data: Vec<MilvusVectorData>,
}

/// Milvus vector data for insertion
#[derive(Debug, Clone, Serialize)]
pub struct MilvusVectorData {
    /// Vector embeddings
    pub vector: Vec<f32>,
    /// Additional fields (metadata)
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Milvus search request
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MilvusSearchRequest {
    /// Collection name
    pub collection_name: String,
    /// Database name (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub db_name: Option<String>,
    /// Query vectors
    pub vector: Vec<f32>,
    /// Number of results to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// Top K results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_k: Option<u32>,
    /// Filter expression
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    /// Output fields to return
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_fields: Option<Vec<String>>,
    /// Search parameters
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<HashMap<String, serde_json::Value>>,
}

/// Milvus API response wrapper
#[derive(Debug, Clone, Deserialize)]
pub struct MilvusResponse<T> {
    /// Response code (0 = success)
    pub code: i32,
    /// Response data
    pub data: Option<T>,
    /// Error message (if any)
    pub message: Option<String>,
}

/// Milvus search result
#[derive(Debug, Clone, Deserialize)]
pub struct MilvusSearchResult {
    /// Result ID
    pub id: serde_json::Value,
    /// Distance/similarity score
    pub distance: f32,
    /// Additional fields
    #[serde(flatten)]
    pub fields: HashMap<String, serde_json::Value>,
}

/// Milvus provider implementation
#[derive(Debug, Clone)]
pub struct MilvusProvider {
    config: MilvusConfig,
    pool_manager: Arc<GlobalPoolManager>,
    models: Vec<ModelInfo>,
}

impl MilvusProvider {
    /// Create a new Milvus provider instance
    pub async fn new(config: MilvusConfig) -> Result<Self, MilvusError> {
        // Validate configuration
        config
            .validate()
            .map_err(|e| MilvusError::configuration(PROVIDER_NAME, e))?;

        // Create pool manager
        let pool_manager = Arc::new(GlobalPoolManager::new().map_err(|e| {
            MilvusError::configuration(
                PROVIDER_NAME,
                format!("Failed to create pool manager: {}", e),
            )
        })?);

        // Build model list from static configuration
        let models = get_available_models()
            .iter()
            .filter_map(|id| get_model_info(id))
            .map(|info| ModelInfo {
                id: info.model_id.to_string(),
                name: info.display_name.to_string(),
                provider: PROVIDER_NAME.to_string(),
                max_context_length: 0, // Milvus doesn't have context length
                max_output_length: None,
                supports_streaming: false,
                supports_tools: false,
                supports_multimodal: false,
                input_cost_per_1k_tokens: None, // Milvus is self-hosted, no per-token cost
                output_cost_per_1k_tokens: None,
                currency: "USD".to_string(),
                capabilities: vec![ProviderCapability::Embeddings],
                created_at: None,
                updated_at: None,
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert(
                        "embedding_dimensions".to_string(),
                        serde_json::json!(info.dimensions),
                    );
                    meta.insert(
                        "recommended_metric".to_string(),
                        serde_json::json!(info.recommended_metric.as_str()),
                    );
                    meta
                },
            })
            .collect();

        Ok(Self {
            config,
            pool_manager,
            models,
        })
    }

    /// Create provider with host only (using defaults)
    pub async fn with_host(host: impl Into<String>) -> Result<Self, MilvusError> {
        let config = MilvusConfig::new(host);
        Self::new(config).await
    }

    /// Create provider with host and port
    pub async fn with_host_port(host: impl Into<String>, port: u16) -> Result<Self, MilvusError> {
        let config = MilvusConfig::with_host_port(host, port);
        Self::new(config).await
    }

    /// Create provider from environment variables
    pub async fn from_env() -> Result<Self, MilvusError> {
        let config = MilvusConfig::from_env();
        Self::new(config).await
    }

    /// Get the configuration
    pub fn config(&self) -> &MilvusConfig {
        &self.config
    }

    /// Build request headers
    fn build_headers(&self) -> Vec<HeaderPair> {
        let mut headers = vec![header("Content-Type", "application/json".to_string())];

        // Add authentication headers
        for (key, value) in self.config.get_auth_headers() {
            headers.push(header_owned(key, value));
        }

        headers
    }

    /// Execute an HTTP request to Milvus
    async fn execute_request(
        &self,
        endpoint: &str,
        body: serde_json::Value,
    ) -> Result<serde_json::Value, MilvusError> {
        let url = self.config.get_endpoint_url(endpoint);
        let headers = self.build_headers();

        debug!("Milvus request to {}: {:?}", url, body);

        let response = self
            .pool_manager
            .execute_request(&url, HttpMethod::POST, headers, Some(body))
            .await
            .map_err(|e| MilvusError::network(PROVIDER_NAME, e.to_string()))?;

        let status = response.status();
        let response_bytes = response
            .bytes()
            .await
            .map_err(|e| MilvusError::network(PROVIDER_NAME, e.to_string()))?;

        debug!("Milvus response status: {}", status);

        if !status.is_success() {
            let body_str = String::from_utf8_lossy(&response_bytes);
            return Err(self.map_http_error(status.as_u16(), &body_str));
        }

        serde_json::from_slice(&response_bytes).map_err(|e| {
            MilvusError::response_parsing(PROVIDER_NAME, format!("Failed to parse response: {}", e))
        })
    }

    /// Map HTTP status codes to provider errors
    fn map_http_error(&self, status: u16, body: &str) -> MilvusError {
        match status {
            401 | 403 => MilvusError::authentication(PROVIDER_NAME, "Authentication failed"),
            404 => MilvusError::invalid_request(PROVIDER_NAME, "Resource not found"),
            429 => MilvusError::rate_limit(PROVIDER_NAME, None),
            400 => MilvusError::invalid_request(PROVIDER_NAME, body),
            500..=599 => MilvusError::provider_unavailable(PROVIDER_NAME, body),
            _ => MilvusError::api_error(PROVIDER_NAME, status, body),
        }
    }

    /// Insert vectors into a collection
    pub async fn insert_vectors(
        &self,
        collection_name: &str,
        vectors: Vec<Vec<f32>>,
        metadata: Option<Vec<HashMap<String, serde_json::Value>>>,
    ) -> Result<serde_json::Value, MilvusError> {
        let data: Vec<MilvusVectorData> = vectors
            .into_iter()
            .enumerate()
            .map(|(i, vector)| MilvusVectorData {
                vector,
                fields: metadata
                    .as_ref()
                    .and_then(|m| m.get(i).cloned())
                    .unwrap_or_default(),
            })
            .collect();

        let request = MilvusInsertRequest {
            collection_name: collection_name.to_string(),
            db_name: self.config.database.clone(),
            data,
        };

        let body = serde_json::to_value(&request).map_err(|e| {
            MilvusError::serialization(PROVIDER_NAME, format!("Failed to serialize request: {}", e))
        })?;

        self.execute_request(endpoints::VECTOR_INSERT, body).await
    }

    /// Search for similar vectors
    pub async fn search_vectors(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        top_k: u32,
        filter: Option<&str>,
        output_fields: Option<Vec<String>>,
    ) -> Result<Vec<MilvusSearchResult>, MilvusError> {
        let request = MilvusSearchRequest {
            collection_name: collection_name.to_string(),
            db_name: self.config.database.clone(),
            vector: query_vector,
            limit: Some(top_k),
            top_k: Some(top_k),
            filter: filter.map(|s| s.to_string()),
            output_fields,
            params: None,
        };

        let body = serde_json::to_value(&request).map_err(|e| {
            MilvusError::serialization(PROVIDER_NAME, format!("Failed to serialize request: {}", e))
        })?;

        let response = self.execute_request(endpoints::VECTOR_SEARCH, body).await?;

        // Parse the response
        let milvus_response: MilvusResponse<Vec<MilvusSearchResult>> =
            serde_json::from_value(response).map_err(|e| {
                MilvusError::response_parsing(
                    PROVIDER_NAME,
                    format!("Failed to parse search response: {}", e),
                )
            })?;

        if milvus_response.code != 0 {
            return Err(MilvusError::api_error(
                PROVIDER_NAME,
                milvus_response.code as u16,
                milvus_response
                    .message
                    .unwrap_or_else(|| "Unknown error".to_string()),
            ));
        }

        Ok(milvus_response.data.unwrap_or_default())
    }
}
