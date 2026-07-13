use std::borrow::Cow;
use std::sync::Arc;
use std::sync::LazyLock;
use std::time::Duration;

use futures::StreamExt;
use reqwest::{Client, RequestBuilder, Response};
use serde_json;

use crate::core::net::{ProviderEndpointAccess, ProviderEndpointPolicy};
use crate::core::providers::unified_provider::ProviderError;
use crate::utils::net::http::{
    HttpClientPoolConfig, ProviderHttpClient, ProviderRequestBuilder,
    create_custom_client_with_config, create_streaming_client,
};

/// HTTP headers using `Cow` to avoid allocations for static strings.
pub type HeaderPair = (Cow<'static, str>, Cow<'static, str>);

/// Helper to create a header from static key and dynamic value.
#[inline]
pub fn header(key: &'static str, value: String) -> HeaderPair {
    (Cow::Borrowed(key), Cow::Owned(value))
}

/// Helper to create a header from both static key and static value (zero allocation).
#[inline]
pub fn header_static(key: &'static str, value: &'static str) -> HeaderPair {
    (Cow::Borrowed(key), Cow::Borrowed(value))
}

/// Helper to create a header from both dynamic key and value.
#[inline]
pub fn header_owned(key: String, value: String) -> HeaderPair {
    (Cow::Owned(key), Cow::Owned(value))
}

/// Apply a list of `HeaderPair`s to a `reqwest::RequestBuilder`.
///
/// This bridges the `Vec<HeaderPair>` pattern with providers that still use
/// `reqwest::Client` directly instead of `GlobalPoolManager`.
#[inline]
pub fn apply_headers(
    mut builder: reqwest::RequestBuilder,
    headers: Vec<HeaderPair>,
) -> reqwest::RequestBuilder {
    for (key, value) in headers {
        builder = builder.header(key.as_ref(), value.as_ref());
    }
    builder
}

#[derive(Debug, Clone, Copy)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
}

/// Unified connection pool configuration
pub struct PoolConfig;
impl PoolConfig {
    pub const TIMEOUT_SECS: u64 = 600;
    pub const POOL_SIZE: usize = 80;
    pub const KEEPALIVE_SECS: u64 = 90;
}

