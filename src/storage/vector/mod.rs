//! Vector database implementation
//!
//! This module provides vector storage and similarity search functionality.

mod backend;
mod pinecone;
mod qdrant;
#[cfg(test)]
mod tests;
mod types;
mod weaviate;

// Re-export public types and traits
pub use backend::VectorStoreBackend;
pub use pinecone::PineconeStore;
pub use qdrant::QdrantStore;
pub use types::{SearchResult, VectorData, VectorPoint, VectorStore};
pub use weaviate::WeaviateStore;
