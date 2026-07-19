//! Tests for observability module

use super::destinations::LogDestination;
#[cfg(test)]
use super::histogram::{BoundedHistogram, HISTOGRAM_MAX_SAMPLES};
use super::logging::{LOG_WEBHOOK_TIMEOUT, LogAggregator};
use super::metrics::MetricsCollector;
use super::types::{LogEntry, TokenUsage};
use crate::core::net::ProviderEndpointPolicy;
use crate::utils::net::http::{ProviderHttpClient, ProviderHttpClientError};
use chrono::Utc;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::collections::HashMap;
use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

type TestResult = Result<(), Box<dyn std::error::Error>>;

#[derive(Clone)]
struct FixedDnsResolver(SocketAddr);

impl Resolve for FixedDnsResolver {
    fn resolve(&self, _name: Name) -> Resolving {
        let address = self.0;
        Box::pin(async move { Ok(Box::new(std::iter::once(address)) as Addrs) })
    }
}

fn policy_client(address: SocketAddr) -> Result<ProviderHttpClient, ProviderHttpClientError> {
    ProviderHttpClient::build_with_dns_resolver_for_test(
        ProviderEndpointPolicy::public_only(),
        LOG_WEBHOOK_TIMEOUT,
        true,
        Arc::new(FixedDnsResolver(address)),
    )
}

fn test_log_entry() -> LogEntry {
    LogEntry {
        timestamp: Utc::now(),
        level: "INFO".to_string(),
        message: "T004-entry".to_string(),
        module: Some("observability".to_string()),
        request_id: Some("req-t004".to_string()),
        metadata: HashMap::from([("scope".to_string(), serde_json::json!("webhook"))]),
    }
}

async fn read_http_request(stream: &mut tokio::net::TcpStream) -> io::Result<String> {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        let read = tokio::time::timeout(Duration::from_secs(1), stream.read(&mut buffer))
            .await
            .map_err(|_| io::Error::other("request read timed out"))??;
        if read == 0 {
            return Err(io::Error::other("request closed before log payload"));
        }
        request.extend_from_slice(&buffer[..read]);
        if request
            .windows(b"T004-entry".len())
            .any(|window| window == b"T004-entry")
        {
            return String::from_utf8(request).map_err(io::Error::other);
        }
    }
}

async fn assert_listener_did_not_accept(listener: &tokio::net::TcpListener, context: &str) {
    match tokio::time::timeout(Duration::from_millis(100), listener.accept()).await {
        Err(_) => {}
        Ok(Ok((_stream, peer))) => panic!("{context}: unexpectedly accepted {peer}"),
        Ok(Err(error)) => panic!("{context}: listener failed: {error}"),
    }
}

#[derive(Clone)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for CapturedLogs {
    type Writer = Self;

    fn make_writer(&'a self) -> Self::Writer {
        self.clone()
    }
}

impl Write for CapturedLogs {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.lock().expect("log lock").extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn test_bounded_histogram_basic() {
    let mut histogram = BoundedHistogram::new(5);

    // Record some values
    histogram.record(1.0);
    histogram.record(2.0);
    histogram.record(3.0);