pub const STREAMING_HEADER_TIMEOUT_SECS: u64 = PoolConfig::TIMEOUT_SECS;
pub const STREAMING_ERROR_BODY_TIMEOUT_SECS: u64 = 10;
pub const STREAMING_ERROR_BODY_MAX_BYTES: usize = 64 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum StreamingRequestError {
    #[error("streaming request did not receive response headers within {timeout:?}")]
    HeaderTimeout { timeout: Duration },
    #[error("streaming error body did not finish within {timeout:?}")]
    ErrorBodyTimeout { timeout: Duration },
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

impl StreamingRequestError {
    pub fn is_timeout(&self) -> bool {
        match self {
            Self::HeaderTimeout { .. } | Self::ErrorBodyTimeout { .. } => true,
            Self::Request(err) => err.is_timeout(),
        }
    }

    pub fn as_reqwest_error(&self) -> Option<&reqwest::Error> {
        match self {
            Self::HeaderTimeout { .. } | Self::ErrorBodyTimeout { .. } => None,
            Self::Request(err) => Some(err),
        }
    }

    pub fn into_provider_error(self, provider: &'static str) -> ProviderError {
        if self.is_timeout() {
            ProviderError::timeout(provider, self.to_string())
        } else {
            ProviderError::network(provider, self.to_string())
        }
    }
}

#[inline]
fn pool_http_config() -> HttpClientPoolConfig {
    HttpClientPoolConfig {
        pool_max_idle_per_host: PoolConfig::POOL_SIZE,
        pool_idle_timeout: Duration::from_secs(PoolConfig::KEEPALIVE_SECS),
        ..HttpClientPoolConfig::default()
    }
}

/// Global HTTP client singleton.
static GLOBAL_CLIENT: LazyLock<Arc<Client>> = LazyLock::new(|| {
    let client = create_custom_client_with_config(
        Duration::from_secs(PoolConfig::TIMEOUT_SECS),
        &pool_http_config(),
    )
    .unwrap_or_else(|e| {
        tracing::error!("Failed to create global HTTP client: {}", e);
        crate::core::http::outbound::default_outbound_client().clone()
    });
    Arc::new(client)
});

static STREAMING_CLIENT: LazyLock<Arc<Client>> = LazyLock::new(|| {
    let client = create_streaming_client().unwrap_or_else(|err| {
        tracing::error!("Failed to create streaming HTTP client: {err}");
        crate::core::http::outbound::default_outbound_client().clone()
    });
    Arc::new(client)
});

/// Get the global HTTP client
///
/// Returns a reference to the shared global HTTP client instance.
/// This is the preferred way to access the HTTP client for connection pooling.
#[inline]
pub fn global_client() -> Arc<Client> {
    Arc::clone(&GLOBAL_CLIENT)
}

/// Get the legacy bounded streaming client for providers not yet migrated.
#[inline]
pub fn streaming_client() -> Arc<Client> {
    global_client()
}

/// Get the legacy unbounded streaming client for providers not yet migrated.
#[inline]
pub fn streaming_unbounded_client() -> Arc<Client> {
    Arc::clone(&STREAMING_CLIENT)
}

/// Send a streaming request with a bounded pre-header phase.
///
/// The timeout wraps only `RequestBuilder::send()`, which completes when
/// response headers arrive. It does not impose a total timeout on the response
/// body stream.
pub async fn send_streaming_request(
    request_builder: RequestBuilder,
    provider: &'static str,
) -> Result<Response, ProviderError> {
    send_streaming_request_with_timeout(
        request_builder,
        Duration::from_secs(STREAMING_HEADER_TIMEOUT_SECS),
    )
    .await
    .map_err(|err| err.into_provider_error(provider))
}

pub async fn send_streaming_request_with_timeout(
    request_builder: RequestBuilder,
    timeout: Duration,
) -> Result<Response, StreamingRequestError> {
    match tokio::time::timeout(timeout, request_builder.send()).await {
        Ok(Ok(response)) => Ok(response),
        Ok(Err(err)) => Err(StreamingRequestError::Request(err)),
        Err(_) => Err(StreamingRequestError::HeaderTimeout { timeout }),
    }
}

/// Read a non-success streaming response body with bounded time and memory.
///
/// Successful SSE bodies must stay unbounded, but error bodies are finite
/// diagnostics. This prevents a provider that sends 4xx/5xx headers and then
/// stalls the body from tying up the task forever.
pub async fn read_streaming_error_body(
    response: Response,
) -> Result<String, StreamingRequestError> {
    read_streaming_error_body_with_limits(
        response,
        Duration::from_secs(STREAMING_ERROR_BODY_TIMEOUT_SECS),
        STREAMING_ERROR_BODY_MAX_BYTES,
    )
    .await
}

pub async fn read_streaming_error_body_with_limits(
    response: Response,
    timeout: Duration,
    max_bytes: usize,
) -> Result<String, StreamingRequestError> {
    if max_bytes == 0 {
        return Ok(String::new());
    }

    let bytes = tokio::time::timeout(timeout, async move {
        let mut stream = response.bytes_stream();
        let mut body = Vec::new();

        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            let remaining = max_bytes.saturating_sub(body.len());
            if remaining == 0 {
                break;
            }

            if chunk.len() > remaining {
                body.extend_from_slice(&chunk[..remaining]);
                break;
            }

            body.extend_from_slice(&chunk);
            if body.len() >= max_bytes {
                break;
            }
        }

        Ok::<_, reqwest::Error>(body)
    })
    .await
    .map_err(|_| StreamingRequestError::ErrorBodyTimeout { timeout })??;

    let body = String::from_utf8(bytes)
        .unwrap_or_else(|err| String::from_utf8_lossy(err.as_bytes()).into_owned());
    Ok(body)
}

/// Simplified connection pool without generic complexity
#[derive(Debug, Clone)]
pub struct ConnectionPool {
    client: Arc<Client>,
}

