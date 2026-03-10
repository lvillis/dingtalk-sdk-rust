use std::time::{Duration, SystemTime, SystemTimeError};

use std::fmt;
use thiserror::Error;

/// SDK result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Structured HTTP error context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpError {
    /// HTTP status code.
    pub status: u16,
    /// Optional short message from upstream/client.
    pub message: Option<String>,
    /// Optional request identifier from upstream.
    pub request_id: Option<String>,
    /// Optional redacted response body snippet.
    pub body_snippet: Option<String>,
}

impl fmt::Display for HttpError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTTP {}", self.status)?;
        if let Some(message) = &self.message {
            write!(f, ": {message}")?;
        }
        if let Some(request_id) = &self.request_id {
            write!(f, " [request-id: {request_id}]")?;
        }
        Ok(())
    }
}

/// Structured transport error context.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransportError {
    /// Optional HTTP status code if available.
    pub status: Option<u16>,
    /// Optional short message from client/runtime.
    pub message: Option<String>,
    /// Optional request identifier from upstream.
    pub request_id: Option<String>,
    /// Optional redacted response body snippet.
    pub body_snippet: Option<String>,
    /// Optional retry-after hint from upstream.
    pub retry_after: Option<Duration>,
    /// Whether the error is likely retryable.
    pub retryable: bool,
    /// Stable transport error code derived from reqx.
    pub code: &'static str,
    /// Optional HTTP method for the failed request.
    pub method: Option<String>,
    /// Optional redacted request URI/path for the failed request.
    pub uri: Option<String>,
    /// Optional timeout phase when the transport failed due to timeout.
    pub timeout_phase: Option<&'static str>,
    /// Optional lower-level transport kind when available.
    pub transport_kind: Option<&'static str>,
}

