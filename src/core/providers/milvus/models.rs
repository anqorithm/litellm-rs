//! Milvus Model Information
//!
//! This module defines the embedding models and vector operations supported by Milvus.
//! Milvus is primarily a vector database, so the "models" here represent different
//! embedding dimensions and index types supported for vector storage and retrieval.
//!
//! Reference: <https://milvus.io/docs/index.md>

/// Supported metric types for similarity search in Milvus
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MetricType {
    /// Euclidean distance (L2)
    #[default]
    L2,
    /// Inner product
    IP,
    /// Cosine similarity
    Cosine,
    /// Hamming distance (for binary vectors)
    Hamming,
    /// Jaccard distance (for binary vectors)
    Jaccard,
}

impl MetricType {
    /// Convert to Milvus API string
    pub fn as_str(&self) -> &'static str {
        match self {
            MetricType::L2 => "L2",
            MetricType::IP => "IP",
            MetricType::Cosine => "COSINE",
            MetricType::Hamming => "HAMMING",
            MetricType::Jaccard => "JACCARD",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "L2" | "EUCLIDEAN" => Some(MetricType::L2),
            "IP" | "INNER_PRODUCT" => Some(MetricType::IP),
            "COSINE" => Some(MetricType::Cosine),
            "HAMMING" => Some(MetricType::Hamming),
            "JACCARD" => Some(MetricType::Jaccard),
            _ => None,
        }
    }
}

/// Supported index types for vector storage
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum IndexType {
    /// Flat index (brute force, exact search)
    Flat,
    /// IVF Flat
    IvfFlat,
    /// IVF with Scalar Quantization
    IvfSq8,
    /// IVF with Product Quantization
    IvfPq,
    /// Hierarchical Navigable Small World graph
    #[default]
    Hnsw,
    /// Approximate Nearest Neighbor OH Yeah
    Annoy,
    /// Disk-based Approximate Nearest Neighbor
    DiskAnn,
}

impl IndexType {
    /// Convert to Milvus API string
    pub fn as_str(&self) -> &'static str {
        match self {
            IndexType::Flat => "FLAT",
            IndexType::IvfFlat => "IVF_FLAT",
            IndexType::IvfSq8 => "IVF_SQ8",
            IndexType::IvfPq => "IVF_PQ",
            IndexType::Hnsw => "HNSW",
            IndexType::Annoy => "ANNOY",
            IndexType::DiskAnn => "DISKANN",
        }
    }

    /// Parse from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_uppercase().as_str() {
            "FLAT" => Some(IndexType::Flat),
            "IVF_FLAT" => Some(IndexType::IvfFlat),
            "IVF_SQ8" => Some(IndexType::IvfSq8),
            "IVF_PQ" => Some(IndexType::IvfPq),
            "HNSW" => Some(IndexType::Hnsw),
            "ANNOY" => Some(IndexType::Annoy),
            "DISKANN" => Some(IndexType::DiskAnn),
            _ => None,
        }
    }
}

/// Milvus embedding model information
///
/// Since Milvus is a vector database (not an embedding model provider),
/// this represents common embedding dimensions from popular models that
/// can be stored in Milvus.
#[derive(Debug, Clone)]
pub struct MilvusEmbeddingModel {
    /// Model identifier (e.g., "openai-ada-002", "bge-base")
    pub model_id: &'static str,
    /// Display name for the model
    pub display_name: &'static str,
    /// Embedding dimension size
    pub dimensions: u32,
    /// Recommended metric type for this embedding
    pub recommended_metric: MetricType,
    /// Description of the model
    pub description: &'static str,
}