impl ConnectionPool {
    /// Create a new connection pool with optimized settings
    ///
    /// Note: This now uses the global client singleton instead of creating a new client.
    /// For true isolation (rare use cases), use `new_isolated()`.
    pub fn new() -> Result<Self, ProviderError> {
        Ok(Self {
            client: global_client(),
        })
    }

    /// Create an isolated connection pool with its own client
    ///
    /// Use this only when you need a separate connection pool from the global one.
    /// Most use cases should use `new()` which shares the global client.
    pub fn new_isolated() -> Result<Self, ProviderError> {
        let client = create_custom_client_with_config(
            Duration::from_secs(PoolConfig::TIMEOUT_SECS),
            &pool_http_config(),
        )
        .map_err(|e| ProviderError::configuration("Failed to create HTTP client", e.to_string()))?;

        Ok(Self {
            client: Arc::new(client),
        })
    }

    /// Get the underlying reqwest client
    pub fn client(&self) -> &Client {
        &self.client
    }
}

#[derive(Debug, Clone)]
struct PolicyClients {
    ordinary: ProviderHttpClient,
    streaming: ProviderHttpClient,
    health: ProviderHttpClient,
}

/// Shared pool manager with an optional provider-scoped endpoint policy.
#[derive(Debug, Clone)]
pub struct GlobalPoolManager {
    pool: Arc<ConnectionPool>,
    policy_clients: Option<PolicyClients>,
    provider: &'static str,
}

impl GlobalPoolManager {
    /// Create a manager backed by the global client.
    pub fn new() -> Result<Self, ProviderError> {
        Ok(Self {
            pool: Arc::new(ConnectionPool::new()?),
            policy_clients: None,
            provider: "common",
        })
    }

    /// Get a manager backed by the global client.
    pub fn shared() -> Self {
        Self {
            pool: Arc::new(ConnectionPool {
                client: global_client(),
            }),
            policy_clients: None,
            provider: "common",
        }
    }

    /// Create a manager that enforces one provider endpoint policy on every path.
    pub fn for_provider_endpoint(
        provider: &'static str,
        api_base: &str,
        endpoint_access: ProviderEndpointAccess,
        timeout: Duration,
    ) -> Result<Self, ProviderError> {
        let policy = ProviderEndpointPolicy::for_base_url(endpoint_access, api_base)
            .map_err(|error| ProviderError::configuration(provider, error.to_string()))?;
        let map_error = |error: crate::utils::net::http::ProviderHttpClientError| {
            ProviderError::initialization(provider, error.to_string())
        };
        let policy_clients = PolicyClients {
            ordinary: ProviderHttpClient::new(policy.clone(), timeout).map_err(map_error)?,
            streaming: ProviderHttpClient::streaming(policy.clone()).map_err(map_error)?,
            health: ProviderHttpClient::no_redirect(policy, timeout).map_err(map_error)?,
        };
        Ok(Self {
            policy_clients: Some(policy_clients),
            provider,
            ..Self::shared()
        })
    }

    fn policy_request(
        &self,
        client: &ProviderHttpClient,
        url: &str,
        method: HttpMethod,
        headers: Vec<HeaderPair>,
        body: Option<serde_json::Value>,
    ) -> Result<ProviderRequestBuilder, ProviderError> {
        let method = match method {
            HttpMethod::GET => reqwest::Method::GET,
            HttpMethod::POST => reqwest::Method::POST,
            HttpMethod::PUT => reqwest::Method::PUT,
            HttpMethod::DELETE => reqwest::Method::DELETE,
        };
        let mut request = client
            .request(method, url)
            .map_err(|error| ProviderError::configuration(self.provider, error.to_string()))?;
        for (key, value) in headers {
            request = request.header(key.as_ref(), value.as_ref());
        }
        if let Some(body) = body {
            request = request
                .header("Content-Type", "application/json")
                .json(&body);
        }
        Ok(request)
    }

