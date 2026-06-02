//! Rate limiting middleware

use crate::core::rate_limiter::get_global_rate_limiter;
use crate::core::types::context::RequestContext;
use crate::server::state::AppState;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready};
use actix_web::http::StatusCode;
use actix_web::web;
use actix_web::{HttpMessage, HttpResponse, ResponseError};
use dashmap::DashMap;
use futures::future::{Ready, ready};
use std::fmt;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::OnceLock;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Maximum number of distinct client trackers retained by the fallback store.
///
/// The fallback path runs only when the global rate limiter is not initialized.
/// Without a cap, every distinct client IP creates a new entry that lives for
/// the entire process lifetime, which is a memory-exhaustion vector when an
/// attacker rotates source addresses. The value matches `AuthRateLimiter`'s
/// `DEFAULT_MAX_ENTRIES`.
const MAX_FALLBACK_ENTRIES: usize = 10_000;

static AUTH_REJECTION_FALLBACK_STORE: OnceLock<Arc<DashMap<String, KeyTracker>>> = OnceLock::new();

const GLOBAL_LIMITER_SOURCE: &str = "global";
const FALLBACK_LIMITER_SOURCE: &str = "fallback";

/// Fallback per-key tracker for sliding window when global rate limiter is unavailable
struct KeyTracker {
    timestamps: Vec<Instant>,
}

impl KeyTracker {
    fn new() -> Self {
        Self {
            timestamps: Vec::new(),
        }
    }

    fn check(&mut self, limit: u32, window: Duration) -> (bool, u64) {
        let now = Instant::now();
        self.timestamps
            .retain(|&ts| now.duration_since(ts) < window);

        let count = self.timestamps.len() as u32;
        if count >= limit {
            let retry_after = self
                .timestamps
                .first()
                .map(|&ts| {
                    let age = now.duration_since(ts);
                    window.saturating_sub(age).as_secs().max(1)
                })
                .unwrap_or(window.as_secs());
            return (false, retry_after);
        }

        (true, 0)
    }

    /// Check-and-record atomically: returns (allowed, retry_after_secs)
    fn check_and_record(&mut self, limit: u32, window: Duration) -> (bool, u64) {
        let (allowed, retry_after) = self.check(limit, window);
        if allowed {
            self.timestamps.push(Instant::now());
        }

        (allowed, retry_after)
    }
}

struct RateLimitPass {
    source: &'static str,
    limit: u32,
    remaining: u32,
}

struct RateLimitRejection {
    source: &'static str,
    retry_after: u64,
    limit: u32,
}

/// Evict trackers when the fallback store exceeds the cap.
///
/// Two-pass strategy: first drop trackers whose latest timestamp is already
/// outside the rate-limit window (they would re-allow on the next request
/// anyway). If that does not free enough room, drop the trackers whose most
/// recent activity is oldest until the map is back under cap.
fn enforce_fallback_capacity(store: &DashMap<String, KeyTracker>, window: Duration) {
    let now = Instant::now();
    store.retain(|_, tracker| {
        tracker
            .timestamps
            .last()
            .is_some_and(|ts| now.duration_since(*ts) < window)
    });

    let overflow = store.len().saturating_sub(MAX_FALLBACK_ENTRIES);
    if overflow == 0 {
        return;
    }

    let mut candidates: Vec<(Option<Instant>, String)> = store
        .iter()
        .map(|e| (e.value().timestamps.last().copied(), e.key().clone()))
        .collect();
    // Smallest (oldest) first; None first.
    candidates.sort_by_key(|(ts, _)| *ts);
    for (_, key) in candidates.into_iter().take(overflow) {
        store.remove(&key);
    }
}

