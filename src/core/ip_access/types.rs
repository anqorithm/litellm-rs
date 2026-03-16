//! Core types for the IP Access Control system

use crate::config::models::defaults::default_true;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;
use std::str::FromStr;
use thiserror::Error;

/// Error types for IP access control operations
#[derive(Debug, Error)]
pub enum IpAccessError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Invalid IP address
    #[error("Invalid IP address: {0}")]
    InvalidIp(String),

    /// Invalid CIDR notation
    #[error("Invalid CIDR notation: {0}")]
    InvalidCidr(String),

    /// Access denied
    #[error("Access denied for IP: {0}")]
    AccessDenied(String),
}

/// Result type for IP access control operations
pub type IpAccessResult<T> = Result<T, IpAccessError>;

/// IP access control mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IpAccessMode {
    /// Allowlist mode - only listed IPs are allowed
    Allowlist,
    /// Blocklist mode - listed IPs are blocked, all others allowed
    #[default]
    Blocklist,
    /// Disabled - no IP filtering
    Disabled,
}

/// An IP rule (single IP or CIDR range)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpRule {
    /// The IP or CIDR string
    pub value: String,
    /// Parsed IP address (for single IPs)
    #[serde(skip)]
    pub ip: Option<IpAddr>,
    /// Parsed CIDR network
    #[serde(skip)]
    pub cidr: Option<CidrRange>,
    /// Optional description
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this rule is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl IpRule {
    /// Create a new IP rule from a string
    pub fn new(value: impl Into<String>) -> IpAccessResult<Self> {
        let value = value.into();
        let (ip, cidr) = Self::parse(&value)?;

        Ok(Self {
            value,
            ip,
            cidr,
            description: None,
            enabled: true,
        })
    }

    /// Parse an IP or CIDR string
    fn parse(value: &str) -> IpAccessResult<(Option<IpAddr>, Option<CidrRange>)> {
        let value = value.trim();

        // Try parsing as CIDR first
        if value.contains('/') {
            let cidr = CidrRange::from_str(value)?;
            return Ok((None, Some(cidr)));
        }

        // Try parsing as single IP
        let ip =
            IpAddr::from_str(value).map_err(|_| IpAccessError::InvalidIp(value.to_string()))?;

        Ok((Some(ip), None))
    }

    /// Set description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Check if this rule matches an IP address
    pub fn matches(&self, ip: &IpAddr) -> bool {
        if !self.enabled {
            return false;
        }

        if let Some(ref rule_ip) = self.ip {
            return rule_ip == ip;
        }

        if let Some(ref cidr) = self.cidr {
            return cidr.contains(ip);
        }

        false
    }
}

/// A CIDR range
#[derive(Debug, Clone)]
pub struct CidrRange {
    /// Network address
    pub network: IpAddr,
    /// Prefix length
    pub prefix_len: u8,
    /// Mask for IPv4
    mask_v4: Option<u32>,
    /// Mask for IPv6
    mask_v6: Option<u128>,
}

impl CidrRange {
    /// Create a new CIDR range
    pub fn new(network: IpAddr, prefix_len: u8) -> IpAccessResult<Self> {
        let (mask_v4, mask_v6) = match network {
            IpAddr::V4(_) => {
                if prefix_len > 32 {
                    return Err(IpAccessError::InvalidCidr(format!(
                        "IPv4 prefix length {} exceeds 32",
                        prefix_len
                    )));
                }
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !0u32 << (32 - prefix_len)
                };
                (Some(mask), None)
            }
            IpAddr::V6(_) => {
                if prefix_len > 128 {
                    return Err(IpAccessError::InvalidCidr(format!(
                        "IPv6 prefix length {} exceeds 128",
                        prefix_len
                    )));
                }
                let mask = if prefix_len == 0 {
                    0
                } else {
                    !0u128 << (128 - prefix_len)
                };
                (None, Some(mask))
            }
        };

        Ok(Self {
            network,
            prefix_len,
            mask_v4,
            mask_v6,
        })
    }

    /// Check if this CIDR range contains an IP address
    pub fn contains(&self, ip: &IpAddr) -> bool {
        match (self.network, ip) {
            (IpAddr::V4(net), IpAddr::V4(addr)) => {
                if let Some(mask) = self.mask_v4 {
                    let net_bits = u32::from(net);
                    let addr_bits = u32::from(*addr);
                    (net_bits & mask) == (addr_bits & mask)
                } else {
                    false
                }
            }
            (IpAddr::V6(net), IpAddr::V6(addr)) => {
                if let Some(mask) = self.mask_v6 {
                    let net_bits = u128::from(net);
                    let addr_bits = u128::from(*addr);
                    (net_bits & mask) == (addr_bits & mask)
                } else {
                    false
                }
            }
            _ => false, // IPv4 and IPv6 don't match
        }
    }
}

