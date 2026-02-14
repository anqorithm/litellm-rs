//! Tests for vector store

#[cfg(test)]
use super::types::{SearchResult, VectorPoint};

#[test]
fn test_vector_point_creation() {
    let point = VectorPoint {
        id: "test-1".to_string(),
        vector: vec![0.1, 0.2, 0.3],
        metadata: Some(serde_json::json!({"text": "test document"})),
    };

    assert_eq!(point.id, "test-1");
    assert_eq!(point.vector.len(), 3);
    assert!(point.metadata.is_some());
}

#[test]
fn test_search_result_creation() {
    let result = SearchResult {
        id: "result-1".to_string(),
        score: 0.95,
        metadata: Some(serde_json::json!({"category": "test"})),
        vector: None,
    };

    assert_eq!(result.id, "result-1");
    assert_eq!(result.score, 0.95);
    assert!(result.metadata.is_some());
    assert!(result.vector.is_none());
}
