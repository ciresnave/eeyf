use thiserror::Error;
use std::time::SystemTime;

use crate::quotes::YErrorMessage;

/// Error codes for programmatic error handling
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum YahooErrorCode {
    /// Network-related fetch failures
    FetchFailed,
    /// JSON deserialization failures
    DeserializeFailed,
    /// Connection errors (timeout, DNS, etc.)
    ConnectionFailed,
    /// Yahoo API returned an error response
    ApiError,
    /// No data available for the request
    NoResult,
    /// Data inconsistency detected
    DataInconsistency,
    /// Client builder configuration error
    BuilderFailed,
    /// Missing cookies in response
    NoCookies,
    /// Invalid cookie format
    InvalidCookie,
    /// Authentication/authorization failure
    Unauthorized,
    /// Invalid crumb token
    InvalidCrumb,
    /// Rate limit exceeded
    RateLimit,
    /// Invalid URL format
    InvalidUrl,
    /// Invalid date format
    InvalidDateFormat,
    /// Missing required field
    MissingField,
    /// Invalid HTTP status code
    InvalidStatusCode,
}

impl std::fmt::Display for YahooErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            YahooErrorCode::FetchFailed => write!(f, "FETCH_FAILED"),
            YahooErrorCode::DeserializeFailed => write!(f, "DESERIALIZE_FAILED"),
            YahooErrorCode::ConnectionFailed => write!(f, "CONNECTION_FAILED"),
            YahooErrorCode::ApiError => write!(f, "API_ERROR"),
            YahooErrorCode::NoResult => write!(f, "NO_RESULT"),
            YahooErrorCode::DataInconsistency => write!(f, "DATA_INCONSISTENCY"),
            YahooErrorCode::BuilderFailed => write!(f, "BUILDER_FAILED"),
            YahooErrorCode::NoCookies => write!(f, "NO_COOKIES"),
            YahooErrorCode::InvalidCookie => write!(f, "INVALID_COOKIE"),
            YahooErrorCode::Unauthorized => write!(f, "UNAUTHORIZED"),
            YahooErrorCode::InvalidCrumb => write!(f, "INVALID_CRUMB"),
            YahooErrorCode::RateLimit => write!(f, "RATE_LIMIT"),
            YahooErrorCode::InvalidUrl => write!(f, "INVALID_URL"),
            YahooErrorCode::InvalidDateFormat => write!(f, "INVALID_DATE_FORMAT"),
            YahooErrorCode::MissingField => write!(f, "MISSING_FIELD"),
            YahooErrorCode::InvalidStatusCode => write!(f, "INVALID_STATUS_CODE"),
        }
    }
}

/// Rich error context for enhanced debugging and observability
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// The symbol being requested (if applicable)
    pub symbol: Option<String>,
    /// The endpoint being called (if applicable)
    pub endpoint: Option<String>,
    /// When the error occurred
    pub timestamp: SystemTime,
    /// Unique request ID for distributed tracing
    pub request_id: Option<String>,
    /// Additional contextual information
    pub metadata: std::collections::HashMap<String, String>,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new() -> Self {
        Self {
            symbol: None,
            endpoint: None,
            timestamp: SystemTime::now(),
            request_id: None,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set the symbol for this context
    pub fn with_symbol(mut self, symbol: impl Into<String>) -> Self {
        self.symbol = Some(symbol.into());
        self
    }

    /// Set the endpoint for this context
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.endpoint = Some(endpoint.into());
        self
    }

    /// Set the request ID for this context
    pub fn with_request_id(mut self, request_id: impl Into<String>) -> Self {
        self.request_id = Some(request_id.into());
        self
    }

    /// Add metadata to this context
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for ErrorContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Error, Debug, Clone)]
pub enum YahooError {
    #[error("fetching the data from yahoo! finance failed: {0}")]
    FetchFailed(String),
    #[error("deserializing response from yahoo! finance failed: {0}")]
    DeserializeFailed(String),

