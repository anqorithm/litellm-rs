//! Network validation and safety utilities.

pub mod ssrf_guard;

pub use ssrf_guard::{
    SsrfError, extract_url_host, is_private_or_reserved_host, is_private_or_reserved_ip,
    validate_outbound_url, validate_outbound_url_str,
};