/// Common embedding models that can be stored in Milvus
const MILVUS_EMBEDDING_MODELS: &[MilvusEmbeddingModel] = &[
    // OpenAI models
    MilvusEmbeddingModel {
        model_id: "text-embedding-ada-002",
        display_name: "OpenAI Ada 002",
        dimensions: 1536,
        recommended_metric: MetricType::Cosine,
        description: "OpenAI's second generation embedding model",
    },
    MilvusEmbeddingModel {
        model_id: "text-embedding-3-small",
        display_name: "OpenAI Embedding 3 Small",
        dimensions: 1536,
        recommended_metric: MetricType::Cosine,
        description: "OpenAI's compact third generation embedding model",
    },
    MilvusEmbeddingModel {
        model_id: "text-embedding-3-large",
        display_name: "OpenAI Embedding 3 Large",
        dimensions: 3072,
        recommended_metric: MetricType::Cosine,
        description: "OpenAI's large third generation embedding model",
    },
    // Voyage models
    MilvusEmbeddingModel {
        model_id: "voyage-3",
        display_name: "Voyage 3",
        dimensions: 1024,
        recommended_metric: MetricType::Cosine,
        description: "Voyage AI's latest embedding model",
    },
    MilvusEmbeddingModel {
        model_id: "voyage-3-lite",
        display_name: "Voyage 3 Lite",
        dimensions: 512,
        recommended_metric: MetricType::Cosine,
        description: "Voyage AI's compact embedding model",
    },
    // Cohere models
    MilvusEmbeddingModel {
        model_id: "embed-english-v3.0",
        display_name: "Cohere Embed English v3",
        dimensions: 1024,
        recommended_metric: MetricType::Cosine,
        description: "Cohere's English embedding model",
    },
    MilvusEmbeddingModel {
        model_id: "embed-multilingual-v3.0",
        display_name: "Cohere Embed Multilingual v3",
        dimensions: 1024,
        recommended_metric: MetricType::Cosine,
        description: "Cohere's multilingual embedding model",
    },
    // BGE models (open source)
    MilvusEmbeddingModel {
        model_id: "bge-small-en-v1.5",
        display_name: "BGE Small English",
        dimensions: 384,
        recommended_metric: MetricType::Cosine,
        description: "BAAI General Embedding small model",
    },
    MilvusEmbeddingModel {
        model_id: "bge-base-en-v1.5",
        display_name: "BGE Base English",
        dimensions: 768,
        recommended_metric: MetricType::Cosine,
        description: "BAAI General Embedding base model",
    },
    MilvusEmbeddingModel {
        model_id: "bge-large-en-v1.5",
        display_name: "BGE Large English",
        dimensions: 1024,
        recommended_metric: MetricType::Cosine,
        description: "BAAI General Embedding large model",
    },
    // Sentence Transformers
    MilvusEmbeddingModel {
        model_id: "all-MiniLM-L6-v2",
        display_name: "MiniLM L6 v2",
        dimensions: 384,
        recommended_metric: MetricType::Cosine,
        description: "Compact sentence transformer model",
    },
    MilvusEmbeddingModel {
        model_id: "all-mpnet-base-v2",
        display_name: "MPNet Base v2",
        dimensions: 768,
        recommended_metric: MetricType::Cosine,
        description: "High quality sentence transformer model",
    },
    // Custom/generic dimensions
    MilvusEmbeddingModel {
        model_id: "custom-128",
        display_name: "Custom 128D",
        dimensions: 128,
        recommended_metric: MetricType::L2,
        description: "Custom 128-dimensional vectors",
    },
    MilvusEmbeddingModel {
        model_id: "custom-256",
        display_name: "Custom 256D",
        dimensions: 256,
        recommended_metric: MetricType::L2,
        description: "Custom 256-dimensional vectors",
    },
    MilvusEmbeddingModel {
        model_id: "custom-512",
        display_name: "Custom 512D",
        dimensions: 512,
        recommended_metric: MetricType::L2,
        description: "Custom 512-dimensional vectors",
    },
    MilvusEmbeddingModel {
        model_id: "custom-768",
        display_name: "Custom 768D",
        dimensions: 768,
        recommended_metric: MetricType::L2,
        description: "Custom 768-dimensional vectors",
    },
    MilvusEmbeddingModel {
        model_id: "custom-1024",
        display_name: "Custom 1024D",
        dimensions: 1024,
        recommended_metric: MetricType::L2,
        description: "Custom 1024-dimensional vectors",
    },
    MilvusEmbeddingModel {
        model_id: "custom-1536",
        display_name: "Custom 1536D",
        dimensions: 1536,
        recommended_metric: MetricType::L2,
        description: "Custom 1536-dimensional vectors",
    },
];

/// Get all available embedding model IDs
pub fn get_available_models() -> Vec<&'static str> {
    MILVUS_EMBEDDING_MODELS.iter().map(|m| m.model_id).collect()
}

/// Get embedding model information by ID
pub fn get_model_info(model_id: &str) -> Option<&'static MilvusEmbeddingModel> {
    // Try exact match first
    if let Some(model) = MILVUS_EMBEDDING_MODELS
        .iter()
        .find(|m| m.model_id == model_id)
    {
        return Some(model);
    }

    // Try normalized match (remove common prefixes)
    let normalized = normalize_model_id(model_id);
    MILVUS_EMBEDDING_MODELS.iter().find(|m| {
        m.model_id == normalized || m.display_name.to_lowercase() == normalized.to_lowercase()
    })
}

/// Normalize model ID by removing common prefixes
fn normalize_model_id(model_id: &str) -> &str {
    model_id
        .trim_start_matches("milvus/")
        .trim_start_matches("openai/")
        .trim_start_matches("voyage/")
        .trim_start_matches("cohere/")
}

/// Get default embedding model
#[cfg(test)]
pub fn get_default_model() -> &'static str {
    "text-embedding-ada-002"
}