impl fmt::Display for TransportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(status) = self.status {
            write!(f, "HTTP {status}")?;
        } else {
            write!(f, "request failed")?;
        }
        if let Some(message) = &self.message {
            write!(f, ": {message}")?;
        }
        if let Some(request_id) = &self.request_id {
            write!(f, " [request-id: {request_id}]")?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
/// Stable high-level error category.
pub enum ErrorKind {
    /// Authentication and authorization errors.
    Auth,
    /// Resource not found.
    NotFound,
    /// Resource conflict or precondition conflict.
    Conflict,
    /// Request rate-limited.
    RateLimited,
    /// DingTalk API business error (`errcode != 0`).
    Api,
    /// Transport-level client/network error.
    Transport,
    /// Serialization or deserialization error.
    Serialization,
    /// Timestamp generation failure.
    Timestamp,
    /// Signature generation failure.
    Signature,
    /// Invalid SDK configuration.
    InvalidConfig,
}

#[derive(Debug, Error)]
#[non_exhaustive]
/// Unified SDK error type.
pub enum Error {
    /// API business error returned by DingTalk.
    #[error("API error (code={code}): {message}")]
    Api {
        /// DingTalk error code.
        code: i64,
        /// DingTalk error message.
        message: String,
        /// Optional request identifier.
        request_id: Option<String>,
        /// Optional redacted response body snippet.
        body_snippet: Option<String>,
    },

    /// Authentication/authorization HTTP error.
    #[error("Authentication failed: {0}")]
    Auth(HttpError),

    /// Not-found HTTP error.
    #[error("Resource not found: {0}")]
    NotFound(HttpError),

    /// Conflict HTTP error.
    #[error("Resource conflict: {0}")]
    Conflict(HttpError),

    /// Rate limit error with optional retry hint.
    #[error("Rate limited: {error}")]
    RateLimited {
        /// Underlying HTTP error.
        error: HttpError,
        /// Retry-after hint parsed from response headers if available.
        retry_after: Option<Duration>,
    },

    /// Transport error from HTTP runtime/client.
    #[error("HTTP transport error: {0}")]
    Transport(Box<TransportError>),

    /// JSON serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// System timestamp retrieval error.
    #[error("Timestamp generation failed: {0}")]
    Timestamp(#[from] SystemTimeError),

    /// Signature generation error.
    #[error("Signature generation failed")]
    Signature,

    /// Invalid runtime configuration.
    #[error("Invalid configuration: {message}")]
    InvalidConfig {
        /// Human-readable reason.
        message: String,
        /// Optional source error.
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

fn reqx_timeout_phase_name(error: &reqx::Error) -> Option<&'static str> {
    match error {
        reqx::Error::Timeout { phase, .. } => Some(match phase {
            reqx::TimeoutPhase::Transport => "transport",
            reqx::TimeoutPhase::ResponseBody => "response_body",
        }),
        _ => None,
    }
}

fn reqx_transport_kind_name(error: &reqx::Error) -> Option<&'static str> {
    match error {
        reqx::Error::Transport { kind, .. } => Some(match kind {
            reqx::TransportErrorKind::Dns => "dns",
            reqx::TransportErrorKind::Connect => "connect",
            reqx::TransportErrorKind::Tls => "tls",
            reqx::TransportErrorKind::Read => "read",
            reqx::TransportErrorKind::Other => "other",
        }),
        _ => None,
    }
}

impl From<reqx::Error> for Error {
    fn from(source: reqx::Error) -> Self {
        let code = source.code().as_str();
        let status = source.status_code();
        let request_id = source.request_id().map(ToOwned::to_owned);
        let retry_after = source.retry_after(SystemTime::now());
        let method = source.request_method().map(ToString::to_string);
        let uri = source.request_uri_redacted_owned();
        let timeout_phase = reqx_timeout_phase_name(&source);
        let transport_kind = reqx_transport_kind_name(&source);
        let retryable = match source.code() {
            reqx::ErrorCode::Timeout
            | reqx::ErrorCode::DeadlineExceeded
            | reqx::ErrorCode::Transport
            | reqx::ErrorCode::RetryBudgetExhausted
            | reqx::ErrorCode::CircuitOpen => true,
            reqx::ErrorCode::HttpStatus => matches!(status, Some(429 | 500..=599)),
            _ => false,
        };

        if let Some(status) = status {
            let error = HttpError {
                status,
                message: None,
                request_id: request_id.clone(),
                body_snippet: None,
            };

            return match status {
                401 | 403 => Self::Auth(error),
                404 => Self::NotFound(error),
                409 | 412 => Self::Conflict(error),
                429 => Self::RateLimited { retry_after, error },
                _ => Self::Transport(Box::new(TransportError {
                    status: Some(status),
                    message: None,
                    request_id,
                    body_snippet: None,
                    retry_after,
                    retryable,
                    code,
                    method,
                    uri,
                    timeout_phase,
                    transport_kind,
                })),
            };
        }

        Self::Transport(Box::new(TransportError {
            status: None,
            message: Some(source.to_string()),
            request_id,
            body_snippet: None,
            retry_after,
            retryable,
            code,
            method,
            uri,
            timeout_phase,
            transport_kind,
        }))
    }
}

impl Error {
    /// Returns a stable high-level error category.
    #[must_use]
    pub fn kind(&self) -> ErrorKind {
        match self {
            Self::Auth(_) => ErrorKind::Auth,
            Self::NotFound(_) => ErrorKind::NotFound,
            Self::Conflict(_) => ErrorKind::Conflict,
            Self::RateLimited { .. } => ErrorKind::RateLimited,
            Self::Api { .. } => ErrorKind::Api,
            Self::Transport(_) => ErrorKind::Transport,
            Self::Serialization(_) => ErrorKind::Serialization,
            Self::Timestamp(_) => ErrorKind::Timestamp,
            Self::Signature => ErrorKind::Signature,
            Self::InvalidConfig { .. } => ErrorKind::InvalidConfig,
        }
    }

    /// Returns HTTP status code when present.
    #[must_use]
    pub fn status(&self) -> Option<u16> {
        match self {
            Self::Auth(error) | Self::NotFound(error) | Self::Conflict(error) => Some(error.status),
            Self::RateLimited { error, .. } => Some(error.status),
            Self::Transport(error) => error.status,
            _ => None,
        }
    }

    /// Returns DingTalk/transport request-id when present.
    #[must_use]
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Api { request_id, .. } => request_id.as_deref(),
            Self::Auth(error) | Self::NotFound(error) | Self::Conflict(error) => {
                error.request_id.as_deref()
            }
            Self::RateLimited { error, .. } => error.request_id.as_deref(),
            Self::Transport(error) => error.request_id.as_deref(),
            _ => None,
        }
    }

    /// Returns redacted body snippet when retained by the SDK.
    #[must_use]
    pub fn body_snippet(&self) -> Option<&str> {
        match self {
            Self::Api { body_snippet, .. } => body_snippet.as_deref(),
            Self::Auth(error) | Self::NotFound(error) | Self::Conflict(error) => {
                error.body_snippet.as_deref()
            }
            Self::RateLimited { error, .. } => error.body_snippet.as_deref(),
            Self::Transport(error) => error.body_snippet.as_deref(),
            _ => None,
        }
    }

    /// Returns `true` if the error is an auth/authz failure.
    #[must_use]
    pub fn is_auth_error(&self) -> bool {
        matches!(self, Self::Auth(_))
    }

    /// Returns `true` if the error is likely transient and safe to retry.
    #[must_use]
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::RateLimited { .. } => true,
            Self::Transport(error) => error.retryable,
            Self::Api { code, .. } => matches!(*code, 130101 | 130102),
            _ => false,
        }
    }

    /// Returns retry-after hint when the upstream provided one.
    #[must_use]
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after, .. } => *retry_after,
            Self::Transport(error) => error.retry_after,
            _ => None,
        }
    }
}
