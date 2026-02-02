//! IP Access Control implementation
//!
//! The main controller for IP-based access control.

use std::net::IpAddr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::config::IpAccessConfig;
use super::types::{IpAccessError, IpAccessMode, IpAccessResult, IpRule};

/// IP Access Control controller
pub struct IpAccessControl {
    config: IpAccessConfig,
    allowlist: Arc<RwLock<Vec<IpRule>>>,
    blocklist: Arc<RwLock<Vec<IpRule>>>,
    always_allow: Arc<RwLock<Vec<IpRule>>>,
}

impl IpAccessControl {
    /// Create a new IP access controller
    pub fn new(config: IpAccessConfig) -> IpAccessResult<Self> {
        let allowlist = config.build_allowlist_rules()?;
        let blocklist = config.build_blocklist_rules()?;
        let always_allow = config.build_always_allow_rules()?;

        info!(
            "IP access control initialized: mode={:?}, allowlist={}, blocklist={}, always_allow={}",
            config.mode,
            allowlist.len(),
            blocklist.len(),
            always_allow.len()
        );

        Ok(Self {
            config,
            allowlist: Arc::new(RwLock::new(allowlist)),
            blocklist: Arc::new(RwLock::new(blocklist)),
            always_allow: Arc::new(RwLock::new(always_allow)),
        })
    }

    /// Create a shared controller
    pub fn shared(config: IpAccessConfig) -> IpAccessResult<Arc<Self>> {
        Ok(Arc::new(Self::new(config)?))
    }