    /// Execute an HTTP request
    ///
    /// Uses `HeaderPair` (Cow-based) for headers to avoid allocations for static strings.
    /// Use `header("Key", value)` for static keys or `header_owned(key, value)` for dynamic keys.
    pub async fn execute_request(
        &self,
        url: &str,
        method: HttpMethod,
        headers: Vec<HeaderPair>,
        body: Option<serde_json::Value>,
    ) -> Result<reqwest::Response, ProviderError> {
        if let Some(clients) = &self.policy_clients {
            return self
                .policy_request(&clients.ordinary, url, method, headers, body)?
                .send()
                .await
                .map_err(|error| ProviderError::network(self.provider, error.to_string()));
        }
        let client = self.pool.client();

        let mut request_builder = match method {
            HttpMethod::GET => client.get(url),
            HttpMethod::POST => client.post(url),
            HttpMethod::PUT => client.put(url),
            HttpMethod::DELETE => client.delete(url),
        };

        // Add headers - Cow allows zero-copy for static strings
        for (key, value) in headers {
            request_builder = request_builder.header(key.as_ref(), value.as_ref());
        }

        // Add body if present
        if let Some(body_data) = body {
            request_builder = request_builder
                .header("Content-Type", "application/json")
                .json(&body_data);
        }

        request_builder
            .send()
            .await
            .map_err(|e| ProviderError::network("common", e.to_string()))
    }

    pub async fn execute_streaming_request(
        &self,
        url: &str,
        method: HttpMethod,
        headers: Vec<HeaderPair>,
        body: Option<serde_json::Value>,
    ) -> Result<Response, ProviderError> {
        let clients = self.policy_clients.as_ref().ok_or_else(|| {
            ProviderError::configuration(self.provider, "streaming endpoint policy is required")
        })?;
        let request = self.policy_request(&clients.streaming, url, method, headers, body)?;
        tokio::time::timeout(
            Duration::from_secs(STREAMING_HEADER_TIMEOUT_SECS),
            request.send(),
        )
        .await
        .map_err(|_| ProviderError::timeout(self.provider, "streaming response headers timed out"))?
        .map_err(|error| ProviderError::network(self.provider, error.to_string()))
    }

    pub async fn execute_health_request(
        &self,
        url: &str,
        headers: Vec<HeaderPair>,
    ) -> Result<Response, ProviderError> {
        let clients = self.policy_clients.as_ref().ok_or_else(|| {
            ProviderError::configuration(self.provider, "health endpoint policy is required")
        })?;
        self.policy_request(&clients.health, url, HttpMethod::GET, headers, None)?
            .send()
            .await
            .map_err(|error| ProviderError::network(self.provider, error.to_string()))
    }

    /// Get the underlying client for direct use
    pub fn client(&self) -> &Client {
        self.pool.client()
    }
}

