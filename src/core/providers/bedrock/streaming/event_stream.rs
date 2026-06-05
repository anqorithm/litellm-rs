//! AWS Event Stream parsing for Bedrock streaming responses.

use super::BedrockStream;
use crate::core::providers::unified_provider::ProviderError;
use bytes::Bytes;
use serde_json::Value;

/// AWS Event Stream message
#[derive(Debug)]
pub struct EventStreamMessage {
    pub headers: Vec<EventStreamHeader>,
    pub payload: Bytes,
}

/// Event stream header
#[derive(Debug)]
pub struct EventStreamHeader {
    pub name: String,
    pub value: HeaderValue,
}

/// Header value types
#[derive(Debug)]
pub enum HeaderValue {
    String(String),
    ByteArray(Vec<u8>),
    Boolean(bool),
    Byte(i8),
    Short(i16),
    Integer(i32),
    Long(i64),
    UUID(String),
    Timestamp(i64),
}

impl BedrockStream {
    /// Parse event stream message from bytes
    pub(super) fn parse_event_message(data: &[u8]) -> Result<EventStreamMessage, ProviderError> {
        if data.len() < 16 {
            return Err(ProviderError::response_parsing(
                "bedrock",
                "Invalid event stream message",
            ));
        }

        // Parse prelude (12 bytes)
        let total_length = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let headers_length = u32::from_be_bytes([data[4], data[5], data[6], data[7]]) as usize;
        // let prelude_crc = u32::from_be_bytes([data[8], data[9], data[10], data[11]]);

        if data.len() < total_length {
            return Err(ProviderError::response_parsing(
                "bedrock",
                "Incomplete event stream message",
            ));
        }

        // Parse headers
        let mut headers = Vec::new();
        let mut offset = 12;
        let headers_end = 12 + headers_length;

        while offset < headers_end {
            if offset + 1 > data.len() {
                break;
            }

            let name_length = data[offset] as usize;
            offset += 1;

            if offset + name_length > data.len() {
                break;
            }

            let name = String::from_utf8_lossy(&data[offset..offset + name_length]).to_string();
            offset += name_length;

            if offset >= data.len() {
                break;
            }

            let header_type = data[offset];
            offset += 1;

            let value = match header_type {
                5 | 7 => {
                    // String type
                    if offset + 2 > data.len() {
                        break;
                    }
                    let string_length =
                        u16::from_be_bytes([data[offset], data[offset + 1]]) as usize;
                    offset += 2;
                    if offset + string_length > data.len() {
                        break;
                    }
                    let string_value =
                        String::from_utf8_lossy(&data[offset..offset + string_length]).to_string();
                    offset += string_length;
                    HeaderValue::String(string_value)
                }
                _ => {
                    // Skip unknown header types
                    HeaderValue::String(String::new())
                }
            };

            headers.push(EventStreamHeader { name, value });
        }

        // Extract payload
        let payload_start = headers_end;
        let payload_end = total_length - 4; // Exclude message CRC
        let payload = if payload_start < payload_end && payload_end <= data.len() {
            Bytes::copy_from_slice(&data[payload_start..payload_end])
        } else {
            Bytes::new()
        };

        Ok(EventStreamMessage { headers, payload })
    }

    fn header_value<'a>(message: &'a EventStreamMessage, name: &str) -> Option<&'a str> {
        message.headers.iter().find_map(|header| {
            (header.name == name)
                .then_some(&header.value)
                .and_then(|value| match value {
                    HeaderValue::String(value) => Some(value.as_str()),
                    _ => None,
                })
        })
    }

    fn stream_exception_from_payload(value: &Value) -> Option<(String, String)> {
        let object = value.as_object()?;

        for (code, detail) in object {
            if code.ends_with("Exception") || code.ends_with("exception") {
                let message = detail
                    .get("message")
                    .and_then(Value::as_str)
                    .or_else(|| detail.as_str())
                    .unwrap_or("");
                return Some((code.clone(), message.to_string()));
            }
        }

        None
    }

    fn stream_error(code: &str, message: &str) -> ProviderError {
        let details = if message.is_empty() {
            format!("Bedrock stream error: {code}")
        } else {
            format!("Bedrock stream error {code}: {message}")
        };

        if code.eq_ignore_ascii_case("validationException") {
            ProviderError::invalid_request("bedrock", details)
        } else {
            ProviderError::api_error("bedrock", 500, details)
        }
    }

    pub(super) fn check_stream_error(message: &EventStreamMessage) -> Result<(), ProviderError> {
        let message_type = Self::header_value(message, ":message-type");
        let exception_type = Self::header_value(message, ":exception-type");

        if matches!(message_type, Some("exception" | "error")) || exception_type.is_some() {
            let payload = serde_json::from_slice::<Value>(&message.payload).ok();
            let payload_message = payload
                .as_ref()
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or("");
            let code = exception_type.unwrap_or("streamException");
            return Err(Self::stream_error(code, payload_message));
        }

        if let Ok(payload) = serde_json::from_slice::<Value>(&message.payload)
            && let Some((code, message)) = Self::stream_exception_from_payload(&payload)
        {
            return Err(Self::stream_error(&code, &message));
        }

        Ok(())
    }
}
