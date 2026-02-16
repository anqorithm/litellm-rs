//! Type utilities for better API design
//!
//! This module provides utilities to create more ergonomic and type-safe APIs
//! following Rust best practices.

#![allow(dead_code)] // Tool module - functions may be used in the future

use serde::{Deserialize, Serialize};
use std::fmt;
use std::marker::PhantomData;

/// A type-safe wrapper for string-based identifiers
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TypedId<T> {
    id: String,
    _phantom: PhantomData<T>,
}

impl<T> TypedId<T> {
    /// Create a new typed ID
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            _phantom: PhantomData,
        }
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.id
    }

    /// Convert to the inner string
    pub fn into_string(self) -> String {
        self.id
    }
}

impl<T> fmt::Display for TypedId<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.id)
    }
}

impl<T> From<String> for TypedId<T> {
    fn from(id: String) -> Self {
        Self::new(id)
    }
}

impl<T> From<&str> for TypedId<T> {
    fn from(id: &str) -> Self {
        Self::new(id)
    }
}

impl<T> AsRef<str> for TypedId<T> {
    fn as_ref(&self) -> &str {
        &self.id
    }
}

/// Marker types for different ID types
///
/// Marker type for user identifiers
pub struct User;
/// Marker type for provider identifiers
pub struct Provider;
/// Marker type for model identifiers
pub struct Model;
/// Marker type for request identifiers
pub struct Request;
/// Marker type for session identifiers
pub struct Session;

/// Type aliases for common ID types
///
/// Type-safe user identifier
pub type UserId = TypedId<User>;
/// Type-safe provider identifier
pub type ProviderId = TypedId<Provider>;
/// Type-safe model identifier
pub type ModelId = TypedId<Model>;
/// Type-safe request identifier
pub type RequestId = TypedId<Request>;
/// Type-safe session identifier
pub type SessionId = TypedId<Session>;

/// A builder pattern trait for creating complex types
pub trait Builder<T> {
    /// Build the final type
    fn build(self) -> T;
}

/// A trait for types that can provide default configurations
pub trait DefaultConfig {
    /// Get the default configuration
    fn default_config() -> Self;
}

/// A wrapper type for optional values with better ergonomics
/// Note: This is mainly for demonstration - in most cases, use `Option<T>` directly
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Optional<T> {
    value: Option<T>,
}

impl<T> Optional<T> {
    /// Create a new optional value
    pub fn new(value: Option<T>) -> Self {
        Self { value }
    }

    /// Create an optional with a value
    pub fn some(value: T) -> Self {
        Self { value: Some(value) }
    }

    /// Create an empty optional
    pub fn none() -> Self {
        Self { value: None }
    }

    /// Check if the optional has a value
    pub fn is_some(&self) -> bool {
        self.value.is_some()
    }

    /// Check if the optional is empty
    pub fn is_none(&self) -> bool {
        self.value.is_none()
    }

    /// Get the inner value
    pub fn into_inner(self) -> Option<T> {
        self.value
    }

    /// Get a reference to the inner value
    pub fn as_ref(&self) -> Option<&T> {
        self.value.as_ref()
    }

    /// Map the inner value
    pub fn map<U, F>(self, f: F) -> Optional<U>
    where
        F: FnOnce(T) -> U,
    {
        Optional::new(self.value.map(f))
    }

    /// Apply a function if the value exists
    pub fn and_then<U, F>(self, f: F) -> Optional<U>
    where
        F: FnOnce(T) -> Optional<U>,
    {
        match self.value {
            Some(value) => f(value),
            None => Optional::none(),
        }
    }

    /// Get the value or a default
    pub fn unwrap_or(self, default: T) -> T {
        self.value.unwrap_or(default)
    }

    /// Get the value or compute a default
    pub fn unwrap_or_else<F>(self, f: F) -> T
    where
        F: FnOnce() -> T,
    {
        self.value.unwrap_or_else(f)
    }
}

impl<T> Default for Optional<T> {
    fn default() -> Self {
        Self::none()
    }
}

impl<T> From<Option<T>> for Optional<T> {
    fn from(value: Option<T>) -> Self {
        Self::new(value)
    }
}

impl<T> From<T> for Optional<T> {
    fn from(value: T) -> Self {
        Self::some(value)
    }
}

impl<T> From<Optional<T>> for Option<T> {
    fn from(val: Optional<T>) -> Self {
        val.value
    }
}

/// A non-empty string type
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NonEmptyString {
    value: String,
}

impl NonEmptyString {
    /// Create a new non-empty string
    pub fn new(value: String) -> Result<Self, &'static str> {
        if value.trim().is_empty() {
            Err("String cannot be empty")
        } else {
            Ok(Self { value })
        }
    }

    /// Get the inner string value
    pub fn as_str(&self) -> &str {
        &self.value
    }

    /// Convert to the inner string
    pub fn into_string(self) -> String {
        self.value
    }
}

impl fmt::Display for NonEmptyString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl TryFrom<String> for NonEmptyString {
    type Error = &'static str;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl TryFrom<&str> for NonEmptyString {
    type Error = &'static str;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value.to_string())
    }
}

impl From<NonEmptyString> for String {
    fn from(val: NonEmptyString) -> Self {
        val.value
    }
}

impl AsRef<str> for NonEmptyString {
    fn as_ref(&self) -> &str {
        &self.value
    }
}

/// A positive number type
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct PositiveF64 {
    value: f64,
}

impl PositiveF64 {
    /// Create a new positive number
    pub fn new(value: f64) -> Result<Self, &'static str> {
        if value > 0.0 && value.is_finite() {
            Ok(Self { value })
        } else {
            Err("Value must be positive and finite")
        }
    }

    /// Get the inner value
    pub fn get(self) -> f64 {
        self.value
    }
}

impl TryFrom<f64> for PositiveF64 {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<PositiveF64> for f64 {
    fn from(val: PositiveF64) -> Self {
        val.value
    }
}

// Note: Macros removed for simplicity - create typed IDs and builders manually as needed

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_typed_id() {
        let user_id: UserId = "user123".into();
        let provider_id: ProviderId = "openai".into();

        assert_eq!(user_id.as_str(), "user123");
        assert_eq!(provider_id.as_str(), "openai");

        // Type safety - these would not compile:
        // let _: UserId = provider_id; // Error: mismatched types
    }

    #[test]
    fn test_optional() {
        let opt1: Optional<i32> = Optional::some(42);
        let opt2: Optional<i32> = Optional::none();

        assert!(opt1.is_some());
        assert!(opt2.is_none());

        let mapped = opt1.map(|x| x * 2);
        assert_eq!(mapped.unwrap_or(0), 84);
    }

    #[test]
    fn test_non_empty_string() {
        let valid = NonEmptyString::new("hello".to_string());
        let invalid = NonEmptyString::new("".to_string());

        assert!(valid.is_ok());
        assert!(invalid.is_err());

        let valid_str = valid.unwrap();
        assert_eq!(valid_str.as_str(), "hello");
    }

    #[test]
    fn test_positive_f64() {
        let valid = PositiveF64::new(42.5);
        let invalid = PositiveF64::new(-1.0);
        let invalid_nan = PositiveF64::new(f64::NAN);

        assert!(valid.is_ok());
        assert!(invalid.is_err());
        assert!(invalid_nan.is_err());

        let positive = valid.unwrap();
        assert_eq!(positive.get(), 42.5);
    }
}