    #[error("deserializing response from yahoo! finance failed, full response body: {0}")]
    DeserializeFailedDebug(String),

    #[error("connection to yahoo! finance server failed: {0}")]
    ConnectionFailed(String),
    #[error("yahoo! finance returned api error: {0:?}")]
    ApiError(YErrorMessage),
    #[error("yahoo! finance returned an empty data set")]
    NoResult,
    #[error("yahoo! finance returned an empty data set")]
    NoQuotes,
    #[error("yahoo! finance returned inconsistent data")]
    DataInconsistency,
    #[error("constructing yahoo! finance client failed")]
    BuilderFailed,
    #[error("No cookies in response headers")]
    NoCookies,
    #[error("Invisible characters in cookies")]
    InvisibleAsciiInCookies,
    #[error("No response")]
    NoResponse,
    #[error("Invalid cookie")]
    InvalidCookie,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Invalid crumb")]
    InvalidCrumb,
    #[error("Too many requests (rate limited by Yahoo) during: {0}")]
    TooManyRequests(String),
    #[error("Invalid URL format")]
    InvalidUrl,
    #[error("Invalid date format")]
    InvalidDateFormat,
    #[error("Missing required field: {0}")]
    MissingField(String),
    #[error("Invalid status code or configuration: {0}")]
    InvalidStatusCode(String),
}

impl YahooError {
    /// Get the error code for programmatic handling
    pub fn error_code(&self) -> YahooErrorCode {
        match self {
            YahooError::FetchFailed(_) => YahooErrorCode::FetchFailed,
            YahooError::DeserializeFailed(_) | YahooError::DeserializeFailedDebug(_) => {
                YahooErrorCode::DeserializeFailed
            }
            YahooError::ConnectionFailed(_) => YahooErrorCode::ConnectionFailed,
            YahooError::ApiError(_) => YahooErrorCode::ApiError,
            YahooError::NoResult | YahooError::NoQuotes | YahooError::NoResponse => {
                YahooErrorCode::NoResult
            }
            YahooError::DataInconsistency => YahooErrorCode::DataInconsistency,
            YahooError::BuilderFailed => YahooErrorCode::BuilderFailed,
            YahooError::NoCookies | YahooError::InvisibleAsciiInCookies => {
                YahooErrorCode::NoCookies
            }
            YahooError::InvalidCookie => YahooErrorCode::InvalidCookie,
            YahooError::Unauthorized => YahooErrorCode::Unauthorized,
            YahooError::InvalidCrumb => YahooErrorCode::InvalidCrumb,
            YahooError::TooManyRequests(_) => YahooErrorCode::RateLimit,
            YahooError::InvalidUrl => YahooErrorCode::InvalidUrl,
            YahooError::InvalidDateFormat => YahooErrorCode::InvalidDateFormat,
            YahooError::MissingField(_) => YahooErrorCode::MissingField,
            YahooError::InvalidStatusCode(_) => YahooErrorCode::InvalidStatusCode,
        }
    }

    /// Check if this error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            // Transient errors - safe to retry
            YahooError::ConnectionFailed(_)
            | YahooError::FetchFailed(_)
            | YahooError::DeserializeFailed(_)
            | YahooError::DeserializeFailedDebug(_)
            | YahooError::DataInconsistency
            | YahooError::NoResponse => true,

            // Rate limiting - retry with backoff
            YahooError::TooManyRequests(_) => true,

            // Server errors indicated in messages - might be retryable
            YahooError::InvalidStatusCode(msg) if msg.contains("5") => true,