    assert_eq!(histogram.count(), 3);
    assert_eq!(histogram.window_size(), 3);
    assert!((histogram.mean() - 2.0).abs() < 0.001);
    assert!((histogram.min() - 1.0).abs() < 0.001);
    assert!((histogram.max() - 3.0).abs() < 0.001);
}

#[test]
fn test_bounded_histogram_rolling_window() {
    let mut histogram = BoundedHistogram::new(3);

    // Fill the histogram
    histogram.record(1.0);
    histogram.record(2.0);
    histogram.record(3.0);

    assert_eq!(histogram.window_size(), 3);
    assert!((histogram.mean() - 2.0).abs() < 0.001);

    // Add more values - oldest should be evicted
    histogram.record(4.0);
    histogram.record(5.0);

    // Window should still be 3, but now contains [3.0, 4.0, 5.0]
    assert_eq!(histogram.window_size(), 3);
    assert_eq!(histogram.count(), 5); // Total count should be 5
    assert!((histogram.mean() - 4.0).abs() < 0.001); // (3+4+5)/3 = 4
    assert!((histogram.min() - 3.0).abs() < 0.001);
    assert!((histogram.max() - 5.0).abs() < 0.001);
}

#[test]
fn test_bounded_histogram_percentile() {
    let mut histogram = BoundedHistogram::new(100);

    // Record values 1-100
    for i in 1..=100 {
        histogram.record(i as f64);
    }

    // Test percentiles
    assert!((histogram.percentile(50.0) - 50.0).abs() < 1.0);
    assert!((histogram.percentile(90.0) - 90.0).abs() < 1.0);
    assert!((histogram.percentile(99.0) - 99.0).abs() < 1.0);
}

#[test]
fn test_bounded_histogram_prevents_memory_leak() {
    let mut histogram = BoundedHistogram::new(100);

    // Record many more values than capacity
    for i in 0..10000 {
        histogram.record(i as f64);
    }

    // Window size should be capped at 100
    assert_eq!(histogram.window_size(), 100);
    // But total count should reflect all recordings
    assert_eq!(histogram.count(), 10000);
}

#[tokio::test]
async fn test_metrics_collection() {
    let collector = MetricsCollector::new();

    collector
        .record_request(
            "openai",
            "gpt-4",
            Duration::from_millis(500),
            Some(TokenUsage {
                prompt_tokens: 100,
                completion_tokens: 50,
                total_tokens: 150,
            }),
            Some(0.01),
            true,
        )
        .await;

    let prometheus_output = collector.export_prometheus().await;
    assert!(prometheus_output.contains("litellm_requests_total"));
    assert!(prometheus_output.contains("provider=\"openai\""));
    assert!(prometheus_output.contains("model=\"gpt-4\""));
}

#[tokio::test]
async fn test_metrics_histogram_bounded() {
    let collector = MetricsCollector::new();

    // Record many requests to test histogram bounding
    for i in 0..2000 {
        collector
            .record_request(
                "openai",
                "gpt-4",
                Duration::from_millis(i),
                None,
                None,
                true,
            )
            .await;
    }

    // Verify histogram is bounded
    let metrics = collector.prometheus_metrics.read().await;
    let histogram = metrics.request_duration.get("openai:gpt-4").unwrap();

    // Window should be capped at HISTOGRAM_MAX_SAMPLES
    assert!(histogram.window_size() <= HISTOGRAM_MAX_SAMPLES);
    // But count should reflect all recordings
    assert_eq!(histogram.count(), 2000);
}

#[tokio::test]
async fn test_log_aggregation() {
    let aggregator = LogAggregator::new();

    let entry = LogEntry {
        timestamp: Utc::now(),
        level: "INFO".to_string(),
        message: "Test log entry".to_string(),
        module: Some("observability".to_string()),
        request_id: Some("req-123".to_string()),
        metadata: HashMap::new(),
    };

    aggregator.log(entry).await;

    let buffer = aggregator.buffer.read().await;
    assert_eq!(buffer.len(), 1);
    assert_eq!(buffer[0].message, "Test log entry");
}

#[test]
fn log_webhook_admission_rejects_complete_unsafe_url_table() {
    for url in "\n \nnot a url\nfile:///tmp/hook\nftp://example.com/hook\nws://secret-user:secret-pass@example.com/hook?token=secret-query\nwss://secret-user:secret-pass@example.com/hook?token=secret-query\nhttp://secret-user:secret-pass@127.0.0.1/hook?token=secret-query\nhttp://10.0.0.1/hook\nhttp://169.254.169.254/latest/meta-data\nhttp://localhost/hook\nhttp://foo.localhost/hook\nhttp://internal/hook\nhttp://foo.internal/hook\nhttp://local/hook\nhttp://foo.local/hook\nhttp://metadata/hook\nhttp://metadata.google.internal/hook\nhttp://metadata.goog/hook\nhttp://[::1]/hook\nhttp://[fd00::1]/hook\nhttp://[fe80::1]/hook\nhttp://[::ffff:169.254.169.254]/hook\nhttp://[64:ff9b::a9fe:a9fe]/hook".split('\n') {
        let result = LogAggregator::new().add_destination(LogDestination::Webhook {
            url: url.to_string(),
            headers: HashMap::new(),
        });
        let Err(error) = result else {
            panic!("unsafe log webhook URL was accepted: {url}");
        };
        for secret in ["secret-user", "secret-pass", "secret-query", "token="] {
            assert!(!error.to_string().contains(secret), "{error}");
        }
    }
}

#[test]
fn log_destination_debug_redacts_webhook_url_and_headers() {
    let destination = LogDestination::Webhook {
        url: "https://du:dp@example.com/hook?token=dq".to_string(),
        headers: HashMap::from([("auth".to_string(), "dh".to_string())]),
    };
    let debug = format!("{destination:?}");
    assert_eq!(debug, "LogDestination { redacted }");
    for secret in "du dp dq auth dh token=".split_whitespace() {
        assert!(!debug.contains(secret), "{debug}");
    }
}

#[tokio::test]
async fn legal_log_webhook_preserves_entries_headers_and_120_second_timeout() -> TestResult {
    assert_eq!(LOG_WEBHOOK_TIMEOUT, Duration::from_secs(120));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await?;
        let request = read_http_request(&mut stream).await?;
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .await?;
        Ok::<_, io::Error>(request)
    });
    let url = format!("http://logs.test:{}/hook", address.port());
    let headers = HashMap::from([("x-log-hook".to_string(), "preserved".to_string())]);
    let destination = LogDestination::Webhook {
        url: url.clone(),
        headers: headers.clone(),
    };
    let aggregator = LogAggregator::new()
        .with_webhook_client_for_test(policy_client(address)?)
        .add_destination(destination.clone())?;

    aggregator
        .send_to_destination(&destination, &[test_log_entry()])
        .await?;
    let request = server.await??.to_ascii_lowercase();
    for expected in ["x-log-hook: preserved", "t004-entry", "req-t004", "webhook"] {
        assert!(request.contains(expected), "{request}");
    }
    Ok(())
}

