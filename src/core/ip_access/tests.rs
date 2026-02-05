//! Integration tests for the IP Access Control system

use self::config::IpAccessConfig;
use self::control::IpAccessControl;
use self::types::{IpAccessMode, IpRule};
use super::*;
use std::sync::Arc;

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_full_allowlist_pipeline() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.0/24")
        .allow_ip("10.0.0.0/8")
        .allow_ip("172.16.0.1");

    let controller = IpAccessControl::new(config).unwrap();

    // Test various IPs
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(controller.is_allowed("192.168.1.255").await);
    assert!(controller.is_allowed("10.0.0.1").await);
    assert!(controller.is_allowed("10.255.255.255").await);
    assert!(controller.is_allowed("172.16.0.1").await);

    assert!(!controller.is_allowed("192.168.2.1").await);
    assert!(!controller.is_allowed("172.16.0.2").await);
    assert!(!controller.is_allowed("8.8.8.8").await);
}

#[tokio::test]
async fn test_full_blocklist_pipeline() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Blocklist)
        .block_ip("192.168.1.100")
        .block_ip("10.0.0.0/8")
        .block_ip("172.16.0.0/12");

    let controller = IpAccessControl::new(config).unwrap();

    // Blocked IPs
    assert!(!controller.is_allowed("192.168.1.100").await);
    assert!(!controller.is_allowed("10.0.0.1").await);
    assert!(!controller.is_allowed("172.16.0.1").await);
    assert!(!controller.is_allowed("172.31.255.255").await);

    // Allowed IPs
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(controller.is_allowed("192.168.1.99").await);
    assert!(controller.is_allowed("8.8.8.8").await);
    assert!(controller.is_allowed("1.1.1.1").await);
}

#[tokio::test]
async fn test_always_allow_bypass() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Blocklist)
        .block_ip("0.0.0.0/0") // Block everything
        .always_allow_ip("192.168.1.1")
        .always_allow_ip("10.0.0.0/24");

    let controller = IpAccessControl::new(config).unwrap();

    // Always-allowed IPs should bypass blocklist
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(controller.is_allowed("10.0.0.1").await);
    assert!(controller.is_allowed("10.0.0.255").await);

    // Other IPs should be blocked
    assert!(!controller.is_allowed("192.168.1.2").await);
    assert!(!controller.is_allowed("8.8.8.8").await);
}

#[tokio::test]
async fn test_dynamic_rule_updates() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.1");

    let controller = IpAccessControl::new(config).unwrap();

    // Initial state
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(!controller.is_allowed("192.168.1.2").await);

    // Add new IP
    controller.add_to_allowlist("192.168.1.2").await.unwrap();
    assert!(controller.is_allowed("192.168.1.2").await);

    // Add CIDR range
    controller.add_to_allowlist("10.0.0.0/24").await.unwrap();
    assert!(controller.is_allowed("10.0.0.1").await);
    assert!(controller.is_allowed("10.0.0.255").await);

    // Remove IP
    assert!(controller.remove_from_allowlist("192.168.1.2").await);
    assert!(!controller.is_allowed("192.168.1.2").await);

    // Verify list contents
    let allowlist = controller.get_allowlist().await;
    assert!(allowlist.contains(&"192.168.1.1".to_string()));
    assert!(allowlist.contains(&"10.0.0.0/24".to_string()));
    assert!(!allowlist.contains(&"192.168.1.2".to_string()));
}

#[tokio::test]
async fn test_ipv6_support() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("2001:db8::/32")
        .allow_ip("::1")
        .allow_ip("fe80::/10");

    let controller = IpAccessControl::new(config).unwrap();

    // Allowed IPv6
    assert!(controller.is_allowed("2001:db8::1").await);
    assert!(controller.is_allowed("2001:db8:ffff::1").await);
    assert!(controller.is_allowed("::1").await);
    assert!(controller.is_allowed("fe80::1").await);

    // Not allowed IPv6
    assert!(!controller.is_allowed("2001:db9::1").await);
    assert!(!controller.is_allowed("::2").await);
}

#[tokio::test]
async fn test_mixed_ipv4_ipv6() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.0/24")
        .allow_ip("2001:db8::/32");

    let controller = IpAccessControl::new(config).unwrap();

    // IPv4
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(!controller.is_allowed("192.168.2.1").await);

    // IPv6
    assert!(controller.is_allowed("2001:db8::1").await);
    assert!(!controller.is_allowed("2001:db9::1").await);
}

#[test]
fn test_config_serialization_roundtrip() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.0/24")
        .allow_ip("10.0.0.1")
        .always_allow_ip("127.0.0.1")
        .trust_proxy(true)
        .with_max_proxy_hops(2)
        .with_blocked_message("Access denied")
        .with_blocked_status(403);

    let json = serde_json::to_string(&config).unwrap();
    let deserialized: IpAccessConfig = serde_json::from_str(&json).unwrap();

    assert_eq!(config.enabled, deserialized.enabled);
    assert_eq!(config.mode, deserialized.mode);
    assert_eq!(config.allowlist.len(), deserialized.allowlist.len());
    assert_eq!(config.trust_proxy, deserialized.trust_proxy);
}