            // Permanent errors - do not retry
            YahooError::ApiError(_)
            | YahooError::NoResult
            | YahooError::NoQuotes
            | YahooError::BuilderFailed
            | YahooError::NoCookies
            | YahooError::InvisibleAsciiInCookies
            | YahooError::InvalidCookie
            | YahooError::Unauthorized
            | YahooError::InvalidCrumb
            | YahooError::InvalidUrl
            | YahooError::InvalidDateFormat
            | YahooError::MissingField(_)
            | YahooError::InvalidStatusCode(_) => false,
        }
    }

    /// Get a user-friendly suggested action for this error
    pub fn suggested_action(&self) -> &'static str {
        match self {
            YahooError::TooManyRequests(_) => {
                "Rate limit exceeded. Wait 60 seconds before retrying, or reduce request frequency. Consider using the built-in rate limiter."
            }
            YahooError::ConnectionFailed(_) => {
                "Network connection failed. Check your internet connection and firewall settings. Retry in a few seconds."
            }
            YahooError::Unauthorized | YahooError::InvalidCrumb => {
                "Authentication failed. The session may have expired. Recreate the YahooConnector client to obtain fresh credentials."
            }
            YahooError::InvalidCookie | YahooError::NoCookies | YahooError::InvisibleAsciiInCookies => {
                "Cookie validation failed. This usually indicates a Yahoo service issue. Recreate the client and try again."
            }
            YahooError::NoResult | YahooError::NoQuotes => {
                "No data available for this symbol. Verify the symbol is correct and that market data is available for the requested time period."
            }
            YahooError::DataInconsistency => {
                "Data inconsistency detected in Yahoo's response. This is usually temporary. Retry the request."
            }
            YahooError::InvalidUrl => {
                "Invalid URL format. Check the symbol and parameters passed to the API."
            }
            YahooError::InvalidDateFormat => {
                "Invalid date format. Use Unix timestamps or ensure dates are in the correct format."
            }
            YahooError::MissingField(_) => {
                "Required field missing in response. This may indicate an API change. Check if you're using the latest version of this library."
            }
            YahooError::DeserializeFailed(_) | YahooError::DeserializeFailedDebug(_) => {
                "Failed to parse Yahoo's response. This may indicate an API change or data corruption. Check logs for details and consider reporting this issue."
            }
            YahooError::FetchFailed(_) => {
                "Failed to fetch data from Yahoo Finance. Check your network connection and retry. If the problem persists, Yahoo's service may be experiencing issues."
            }
            YahooError::ApiError(_) => {
                "Yahoo Finance API returned an error. Check the error message for specific details. The requested operation may not be supported."
            }
            YahooError::BuilderFailed => {
                "Failed to build the YahooConnector client. Check your configuration and ensure all required settings are valid."
            }
            YahooError::InvalidStatusCode(_) => {
                "Unexpected HTTP status code received. Check the error message for details. This may indicate a temporary service issue."
            }
            YahooError::NoResponse => {
                "No response received from Yahoo Finance. This may indicate a network timeout or service outage. Retry the request."
            }
        }
    }

    /// Create an error with context
    pub fn with_context(self, context: ErrorContext) -> YahooErrorWithContext {
        YahooErrorWithContext {
            error: self,
            context,
        }
    }
}

/// Yahoo error with rich context for enhanced debugging
#[derive(Debug, Clone)]
pub struct YahooErrorWithContext {
    pub error: YahooError,
    pub context: ErrorContext,
}

impl std::fmt::Display for YahooErrorWithContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        
        if let Some(ref symbol) = self.context.symbol {
            write!(f, " [symbol: {}]", symbol)?;
        }
        
        if let Some(ref endpoint) = self.context.endpoint {
            write!(f, " [endpoint: {}]", endpoint)?;
        }
        
        if let Some(ref request_id) = self.context.request_id {
            write!(f, " [request_id: {}]", request_id)?;
        }
        
        if let Ok(elapsed) = self.context.timestamp.elapsed() {
            write!(f, " [occurred: {:?} ago]", elapsed)?;
        }
        
        Ok(())
    }
}

impl std::error::Error for YahooErrorWithContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.error)
    }
}

impl From<serde_json::Error> for YahooError {
    fn from(error: serde_json::Error) -> Self {
        YahooError::DeserializeFailed(error.to_string())
    }
}

impl From<reqwest::Error> for YahooError {
    fn from(error: reqwest::Error) -> Self {
        YahooError::ConnectionFailed(error.to_string())
    }
}