/// Get model dimensions
pub fn get_model_dimensions(model_id: &str) -> Option<u32> {
    get_model_info(model_id).map(|m| m.dimensions)
}

/// Get recommended metric type for a model
pub fn get_recommended_metric(model_id: &str) -> MetricType {
    get_model_info(model_id)
        .map(|m| m.recommended_metric)
        .unwrap_or_default()
}

/// Check if a model ID is recognized
pub fn is_known_model(model_id: &str) -> bool {
    get_model_info(model_id).is_some()
}

/// Supported consistency levels for Milvus operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConsistencyLevel {
    /// Strong consistency
    Strong,
    /// Session consistency
    Session,
    /// Bounded staleness
    #[default]
    Bounded,
    /// Eventually consistent
    Eventually,
}

impl ConsistencyLevel {
    /// Convert to Milvus API string
    pub fn as_str(&self) -> &'static str {
        match self {
            ConsistencyLevel::Strong => "Strong",
            ConsistencyLevel::Session => "Session",
            ConsistencyLevel::Bounded => "Bounded",
            ConsistencyLevel::Eventually => "Eventually",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_available_models() {
        let models = get_available_models();
        assert!(!models.is_empty());
        assert!(models.contains(&"text-embedding-ada-002"));
        assert!(models.contains(&"voyage-3"));
        assert!(models.contains(&"bge-base-en-v1.5"));
    }

    #[test]
    fn test_get_model_info() {
        let model = get_model_info("text-embedding-ada-002").unwrap();
        assert_eq!(model.display_name, "OpenAI Ada 002");
        assert_eq!(model.dimensions, 1536);
        assert_eq!(model.recommended_metric, MetricType::Cosine);
    }

    #[test]
    fn test_get_model_info_with_prefix() {
        let model = get_model_info("openai/text-embedding-ada-002");
        assert!(model.is_some());
    }

    #[test]
    fn test_get_model_info_not_found() {
        let model = get_model_info("unknown-model");
        assert!(model.is_none());
    }

    #[test]
    fn test_get_default_model() {
        assert_eq!(get_default_model(), "text-embedding-ada-002");
    }

    #[test]
    fn test_get_model_dimensions() {
        assert_eq!(get_model_dimensions("text-embedding-ada-002"), Some(1536));
        assert_eq!(get_model_dimensions("voyage-3"), Some(1024));
        assert_eq!(get_model_dimensions("bge-small-en-v1.5"), Some(384));
        assert_eq!(get_model_dimensions("unknown"), None);
    }

    #[test]
    fn test_metric_type() {
        assert_eq!(MetricType::L2.as_str(), "L2");
        assert_eq!(MetricType::IP.as_str(), "IP");
        assert_eq!(MetricType::Cosine.as_str(), "COSINE");

        assert_eq!(MetricType::parse("L2"), Some(MetricType::L2));
        assert_eq!(MetricType::parse("COSINE"), Some(MetricType::Cosine));
        assert_eq!(MetricType::parse("invalid"), None);
    }

    #[test]
    fn test_index_type() {
        assert_eq!(IndexType::Hnsw.as_str(), "HNSW");
        assert_eq!(IndexType::Flat.as_str(), "FLAT");
        assert_eq!(IndexType::IvfFlat.as_str(), "IVF_FLAT");

        assert_eq!(IndexType::parse("HNSW"), Some(IndexType::Hnsw));
        assert_eq!(IndexType::parse("IVF_FLAT"), Some(IndexType::IvfFlat));
        assert_eq!(IndexType::parse("invalid"), None);
    }

    #[test]
    fn test_consistency_level() {
        assert_eq!(ConsistencyLevel::Strong.as_str(), "Strong");
        assert_eq!(ConsistencyLevel::Eventually.as_str(), "Eventually");
        assert_eq!(ConsistencyLevel::default(), ConsistencyLevel::Bounded);
    }

    #[test]
    fn test_is_known_model() {
        assert!(is_known_model("text-embedding-ada-002"));
        assert!(is_known_model("voyage-3"));
        assert!(!is_known_model("unknown-model"));
    }

    #[test]
    fn test_get_recommended_metric() {
        assert_eq!(
            get_recommended_metric("text-embedding-ada-002"),
            MetricType::Cosine
        );
        assert_eq!(get_recommended_metric("custom-768"), MetricType::L2);
        // Unknown model defaults to L2
        assert_eq!(get_recommended_metric("unknown"), MetricType::L2);
    }

    #[test]
    fn test_custom_dimension_models() {
        let dims = [128, 256, 512, 768, 1024, 1536];
        for dim in dims {
            let model_id = format!("custom-{}", dim);
            let model = get_model_info(&model_id).unwrap();
            assert_eq!(model.dimensions, dim);
        }
    }
}
