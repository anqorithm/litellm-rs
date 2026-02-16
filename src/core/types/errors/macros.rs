//! Macros for reducing boilerplate in error type conversions.
//!
//! These macros standardize the `From<reqwest::Error>` and `From<serde_json::Error>`
//! implementations across different error types (ProviderError, A2AError, McpError, etc.).

/// Implement `From<reqwest::Error>` for an error type with timeout/connect/other branching.
///
/// # Usage
/// ```rust,ignore
/// impl_from_reqwest_error!(MyError,
///     timeout  => |e| MyError::Timeout(e.to_string()),
///     connect  => |e| MyError::Network(e.to_string()),
///     other    => |e| MyError::Other(e.to_string())
/// );
/// ```
#[macro_export]
macro_rules! impl_from_reqwest_error {
    ($error_type:ty,
     timeout => |$t:ident| $timeout_expr:expr,
     connect => |$c:ident| $connect_expr:expr,
     other   => |$o:ident| $other_expr:expr
    ) => {
        impl From<reqwest::Error> for $error_type {
            fn from(err: reqwest::Error) -> Self {
                if err.is_timeout() {
                    let $t = &err;
                    $timeout_expr
                } else if err.is_connect() || err.is_request() {
                    let $c = &err;
                    $connect_expr
                } else {
                    let $o = &err;
                    $other_expr
                }
            }
        }
    };
}

/// Implement `From<serde_json::Error>` for an error type.
///
/// # Usage
/// ```rust,ignore
/// impl_from_serde_error!(MyError, |e| MyError::Parsing(e.to_string()));
/// ```
#[macro_export]
macro_rules! impl_from_serde_error {
    ($error_type:ty, |$e:ident| $expr:expr) => {
        impl From<serde_json::Error> for $error_type {
            fn from(err: serde_json::Error) -> Self {
                let $e = &err;
                $expr
            }
        }
    };
}
