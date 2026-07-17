//! SSRF (Server-Side Request Forgery) protection utilities
//!
//! This module provides validation functions to protect against SSRF attacks
//! by checking URLs for private/internal IP addresses and blocked hosts.

use crate::core::net::{ProviderEndpointAccess, validate_provider_endpoint_url};
use url::Url;

/// Validate a URL against SSRF attacks
///
/// This function checks that:
/// - The URL is well-formed
/// - The host is not a private/internal IP address
/// - The host is not localhost or a loopback address
/// - The host is not a cloud metadata endpoint
pub fn validate_url_against_ssrf(url_str: &str, context: &str) -> Result<(), String> {
    let url =
        Url::parse(url_str).map_err(|e| format!("{} has invalid URL format: {}", context, e))?;

    // Ensure scheme is http or https
    match url.scheme() {
        "http" | "https" => {}
        scheme => {
            return Err(format!(
                "{} must use http:// or https:// scheme, got: {}",
                context, scheme
            ));
        }
    }

    validate_provider_endpoint_url(&url, ProviderEndpointAccess::PublicOnly)
        .map_err(|error| format!("{context} failed SSRF validation: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== Valid Public URLs ====================

    #[test]
    fn test_valid_public_https_url() {
        let result = validate_url_against_ssrf("https://8.8.8.8/api", "API endpoint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_public_http_url() {
        let result = validate_url_against_ssrf("http://1.1.1.1/v1", "OpenAI API");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_port() {
        // Use a literal public IP to avoid DNS dependency in tests
        let result = validate_url_against_ssrf("https://8.8.8.8:8443/v1", "API endpoint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_path() {
        let result =
            validate_url_against_ssrf("https://8.8.8.8/api/v1/chat/completions", "Chat endpoint");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_url_with_query() {
        let result = validate_url_against_ssrf("https://8.8.8.8/api?key=value", "API with query");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_subdomain() {
        // Use a literal public IP — subdomain DNS is not reliably available in all test envs
        let result = validate_url_against_ssrf("https://1.1.1.1", "Subdomain API");
        assert!(result.is_ok());
    }

    // ==================== Invalid Scheme Tests ====================

    #[test]
    fn test_invalid_ftp_scheme() {
        let result = validate_url_against_ssrf("ftp://example.com/file", "FTP endpoint");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("http:// or https://"));
    }

    #[test]
    fn test_invalid_file_scheme() {
        let result = validate_url_against_ssrf("file:///etc/passwd", "File path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("http:// or https://"));
    }

    #[test]
    fn test_invalid_javascript_scheme() {
        let result = validate_url_against_ssrf("javascript:alert(1)", "JS");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_data_scheme() {
        let result = validate_url_against_ssrf("data:text/html,<h1>Hi</h1>", "Data URI");
        assert!(result.is_err());
    }

    // ==================== Localhost/Loopback Tests ====================

    #[test]
    fn test_blocked_localhost() {
        let result = validate_url_against_ssrf("http://localhost/api", "Local API");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_127_0_0_1() {
        let result = validate_url_against_ssrf("http://127.0.0.1/api", "Loopback API");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_127_0_0_1_with_port() {
        let result = validate_url_against_ssrf("http://127.0.0.1:8080/api", "Loopback with port");
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_ipv6_loopback() {
        let result = validate_url_against_ssrf("http://[::1]/api", "IPv6 loopback");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_0_0_0_0() {
        let result = validate_url_against_ssrf("http://0.0.0.0/api", "Unspecified");
        assert!(result.is_err());
    }

    // ==================== Private IP Address Tests ====================

    #[test]
    fn test_blocked_private_10_network() {
        let result = validate_url_against_ssrf("http://10.0.0.1/api", "Private 10.x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_private_172_16_network() {
        let result = validate_url_against_ssrf("http://172.16.0.1/api", "Private 172.16.x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_private_192_168_network() {
        let result = validate_url_against_ssrf("http://192.168.1.1/api", "Private 192.168.x");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_private_172_31_network() {
        let result = validate_url_against_ssrf("http://172.31.255.255/api", "Private 172.31.x");
        assert!(result.is_err());
    }

    // ==================== Cloud Metadata Endpoint Tests ====================

    #[test]
    fn test_blocked_aws_metadata_endpoint() {
        let result =
            validate_url_against_ssrf("http://169.254.169.254/latest/meta-data/", "AWS metadata");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_link_local_ip() {
        let result = validate_url_against_ssrf("http://169.254.1.1/api", "Link local");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_gcp_metadata_hostname() {
        let result =
            validate_url_against_ssrf("http://metadata.google.internal/v1/", "GCP metadata");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_metadata_hostname() {
        let result = validate_url_against_ssrf("http://metadata/v1/", "Metadata shortname");
        assert!(result.is_err());
    }

    // ==================== Decimal/Octal/Hex Encoded IP Tests ====================

    #[test]
    fn test_blocked_decimal_encoded_loopback() {
        // 2130706433 = 127.0.0.1
        // Note: URL parser resolves this to 127.0.0.1 before our check runs
        let result = validate_url_against_ssrf("http://2130706433/api", "Decimal encoded");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_hex_encoded_loopback() {
        // 0x7f000001 = 127.0.0.1
        // Note: URL parser resolves this to 127.0.0.1 before our check runs
        let result = validate_url_against_ssrf("http://0x7f000001/api", "Hex encoded");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("SSRF protection"));
    }

    #[test]
    fn test_blocked_hex_encoded_private() {
        // 0x0a000001 = 10.0.0.1
        let result = validate_url_against_ssrf("http://0x0a000001/api", "Hex private");
        assert!(result.is_err());
    }

    // ==================== IPv6 Private Address Tests ====================

    #[test]
    fn test_blocked_ipv6_unique_local() {
        // fc00::/7 - unique local addresses
        let result = validate_url_against_ssrf("http://[fc00::1]/api", "IPv6 unique local");
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_ipv6_link_local() {
        // fe80::/10 - link local addresses
        let result = validate_url_against_ssrf("http://[fe80::1]/api", "IPv6 link local");
        assert!(result.is_err());
    }

    // ==================== Reserved IP Range Tests ====================

    #[test]
    fn test_blocked_reserved_240_range() {
        let result = validate_url_against_ssrf("http://240.0.0.1/api", "Reserved 240.x");
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_reserved_255_range() {
        let result = validate_url_against_ssrf("http://255.255.255.255/api", "Broadcast");
        assert!(result.is_err());
    }

    #[test]
    fn test_blocked_shared_address_space() {
        // 100.64.0.0/10 - carrier-grade NAT (RFC 6598)
        let result = validate_url_against_ssrf("http://100.64.0.1/api", "CGN address");
        assert!(result.is_err());
    }

    #[test]
    fn test_canonical_special_purpose_ranges_blocked() {
        let blocked_urls = [
            "http://224.0.0.1/api",
            "http://198.18.0.1/api",
            "http://[ff02::1]/api",
            "http://[2001:db8::1]/api",
        ];

        for url in blocked_urls {
            let error = validate_url_against_ssrf(url, "Canonical policy")
                .expect_err("special-purpose address must be blocked");
            assert!(
                error.contains("Canonical policy"),
                "unexpected error: {error}"
            );
            assert!(
                error.contains("SSRF protection"),
                "unexpected error: {error}"
            );
        }
    }

    // ==================== Malformed URL Tests ====================

    #[test]
    fn test_invalid_url_format() {
        let result = validate_url_against_ssrf("not-a-valid-url", "Invalid URL");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("invalid URL format"));
    }

    #[test]
    fn test_empty_url() {
        let result = validate_url_against_ssrf("", "Empty URL");
        assert!(result.is_err());
    }

    #[test]
    fn test_url_without_host() {
        let result = validate_url_against_ssrf("http:///path", "No host");
        let error = result.expect_err("URL without a valid host must be rejected");
        assert!(error.contains("No host"));
    }

    // ==================== Edge Cases ====================

    #[test]
    fn test_localhost_with_subdomain_blocked() {
        // Subdomains of blocked hosts should also be blocked
        let result =
            validate_url_against_ssrf("http://sub.localhost/api", "Subdomain of localhost");
        assert!(result.is_err());
    }

    #[test]
    fn test_internal_hostname_blocked() {
        let result = validate_url_against_ssrf("http://internal/api", "Internal hostname");
        assert!(result.is_err());
    }

    #[test]
    fn test_local_hostname_blocked() {
        let result = validate_url_against_ssrf("http://local/api", "Local hostname");
        assert!(result.is_err());
    }

    #[test]
    fn test_subdomain_of_internal_blocked() {
        let result = validate_url_against_ssrf("http://api.internal/v1", "Subdomain of internal");
        assert!(result.is_err());
    }

    #[test]
    fn test_valid_external_ip() {
        // 8.8.8.8 is Google's public DNS - should be allowed
        let result = validate_url_against_ssrf("http://8.8.8.8/api", "Public IP");
        assert!(result.is_ok());
    }

    #[test]
    fn test_valid_external_ip_2() {
        // 1.1.1.1 is Cloudflare's public DNS - should be allowed
        let result = validate_url_against_ssrf("http://1.1.1.1/api", "Cloudflare DNS");
        assert!(result.is_ok());
    }

    // ==================== Context Message Tests ====================

    #[test]
    fn test_context_in_error_message() {
        let result = validate_url_against_ssrf("http://localhost/api", "Webhook URL");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Webhook URL"));
    }

    #[test]
    fn test_context_in_scheme_error() {
        let result = validate_url_against_ssrf("ftp://example.com/file", "Callback endpoint");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Callback endpoint"));
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_real_world_api_endpoints() {
        // Preserve common API path shapes without depending on external DNS.
        let valid_endpoints = vec![
            "https://8.8.8.8/v1/chat/completions",
            "https://1.1.1.1/v1/messages",
            "https://8.8.4.4/v1/models",
            "https://1.0.0.1/v1/generate",
        ];

        for endpoint in valid_endpoints {
            let result = validate_url_against_ssrf(endpoint, "API endpoint");
            assert!(result.is_ok(), "Expected {} to be valid", endpoint);
        }
    }

    #[test]
    fn test_ssrf_attack_vectors() {
        // Test common SSRF attack vectors that should be blocked
        let attack_vectors = vec![
            "http://localhost/admin",
            "http://127.0.0.1/admin",
            "http://[::1]/admin",
            "http://169.254.169.254/latest/meta-data/",
            "http://10.0.0.1/internal",
            "http://192.168.1.1/router",
            "http://2130706433/decimal-bypass",
            "http://0x7f000001/hex-bypass",
        ];

        for vector in attack_vectors {
            let result = validate_url_against_ssrf(vector, "Attack vector");
            assert!(result.is_err(), "Expected {} to be blocked", vector);
        }
    }
}
