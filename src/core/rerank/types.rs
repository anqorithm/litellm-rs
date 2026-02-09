//! Rerank types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Rerank request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankRequest {
    /// Model to use for reranking (e.g., "cohere/rerank-english-v3.0")
    pub model: String,

    /// The query to compare documents against
    pub query: String,

    /// List of documents to rerank
    pub documents: Vec<RerankDocument>,

    /// Number of top results to return (default: all documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_n: Option<usize>,

    /// Whether to return the document text in results
    #[serde(skip_serializing_if = "Option::is_none")]
    pub return_documents: Option<bool>,

    /// Maximum number of chunks per document (for long documents)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_chunks_per_doc: Option<usize>,

    /// Additional provider-specific parameters
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub extra_params: HashMap<String, serde_json::Value>,
}

impl Default for RerankRequest {
    fn default() -> Self {
        Self {
            model: "cohere/rerank-english-v3.0".to_string(),
            query: String::new(),
            documents: Vec::new(),
            top_n: None,
            return_documents: Some(true),
            max_chunks_per_doc: None,
            extra_params: HashMap::new(),
        }
    }
}

/// Document for reranking - can be a simple string or structured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RerankDocument {
    /// Simple text document
    Text(String),
    /// Structured document with metadata
    Structured {
        /// Document text content
        text: String,
        /// Optional document title
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        /// Optional document ID for tracking
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<String>,
        /// Additional metadata
        #[serde(default, skip_serializing_if = "HashMap::is_empty")]
        metadata: HashMap<String, serde_json::Value>,
    },
}

impl RerankDocument {
    /// Create a simple text document
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Create a structured document
    pub fn structured(text: impl Into<String>) -> Self {
        Self::Structured {
            text: text.into(),
            title: None,
            id: None,
            metadata: HashMap::new(),
        }
    }

    /// Get the text content of the document
    pub fn get_text(&self) -> &str {
        match self {
            Self::Text(t) => t,
            Self::Structured { text, .. } => text,
        }
    }

    /// Get the document ID if available
    pub fn get_id(&self) -> Option<&str> {
        match self {
            Self::Text(_) => None,
            Self::Structured { id, .. } => id.as_deref(),
        }
    }
}

impl From<String> for RerankDocument {
    fn from(s: String) -> Self {
        Self::Text(s)
    }
}

impl From<&str> for RerankDocument {
    fn from(s: &str) -> Self {
        Self::Text(s.to_string())
    }
}

/// Rerank response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResponse {
    /// Unique response ID
    pub id: String,

    /// Reranked results ordered by relevance (highest first)
    pub results: Vec<RerankResult>,

    /// Model used for reranking
    pub model: String,

    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<RerankUsage>,

    /// Response metadata
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub meta: HashMap<String, serde_json::Value>,
}

/// Individual rerank result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RerankResult {
    /// Original index of the document in the input list
    pub index: usize,

    /// Relevance score (typically 0.0 to 1.0, higher is more relevant)
    pub relevance_score: f64,

    /// The document text (if return_documents was true)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document: Option<RerankDocument>,
}

/// Token usage for reranking
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RerankUsage {
    /// Number of tokens in the query
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_tokens: Option<u32>,

    /// Number of tokens in all documents
    #[serde(skip_serializing_if = "Option::is_none")]
    pub document_tokens: Option<u32>,

    /// Total tokens processed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_tokens: Option<u32>,

    /// Search units consumed (Cohere-specific)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_units: Option<u32>,
}


#[cfg(test)]
#[path = "types_tests.rs"]
mod tests;
