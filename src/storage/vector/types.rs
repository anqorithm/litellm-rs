//! Type definitions for vector storage

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::utils::error::error::Result;

/// Vector data for storage and retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorData {
    /// Unique identifier
    pub id: String,
    /// Vector embedding
    pub vector: Vec<f32>,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Search result from vector database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Result ID
    pub id: String,
    /// Similarity score
    pub score: f32,
    /// Associated metadata
    pub metadata: Option<serde_json::Value>,
    /// Vector data (optional)
    pub vector: Option<Vec<f32>>,
}

/// Vector point for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorPoint {
    /// Point ID
    pub id: String,
    /// Vector data
    pub vector: Vec<f32>,
    /// Associated metadata
    pub metadata: Option<serde_json::Value>,
}

/// Vector store trait
#[async_trait::async_trait]
pub trait VectorStore: Send + Sync {
    /// Search for similar vectors
    async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<SearchResult>>;

    /// Insert vectors
    async fn insert(&self, vectors: Vec<VectorData>) -> Result<()>;

    /// Delete vectors by ID
    async fn delete(&self, ids: Vec<String>) -> Result<()>;
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== VectorData Tests ====================

    #[test]
    fn test_vector_data_new() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), serde_json::json!("value"));

        let data = VectorData {
            id: "vec-123".to_string(),
            vector: vec![0.1, 0.2, 0.3, 0.4],
            metadata,
        };

        assert_eq!(data.id, "vec-123");
        assert_eq!(data.vector.len(), 4);
        assert!((data.vector[0] - 0.1).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vector_data_clone() {
        let data = VectorData {
            id: "vec-456".to_string(),
            vector: vec![1.0, 2.0, 3.0],
            metadata: HashMap::new(),
        };

        let cloned = data.clone();
        assert_eq!(data.id, cloned.id);
        assert_eq!(data.vector, cloned.vector);
    }

    #[test]
    fn test_vector_data_serialization() {
        let mut metadata = HashMap::new();
        metadata.insert("category".to_string(), serde_json::json!("test"));

        let data = VectorData {
            id: "vec-789".to_string(),
            vector: vec![0.5, 0.6],
            metadata,
        };

        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["id"], "vec-789");
        assert!(json["vector"].is_array());
        assert!(json["metadata"]["category"].is_string());
    }

    #[test]
    fn test_vector_data_empty_vector() {
        let data = VectorData {
            id: "empty".to_string(),
            vector: vec![],
            metadata: HashMap::new(),
        };

        assert!(data.vector.is_empty());
    }

    #[test]
    fn test_vector_data_empty_metadata() {
        let data = VectorData {
            id: "no-meta".to_string(),
            vector: vec![1.0],
            metadata: HashMap::new(),
        };

        assert!(data.metadata.is_empty());
    }

    // ==================== SearchResult Tests ====================

    #[test]
    fn test_search_result_structure() {
        let result = SearchResult {
            id: "result-123".to_string(),
            score: 0.95,
            metadata: Some(serde_json::json!({"type": "document"})),
            vector: Some(vec![0.1, 0.2, 0.3]),
        };

        assert_eq!(result.id, "result-123");
        assert!((result.score - 0.95).abs() < f32::EPSILON);
        assert!(result.metadata.is_some());
        assert!(result.vector.is_some());
    }

    #[test]
    fn test_search_result_minimal() {
        let result = SearchResult {
            id: "result-456".to_string(),
            score: 0.5,
            metadata: None,
            vector: None,
        };

        assert_eq!(result.id, "result-456");
        assert!(result.metadata.is_none());
        assert!(result.vector.is_none());
    }

    #[test]
    fn test_search_result_clone() {
        let result = SearchResult {
            id: "result-789".to_string(),
            score: 0.8,
            metadata: None,
            vector: None,
        };

        let cloned = result.clone();
        assert_eq!(result.id, cloned.id);
        assert_eq!(result.score, cloned.score);
    }

    #[test]
    fn test_search_result_serialization() {
        let result = SearchResult {
            id: "result-abc".to_string(),
            score: 0.75,
            metadata: Some(serde_json::json!({"source": "test"})),
            vector: None,
        };

        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["id"], "result-abc");
        assert_eq!(json["score"], 0.75);
    }

    #[test]
    fn test_search_result_zero_score() {
        let result = SearchResult {
            id: "zero-score".to_string(),
            score: 0.0,
            metadata: None,
            vector: None,
        };

        assert_eq!(result.score, 0.0);
    }

    #[test]
    fn test_search_result_perfect_score() {
        let result = SearchResult {
            id: "perfect".to_string(),
            score: 1.0,
            metadata: None,
            vector: None,
        };

        assert_eq!(result.score, 1.0);
    }

    // ==================== VectorPoint Tests ====================

    #[test]
    fn test_vector_point_structure() {
        let point = VectorPoint {
            id: "point-123".to_string(),
            vector: vec![0.1, 0.2, 0.3],
            metadata: Some(serde_json::json!({"label": "test"})),
        };

        assert_eq!(point.id, "point-123");
        assert_eq!(point.vector.len(), 3);
        assert!(point.metadata.is_some());
    }

    #[test]
    fn test_vector_point_no_metadata() {
        let point = VectorPoint {
            id: "point-456".to_string(),
            vector: vec![1.0, 2.0],
            metadata: None,
        };

        assert!(point.metadata.is_none());
    }

    #[test]
    fn test_vector_point_clone() {
        let point = VectorPoint {
            id: "point-789".to_string(),
            vector: vec![0.5],
            metadata: None,
        };

        let cloned = point.clone();
        assert_eq!(point.id, cloned.id);
        assert_eq!(point.vector, cloned.vector);
    }

    #[test]
    fn test_vector_point_serialization() {
        let point = VectorPoint {
            id: "point-abc".to_string(),
            vector: vec![0.1, 0.2],
            metadata: Some(serde_json::json!({"key": "value"})),
        };

        let json = serde_json::to_value(&point).unwrap();
        assert_eq!(json["id"], "point-abc");
        assert!(json["vector"].is_array());
    }

    // ==================== Deserialization Tests ====================

    #[test]
    fn test_vector_data_deserialization() {
        let json = r#"{
            "id": "deser-123",
            "vector": [0.1, 0.2, 0.3],
            "metadata": {"type": "test"}
        }"#;

        let data: VectorData = serde_json::from_str(json).unwrap();
        assert_eq!(data.id, "deser-123");
        assert_eq!(data.vector.len(), 3);
        assert!(data.metadata.contains_key("type"));
    }

    #[test]
    fn test_search_result_deserialization() {
        let json = r#"{
            "id": "deser-456",
            "score": 0.85,
            "metadata": null,
            "vector": null
        }"#;

        let result: SearchResult = serde_json::from_str(json).unwrap();
        assert_eq!(result.id, "deser-456");
        assert!((result.score - 0.85).abs() < f32::EPSILON);
    }

    #[test]
    fn test_vector_point_deserialization() {
        let json = r#"{
            "id": "deser-789",
            "vector": [1.0, 2.0, 3.0],
            "metadata": {"index": 0}
        }"#;

        let point: VectorPoint = serde_json::from_str(json).unwrap();
        assert_eq!(point.id, "deser-789");
        assert_eq!(point.vector.len(), 3);
    }
}
