//! Module
//!
//! Contains base components shared by all providers

pub mod config;
pub mod connection_pool;
pub mod http;
pub mod pricing;
pub mod sse;

pub use config::BaseConfig;
pub use connection_pool::{
    ConnectionPool, GlobalPoolManager, HeaderPair, HttpMethod, PoolConfig, global_client, header,
    header_owned, streaming_client,
};
pub use http::create_http_client;
pub use pricing::{PricingDatabase, get_pricing_db};
pub use sse::{
    AnthropicTransformer, CohereTransformer, DatabricksTransformer, GeminiTransformer,
    OpenAICompatibleTransformer, SSEEvent, SSEEventType, SSETransformer, UnifiedSSEParser,
    UnifiedSSEStream,
};
