//! Alert system tests

use super::channels::{NotificationChannel, SLACK_WEBHOOK_TIMEOUT, SlackChannel};
use super::types::{AlertRule, AlertStats, ComparisonOperator};
use crate::core::net::ProviderEndpointPolicy;
use crate::monitoring::types::{Alert, AlertSeverity};
use crate::utils::net::http::{ProviderHttpClient, ProviderHttpClientError};
use chrono::Utc;
use reqwest::dns::{Addrs, Name, Resolve, Resolving};
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
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
        SLACK_WEBHOOK_TIMEOUT,
        true,
        Arc::new(FixedDnsResolver(address)),
    )
}

fn test_slack_channel(
    url: String,
    client: ProviderHttpClient,
) -> crate::utils::error::gateway_error::Result<SlackChannel> {
    SlackChannel::with_client_for_test(url, None, None, AlertSeverity::Info, client)
}

fn test_alert(severity: AlertSeverity) -> Alert {
    Alert {
        id: "alert-id".to_string(),
        title: "Policy Alert".to_string(),
        description: "Deterministic payload".to_string(),
        severity,
        source: "monitoring-test".to_string(),
        timestamp: Utc::now(),
        metadata: serde_json::json!({}),
        resolved: false,
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
            return Err(io::Error::other("request closed before Slack payload"));
        }
        request.extend_from_slice(&buffer[..read]);
        if request
            .windows(b"monitoring-test".len())
            .any(|window| window == b"monitoring-test")
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

#[test]
fn test_alert_rule_creation() {
    let rule = AlertRule {
        id: "test-rule".to_string(),
        name: "High CPU Usage".to_string(),
        description: "Alert when CPU usage exceeds 80%".to_string(),
        metric: "cpu_usage".to_string(),
        threshold: 80.0,
        operator: ComparisonOperator::GreaterThan,
        severity: AlertSeverity::Warning,
        interval: Duration::from_secs(60),
        enabled: true,
        channels: vec!["slack".to_string()],
    };

    assert_eq!(rule.name, "High CPU Usage");
    assert_eq!(rule.threshold, 80.0);
    assert!(rule.enabled);
}

#[test]
fn test_comparison_operators() {
    assert_eq!(
        ComparisonOperator::GreaterThan,
        ComparisonOperator::GreaterThan
    );
    assert_ne!(
        ComparisonOperator::GreaterThan,
        ComparisonOperator::LessThan
    );
}

#[test]
fn test_slack_channel_creation() {
    let Ok(channel) = SlackChannel::new(
        "https://hooks.slack.com/test".to_string(),
        Some("#alerts".to_string()),
        Some("Gateway".to_string()),
        AlertSeverity::Warning,
    ) else {
        panic!("public Slack webhook must be accepted");
    };

    assert_eq!(channel.name(), "slack");
    assert!(channel.supports_severity(AlertSeverity::Critical));
    assert!(!channel.supports_severity(AlertSeverity::Info));
}

#[test]
fn slack_webhook_admission_rejects_complete_unsafe_url_table() {
    for url in [
        "",
        " ",
        "not a url",
        "file:///tmp/hook",
        "ftp://example.com/hook",
        "http://127.0.0.1/hook",
        "http://10.0.0.1/hook",
        "http://169.254.169.254/latest/meta-data",
        "http://localhost/hook",
        "http://foo.localhost/hook",
        "http://internal/hook",
        "http://foo.internal/hook",
        "http://local/hook",
        "http://foo.local/hook",
        "http://metadata/hook",
        "http://metadata.google.internal/hook",
        "http://metadata.goog/hook",
        "http://[::1]/hook",
        "http://[fd00::1]/hook",
        "http://[fe80::1]/hook",
        "http://[::ffff:169.254.169.254]/hook",
        "http://[64:ff9b::a9fe:a9fe]/hook",
    ] {
        assert!(
            SlackChannel::new(url.to_string(), None, None, AlertSeverity::Info).is_err(),
            "unsafe Slack webhook URL was accepted: {url}"
        );
    }
}

#[tokio::test]
async fn legal_slack_delivery_preserves_payload_and_120_second_timeout() -> TestResult {
    assert_eq!(SLACK_WEBHOOK_TIMEOUT, Duration::from_secs(120));
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
    let channel = SlackChannel::with_client_for_test(
        format!("http://slack.test:{}/hook", address.port()),
        Some("#alerts".to_string()),
        Some("AlertBot".to_string()),
        AlertSeverity::Info,
        policy_client(address)?,
    )?;

    channel.send(&test_alert(AlertSeverity::Critical)).await?;
    let request = server.await??;
    for expected in [
        "\"username\":\"AlertBot\"",
        "\"channel\":\"#alerts\"",
        "\"color\":\"#ff0000\"",
        "\"title\":\"Policy Alert\"",
        "\"text\":\"Deterministic payload\"",
        "\"value\":\"Critical\"",
        "\"value\":\"monitoring-test\"",
    ] {
        assert!(request.contains(expected), "{request}");
    }
    Ok(())
}

#[tokio::test]
async fn slack_redirect_is_not_followed_and_secrets_are_redacted() -> TestResult {
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
        "http://fake-user:fake-password@source.test:{}/hook?token=fake-query",
        source_address.port()
    );
    let channel = test_slack_channel(url, policy_client(source_address)?)?;
    let debug = format!("{channel:?}");
    let error = channel
        .send(&test_alert(AlertSeverity::Info))
        .await
        .expect_err("redirect response must be an explicit error")
        .to_string();
    server.await??;
    assert_listener_did_not_accept(&target, "Slack redirect target").await;
    assert!(error.contains("302"), "{error}");
    for secret in ["fake-user", "fake-password", "fake-query", "token="] {
        assert!(!error.contains(secret), "{error}");
        assert!(!debug.contains(secret), "{debug}");
    }
    Ok(())
}

#[tokio::test]
async fn slack_public_then_private_rebind_never_reaches_listener() -> TestResult {
    for blocked_ip in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "::1",
        "fd00::1",
        "::ffff:169.254.169.254",
        "64:ff9b::a9fe:a9fe",
    ] {
        let tripwire = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
        let tripwire_address = tripwire.local_addr()?;
        let blocked = SocketAddr::new(blocked_ip.parse()?, tripwire_address.port());
        let client = ProviderHttpClient::build_public_then_private_tripwire_for_test(
            blocked,
            tripwire_address,
        )
        .await?;
        let url = format!(
            "http://rebind-user:rebind-password@rebind.test:{}/hook?token=rebind-secret",
            tripwire_address.port()
        );
        let channel = test_slack_channel(url, client)?;
        let error = channel
            .send(&test_alert(AlertSeverity::Info))
            .await
            .expect_err("private rebinding answer must be rejected")
            .to_string();

        assert!(error.contains("outbound endpoint policy"), "{error}");
        for secret in ["rebind-user", "rebind-password", "rebind-secret", "token="] {
            assert!(!error.contains(secret), "{error}");
        }
        assert_listener_did_not_accept(&tripwire, "Slack rebinding target").await;
    }
    Ok(())
}

#[test]
fn test_alert_stats() {
    let mut stats = AlertStats {
        total_alerts: 10,
        ..Default::default()
    };
    stats.alerts_by_severity.insert("Warning".to_string(), 5);
    stats.alerts_by_severity.insert("Critical".to_string(), 3);

    assert_eq!(stats.total_alerts, 10);
    assert_eq!(stats.alerts_by_severity.get("Warning"), Some(&5));
}