impl Default for GlobalPoolManager {
    /// Create a default GlobalPoolManager
    ///
    /// Uses the global client singleton, so this is always cheap and fast.
    fn default() -> Self {
        Self::shared()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn delayed_response_url(
        header_delay: Duration,
        body_delay: Duration,
    ) -> std::io::Result<String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let (mut socket, _) = match listener.accept().await {
                Ok(connection) => connection,
                Err(err) => panic!("test server accept failed: {err}"),
            };
            let mut buffer = [0_u8; 1024];
            if let Err(err) = socket.read(&mut buffer).await {
                panic!("test server failed to read request: {err}");
            }

            tokio::time::sleep(header_delay).await;
            if let Err(err) = socket
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 5\r\nConnection: close\r\n\r\n")
                .await
            {
                if !matches!(
                    err.kind(),
                    std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                ) {
                    panic!("test server failed to write headers: {err}");
                }
                return;
            }

            tokio::time::sleep(body_delay).await;
            match socket.write_all(b"hello").await {
                Ok(()) => {}
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                    ) => {}
                Err(err) => panic!("test server failed to write body: {err}"),
            }
        });

        Ok(format!("http://{addr}"))
    }

    async fn delayed_error_body_url(body_delay: Duration) -> std::io::Result<String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let (mut socket, _) = match listener.accept().await {
                Ok(connection) => connection,
                Err(err) => panic!("test server accept failed: {err}"),
            };
            let mut buffer = [0_u8; 1024];
            if let Err(err) = socket.read(&mut buffer).await {
                panic!("test server failed to read request: {err}");
            }

            if let Err(err) = socket
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\nContent-Length: 5\r\nConnection: close\r\n\r\n")
                .await
            {
                if !matches!(
                    err.kind(),
                    std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                ) {
                    panic!("test server failed to write headers: {err}");
                }
                return;
            }

            tokio::time::sleep(body_delay).await;
            match socket.write_all(b"error").await {
                Ok(()) => {}
                Err(err)
                    if matches!(
                        err.kind(),
                        std::io::ErrorKind::BrokenPipe | std::io::ErrorKind::ConnectionReset
                    ) => {}
                Err(err) => panic!("test server failed to write body: {err}"),
            }
        });

        Ok(format!("http://{addr}"))
    }

    async fn error_body_then_stall_url(body: &'static [u8]) -> std::io::Result<String> {
        let listener = TcpListener::bind(("127.0.0.1", 0)).await?;
        let addr = listener.local_addr()?;

        tokio::spawn(async move {
            let (mut socket, _) = match listener.accept().await {
                Ok(connection) => connection,
                Err(err) => panic!("test server accept failed: {err}"),
            };
            let mut buffer = [0_u8; 1024];
            if let Err(err) = socket.read(&mut buffer).await {
                panic!("test server failed to read request: {err}");
            }

            if let Err(err) = socket
                .write_all(b"HTTP/1.1 500 Internal Server Error\r\n\r\n")
                .await
            {
                panic!("test server failed to write headers: {err}");
            }
            if let Err(err) = socket.write_all(body).await {
                panic!("test server failed to write body: {err}");
            }
            tokio::time::sleep(Duration::from_secs(5)).await;
        });

        Ok(format!("http://{addr}"))
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = ConnectionPool::new();
        assert!(pool.is_ok());
    }

    #[tokio::test]
    async fn test_global_manager() {
        let manager = GlobalPoolManager::new();
        assert!(manager.is_ok());
    }

    #[tokio::test]
    async fn test_global_client_singleton() {
        // Get the global client twice
        let client1 = global_client();
        let client2 = global_client();

        // They should point to the same underlying Arc (same pointer)
        assert!(Arc::ptr_eq(&client1, &client2));
    }

    #[tokio::test]
    async fn test_streaming_clients_keep_legacy_and_unbounded_semantics() {
        let legacy = streaming_client();
        let stream1 = streaming_unbounded_client();
        let stream2 = streaming_unbounded_client();
        let global = global_client();

        assert!(Arc::ptr_eq(&legacy, &global));
        assert!(Arc::ptr_eq(&stream1, &stream2));
        assert!(!Arc::ptr_eq(&stream1, &global));
    }

    #[tokio::test]
    async fn test_streaming_send_times_out_before_headers() -> Result<(), Box<dyn std::error::Error>>
    {
        let url =
            delayed_response_url(Duration::from_millis(150), Duration::from_millis(0)).await?;

        let err = send_streaming_request_with_timeout(
            streaming_unbounded_client().get(url),
            Duration::from_millis(25),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, StreamingRequestError::HeaderTimeout { .. }));
        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_send_timeout_does_not_bound_body()
    -> Result<(), Box<dyn std::error::Error>> {
        let url =
            delayed_response_url(Duration::from_millis(0), Duration::from_millis(150)).await?;

        let response = send_streaming_request_with_timeout(
            streaming_unbounded_client().get(url),
            Duration::from_millis(25),
        )
        .await?;
        let body = response.text().await?;

        assert_eq!(body, "hello");
        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_error_body_read_is_bounded() -> Result<(), Box<dyn std::error::Error>> {
        let url = delayed_error_body_url(Duration::from_millis(150)).await?;

        let response = send_streaming_request_with_timeout(
            streaming_unbounded_client().get(url),
            Duration::from_secs(1),
        )
        .await?;
        assert_eq!(
            response.status(),
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        );

        let err = read_streaming_error_body_with_limits(
            response,
            Duration::from_millis(25),
            STREAMING_ERROR_BODY_MAX_BYTES,
        )
        .await
        .unwrap_err();

        assert!(matches!(
            err,
            StreamingRequestError::ErrorBodyTimeout { .. }
        ));
        Ok(())
    }

    #[tokio::test]
    async fn test_streaming_error_body_returns_at_exact_byte_cap()
    -> Result<(), Box<dyn std::error::Error>> {
        let url = error_body_then_stall_url(b"error").await?;

        let response = send_streaming_request_with_timeout(
            streaming_unbounded_client().get(url),
            Duration::from_secs(1),
        )
        .await?;

        let body =
            read_streaming_error_body_with_limits(response, Duration::from_millis(25), 5).await?;

        assert_eq!(body, "error");
        Ok(())
    }

    #[tokio::test]
    async fn test_multiple_managers_share_client() {
        // Create multiple managers
        let manager1 = GlobalPoolManager::new().unwrap();
        let manager2 = GlobalPoolManager::new().unwrap();
        let manager3 = GlobalPoolManager::shared();

        // All should share the same underlying client
        let client1 = manager1.pool.client.clone();
        let client2 = manager2.pool.client.clone();
        let client3 = manager3.pool.client.clone();

        assert!(Arc::ptr_eq(&client1, &client2));
        assert!(Arc::ptr_eq(&client2, &client3));
    }

    #[tokio::test]
    async fn test_isolated_pool_is_different() {
        // Get the global client
        let global = global_client();

        // Create an isolated pool
        let isolated = ConnectionPool::new_isolated().unwrap();

        // The isolated pool should have a different client
        assert!(!Arc::ptr_eq(&global, &isolated.client));
    }

    #[test]
    fn test_default_manager() {
        let manager = GlobalPoolManager::default();
        // Should work without panic
        let _client = manager.client();
    }

    #[tokio::test]
    async fn policy_manager_covers_ordinary_streaming_and_health_requests()
    -> Result<(), Box<dyn std::error::Error>> {
        use crate::core::net::ProviderEndpointAccess;

        assert!(
            GlobalPoolManager::for_provider_endpoint(
                "test",
                "http://127.0.0.1:11434/v1",
                ProviderEndpointAccess::PublicOnly,
                Duration::from_secs(1),
            )
            .is_err()
        );

        let blocked = TcpListener::bind(("127.0.0.1", 0)).await?;
        let blocked_url = format!("http://{}/v1", blocked.local_addr()?);
        let manager = GlobalPoolManager::for_provider_endpoint(
            "test",
            "http://127.0.0.1:11434/v1",
            ProviderEndpointAccess::PrivateNetwork,
            Duration::from_secs(1),
        )?;
        let error = manager
            .execute_request(&blocked_url, HttpMethod::GET, vec![], None)
            .await
            .expect_err("private access must not cross authorities");
        assert!(error.to_string().contains("does not match"));
        assert!(
            tokio::time::timeout(Duration::from_millis(25), blocked.accept())
                .await
                .is_err()
        );

        for mode in ["ordinary", "streaming", "health"] {
            let url = delayed_response_url(Duration::ZERO, Duration::ZERO).await?;
            let manager = GlobalPoolManager::for_provider_endpoint(
                "test",
                &url,
                ProviderEndpointAccess::PrivateNetwork,
                Duration::from_secs(1),
            )?;
            let response = match mode {
                "ordinary" => {
                    manager
                        .execute_request(&url, HttpMethod::GET, vec![], None)
                        .await?
                }
                "streaming" => {
                    manager
                        .execute_streaming_request(&url, HttpMethod::GET, vec![], None)
                        .await?
                }
                "health" => manager.execute_health_request(&url, vec![]).await?,
                _ => unreachable!(),
            };
            assert_eq!(response.text().await?, "hello");
        }
        Ok(())
    }
}