impl FromStr for CidrRange {
    type Err = IpAccessError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 2 {
            return Err(IpAccessError::InvalidCidr(format!(
                "Invalid CIDR format: {}",
                s
            )));
        }

        let network = IpAddr::from_str(parts[0]).map_err(|_| {
            IpAccessError::InvalidCidr(format!("Invalid network address: {}", parts[0]))
        })?;

        let prefix_len: u8 = parts[1].parse().map_err(|_| {
            IpAccessError::InvalidCidr(format!("Invalid prefix length: {}", parts[1]))
        })?;

        Self::new(network, prefix_len)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ip_access_mode() {
        assert_eq!(IpAccessMode::default(), IpAccessMode::Blocklist);
    }

    #[test]
    fn test_ip_rule_single_ip() {
        let rule = IpRule::new("192.168.1.1").unwrap();
        assert!(rule.ip.is_some());
        assert!(rule.cidr.is_none());

        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(rule.matches(&ip));

        let other_ip: IpAddr = "192.168.1.2".parse().unwrap();
        assert!(!rule.matches(&other_ip));
    }

    #[test]
    fn test_ip_rule_cidr() {
        let rule = IpRule::new("192.168.1.0/24").unwrap();
        assert!(rule.ip.is_none());
        assert!(rule.cidr.is_some());

        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.255".parse().unwrap();
        let ip3: IpAddr = "192.168.2.1".parse().unwrap();

        assert!(rule.matches(&ip1));
        assert!(rule.matches(&ip2));
        assert!(!rule.matches(&ip3));
    }

    #[test]
    fn test_ip_rule_ipv6() {
        let rule = IpRule::new("2001:db8::1").unwrap();
        assert!(rule.ip.is_some());

        let ip: IpAddr = "2001:db8::1".parse().unwrap();
        assert!(rule.matches(&ip));
    }

    #[test]
    fn test_ip_rule_ipv6_cidr() {
        let rule = IpRule::new("2001:db8::/32").unwrap();
        assert!(rule.cidr.is_some());

        let ip1: IpAddr = "2001:db8::1".parse().unwrap();
        let ip2: IpAddr = "2001:db8:ffff::1".parse().unwrap();
        let ip3: IpAddr = "2001:db9::1".parse().unwrap();

        assert!(rule.matches(&ip1));
        assert!(rule.matches(&ip2));
        assert!(!rule.matches(&ip3));
    }

    #[test]
    fn test_ip_rule_disabled() {
        let mut rule = IpRule::new("192.168.1.1").unwrap();
        rule.enabled = false;

        let ip: IpAddr = "192.168.1.1".parse().unwrap();
        assert!(!rule.matches(&ip));
    }

    #[test]
    fn test_ip_rule_with_description() {
        let rule = IpRule::new("192.168.1.1")
            .unwrap()
            .with_description("Office IP");

        assert_eq!(rule.description, Some("Office IP".to_string()));
    }

    #[test]
    fn test_cidr_range_ipv4() {
        let cidr = CidrRange::from_str("10.0.0.0/8").unwrap();

        let ip1: IpAddr = "10.0.0.1".parse().unwrap();
        let ip2: IpAddr = "10.255.255.255".parse().unwrap();
        let ip3: IpAddr = "11.0.0.1".parse().unwrap();

        assert!(cidr.contains(&ip1));
        assert!(cidr.contains(&ip2));
        assert!(!cidr.contains(&ip3));
    }

    #[test]
    fn test_cidr_range_ipv4_32() {
        let cidr = CidrRange::from_str("192.168.1.1/32").unwrap();

        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "192.168.1.2".parse().unwrap();

        assert!(cidr.contains(&ip1));
        assert!(!cidr.contains(&ip2));
    }

    #[test]
    fn test_cidr_range_ipv4_0() {
        let cidr = CidrRange::from_str("0.0.0.0/0").unwrap();

        let ip1: IpAddr = "192.168.1.1".parse().unwrap();
        let ip2: IpAddr = "10.0.0.1".parse().unwrap();

        assert!(cidr.contains(&ip1));
        assert!(cidr.contains(&ip2));
    }

    #[test]
    fn test_invalid_ip() {
        let result = IpRule::new("invalid");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_cidr() {
        let result = IpRule::new("192.168.1.0/33");
        assert!(result.is_err());

        let result = IpRule::new("192.168.1.0/abc");
        assert!(result.is_err());
    }

    #[test]
    fn test_ipv4_ipv6_mismatch() {
        let cidr = CidrRange::from_str("192.168.1.0/24").unwrap();
        let ipv6: IpAddr = "2001:db8::1".parse().unwrap();

        assert!(!cidr.contains(&ipv6));
    }
}
