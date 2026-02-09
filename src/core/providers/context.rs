//! Request/Response Context - Execution context and metadata
//!
//! This module provides context objects that carry metadata, configuration,
//! and runtime information throughout the provider execution pipeline.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Instant, SystemTime};

use super::ProviderType;

/// Context for request execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    /// Unique request identifier for tracing
    pub request_id: String,

    /// User identifier for authentication/authorization
    pub user_id: Option<String>,

    /// API key or token used for the request
    pub api_key_id: Option<String>,

    /// Original request timestamp
    pub timestamp: SystemTime,

    /// Request execution start time (for latency measurement)
    #[serde(skip, default = "Instant::now")]
    pub start_time: Instant,

    /// Provider configuration overrides
    pub config_overrides: HashMap<String, serde_json::Value>,

    /// Request headers from client
    pub headers: HashMap<String, String>,

    /// Query parameters
    pub query_params: HashMap<String, String>,

    /// IP address of the requesting client
    pub client_ip: Option<String>,

    /// User agent string
    pub user_agent: Option<String>,

    /// Rate limiting information
    pub rate_limit: Option<RateLimitContext>,

    /// Cost tracking information
    pub cost_context: Option<CostContext>,

    /// Security context
    pub security_context: SecurityContext,

    /// Routing context
    pub routing_context: RoutingContext,

    /// Custom metadata
    pub metadata: HashMap<String, String>,

    /// Request priority (0-255, higher = more priority)
    pub priority: u8,

    /// Maximum allowed execution time
    pub timeout_ms: Option<u64>,

    /// Whether to enable detailed logging for this request
    pub debug_mode: bool,
}

/// Context for response processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseContext {
    /// Associated request context
    pub request_context: RequestContext,

    /// Response timestamp
    pub timestamp: SystemTime,

    /// Total execution time
    pub execution_time_ms: f64,

    /// Provider that handled the request
    pub provider_id: String,

    /// Provider type used
    pub provider_type: ProviderType,

    /// Whether request was served from cache
    pub from_cache: bool,

    /// Cache hit information
    pub cache_info: Option<CacheInfo>,

    /// Retry information
    pub retry_info: Option<RetryInfo>,

    /// Cost information
    pub cost_info: Option<CostInfo>,

    /// Performance metrics
    pub metrics: ResponseMetrics,

    /// Any warnings generated during processing
    pub warnings: Vec<String>,

    /// Error information if request failed
    pub error_info: Option<ErrorInfo>,
}

/// Rate limiting context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitContext {
    /// Rate limit key (usually user_id or api_key_id)
    pub key: String,

    /// Remaining requests in current window
    pub remaining_requests: u32,

    /// Total requests allowed in window
    pub limit: u32,

    /// Window reset time
    pub reset_time: SystemTime,

    /// Window duration in seconds
    pub window_seconds: u64,
}

/// Cost tracking context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostContext {
    /// Budget key (user_id, team_id, etc.)
    pub budget_key: String,

    /// Remaining budget
    pub remaining_budget: f64,

    /// Total budget for the period
    pub total_budget: f64,

    /// Budget currency
    pub currency: String,

    /// Budget period end time
    pub period_end: SystemTime,
}

/// Security context for request validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Whether request passed authentication
    pub authenticated: bool,

    /// User roles and permissions
    pub roles: Vec<String>,

    /// Allowed models for this user
    pub allowed_models: Vec<String>,

    /// Content filtering level
    pub content_filter_level: ContentFilterLevel,

    /// Whether PII detection is enabled
    pub pii_detection_enabled: bool,

    /// Security audit tags
    pub audit_tags: Vec<String>,
}

/// Content filtering levels
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum ContentFilterLevel {
    None,
    Low,
    #[default]
    Medium,
    High,
    Strict,
}

/// Routing context for load balancing decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingContext {
    /// Requested provider (if specified)
    pub preferred_provider: Option<ProviderType>,

    /// Routing strategy to use
    pub strategy: RoutingStrategy,

    /// Fallback providers in order of preference
    pub fallback_providers: Vec<ProviderType>,

    /// Geographic region preference
    pub region_preference: Option<String>,

    /// Whether to allow degraded providers
    pub allow_degraded: bool,

    /// Maximum acceptable latency (ms)
    pub max_latency_ms: Option<f64>,
}

/// Routing strategies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub enum RoutingStrategy {
    #[default]
    RoundRobin,
    LeastLatency,
    LeastBusy,
    CostOptimized,
    HealthBased,
    Weighted,
    Random,
}

/// Cache information for responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheInfo {
    /// Cache key used
    pub cache_key: String,

    /// Cache tier that served the response
    pub cache_tier: CacheTier,

    /// Cache hit/miss status
    pub hit: bool,

    /// Time to live remaining (seconds)
    pub ttl_remaining: Option<u64>,

    /// Size of cached response (bytes)
    pub size_bytes: Option<u64>,
}

/// Cache tiers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheTier {
    Memory,
    Redis,
    Database,
    ObjectStorage,
    Semantic,
}