    /// Create a disabled controller
    pub fn disabled() -> Self {
        Self {
            config: IpAccessConfig::default(),
            allowlist: Arc::new(RwLock::new(Vec::new())),
            blocklist: Arc::new(RwLock::new(Vec::new())),
            always_allow: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if IP access control is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled && self.config.mode != IpAccessMode::Disabled
    }

    /// Check if an IP address is allowed
    pub async fn is_allowed(&self, ip_str: &str) -> bool {
        if !self.is_enabled() {
            return true;
        }

        let ip = match IpAddr::from_str(ip_str) {
            Ok(ip) => ip,
            Err(_) => {
                warn!("Invalid IP address: {}", ip_str);
                return false;
            }
        };

        self.is_ip_allowed(&ip).await
    }

    /// Check if an IP address is allowed (parsed)
    pub async fn is_ip_allowed(&self, ip: &IpAddr) -> bool {
        if !self.is_enabled() {
            return true;
        }

        // Check always-allow list first
        {
            let always_allow = self.always_allow.read().await;
            for rule in always_allow.iter() {
                if rule.matches(ip) {
                    debug!("IP {} matched always-allow rule", ip);
                    return true;
                }
            }
        }

        match self.config.mode {
            IpAccessMode::Allowlist => self.check_allowlist(ip).await,
            IpAccessMode::Blocklist => self.check_blocklist(ip).await,
            IpAccessMode::Disabled => true,
        }
    }

    /// Check IP against allowlist
    async fn check_allowlist(&self, ip: &IpAddr) -> bool {
        let allowlist = self.allowlist.read().await;

        for rule in allowlist.iter() {
            if rule.matches(ip) {
                debug!("IP {} matched allowlist rule: {}", ip, rule.value);
                return true;
            }
        }

        debug!("IP {} not in allowlist", ip);
        false
    }

    /// Check IP against blocklist
    async fn check_blocklist(&self, ip: &IpAddr) -> bool {
        let blocklist = self.blocklist.read().await;

        for rule in blocklist.iter() {
            if rule.matches(ip) {
                debug!("IP {} matched blocklist rule: {}", ip, rule.value);
                return false;
            }
        }

        true
    }

    /// Check access and return detailed result
    pub async fn check_access(&self, ip_str: &str) -> Result<(), IpAccessError> {
        if self.is_allowed(ip_str).await {
            Ok(())
        } else {
            Err(IpAccessError::AccessDenied(ip_str.to_string()))
        }
    }

    /// Add an IP to the allowlist at runtime
    pub async fn add_to_allowlist(&self, ip: impl Into<String>) -> IpAccessResult<()> {
        let rule = IpRule::new(ip)?;
        let mut allowlist = self.allowlist.write().await;
        allowlist.push(rule);
        Ok(())
    }

    /// Add an IP to the blocklist at runtime
    pub async fn add_to_blocklist(&self, ip: impl Into<String>) -> IpAccessResult<()> {
        let rule = IpRule::new(ip)?;
        let mut blocklist = self.blocklist.write().await;
        blocklist.push(rule);
        Ok(())
    }

    /// Remove an IP from the allowlist
    pub async fn remove_from_allowlist(&self, ip: &str) -> bool {
        let mut allowlist = self.allowlist.write().await;
        let len_before = allowlist.len();
        allowlist.retain(|r| r.value != ip);
        allowlist.len() < len_before
    }

    /// Remove an IP from the blocklist
    pub async fn remove_from_blocklist(&self, ip: &str) -> bool {
        let mut blocklist = self.blocklist.write().await;
        let len_before = blocklist.len();
        blocklist.retain(|r| r.value != ip);
        blocklist.len() < len_before
    }

    /// Get the current allowlist
    pub async fn get_allowlist(&self) -> Vec<String> {
        let allowlist = self.allowlist.read().await;
        allowlist.iter().map(|r| r.value.clone()).collect()
    }

    /// Get the current blocklist
    pub async fn get_blocklist(&self) -> Vec<String> {
        let blocklist = self.blocklist.read().await;
        blocklist.iter().map(|r| r.value.clone()).collect()
    }

    /// Get configuration
    pub fn config(&self) -> &IpAccessConfig {
        &self.config
    }

    /// Check if a path should be excluded from IP checking
    pub fn is_path_excluded(&self, path: &str) -> bool {
        self.config.is_path_excluded(path)
    }

    /// Extract client IP from request headers
    pub fn extract_client_ip(&self, remote_addr: &str, forwarded_for: Option<&str>) -> String {
        if !self.config.trust_proxy {
            return remote_addr.to_string();
        }

        if let Some(xff) = forwarded_for {
            let ips: Vec<&str> = xff.split(',').map(|s| s.trim()).collect();

            // Get the IP at the configured hop position from the right
            let hop_index = ips.len().saturating_sub(self.config.max_proxy_hops);
            if hop_index < ips.len() {
                return ips[hop_index].to_string();
            }
        }

        remote_addr.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_allowlist_config() -> IpAccessConfig {
        IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("192.168.1.0/24")
            .allow_ip("10.0.0.1")
    }

    fn create_blocklist_config() -> IpAccessConfig {
        IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Blocklist)
            .block_ip("192.168.1.100")
            .block_ip("10.0.0.0/8")
    }

    #[test]
    fn test_controller_creation() {
        let config = create_allowlist_config();
        let controller = IpAccessControl::new(config).unwrap();
        assert!(controller.is_enabled());
    }

    #[test]
    fn test_disabled_controller() {
        let controller = IpAccessControl::disabled();
        assert!(!controller.is_enabled());
    }

    #[tokio::test]
    async fn test_allowlist_mode() {
        let config = create_allowlist_config();
        let controller = IpAccessControl::new(config).unwrap();

        // Allowed IPs
        assert!(controller.is_allowed("192.168.1.1").await);
        assert!(controller.is_allowed("192.168.1.255").await);
        assert!(controller.is_allowed("10.0.0.1").await);

        // Not allowed IPs
        assert!(!controller.is_allowed("192.168.2.1").await);
        assert!(!controller.is_allowed("10.0.0.2").await);
        assert!(!controller.is_allowed("8.8.8.8").await);
    }

    #[tokio::test]
    async fn test_blocklist_mode() {
        let config = create_blocklist_config();
        let controller = IpAccessControl::new(config).unwrap();

        // Blocked IPs
        assert!(!controller.is_allowed("192.168.1.100").await);
        assert!(!controller.is_allowed("10.0.0.1").await);
        assert!(!controller.is_allowed("10.255.255.255").await);

        // Allowed IPs
        assert!(controller.is_allowed("192.168.1.1").await);
        assert!(controller.is_allowed("8.8.8.8").await);
    }

    #[tokio::test]
    async fn test_always_allow() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Blocklist)
            .block_ip("192.168.1.0/24")
            .always_allow_ip("192.168.1.100");

        let controller = IpAccessControl::new(config).unwrap();