#[tokio::test(flavor = "current_thread")]
async fn log_webhook_redirect_is_error_not_followed_and_logs_are_secret_safe() -> TestResult {
    let bytes = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .without_time()
        .with_ansi(false)
        .with_writer(CapturedLogs(bytes.clone()))
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let source = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let source_address = source.local_addr()?;
    let target = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let target_address = target.local_addr()?;
    let server = tokio::spawn(async move {
        let (mut stream, _) = source.accept().await?;
        let _request = read_http_request(&mut stream).await?;
        stream
            .write_all(
                format!(
                    "HTTP/1.1 302 Found\r\nLocation: http://redirect.test:{}/private?token=redirect-secret\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                    target_address.port()
                )
                .as_bytes(),
            )
            .await
    });
    let url = format!(
        "http://source-user:source-pass@source.test:{}/hook?token=source-secret",
        source_address.port()
    );
    let aggregator = LogAggregator::new()
        .with_webhook_client_for_test(policy_client(source_address)?)
        .add_destination(LogDestination::Webhook {
            url,
            headers: HashMap::new(),
        })?;
    aggregator.log(test_log_entry()).await;
    aggregator.flush_buffer().await;
    server.await??;
    assert_listener_did_not_accept(&target, "log webhook redirect target").await;
    let logs = String::from_utf8(bytes.lock().expect("log lock").clone())?;
    assert!(logs.contains("302 Found"), "{logs}");
    for secret in "source-user source-pass source-secret redirect-secret token=".split_whitespace()
    {
        assert!(!logs.contains(secret), "{logs}");
    }
    Ok(())
}

#[tokio::test]
async fn log_webhook_public_then_private_rebind_never_reaches_listener() -> TestResult {
    for blocked_ip in
        "127.0.0.1 10.0.0.1 169.254.169.254 ::1 fd00::1 ::ffff:169.254.169.254 64:ff9b::a9fe:a9fe"
            .split_whitespace()
    {
        let tripwire = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let address = tripwire.local_addr()?;
        let client = ProviderHttpClient::build_public_then_private_tripwire_for_test(
            SocketAddr::new(blocked_ip.parse()?, address.port()),
            address,
        )
        .await?;
        let url = format!(
            "http://rebind-user:rebind-pass@rebind.test:{}/hook?token=rebind-secret",
            address.port()
        );
        let destination = LogDestination::Webhook {
            url: url.clone(),
            headers: HashMap::new(),
        };
        let aggregator = LogAggregator::new()
            .with_webhook_client_for_test(client)
            .add_destination(destination.clone())?;
        let error = aggregator
            .send_to_destination(&destination, &[test_log_entry()])
            .await
            .expect_err("private rebinding answer must be rejected")
            .to_string();
        assert!(error.contains("outbound endpoint policy"), "{error}");
        for secret in ["rebind-user", "rebind-pass", "rebind-secret", "token="] {
            assert!(!error.contains(secret), "{error}");
        }
        assert_listener_did_not_accept(&tripwire, "log webhook rebinding target").await;
    }
    Ok(())
}

#[test]
fn all_four_webhook_senders_avoid_generic_clients_and_private_ip_classification() {
    let sources = [
        include_str!("../webhooks/manager.rs"),
        include_str!("../webhooks/delivery.rs"),
        include_str!("../budget/alerts.rs"),
        include_str!("../../monitoring/alerts/channels.rs"),
        include_str!("logging.rs"),
    ];
    for source in sources {
        for forbidden in
            "default_outbound_client create_custom_client reqwest::Client is_private_or_reserved_ip"
                .split_whitespace()
        {
            assert!(!source.contains(forbidden), "found {forbidden}");
        }
    }
}
