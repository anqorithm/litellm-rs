//! Configuration for the IP Access Control system

use crate::config::models::defaults::default_true;
use serde::{Deserialize, Serialize};

use super::types::{IpAccessMode, IpAccessResult, IpRule};

/// Main configuration for IP access control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpAccessConfig {
    /// Whether IP access control is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Access control mode
    #[serde(default)]
    pub mode: IpAccessMode,

    /// Allowlist rules (used in Allowlist mode)
    #[serde(default)]
    pub allowlist: Vec<IpRuleConfig>,

    /// Blocklist rules (used in Blocklist mode)
    #[serde(default)]
    pub blocklist: Vec<IpRuleConfig>,

    /// Always allowed IPs (bypass blocklist)
    #[serde(default)]
    pub always_allow: Vec<String>,

    /// Paths to exclude from IP checking
    #[serde(default = "default_excluded_paths")]
    pub exclude_paths: Vec<String>,

    /// Whether to trust X-Forwarded-For header
    #[serde(default)]
    pub trust_proxy: bool,

    /// Maximum number of proxy hops to trust
    #[serde(default = "default_max_proxy_hops")]
    pub max_proxy_hops: usize,

    /// Custom message for blocked requests
    #[serde(default = "default_blocked_message")]
    pub blocked_message: String,

    /// HTTP status code for blocked requests
    #[serde(default = "default_blocked_status")]
    pub blocked_status: u16,

    /// Whether to log blocked requests
    #[serde(default = "default_true")]
    pub log_blocked: bool,
}

fn default_excluded_paths() -> Vec<String> {
    vec![
        r"/health".to_string(),
        r"/ready".to_string(),
        r"/live".to_string(),
    ]
}

fn default_max_proxy_hops() -> usize {
    1
}

fn default_blocked_message() -> String {
    "Access denied".to_string()
}

fn default_blocked_status() -> u16 {
    403
}

impl Default for IpAccessConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            mode: IpAccessMode::Blocklist,
            allowlist: Vec::new(),
            blocklist: Vec::new(),
            always_allow: Vec::new(),
            exclude_paths: default_excluded_paths(),
            trust_proxy: false,
            max_proxy_hops: default_max_proxy_hops(),
            blocked_message: default_blocked_message(),
            blocked_status: default_blocked_status(),
            log_blocked: true,
        }
    }
}

impl IpAccessConfig {
    /// Create a new IP access config
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable IP access control
    pub fn enable(mut self) -> Self {
        self.enabled = true;
        self
    }

    /// Set access mode
    pub fn with_mode(mut self, mode: IpAccessMode) -> Self {
        self.mode = mode;
        self
    }

    /// Add an IP to the allowlist
    pub fn allow_ip(mut self, ip: impl Into<String>) -> Self {
        self.allowlist.push(IpRuleConfig {
            value: ip.into(),
            description: None,
            enabled: true,
        });
        self
    }

    /// Add an IP to the blocklist
    pub fn block_ip(mut self, ip: impl Into<String>) -> Self {
        self.blocklist.push(IpRuleConfig {
            value: ip.into(),
            description: None,
            enabled: true,
        });
        self
    }

    /// Add an always-allowed IP
    pub fn always_allow_ip(mut self, ip: impl Into<String>) -> Self {
        self.always_allow.push(ip.into());
        self
    }

    /// Add excluded path
    pub fn exclude_path(mut self, path: impl Into<String>) -> Self {
        self.exclude_paths.push(path.into());
        self
    }

    /// Enable proxy trust
    pub fn trust_proxy(mut self, trust: bool) -> Self {
        self.trust_proxy = trust;
        self
    }

    /// Set max proxy hops
    pub fn with_max_proxy_hops(mut self, hops: usize) -> Self {
        self.max_proxy_hops = hops;
        self
    }

    /// Set blocked message
    pub fn with_blocked_message(mut self, message: impl Into<String>) -> Self {
        self.blocked_message = message.into();
        self
    }

    /// Set blocked status code
    pub fn with_blocked_status(mut self, status: u16) -> Self {
        self.blocked_status = status;
        self
    }

    /// Check if a path should be excluded
    pub fn is_path_excluded(&self, path: &str) -> bool {
        self.exclude_paths.iter().any(|p| path.starts_with(p))
    }

