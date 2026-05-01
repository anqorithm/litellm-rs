//! SSRF guard helpers for outbound URLs.

use std::fmt;
use std::net::IpAddr;
use url::Url;

/// Error returned when an outbound URL is not safe to use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SsrfError {
    InvalidUrl { url: String, message: String },
    UnsupportedScheme { scheme: String },
    MissingHost { url: String },
    PrivateOrReservedHost { host: String },
}

impl fmt::Display for SsrfError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SsrfError::InvalidUrl { url, message } => {
                write!(f, "Outbound URL '{url}' is invalid: {message}")
            }
            SsrfError::UnsupportedScheme { scheme } => {
                write!(f, "Outbound URL scheme '{scheme}' is not allowed")
            }
            SsrfError::MissingHost { url } => {
                write!(f, "Outbound URL has an invalid or missing host: {url}")
            }
            SsrfError::PrivateOrReservedHost { host } => write!(
                f,
                "Outbound URL targets a private or reserved address '{host}', which is not allowed (SSRF protection)"
            ),
        }
    }
}

impl std::error::Error for SsrfError {}

/// Parse and validate an outbound URL string.
pub fn validate_outbound_url_str(raw_url: &str) -> Result<Url, SsrfError> {
    let url = Url::parse(raw_url).map_err(|error| SsrfError::InvalidUrl {
        url: raw_url.to_string(),
        message: error.to_string(),
    })?;

    validate_outbound_url(&url)?;
    Ok(url)
}

/// Validate an already parsed outbound URL.
pub fn validate_outbound_url(url: &Url) -> Result<(), SsrfError> {
    match url.scheme() {
        "http" | "https" | "ws" | "wss" => {}
        scheme => {
            return Err(SsrfError::UnsupportedScheme {
                scheme: scheme.to_string(),
            });
        }
    }

    let host = extract_url_host(url.as_str()).ok_or_else(|| SsrfError::MissingHost {
        url: url.to_string(),
    })?;

    if is_private_or_reserved_host(&host) {
        return Err(SsrfError::PrivateOrReservedHost { host });
    }

    Ok(())
}

/// Extract the lowercase host portion from a URL string.
pub fn extract_url_host(raw_url: &str) -> Option<String> {
    let url = Url::parse(raw_url).ok()?;
    if !matches!(url.scheme(), "http" | "https" | "ws" | "wss") {
        return None;
    }

    url.host_str()
        .map(|host| host.trim_matches(['[', ']']).to_ascii_lowercase())
}

/// Returns true for private, loopback, link-local, or reserved hosts.
pub fn is_private_or_reserved_host(host: &str) -> bool {
    let normalized = host.trim().trim_matches(['[', ']']).to_ascii_lowercase();

    if normalized == "localhost"
        || normalized.ends_with(".localhost")
        || normalized == "metadata.google.internal"
        || normalized == "169.254.169.254"
    {
        return true;
    }

    if let Ok(ip) = normalized.parse::<IpAddr>() {
        return is_private_or_reserved_ip(&ip);
    }

    false
}

/// Check whether a parsed IP address falls within private or reserved ranges.
pub fn is_private_or_reserved_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            let octets = v4.octets();

            octets == [0, 0, 0, 0]
                || octets[0] == 10
                || octets[0] == 127
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || (octets[0] == 169 && octets[1] == 254)
                || (octets[0] == 172 && (16..=31).contains(&octets[1]))
                || (octets[0] == 192 && octets[1] == 168)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 0)
                || (octets[0] == 192 && octets[1] == 0 && octets[2] == 2)
                || (octets[0] == 198 && (18..=19).contains(&octets[1]))
                || (octets[0] == 198 && octets[1] == 51 && octets[2] == 100)
                || (octets[0] == 203 && octets[1] == 0 && octets[2] == 113)
                || v4.is_broadcast()
                || v4.is_multicast()
        }
        IpAddr::V6(v6) => {
            if v6.is_loopback() || v6.is_unspecified() || v6.is_multicast() {
                return true;
            }

            let segments = v6.segments();
            // fc00::/7 unique-local.
            if (segments[0] & 0xfe00) == 0xfc00 {
                return true;
            }
            // fe80::/10 link-local.
            if (segments[0] & 0xffc0) == 0xfe80 {
                return true;
            }
            // ::ffff:0:0/96 IPv4-mapped.
            if let Some(v4) = v6.to_ipv4_mapped() {
                return is_private_or_reserved_ip(&IpAddr::V4(v4));
            }

            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{Ipv4Addr, Ipv6Addr};

    #[test]
    fn extract_url_host_parses_standard_hosts() {
        assert_eq!(
            extract_url_host("https://example.com/path"),
            Some("example.com".to_string())
        );
        assert_eq!(
            extract_url_host("http://10.0.0.1:8080/api"),
            Some("10.0.0.1".to_string())
        );
        assert_eq!(
            extract_url_host("http://[::1]:9000/api"),
            Some("::1".to_string())
        );
        assert_eq!(extract_url_host("not a url"), None);
    }

    #[test]
    fn public_ipv4_addresses_are_allowed() {
        assert!(!is_private_or_reserved_ip(&IpAddr::V4(Ipv4Addr::new(
            8, 8, 8, 8
        ))));
        assert!(!is_private_or_reserved_ip(&IpAddr::V4(Ipv4Addr::new(
            1, 1, 1, 1
        ))));
    }

    #[test]
    fn private_and_reserved_ipv4_addresses_are_rejected() {
        for ip in [
            Ipv4Addr::new(0, 0, 0, 0),
            Ipv4Addr::new(10, 1, 2, 3),
            Ipv4Addr::new(100, 64, 0, 1),
            Ipv4Addr::new(127, 0, 0, 1),
            Ipv4Addr::new(169, 254, 169, 254),
            Ipv4Addr::new(172, 20, 0, 1),
            Ipv4Addr::new(192, 168, 0, 1),
            Ipv4Addr::new(198, 18, 0, 1),
        ] {
            assert!(is_private_or_reserved_ip(&IpAddr::V4(ip)), "{ip}");
        }
    }

    #[test]
    fn private_and_reserved_ipv6_addresses_are_rejected() {
        for ip in [
            Ipv6Addr::LOCALHOST,
            Ipv6Addr::UNSPECIFIED,
            "fc00::1".parse().unwrap(),
            "fd00::1".parse().unwrap(),
            "fe80::1".parse().unwrap(),
            "::ffff:127.0.0.1".parse().unwrap(),
        ] {
            assert!(is_private_or_reserved_ip(&IpAddr::V6(ip)), "{ip}");
        }
    }

    #[test]
    fn private_hostnames_are_rejected() {
        assert!(is_private_or_reserved_host("localhost"));
        assert!(is_private_or_reserved_host("my.localhost"));
        assert!(is_private_or_reserved_host("metadata.google.internal"));
    }

    #[test]
    fn validate_outbound_url_rejects_private_targets() {
        let url = Url::parse("http://169.254.169.254/latest/meta-data/").unwrap();
        assert!(matches!(
            validate_outbound_url(&url),
            Err(SsrfError::PrivateOrReservedHost { .. })
        ));
    }

    #[test]
    fn validate_outbound_url_allows_public_targets() {
        let url = Url::parse("https://api.example.com/v1").unwrap();
        assert!(validate_outbound_url(&url).is_ok());
    }

    #[test]
    fn validate_outbound_url_rejects_unsupported_scheme() {
        let url = Url::parse("file:///tmp/socket").unwrap();
        assert!(matches!(
            validate_outbound_url(&url),
            Err(SsrfError::UnsupportedScheme { .. })
        ));
    }
}