async fn check_rate_limit_key(
    key: &str,
    requests_per_minute: u32,
    fallback_store: &DashMap<String, KeyTracker>,
) -> Result<RateLimitPass, RateLimitRejection> {
    if let Some(global_limiter) = get_global_rate_limiter() {
        let limit = global_limiter.limit();
        let result = global_limiter.check_and_record(key).await;

        if !result.allowed {
            return Err(RateLimitRejection {
                source: GLOBAL_LIMITER_SOURCE,
                retry_after: result.retry_after_secs.unwrap_or(60),
                limit,
            });
        }

        return Ok(RateLimitPass {
            source: GLOBAL_LIMITER_SOURCE,
            limit,
            remaining: result.remaining,
        });
    }

    let window = Duration::from_secs(60);
    let (allowed, retry_after) = {
        let mut tracker = fallback_store
            .entry(key.to_string())
            .or_insert_with(KeyTracker::new);
        tracker.check_and_record(requests_per_minute, window)
    };
    if fallback_store.len() > MAX_FALLBACK_ENTRIES {
        enforce_fallback_capacity(fallback_store, window);
    }

    if !allowed {
        return Err(RateLimitRejection {
            source: FALLBACK_LIMITER_SOURCE,
            retry_after,
            limit: requests_per_minute,
        });
    }

    let remaining = fallback_store
        .get(key)
        .map(|tracker| requests_per_minute.saturating_sub(tracker.timestamps.len() as u32))
        .unwrap_or(requests_per_minute);

    Ok(RateLimitPass {
        source: FALLBACK_LIMITER_SOURCE,
        limit: requests_per_minute,
        remaining,
    })
}

async fn check_rate_limit_key_status(
    key: &str,
    requests_per_minute: u32,
    fallback_store: &DashMap<String, KeyTracker>,
) -> Result<RateLimitPass, RateLimitRejection> {
    if let Some(global_limiter) = get_global_rate_limiter() {
        let limit = global_limiter.limit();
        let result = global_limiter.check(key).await;

        if !result.allowed {
            return Err(RateLimitRejection {
                source: GLOBAL_LIMITER_SOURCE,
                retry_after: result.retry_after_secs.unwrap_or(60),
                limit,
            });
        }

        return Ok(RateLimitPass {
            source: GLOBAL_LIMITER_SOURCE,
            limit,
            remaining: result.remaining,
        });
    }

    let window = Duration::from_secs(60);
    let Some(mut tracker) = fallback_store.get_mut(key) else {
        return Ok(RateLimitPass {
            source: FALLBACK_LIMITER_SOURCE,
            limit: requests_per_minute,
            remaining: requests_per_minute,
        });
    };

    let (allowed, retry_after) = tracker.check(requests_per_minute, window);
    if !allowed {
        return Err(RateLimitRejection {
            source: FALLBACK_LIMITER_SOURCE,
            retry_after,
            limit: requests_per_minute,
        });
    }

    Ok(RateLimitPass {
        source: FALLBACK_LIMITER_SOURCE,
        limit: requests_per_minute,
        remaining: requests_per_minute.saturating_sub(tracker.timestamps.len() as u32),
    })
}

fn auth_rejection_fallback_store() -> Arc<DashMap<String, KeyTracker>> {
    AUTH_REJECTION_FALLBACK_STORE
        .get_or_init(|| Arc::new(DashMap::new()))
        .clone()
}

pub(super) async fn reject_if_rate_limited_for_auth_attempt(
    req: &ServiceRequest,
    requests_per_minute: u32,
    trusted_proxies: &[String],
) -> Result<(), actix_web::Error> {
    let key = extract_client_key(req, trusted_proxies);
    let fallback_store = auth_rejection_fallback_store();

    match check_rate_limit_key_status(&key, requests_per_minute, &fallback_store).await {
        Ok(pass) => {
            debug!(
                client = %key,
                limit = pass.limit,
                remaining = pass.remaining,
                "Rate limit pre-check passed for auth attempt ({} limiter)",
                pass.source
            );
            Ok(())
        }
        Err(rejection) => {
            warn!(
                client = %key,
                "Rate limit exceeded before auth verification ({} limiter): retry after {}s",
                rejection.source,
                rejection.retry_after
            );
            Err(actix_web::Error::from(RateLimitError {
                retry_after: rejection.retry_after,
                limit: rejection.limit,
            }))
        }
    }
}

