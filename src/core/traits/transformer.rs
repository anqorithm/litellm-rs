//! Transformer trait definitions
//!
//! Provides unified interface for request/response format conversion between different providers

use futures::Stream;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};

/// Core transformer trait
///
/// Provides capability to convert from one format to another
pub trait Transform<From, To>: Send + Sync {
    /// Error
    type Error: std::error::Error + Send + Sync + 'static;

    /// Single item transformation
    fn transform(input: From) -> Result<To, Self::Error>;

    /// Batch transformation
    fn transform_batch(inputs: Vec<From>) -> Result<Vec<To>, Self::Error> {
        inputs.into_iter().map(Self::transform).collect()
    }

    /// Stream transformation
    fn transform_stream<S>(stream: S) -> TransformStream<S, Self>
    where
        S: Stream<Item = From> + Send,
        Self: Sized,
    {
        TransformStream::new(stream, PhantomData)
    }
}

/// Bidirectional transformer trait
///
/// Transformer that supports bidirectional conversion
pub trait BidirectionalTransform<A, B>: Transform<A, B> + Transform<B, A> {
    /// A to B conversion
    fn forward(input: A) -> Result<B, <Self as Transform<A, B>>::Error> {
        <Self as Transform<A, B>>::transform(input)
    }

    /// B to A conversion  
    fn backward(input: B) -> Result<A, <Self as Transform<B, A>>::Error> {
        <Self as Transform<B, A>>::transform(input)
    }
}

/// Stream transformation wrapper
pub struct TransformStream<S, T> {
    stream: S,
    _phantom: PhantomData<T>,
}

impl<S, T> TransformStream<S, T> {
    pub fn new(stream: S, _phantom: PhantomData<T>) -> Self {
        Self { stream, _phantom }
    }
}

impl<S, T> Unpin for TransformStream<S, T> where S: Unpin {}

impl<S, T> Stream for TransformStream<S, T>
where
    S: Stream + Unpin,
    T: Send + Sync,
{
    type Item = S::Item;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        Pin::new(&mut this.stream).poll_next(cx)
    }
}

/// Transformer registry
pub struct TransformerRegistry {
    // Here we can store mappings of different type transformers
    // Simplified implementation for now
}

impl TransformerRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub fn register_transformer<T>(&mut self, _transformer: T)
    where
        T: Transform<(), ()> + 'static,
    {
        // Placeholder: transformer registration not yet implemented
        // Transformers are currently managed through static configuration
    }
}

impl Default for TransformerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Error
#[derive(Debug, thiserror::Error)]
pub enum TransformError {
    #[error("Conversion failed: {0}")]
    ConversionFailed(String),

    #[error("Unsupported conversion from {from} to {to}")]
    UnsupportedConversion { from: String, to: String },

    #[error("Missing required field: {field}")]
    MissingField { field: String },

    #[error("Invalid value for field {field}: {value}")]
    InvalidValue { field: String, value: String },

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Other transform error: {0}")]
    Other(String),
}

impl TransformError {
    pub fn conversion_failed(msg: impl Into<String>) -> Self {
        Self::ConversionFailed(msg.into())
    }

    pub fn unsupported_conversion(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self::UnsupportedConversion {
            from: from.into(),
            to: to.into(),
        }
    }

    pub fn missing_field(field: impl Into<String>) -> Self {
        Self::MissingField {
            field: field.into(),
        }
    }

    pub fn invalid_value(field: impl Into<String>, value: impl Into<String>) -> Self {
        Self::InvalidValue {
            field: field.into(),
            value: value.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ==================== TransformStream Tests ====================

    #[test]
    fn test_transform_stream_new() {
        use futures::stream;
        let inner_stream = stream::iter(vec![1, 2, 3]);
        let transform_stream: TransformStream<_, ()> =
            TransformStream::new(inner_stream, PhantomData);
        assert!(format!("{:?}", transform_stream._phantom).contains("PhantomData"));
    }

    // ==================== TransformerRegistry Tests ====================

    #[test]
    fn test_transformer_registry_new() {
        let registry = TransformerRegistry::new();
        // Just verify it creates successfully
        let _ = registry;
    }

    #[test]
    fn test_transformer_registry_default() {
        let registry = TransformerRegistry::default();
        let _ = registry;
    }

    // ==================== TransformError Tests ====================

    #[test]
    fn test_transform_error_conversion_failed() {
        let err = TransformError::conversion_failed("Cannot convert JSON to struct");
        assert!(matches!(err, TransformError::ConversionFailed(_)));
        assert!(err.to_string().contains("Conversion failed"));
        assert!(err.to_string().contains("Cannot convert JSON to struct"));
    }

    #[test]
    fn test_transform_error_unsupported_conversion() {
        let err = TransformError::unsupported_conversion("OpenAI", "Custom");
        assert!(matches!(err, TransformError::UnsupportedConversion { .. }));
        assert!(err.to_string().contains("Unsupported conversion"));
        assert!(err.to_string().contains("OpenAI"));
        assert!(err.to_string().contains("Custom"));
    }

    #[test]
    fn test_transform_error_missing_field() {
        let err = TransformError::missing_field("model");
        assert!(matches!(err, TransformError::MissingField { .. }));
        assert!(err.to_string().contains("Missing required field"));
        assert!(err.to_string().contains("model"));
    }

    #[test]
    fn test_transform_error_invalid_value() {
        let err = TransformError::invalid_value("temperature", "2.5");
        assert!(matches!(err, TransformError::InvalidValue { .. }));
        assert!(err.to_string().contains("Invalid value"));
        assert!(err.to_string().contains("temperature"));
        assert!(err.to_string().contains("2.5"));
    }

    #[test]
    fn test_transform_error_serialization() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let err = TransformError::Serialization(json_err);
        assert!(matches!(err, TransformError::Serialization(_)));
    }

    #[test]
    fn test_transform_error_other() {
        let err = TransformError::Other("Unknown transformation error".to_string());
        assert!(matches!(err, TransformError::Other(_)));
        assert!(err.to_string().contains("Unknown transformation error"));
    }

    #[test]
    fn test_transform_error_display() {
        let err = TransformError::missing_field("content");
        let display = format!("{}", err);
        assert!(!display.is_empty());
        assert!(display.contains("content"));
    }

    #[test]
    fn test_transform_error_debug() {
        let err = TransformError::ConversionFailed("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ConversionFailed"));
    }

    // ==================== Transform Trait Tests ====================

    // Test implementation of Transform trait
    struct StringToInt;

    impl Transform<String, i32> for StringToInt {
        type Error = TransformError;

        fn transform(input: String) -> Result<i32, Self::Error> {
            input.parse::<i32>().map_err(|_| {
                TransformError::conversion_failed(format!("Cannot parse '{}' as integer", input))
            })
        }
    }

    #[test]
    fn test_transform_single_item() {
        let result = StringToInt::transform("42".to_string());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_transform_single_item_error() {
        let result = StringToInt::transform("not_a_number".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_batch_success() {
        let inputs = vec!["1".to_string(), "2".to_string(), "3".to_string()];
        let result = StringToInt::transform_batch(inputs);
        assert!(result.is_ok());
        let values = result.unwrap();
        assert_eq!(values, vec![1, 2, 3]);
    }

    #[test]
    fn test_transform_batch_partial_error() {
        let inputs = vec!["1".to_string(), "invalid".to_string(), "3".to_string()];
        let result = StringToInt::transform_batch(inputs);
        assert!(result.is_err());
    }

    #[test]
    fn test_transform_batch_empty() {
        let inputs: Vec<String> = vec![];
        let result = StringToInt::transform_batch(inputs);
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