/// Retry attempt information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryInfo {
    /// Number of retry attempts made
    pub attempts: u32,

    /// Maximum retries allowed
    pub max_attempts: u32,

    /// Providers tried in order
    pub providers_tried: Vec<String>,

    /// Errors encountered during retries
    pub retry_errors: Vec<String>,

    /// Total retry delay time (ms)
    pub total_retry_delay_ms: f64,
}

/// Cost information for billing/tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostInfo {
    /// Provider cost calculation
    pub provider_cost: f64,

    /// Currency of the cost
    pub currency: String,

    /// Input tokens consumed
    pub input_tokens: u32,

    /// Output tokens generated
    pub output_tokens: u32,

    /// Cost breakdown by component
    pub cost_breakdown: HashMap<String, f64>,

    /// Cost estimation vs actual
    pub estimated_cost: Option<f64>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetrics {
    /// Time spent on authentication (ms)
    pub auth_time_ms: f64,

    /// Time spent on routing/load balancing (ms)
    pub routing_time_ms: f64,

    /// Time spent on request transformation (ms)
    pub transform_request_time_ms: f64,

    /// Time spent calling provider (ms)
    pub provider_call_time_ms: f64,

    /// Time spent on response transformation (ms)
    pub transform_response_time_ms: f64,

    /// Time spent on caching operations (ms)
    pub cache_time_ms: f64,

    /// Queue wait time (ms)
    pub queue_wait_time_ms: f64,

    /// Total time from start to finish (ms)
    pub total_time_ms: f64,

    /// First byte time from provider (ms)
    pub first_byte_time_ms: Option<f64>,

    /// Tokens per second (for streaming)
    pub tokens_per_second: Option<f64>,
}

/// Error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorInfo {
    /// Error code
    pub error_code: String,

    /// Human-readable error message
    pub message: String,

    /// Technical error details
    pub details: Option<String>,

    /// HTTP status code (if applicable)
    pub http_status: Option<u16>,

    /// Provider-specific error code
    pub provider_error_code: Option<String>,

    /// Whether error is retryable
    pub retryable: bool,

    /// Error category
    pub category: ErrorCategory,
}

/// Error categories for better error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    Authentication,
    Authorization,
    RateLimit,
    Validation,
    Provider,
    Network,
    Timeout,
    Internal,
    Configuration,
    Cost,
}

impl RequestContext {
    /// Create a new request context
    pub fn new(request_id: String) -> Self {
        Self {
            request_id,
            user_id: None,
            api_key_id: None,
            timestamp: SystemTime::now(),
            start_time: Instant::now(),
            config_overrides: HashMap::new(),
            headers: HashMap::new(),
            query_params: HashMap::new(),
            client_ip: None,
            user_agent: None,
            rate_limit: None,
            cost_context: None,
            security_context: SecurityContext::default(),
            routing_context: RoutingContext::default(),
            metadata: HashMap::new(),
            priority: 128, // medium priority
            timeout_ms: None,
            debug_mode: false,
        }
    }

    /// Get elapsed time since request started
    pub fn elapsed_ms(&self) -> f64 {
        self.start_time.elapsed().as_millis() as f64
    }

    /// Check if request has timed out
    pub fn is_timed_out(&self) -> bool {
        if let Some(timeout_ms) = self.timeout_ms {
            self.elapsed_ms() > timeout_ms as f64
        } else {
            false
        }
    }

    /// Add metadata entry
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }

    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
}

impl ResponseContext {
    /// Create response context from request context
    pub fn from_request(
        request_context: RequestContext,
        provider_id: String,
        provider_type: ProviderType,
    ) -> Self {
        let execution_time_ms = request_context.elapsed_ms();

        Self {
            request_context,
            timestamp: SystemTime::now(),
            execution_time_ms,
            provider_id,
            provider_type,
            from_cache: false,
            cache_info: None,
            retry_info: None,
            cost_info: None,
            metrics: ResponseMetrics::default(),
            warnings: Vec::new(),
            error_info: None,
        }
    }

    /// Add a warning message
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Set error information
    pub fn set_error(&mut self, error_info: ErrorInfo) {
        self.error_info = Some(error_info);
    }

    /// Check if response has errors
    pub fn has_error(&self) -> bool {
        self.error_info.is_some()
    }
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            authenticated: false,
            roles: Vec::new(),
            allowed_models: Vec::new(),
            content_filter_level: ContentFilterLevel::Medium,
            pii_detection_enabled: true,
            audit_tags: Vec::new(),
        }
    }
}

impl Default for RoutingContext {
    fn default() -> Self {
        Self {
            preferred_provider: None,
            strategy: RoutingStrategy::RoundRobin,
            fallback_providers: Vec::new(),
            region_preference: None,
            allow_degraded: false,
            max_latency_ms: None,
        }
    }
}

impl Default for ResponseMetrics {
    fn default() -> Self {
        Self {
            auth_time_ms: 0.0,
            routing_time_ms: 0.0,
            transform_request_time_ms: 0.0,
            provider_call_time_ms: 0.0,
            transform_response_time_ms: 0.0,
            cache_time_ms: 0.0,
            queue_wait_time_ms: 0.0,
            total_time_ms: 0.0,
            first_byte_time_ms: None,
            tokens_per_second: None,
        }
    }
}


#[cfg(test)]
#[path = "context_tests.rs"]
mod tests;