pub(super) async fn enforce_rate_limit_for_rejected_auth(
    req: &ServiceRequest,
    requests_per_minute: u32,
    trusted_proxies: &[String],
) -> Result<(), actix_web::Error> {
    let key = extract_client_key(req, trusted_proxies);
    let fallback_store = auth_rejection_fallback_store();

    match check_rate_limit_key(&key, requests_per_minute, &fallback_store).await {
        Ok(pass) => {
            debug!(
                client = %key,
                limit = pass.limit,
                remaining = pass.remaining,
                "Rate limit check passed for rejected auth path ({} limiter)",
                pass.source
            );
            Ok(())
        }
        Err(rejection) => {
            warn!(
                client = %key,
                "Rate limit exceeded for rejected auth path ({} limiter): retry after {}s",
                rejection.source,
                rejection.retry_after
            );
            Err(actix_web::Error::from(RateLimitError {
                retry_after: rejection.retry_after,
                limit: rejection.limit,
            }))
        }
    }
}

/// Lightweight in-process rate limit error for 429 responses
#[derive(Debug)]
struct RateLimitError {
    retry_after: u64,
    limit: u32,
}

impl fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Too Many Requests")
    }
}

impl ResponseError for RateLimitError {
    fn status_code(&self) -> StatusCode {
        StatusCode::TOO_MANY_REQUESTS
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::TooManyRequests()
            .insert_header(("Retry-After", self.retry_after.to_string()))
            .insert_header(("X-RateLimit-Limit", self.limit.to_string()))
            .json(serde_json::json!({
                "error": {
                    "message": "Rate limit exceeded. Please retry after the indicated seconds.",
                    "type": "rate_limit_error",
                    "code": 429
                }
            }))
    }
}

/// Rate limit middleware for Actix-web
pub struct RateLimitMiddleware {
    requests_per_minute: u32,
}

impl RateLimitMiddleware {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
        }
    }
}

impl Default for RateLimitMiddleware {
    fn default() -> Self {
        Self::new(60)
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimitMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type InitError = ();
    type Transform = RateLimitMiddlewareService<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimitMiddlewareService {
            service,
            requests_per_minute: self.requests_per_minute,
            fallback_store: Arc::new(DashMap::new()),
        }))
    }
}

/// Service implementation for rate limit middleware
pub struct RateLimitMiddlewareService<S> {
    service: S,
    requests_per_minute: u32,
    /// Fallback in-process store used when the global rate limiter is not initialized
    fallback_store: Arc<DashMap<String, KeyTracker>>,
}

/// Extract the IP address (without port) from a peer address string.
///
/// Handles IPv4 (`1.2.3.4:5678` → `1.2.3.4`) and IPv6 (`[::1]:5678` → `::1`).
fn parse_peer_ip(peer: &str) -> String {
    peer.parse::<SocketAddr>()
        .map(|addr| addr.ip().to_string())
        .unwrap_or_else(|_| peer.to_string())
}

/// Extract a client identifier from the request.
///
/// Priority:
/// 1. Authenticated API key ID / user ID from `RequestContext`, when present
/// 2. `X-Forwarded-For` first address — only when peer IP is in `trusted_proxies`
/// 3. Direct peer address from connection info
fn extract_client_key(req: &ServiceRequest, trusted_proxies: &[String]) -> String {
    if let Some(identity) = authenticated_client_key(req) {
        return identity;
    }

    network_client_key(req, trusted_proxies)
}

fn authenticated_client_key(req: &ServiceRequest) -> Option<String> {
    let extensions = req.extensions();
    let context = extensions.get::<RequestContext>()?;

    if let Some(api_key_id) = context.api_key_id() {
        return Some(format!("api_key:{}", api_key_id));
    }

    context
        .user_id
        .as_deref()
        .map(str::trim)
        .filter(|user_id| !user_id.is_empty())
        .map(|user_id| format!("user:{}", user_id))
}

fn network_client_key(req: &ServiceRequest, trusted_proxies: &[String]) -> String {
    let conn = req.connection_info();
    let peer = conn.peer_addr().unwrap_or("unknown");
    let peer_ip = parse_peer_ip(peer);

    if trusted_proxies.iter().any(|p| p == &peer_ip)
        && let Some(forwarded) = req.headers().get("X-Forwarded-For")
        && let Ok(val) = forwarded.to_str()
        && let first = val.split(',').next().unwrap_or("").trim()
        && !first.is_empty()
    {
        return format!("ip:{}", first);
    }

    format!("ip:{}", peer_ip)
}

