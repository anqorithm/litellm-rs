//! PII detection patterns
//!
//! Pre-compiled regex patterns for detecting personally identifiable information.

use regex::Regex;
use std::sync::LazyLock;

// Pre-compiled regex patterns for PII detection
// These patterns are validated at compile time to ensure they never fail
// Note: unwrap() is used because these are static patterns that are known-good
// If a pattern fails, it indicates a code error that should be caught in tests

/// Fallback regex that never matches anything.
/// `[^\s\S]` matches "neither whitespace nor non-whitespace" = empty set.
/// This is a trivially valid pattern, so `expect` here is safe.
fn never_matching_regex() -> Regex {
    Regex::new(r"[^\s\S]").expect("static never-matching regex pattern is always valid")
}

/// SSN pattern: XXX-XX-XXXX
pub static SSN_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{3}-\d{2}-\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile SSN regex: {}", e);
        never_matching_regex()
    })
});

/// Email pattern: local@domain.tld
pub static EMAIL_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile email regex: {}", e);
        never_matching_regex()
    })
});

/// Phone pattern: XXX-XXX-XXXX
pub static PHONE_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{3}-\d{3}-\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile phone regex: {}", e);
        never_matching_regex()
    })
});

/// Credit card pattern: XXXX-XXXX-XXXX-XXXX or XXXXXXXXXXXXXXXX
pub static CREDIT_CARD_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{4}[\s-]?\d{4}[\s-]?\d{4}[\s-]?\d{4}\b").unwrap_or_else(|e| {
        tracing::error!("Failed to compile credit card regex: {}", e);
        never_matching_regex()
    })
});

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== SSN Pattern Tests ====================

    #[test]
    fn test_ssn_pattern_valid() {
        assert!(SSN_PATTERN.is_match("123-45-6789"));
        assert!(SSN_PATTERN.is_match("000-00-0000"));
        assert!(SSN_PATTERN.is_match("999-99-9999"));
    }

    #[test]
    fn test_ssn_pattern_in_text() {
        assert!(SSN_PATTERN.is_match("My SSN is 123-45-6789 and more text"));
        assert!(SSN_PATTERN.is_match("SSN: 123-45-6789"));
    }

    #[test]
    fn test_ssn_pattern_invalid() {
        assert!(!SSN_PATTERN.is_match("123456789")); // No dashes
        assert!(!SSN_PATTERN.is_match("12-345-6789")); // Wrong format
        assert!(!SSN_PATTERN.is_match("1234-56-789")); // Wrong format
        assert!(!SSN_PATTERN.is_match("123-45-678")); // Too short last segment
        assert!(!SSN_PATTERN.is_match("123-45-67890")); // Too long last segment
    }

    #[test]
    fn test_ssn_pattern_boundary() {
        // Should not match if not at word boundary
        assert!(!SSN_PATTERN.is_match("a123-45-6789b"));
    }

    // ==================== Email Pattern Tests ====================

    #[test]
    fn test_email_pattern_valid() {
        assert!(EMAIL_PATTERN.is_match("test@example.com"));
        assert!(EMAIL_PATTERN.is_match("user.name@domain.org"));
        assert!(EMAIL_PATTERN.is_match("user+tag@domain.co.uk"));
        assert!(EMAIL_PATTERN.is_match("user123@sub.domain.com"));
    }

    #[test]
    fn test_email_pattern_special_chars() {
        assert!(EMAIL_PATTERN.is_match("user.name+tag@domain.com"));
        assert!(EMAIL_PATTERN.is_match("user%tag@domain.com"));
        assert!(EMAIL_PATTERN.is_match("user_name@domain.com"));
        assert!(EMAIL_PATTERN.is_match("user-name@domain.com"));
    }

    #[test]
    fn test_email_pattern_in_text() {
        assert!(EMAIL_PATTERN.is_match("Contact me at user@example.com for more info"));
        assert!(EMAIL_PATTERN.is_match("Email: admin@test.org"));
    }

    #[test]
    fn test_email_pattern_invalid() {
        assert!(!EMAIL_PATTERN.is_match("not an email"));
        assert!(!EMAIL_PATTERN.is_match("@domain.com"));
        assert!(!EMAIL_PATTERN.is_match("user@"));
        assert!(!EMAIL_PATTERN.is_match("user@domain"));
    }

    // ==================== Phone Pattern Tests ====================

    #[test]
    fn test_phone_pattern_valid() {
        assert!(PHONE_PATTERN.is_match("123-456-7890"));
        assert!(PHONE_PATTERN.is_match("000-000-0000"));
        assert!(PHONE_PATTERN.is_match("999-999-9999"));
    }

    #[test]
    fn test_phone_pattern_in_text() {
        assert!(PHONE_PATTERN.is_match("Call me at 123-456-7890 anytime"));
        assert!(PHONE_PATTERN.is_match("Phone: 123-456-7890"));
    }

    #[test]
    fn test_phone_pattern_invalid() {
        assert!(!PHONE_PATTERN.is_match("12345678901")); // No dashes
        assert!(!PHONE_PATTERN.is_match("1234567890")); // No dashes
        assert!(!PHONE_PATTERN.is_match("12-3456-7890")); // Wrong format
        assert!(!PHONE_PATTERN.is_match("(123) 456-7890")); // Different format
        assert!(!PHONE_PATTERN.is_match("123.456.7890")); // Dots instead of dashes
    }

    #[test]
    fn test_phone_pattern_boundary() {
        // Should not match if not at word boundary
        assert!(!PHONE_PATTERN.is_match("a123-456-7890b"));
    }

    // ==================== Credit Card Pattern Tests ====================

    #[test]
    fn test_credit_card_pattern_with_dashes() {
        assert!(CREDIT_CARD_PATTERN.is_match("1234-5678-9012-3456"));
        assert!(CREDIT_CARD_PATTERN.is_match("0000-0000-0000-0000"));
        assert!(CREDIT_CARD_PATTERN.is_match("9999-9999-9999-9999"));
    }

    #[test]
    fn test_credit_card_pattern_no_dashes() {
        assert!(CREDIT_CARD_PATTERN.is_match("1234567890123456"));
        assert!(CREDIT_CARD_PATTERN.is_match("0000000000000000"));
    }

    #[test]
    fn test_credit_card_pattern_with_spaces() {
        assert!(CREDIT_CARD_PATTERN.is_match("1234 5678 9012 3456"));
    }

    #[test]
    fn test_credit_card_pattern_in_text() {
        assert!(CREDIT_CARD_PATTERN.is_match("Card number: 1234-5678-9012-3456 expires 12/25"));
        assert!(CREDIT_CARD_PATTERN.is_match("Pay with 1234567890123456"));
    }

    #[test]
    fn test_credit_card_pattern_invalid() {
        assert!(!CREDIT_CARD_PATTERN.is_match("123")); // Too short
        assert!(!CREDIT_CARD_PATTERN.is_match("1234-5678")); // Incomplete
        assert!(!CREDIT_CARD_PATTERN.is_match("1234-5678-9012")); // Missing last group
        assert!(!CREDIT_CARD_PATTERN.is_match("12345678901234567")); // Too long
    }

    // ==================== Integration Tests ====================

    #[test]
    fn test_all_patterns_compile() {
        // Verify all static regex patterns compile successfully
        assert!(SSN_PATTERN.is_match("123-45-6789"));
        assert!(EMAIL_PATTERN.is_match("test@example.com"));
        assert!(PHONE_PATTERN.is_match("123-456-7890"));
        assert!(CREDIT_CARD_PATTERN.is_match("1234-5678-9012-3456"));
    }

    #[test]
    fn test_patterns_in_mixed_content() {
        let content = "Contact John at john@example.com or 123-456-7890. SSN: 123-45-6789";
        assert!(EMAIL_PATTERN.is_match(content));
        assert!(PHONE_PATTERN.is_match(content));
        assert!(SSN_PATTERN.is_match(content));
    }

    #[test]
    fn test_no_false_positives_in_normal_text() {
        let content = "This is normal text without any PII data";
        assert!(!SSN_PATTERN.is_match(content));
        assert!(!EMAIL_PATTERN.is_match(content));
        assert!(!PHONE_PATTERN.is_match(content));
        assert!(!CREDIT_CARD_PATTERN.is_match(content));
    }
}