    /// Build IP rules from config
    pub fn build_allowlist_rules(&self) -> IpAccessResult<Vec<IpRule>> {
        self.allowlist
            .iter()
            .filter(|r| r.enabled)
            .map(|r| {
                let mut rule = IpRule::new(&r.value)?;
                if let Some(ref desc) = r.description {
                    rule = rule.with_description(desc);
                }
                Ok(rule)
            })
            .collect()
    }

    /// Build blocklist rules from config
    pub fn build_blocklist_rules(&self) -> IpAccessResult<Vec<IpRule>> {
        self.blocklist
            .iter()
            .filter(|r| r.enabled)
            .map(|r| {
                let mut rule = IpRule::new(&r.value)?;
                if let Some(ref desc) = r.description {
                    rule = rule.with_description(desc);
                }
                Ok(rule)
            })
            .collect()
    }

    /// Build always-allow rules from config
    pub fn build_always_allow_rules(&self) -> IpAccessResult<Vec<IpRule>> {
        self.always_allow.iter().map(IpRule::new).collect()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.enabled {
            match self.mode {
                IpAccessMode::Allowlist if self.allowlist.is_empty() => {
                    return Err("Allowlist mode enabled but no IPs in allowlist".to_string());
                }
                _ => {}
            }
        }
        Ok(())
    }
}

/// IP rule configuration (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRuleConfig {
    /// IP address or CIDR
    pub value: String,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl IpRuleConfig {
    /// Create a new rule config
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            description: None,
            enabled: true,
        }
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = IpAccessConfig::default();
        assert!(!config.enabled);
        assert_eq!(config.mode, IpAccessMode::Blocklist);
        assert!(config.allowlist.is_empty());
        assert!(config.blocklist.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("192.168.1.0/24")
            .allow_ip("10.0.0.1")
            .always_allow_ip("127.0.0.1")
            .exclude_path(r"/internal")
            .trust_proxy(true)
            .with_max_proxy_hops(2)
            .with_blocked_message("Forbidden")
            .with_blocked_status(403);

        assert!(config.enabled);
        assert_eq!(config.mode, IpAccessMode::Allowlist);
        assert_eq!(config.allowlist.len(), 2);
        assert_eq!(config.always_allow.len(), 1);
        assert!(config.trust_proxy);
        assert_eq!(config.max_proxy_hops, 2);
    }

    #[test]
    fn test_blocklist_config() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Blocklist)
            .block_ip("192.168.1.100")
            .block_ip("10.0.0.0/8");

        assert_eq!(config.blocklist.len(), 2);
    }

    #[test]
    fn test_path_exclusion() {
        let config = IpAccessConfig::default();
        assert!(config.is_path_excluded(r"/health"));
        assert!(config.is_path_excluded(r"/health/live"));
        assert!(config.is_path_excluded(r"/ready"));
        assert!(!config.is_path_excluded(r"/api/chat"));
    }

    #[test]
    fn test_build_rules() {
        let config = IpAccessConfig::new()
            .allow_ip("192.168.1.0/24")
            .allow_ip("10.0.0.1");

        let rules = config.build_allowlist_rules().unwrap();
        assert_eq!(rules.len(), 2);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("192.168.1.0/24");
        assert!(valid_config.validate().is_ok());

        let invalid_config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist);
        assert!(invalid_config.validate().is_err());

        // Blocklist mode with empty blocklist is valid (allows all)
        let blocklist_config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Blocklist);
        assert!(blocklist_config.validate().is_ok());
    }

    #[test]
    fn test_config_serialization() {
        let config = IpAccessConfig::new()
            .enable()
            .with_mode(IpAccessMode::Allowlist)
            .allow_ip("192.168.1.0/24");

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: IpAccessConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(config.enabled, deserialized.enabled);
        assert_eq!(config.mode, deserialized.mode);
        assert_eq!(config.allowlist.len(), deserialized.allowlist.len());
    }

    #[test]
    fn test_ip_rule_config() {
        let rule = IpRuleConfig::new("192.168.1.0/24").with_description("Office network");

        assert_eq!(rule.value, "192.168.1.0/24");
        assert_eq!(rule.description, Some("Office network".to_string()));
        assert!(rule.enabled);
    }
}