#[test]
fn test_yaml_config() {
    let yaml = r#"
enabled: true
mode: allowlist
allowlist:
  - value: 192.168.1.0/24
    description: Office network
    enabled: true
  - value: 10.0.0.1
    description: VPN server
    enabled: true
blocklist:
  - value: 192.168.1.100
    description: Blocked device
    enabled: true
always_allow:
  - 127.0.0.1
exclude_paths:
  - /health
  - /metrics
trust_proxy: true
max_proxy_hops: 2
blocked_message: Access denied
blocked_status: 403
log_blocked: true
"#;

    let config: IpAccessConfig = serde_yaml::from_str(yaml).unwrap();
    assert!(config.enabled);
    assert_eq!(config.mode, IpAccessMode::Allowlist);
    assert_eq!(config.allowlist.len(), 2);
    assert_eq!(config.blocklist.len(), 1);
    assert!(config.trust_proxy);
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_disabled_controller() {
    let controller = IpAccessControl::disabled();

    // All IPs should be allowed when disabled
    assert!(controller.is_allowed("192.168.1.1").await);
    assert!(controller.is_allowed("10.0.0.1").await);
    assert!(controller.is_allowed("8.8.8.8").await);
}

#[tokio::test]
async fn test_invalid_ip_handling() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.0/24");

    let controller = IpAccessControl::new(config).unwrap();

    // Invalid IPs should be denied
    assert!(!controller.is_allowed("invalid").await);
    assert!(!controller.is_allowed("").await);
    assert!(!controller.is_allowed("256.256.256.256").await);
}

#[tokio::test]
async fn test_concurrent_access() {
    let config = IpAccessConfig::new()
        .enable()
        .with_mode(IpAccessMode::Allowlist)
        .allow_ip("192.168.1.0/24");

    let controller = Arc::new(IpAccessControl::new(config).unwrap());

    let mut handles = Vec::new();

    for i in 0..10 {
        let controller = controller.clone();
        let handle = tokio::spawn(async move {
            for j in 0..100 {
                let ip = format!("192.168.1.{}", (i * 10 + j) % 256);
                let _ = controller.is_allowed(&ip).await;
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }
}

#[test]
fn test_ip_rule_creation() {
    // Valid single IP
    let rule = IpRule::new("192.168.1.1").unwrap();
    assert!(rule.ip.is_some());
    assert!(rule.cidr.is_none());

    // Valid CIDR
    let rule = IpRule::new("192.168.1.0/24").unwrap();
    assert!(rule.ip.is_none());
    assert!(rule.cidr.is_some());

    // Invalid IP
    assert!(IpRule::new("invalid").is_err());

    // Invalid CIDR
    assert!(IpRule::new("192.168.1.0/33").is_err());
}

#[test]
fn test_cidr_edge_cases() {
    // /0 should match everything
    let rule = IpRule::new("0.0.0.0/0").unwrap();
    let ip1: std::net::IpAddr = "192.168.1.1".parse().unwrap();
    let ip2: std::net::IpAddr = "10.0.0.1".parse().unwrap();
    assert!(rule.matches(&ip1));
    assert!(rule.matches(&ip2));

    // /32 should match only one IP
    let rule = IpRule::new("192.168.1.1/32").unwrap();
    let ip1: std::net::IpAddr = "192.168.1.1".parse().unwrap();
    let ip2: std::net::IpAddr = "192.168.1.2".parse().unwrap();
    assert!(rule.matches(&ip1));
    assert!(!rule.matches(&ip2));
}

#[test]
fn test_proxy_ip_extraction() {
    // No proxy trust
    let config = IpAccessConfig::new();
    let controller = IpAccessControl::new(config).unwrap();
    let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1"));
    assert_eq!(ip, "192.168.1.1");

    // With proxy trust, 1 hop
    let config = IpAccessConfig::new()
        .trust_proxy(true)
        .with_max_proxy_hops(1);
    let controller = IpAccessControl::new(config).unwrap();
    let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1, 172.16.0.1"));
    assert_eq!(ip, "172.16.0.1");

    // With proxy trust, 2 hops
    let config = IpAccessConfig::new()
        .trust_proxy(true)
        .with_max_proxy_hops(2);
    let controller = IpAccessControl::new(config).unwrap();
    let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1, 172.16.0.1, 8.8.8.8"));
    assert_eq!(ip, "172.16.0.1");

    // No X-Forwarded-For header
    let ip = controller.extract_client_ip("192.168.1.1", None);
    assert_eq!(ip, "192.168.1.1");
}
