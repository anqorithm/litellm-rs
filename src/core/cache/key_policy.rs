//! Stable cache key policy helpers.

use serde::Serialize;
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Increment this when request dimensions included in cache keys change.
///
/// Bumping this value intentionally cold-starts older Redis/cache entries so
/// responses produced under an older request identity policy are not reused.
pub const CACHE_KEY_SCHEMA_VERSION: &str = "v3";

/// Generate a stable SHA-256 digest for any serializable value.
pub fn stable_digest<T: Serialize>(value: &T) -> String {
    let value = serde_json::to_value(value).unwrap_or(Value::Null);
    stable_digest_value(&value)
}

/// Generate a stable SHA-256 digest for an already materialized JSON value.
pub fn stable_digest_value(value: &Value) -> String {
    let canonical = canonical_json_string(value);
    let digest = Sha256::digest(canonical.as_bytes());
    hex::encode(digest)
}

/// Convert a JSON value to canonical JSON by sorting object keys and removing
/// fields that cannot affect model output identity.
pub fn canonical_json_string(value: &Value) -> String {
    let canonical = canonicalize_json_value(value);
    serde_json::to_string(&canonical).unwrap_or_else(|_| "null".to_string())
}

/// Parse and canonicalize a JSON string.
pub fn canonical_json_str(raw_json: &str) -> String {
    match serde_json::from_str::<Value>(raw_json) {
        Ok(value) => canonical_json_string(&value),
        Err(_) => raw_json.to_string(),
    }
}

/// Returns true when a serialized field should be excluded from cache identity.
pub fn is_non_deterministic_field(field: &str) -> bool {
    matches!(
        field,
        "timestamp"
            | "request_id"
            | "trace_id"
            | "span_id"
            | "created_at"
            | "updated_at"
            | "id"
            | "stream"
            | "stream_options"
    )
}

/// Format a stable cache key using the shared schema version namespace.
pub fn versioned_key(prefix: &str, namespace: Option<&str>, digest: &str) -> String {
    match namespace {
        Some(namespace) => format!("{prefix}:{namespace}:{CACHE_KEY_SCHEMA_VERSION}:{digest}"),
        None => format!("{prefix}:{CACHE_KEY_SCHEMA_VERSION}:{digest}"),
    }
}

fn canonicalize_json_value(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            for (key, value) in map {
                if is_non_deterministic_field(key) {
                    continue;
                }
                sorted.insert(key.clone(), canonicalize_json_value(value));
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalize_json_value).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_json_sorts_object_keys() {
        assert_eq!(
            canonical_json_str(r#"{"b":2,"a":1}"#),
            canonical_json_str(r#"{"a":1,"b":2}"#)
        );
    }

    #[test]
    fn canonical_json_filters_transport_fields() {
        let without_transport = canonical_json_str(r#"{"message":"hello"}"#);
        let with_transport =
            canonical_json_str(r#"{"message":"hello","stream":true,"request_id":"abc"}"#);

        assert_eq!(without_transport, with_transport);
    }

    #[test]
    fn stable_digest_is_deterministic_and_sha256_sized() {
        let digest = stable_digest_value(&json!({"b": 2, "a": 1}));

        assert_eq!(digest, stable_digest_value(&json!({"a": 1, "b": 2})));
        assert_eq!(digest.len(), 64);
    }

    #[test]
    fn versioned_key_includes_schema_version() {
        assert_eq!(
            versioned_key("chat", Some("gpt-4"), "abc"),
            "chat:gpt-4:v3:abc"
        );
        assert_eq!(versioned_key("raw", None, "abc"), "raw:v3:abc");
    }
}