        // 192.168.1.100 should be allowed despite being in blocked range
        assert!(controller.is_allowed("192.168.1.100").await);

        // Other IPs in range should be blocked
        assert!(!controller.is_allowed("192.168.1.1").await);
    }

    #[tokio::test]
    async fn test_disabled_mode() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Disabled);

        let controller = IpAccessControl::new(config).unwrap();

        // All IPs should be allowed
        assert!(controller.is_allowed("192.168.1.1").await);
        assert!(controller.is_allowed("10.0.0.1").await);
    }

    #[tokio::test]
    async fn test_dynamic_allowlist() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("192.168.1.1");

        let controller = IpAccessControl::new(config).unwrap();

        assert!(controller.is_allowed("192.168.1.1").await);
        assert!(!controller.is_allowed("192.168.1.2").await);

        // Add new IP
        controller.add_to_allowlist("192.168.1.2").await.unwrap();
        assert!(controller.is_allowed("192.168.1.2").await);

        // Remove IP
        controller.remove_from_allowlist("192.168.1.2").await;
        assert!(!controller.is_allowed("192.168.1.2").await);
    }

    #[tokio::test]
    async fn test_dynamic_blocklist() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Blocklist);

        let controller = IpAccessControl::new(config).unwrap();

        assert!(controller.is_allowed("192.168.1.100").await);

        // Add to blocklist
        controller.add_to_blocklist("192.168.1.100").await.unwrap();
        assert!(!controller.is_allowed("192.168.1.100").await);

        // Remove from blocklist
        controller.remove_from_blocklist("192.168.1.100").await;
        assert!(controller.is_allowed("192.168.1.100").await);
    }

    #[tokio::test]
    async fn test_check_access() {
        let config = create_allowlist_config();
        let controller = IpAccessControl::new(config).unwrap();

        assert!(controller.check_access("192.168.1.1").await.is_ok());
        assert!(controller.check_access("8.8.8.8").await.is_err());
    }

    #[tokio::test]
    async fn test_get_lists() {
        let config = create_allowlist_config();
        let controller = IpAccessControl::new(config).unwrap();

        let allowlist = controller.get_allowlist().await;
        assert_eq!(allowlist.len(), 2);
        assert!(allowlist.contains(&"192.168.1.0/24".to_string()));
    }

    #[test]
    fn test_path_exclusion() {
        let config = IpAccessConfig::default();
        let controller = IpAccessControl::new(config).unwrap();

        assert!(controller.is_path_excluded(r"/health"));
        assert!(!controller.is_path_excluded(r"/api/chat"));
    }

    #[test]
    fn test_extract_client_ip_no_proxy() {
        let config = IpAccessConfig::new();
        let controller = IpAccessControl::new(config).unwrap();

        let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1, 172.16.0.1"));
        assert_eq!(ip, "192.168.1.1");
    }

    #[test]
    fn test_extract_client_ip_with_proxy() {
        let config = IpAccessConfig::new()
            .trust_proxy(true)
            .with_max_proxy_hops(1);
        let controller = IpAccessControl::new(config).unwrap();

        // With 1 hop, should get the last IP in X-Forwarded-For
        let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1, 172.16.0.1"));
        assert_eq!(ip, "172.16.0.1");
    }

    #[test]
    fn test_extract_client_ip_with_multiple_hops() {
        let config = IpAccessConfig::new()
            .trust_proxy(true)
            .with_max_proxy_hops(2);
        let controller = IpAccessControl::new(config).unwrap();

        // With 2 hops, should get second-to-last IP
        let ip = controller.extract_client_ip("192.168.1.1", Some("10.0.0.1, 172.16.0.1, 8.8.8.8"));
        assert_eq!(ip, "172.16.0.1");
    }

    #[tokio::test]
    async fn test_invalid_ip() {
        let config = create_allowlist_config();
        let controller = IpAccessControl::new(config).unwrap();

        // Invalid IP should be denied
        assert!(!controller.is_allowed("invalid").await);
    }

    #[tokio::test]
    async fn test_ipv6() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("2001:db8::/32");

        let controller = IpAccessControl::new(config).unwrap();

        assert!(controller.is_allowed("2001:db8::1").await);
        assert!(controller.is_allowed("2001:db8:ffff::1").await);
        assert!(!controller.is_allowed("2001:db9::1").await);
    }
}
