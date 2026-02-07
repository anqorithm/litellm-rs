use std::sync::LazyLock;
use regex::Regex;

static SENSITIVE_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"(?i)api[_-]?key[=:\s]*['\x22]?([a-zA-Z0-9\-_]+)['\x22]?").unwrap(),
        Regex::new(r"(?i)token[=:\s]*['\x22]?([a-zA-Z0-9\-_.]+)['\x22]?").unwrap(),
        Regex::new(r"(?i)password[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?").unwrap(),
        Regex::new(r"(?i)secret[=:\s]*['\x22]?([^\s'\x22]+)['\x22]?").unwrap(),
    ]
});

pub struct Sanitization;

impl Sanitization {
    pub fn sanitize_log_data(data: &str) -> String {
        let mut sanitized = data.to_string();

        for re in SENSITIVE_PATTERNS.iter() {
            sanitized = re.replace_all(&sanitized, "***REDACTED***").to_string();
        }

        sanitized
    }

    pub fn mask_sensitive_data(input: &str) -> String {
        static MASK_PATTERNS: LazyLock<Vec<Regex>> = LazyLock::new(|| {
            let sensitive_keys = [
                "api_key",
                "token",
                "password",
                "secret",
                "auth",
                "credential",
            ];

            let mut patterns = Vec::new();
            for key in &sensitive_keys {
                if let Ok(re) = Regex::new(&format!(r#""{}"\s*:\s*"([^"]+)""#, key)) {
                    patterns.push(re);
                }
                if let Ok(re) = Regex::new(&format!(r#"'{}'\s*:\s*'([^']+)'"#, key)) {
                    patterns.push(re);
                }
                if let Ok(re) = Regex::new(&format!(r#"{}[=:]\s*([^\s,}}\]]+)"#, key)) {
                    patterns.push(re);
                }
            }
            patterns
        });

        let mut result = input.to_string();

        for re in MASK_PATTERNS.iter() {
            result = re
                .replace_all(&result, |caps: &regex::Captures| {
                    let full_match = caps.get(0).unwrap().as_str();
                    let value = caps.get(1).unwrap().as_str();
                    let masked_value = if value.len() > 8 {
                        format!("{}***{}", &value[..2], &value[value.len() - 2..])
                    } else {
                        "***".to_string()
                    };
                    full_match.replace(value, &masked_value)
                })
                .to_string();
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== sanitize_log_data Tests ====================

    #[test]
    fn test_sanitize_api_key() {
        let input = "api_key=sk-abc123xyz";
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
        assert!(!result.contains("sk-abc123xyz"));
    }

    #[test]
    fn test_sanitize_api_key_with_quotes() {
        let input = r#"api_key="sk-secret123""#;
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
    }

    #[test]
    fn test_sanitize_token() {
        let input = "token: bearer_token_value";
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
        assert!(!result.contains("bearer_token_value"));
    }

    #[test]
    fn test_sanitize_password() {
        let input = "password=my_secret_pass";
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
        assert!(!result.contains("my_secret_pass"));
    }

    #[test]
    fn test_sanitize_secret() {
        let input = "secret: super_secret_value";
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
    }

    #[test]
    fn test_sanitize_case_insensitive() {
        let input = "API_KEY=test123";
        let result = Sanitization::sanitize_log_data(input);
        assert!(result.contains("REDACTED"));
    }

    #[test]
    fn test_sanitize_multiple_patterns() {
        let input = "api_key=key1 token=tok1 password=pass1";
        let result = Sanitization::sanitize_log_data(input);
        assert!(!result.contains("key1"));
        assert!(!result.contains("tok1"));
        assert!(!result.contains("pass1"));
    }

    #[test]
    fn test_sanitize_no_sensitive_data() {
        let input = "normal log message with regular content";
        let result = Sanitization::sanitize_log_data(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_sanitize_empty_string() {
        let result = Sanitization::sanitize_log_data("");
        assert_eq!(result, "");
    }

    // ==================== mask_sensitive_data Tests ====================

    #[test]
    fn test_mask_json_api_key() {
        let input = r#"{"api_key": "sk-1234567890abcdef"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("sk***ef"));
        assert!(!result.contains("1234567890abcdef"));
    }

    #[test]
    fn test_mask_json_token() {
        let input = r#"{"token": "tok_abcdefghij"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("to***ij"));
    }

    #[test]
    fn test_mask_json_password() {
        let input = r#"{"password": "mysupersecretpassword"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("my***rd"));
    }

    #[test]
    fn test_mask_json_secret() {
        let input = r#"{"secret": "verysecretvalue123"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("ve***23"));
    }

    #[test]
    fn test_mask_json_auth() {
        let input = r#"{"auth": "bearer_token_here"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("be***re"));
    }

    #[test]
    fn test_mask_json_credential() {
        let input = r#"{"credential": "cred1234567890"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("cr***90"));
    }

    #[test]
    fn test_mask_short_value() {
        let input = r#"{"api_key": "short"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("***"));
    }

    #[test]
    fn test_mask_single_quotes() {
        let input = r#"{'token': 'mytoken12345'}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("my***45"));
    }

    #[test]
    fn test_mask_key_value_format() {
        let input = "api_key=sk_test_abcdefghij";
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("sk***ij"));
    }

    #[test]
    fn test_mask_no_sensitive_data() {
        let input = r#"{"name": "John", "email": "john@example.com"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert_eq!(result, input);
    }

    #[test]
    fn test_mask_empty_string() {
        let result = Sanitization::mask_sensitive_data("");
        assert_eq!(result, "");
    }

    #[test]
    fn test_mask_nested_json() {
        let input = r#"{"config": {"api_key": "sk-nestedvalue123"}}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains("sk***23"));
    }

    #[test]
    fn test_mask_preserves_structure() {
        let input = r#"{"api_key": "sk-1234567890", "other": "value"}"#;
        let result = Sanitization::mask_sensitive_data(input);
        assert!(result.contains(r#""other": "value""#));
    }
}