impl<S, B> Service<ServiceRequest> for RateLimitMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let app_state = req.app_data::<web::Data<AppState>>().cloned();
        let trusted_proxies: Vec<String> = match app_state.as_ref() {
            Some(state) => {
                let cfg = state.config.load();
                cfg.server().trusted_proxies.clone()
            }
            None => Vec::new(),
        };
        let requests_per_minute = self.requests_per_minute;
        let start_time = Instant::now();
        let path = req.path().to_string();
        let method = req.method().to_string();

        let client_key = extract_client_key(&req, &trusted_proxies);

        let fallback_store = self.fallback_store.clone();
        // service.call() returns a lazy future; it only executes on .await.
        // We must call it here because it consumes `req`, but we will NOT
        // await it if the rate check fails, so no downstream work is wasted.
        let fut = self.service.call(req);
        let key = client_key.clone();

        Box::pin(async move {
            let pass = match check_rate_limit_key(&key, requests_per_minute, &fallback_store).await
            {
                Ok(pass) => pass,
                Err(rejection) => {
                    warn!(
                        client = %key,
                        path = %path,
                        "Rate limit exceeded ({} limiter): retry after {}s",
                        rejection.source,
                        rejection.retry_after
                    );
                    let err = RateLimitError {
                        retry_after: rejection.retry_after,
                        limit: rejection.limit,
                    };
                    return Err(actix_web::Error::from(err));
                }
            };

            debug!(
                client = %key,
                limit = pass.limit,
                remaining = pass.remaining,
                "Rate limit check passed ({} limiter)",
                pass.source
            );

            let res = fut.await?;
            let duration = start_time.elapsed();
            info!(
                "{} {} completed in {:?} with status {}",
                method,
                path,
                duration,
                res.status()
            );
            Ok(res)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::test::TestRequest;
    use uuid::Uuid;

    #[test]
    fn test_parse_peer_ip_ipv4_with_port() {
        assert_eq!(parse_peer_ip("127.0.0.1:1234"), "127.0.0.1");
    }

    #[test]
    fn test_parse_peer_ip_ipv4_no_port() {
        assert_eq!(parse_peer_ip("10.0.0.1"), "10.0.0.1");
    }

    #[test]
    fn test_parse_peer_ip_ipv6_with_port() {
        assert_eq!(parse_peer_ip("[::1]:8080"), "::1");
    }

    #[test]
    fn test_parse_peer_ip_unknown_falls_back() {
        assert_eq!(parse_peer_ip("unknown"), "unknown");
    }

    #[test]
    fn test_trusted_proxy_match() {
        let proxies = ["10.0.0.1".to_string()];
        assert!(proxies.iter().any(|p| p == "10.0.0.1"));
    }

    #[test]
    fn test_trusted_proxy_no_match() {
        let proxies = ["10.0.0.1".to_string()];
        assert!(!proxies.iter().any(|p| p == "10.0.0.2"));
    }

    #[test]
    fn test_trusted_proxy_empty_list() {
        let proxies: Vec<String> = vec![];
        assert!(!proxies.iter().any(|p| p == "127.0.0.1"));
    }

    #[test]
    fn test_extract_client_key_ignores_rotating_authorization_headers() {
        let req_a = TestRequest::default()
            .peer_addr("203.0.113.10:1000".parse().unwrap())
            .insert_header(("Authorization", "Bearer bogus-a"))
            .to_srv_request();
        let req_b = TestRequest::default()
            .peer_addr("203.0.113.10:1000".parse().unwrap())
            .insert_header(("Authorization", "Bearer bogus-b"))
            .to_srv_request();

        let key_a = extract_client_key(&req_a, &[]);
        let key_b = extract_client_key(&req_b, &[]);

        assert_eq!(key_a, "ip:203.0.113.10");
        assert_eq!(key_a, key_b);
    }

    #[test]
    fn test_extract_client_key_ignores_rotating_api_key_headers() {
        let req_a = TestRequest::default()
            .peer_addr("203.0.113.20:1000".parse().unwrap())
            .insert_header(("x-api-key", "bogus-a"))
            .to_srv_request();
        let req_b = TestRequest::default()
            .peer_addr("203.0.113.20:1000".parse().unwrap())
            .insert_header(("x-api-key", "bogus-b"))
            .to_srv_request();

        let key_a = extract_client_key(&req_a, &[]);
        let key_b = extract_client_key(&req_b, &[]);

        assert_eq!(key_a, "ip:203.0.113.20");
        assert_eq!(key_a, key_b);
    }

    #[test]
    fn test_extract_client_key_uses_trusted_forwarded_ip() {
        let req = TestRequest::default()
            .peer_addr("10.0.0.1:1000".parse().unwrap())
            .insert_header(("X-Forwarded-For", "198.51.100.7, 10.0.0.2"))
            .to_srv_request();

        let key = extract_client_key(&req, &["10.0.0.1".to_string()]);

        assert_eq!(key, "ip:198.51.100.7");
    }

    #[test]
    fn test_extract_client_key_prefers_authenticated_api_key_id() {
        let api_key_id = Uuid::new_v4();
        let req = TestRequest::default()
            .peer_addr("203.0.113.30:1000".parse().unwrap())
            .to_srv_request();
        req.extensions_mut()
            .insert(RequestContext::new().with_api_key(api_key_id));

        let key = extract_client_key(&req, &[]);

        assert_eq!(key, format!("api_key:{}", api_key_id));
    }

    #[test]
    fn test_extract_client_key_uses_authenticated_user_id_without_api_key() {
        let req = TestRequest::default()
            .peer_addr("203.0.113.40:1000".parse().unwrap())
            .to_srv_request();
        req.extensions_mut()
            .insert(RequestContext::new().with_user_id("user-123"));

        let key = extract_client_key(&req, &[]);

        assert_eq!(key, "user:user-123");
    }

    #[test]
    fn test_key_tracker_status_check_does_not_record() {
        let mut tracker = KeyTracker::new();
        let window = Duration::from_secs(60);

        let (allowed, retry_after) = tracker.check(1, window);

        assert!(allowed);
        assert_eq!(retry_after, 0);
        assert!(tracker.timestamps.is_empty());
    }

    #[test]
    fn test_key_tracker_status_check_rejects_full_bucket_without_recording() {
        let mut tracker = KeyTracker::new();
        let window = Duration::from_secs(60);
        let (allowed, _) = tracker.check_and_record(1, window);
        assert!(allowed);
        let recorded = tracker.timestamps.len();

        let (allowed, retry_after) = tracker.check(1, window);

        assert!(!allowed);
        assert!(retry_after > 0);
        assert_eq!(tracker.timestamps.len(), recorded);
    }

    #[test]
    fn test_enforce_fallback_capacity_evicts_stale_first() {
        let store: DashMap<String, KeyTracker> = DashMap::new();
        let now = Instant::now();
        let window = Duration::from_secs(60);
        // 3 stale + 2 fresh trackers
        for i in 0..3 {
            let mut t = KeyTracker::new();
            t.timestamps.push(now - Duration::from_secs(120));
            store.insert(format!("stale-{i}"), t);
        }
        for i in 0..2 {
            let mut t = KeyTracker::new();
            t.timestamps.push(now);
            store.insert(format!("fresh-{i}"), t);
        }
        assert_eq!(store.len(), 5);
        enforce_fallback_capacity(&store, window);
        // Stale entries dropped, fresh kept; cap is generous so no LRU pass.
        assert_eq!(store.len(), 2);
        assert!(store.contains_key("fresh-0"));
        assert!(store.contains_key("fresh-1"));
    }

    #[test]
    fn test_enforce_fallback_capacity_evicts_oldest_when_all_fresh() {
        // Force the LRU branch by setting a tiny cap via a parallel helper.
        let store: DashMap<String, KeyTracker> = DashMap::new();
        let base = Instant::now();
        for i in 0..MAX_FALLBACK_ENTRIES + 5 {
            let mut t = KeyTracker::new();
            t.timestamps.push(base + Duration::from_millis(i as u64));
            store.insert(format!("k-{i}"), t);
        }
        // All within window so the stale-pass doesn't shrink the map.
        enforce_fallback_capacity(&store, Duration::from_secs(60));
        assert!(store.len() <= MAX_FALLBACK_ENTRIES);
        // The 5 oldest should be evicted, the rest kept.
        assert!(!store.contains_key("k-0"));
        assert!(store.contains_key(&format!("k-{}", MAX_FALLBACK_ENTRIES + 4)));
    }
}
