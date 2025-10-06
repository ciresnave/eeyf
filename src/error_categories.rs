//! Enhanced error categorization for improved error handling and retry logic
//!
//! This module provides detailed error classification to enable intelligent
//! retry strategies, circuit breaker decisions, and observability metrics.

use crate::yahoo_error::YahooError;
use std::fmt;

/// Categories of errors for intelligent error handling
#[derive(Debug, Clone, PartialEq)]
pub enum ErrorCategory {
    /// Temporary network issues that should be retried
    Transient,
    /// Rate limiting errors - should use exponential backoff
    RateLimit,
    /// Authentication/authorization issues - should not retry
    Authentication,
    /// Client errors (4xx) - should not retry
    ClientError,
    /// Server errors (5xx) - may retry with caution
    ServerError,
    /// Configuration or input validation errors - should not retry
    Configuration,
    /// Permanent failures - should not retry
    Permanent,
    /// Unknown errors - handle conservatively
    Unknown,
}

/// Detailed error information for enhanced observability
#[derive(Debug, Clone)]
pub struct ErrorInfo {
    pub category: ErrorCategory,
    pub is_retryable: bool,
    pub suggested_delay_ms: Option<u64>,
    pub context: String,
    pub error_code: Option<String>,
}

impl ErrorCategory {
    /// Determine if errors of this category should be retried
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ErrorCategory::Transient | ErrorCategory::RateLimit | ErrorCategory::ServerError
        )
    }

    /// Get suggested base delay in milliseconds for retry logic
    pub fn base_delay_ms(&self) -> u64 {
        match self {
            ErrorCategory::Transient => 1000,   // 1 second
            ErrorCategory::RateLimit => 5000,   // 5 seconds
            ErrorCategory::ServerError => 2000, // 2 seconds
            _ => 0,                             // No retry
        }
    }

    /// Maximum number of retries for this category
    pub fn max_retries(&self) -> u32 {
        match self {
            ErrorCategory::Transient => 3,
            ErrorCategory::RateLimit => 5,
            ErrorCategory::ServerError => 2,
            _ => 0,
        }
    }
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCategory::Transient => write!(f, "transient"),
            ErrorCategory::RateLimit => write!(f, "rate_limit"),
            ErrorCategory::Authentication => write!(f, "authentication"),
            ErrorCategory::ClientError => write!(f, "client_error"),
            ErrorCategory::ServerError => write!(f, "server_error"),
            ErrorCategory::Configuration => write!(f, "configuration"),
            ErrorCategory::Permanent => write!(f, "permanent"),
            ErrorCategory::Unknown => write!(f, "unknown"),
        }
    }
}

/// Trait for categorizing errors
pub trait ErrorCategorizer {
    fn categorize_error(&self) -> ErrorInfo;
}

impl ErrorCategorizer for YahooError {
    fn categorize_error(&self) -> ErrorInfo {
        match self {
            YahooError::TooManyRequests(_) => ErrorInfo {
                category: ErrorCategory::RateLimit,
                is_retryable: true,
                suggested_delay_ms: Some(5000),
                context: format!("Rate limit exceeded: {}", self),
                error_code: Some("RATE_LIMIT".to_string()),
            },

            YahooError::ConnectionFailed(error_msg) => {
                // Parse common error patterns from the string
                if error_msg.to_lowercase().contains("timeout") {
                    ErrorInfo {
                        category: ErrorCategory::Transient,
                        is_retryable: true,
                        suggested_delay_ms: Some(1000),
                        context: format!("Network timeout: {}", error_msg),
                        error_code: Some("TIMEOUT".to_string()),
                    }
                } else if error_msg.to_lowercase().contains("connect") {
                    ErrorInfo {
                        category: ErrorCategory::Transient,
                        is_retryable: true,
                        suggested_delay_ms: Some(2000),
                        context: format!("Connection failed: {}", error_msg),
                        error_code: Some("CONNECTION_FAILED".to_string()),
                    }
                } else if error_msg.contains("401") || error_msg.contains("403") {
                    ErrorInfo {
                        category: ErrorCategory::Authentication,
                        is_retryable: false,
                        suggested_delay_ms: None,
                        context: format!("Authentication error: {}", error_msg),
                        error_code: Some("AUTH_ERROR".to_string()),
                    }
                } else if error_msg.contains("429") {
                    ErrorInfo {
                        category: ErrorCategory::RateLimit,
                        is_retryable: true,
                        suggested_delay_ms: Some(10000),
                        context: format!("Rate limit response: {}", error_msg),
                        error_code: Some("RATE_LIMIT_429".to_string()),
                    }
                } else if error_msg.contains("400") || error_msg.contains("4") {
                    ErrorInfo {
                        category: ErrorCategory::ClientError,
                        is_retryable: false,
                        suggested_delay_ms: None,
                        context: format!("Client error: {}", error_msg),
                        error_code: Some("CLIENT_ERROR".to_string()),
                    }
                } else if error_msg.contains("500") || error_msg.contains("5") {
                    ErrorInfo {
                        category: ErrorCategory::ServerError,
                        is_retryable: true,
                        suggested_delay_ms: Some(3000),
                        context: format!("Server error: {}", error_msg),
                        error_code: Some("SERVER_ERROR".to_string()),
                    }
                } else {
                    ErrorInfo {
                        category: ErrorCategory::Transient,
                        is_retryable: true,
                        suggested_delay_ms: Some(1000),
                        context: format!("Network error: {}", error_msg),
                        error_code: Some("NETWORK_ERROR".to_string()),
                    }
                }
            }

            YahooError::Unauthorized => ErrorInfo {
                category: ErrorCategory::Authentication,
                is_retryable: false,
                suggested_delay_ms: None,
                context: "Yahoo Finance authentication failed".to_string(),
                error_code: Some("UNAUTHORIZED".to_string()),
            },

            YahooError::InvalidUrl => ErrorInfo {
                category: ErrorCategory::Configuration,
                is_retryable: false,
                suggested_delay_ms: None,
                context: "Invalid URL configuration".to_string(),
                error_code: Some("INVALID_URL".to_string()),
            },

            YahooError::InvalidDateFormat => ErrorInfo {
                category: ErrorCategory::Configuration,
                is_retryable: false,
                suggested_delay_ms: None,
                context: "Invalid date format provided".to_string(),
                error_code: Some("INVALID_DATE".to_string()),
            },

            YahooError::MissingField(_) => ErrorInfo {
                category: ErrorCategory::Configuration,
                is_retryable: false,
                suggested_delay_ms: None,
                context: format!("Missing required field: {}", self),
                error_code: Some("MISSING_FIELD".to_string()),
            },

            YahooError::DeserializeFailed(_) => ErrorInfo {
                category: ErrorCategory::Transient,
                is_retryable: true,
                suggested_delay_ms: Some(500),
                context: format!("JSON deserialization failed: {}", self),
                error_code: Some("DESERIALIZE_FAILED".to_string()),
            },

            YahooError::NoResult | YahooError::NoQuotes => ErrorInfo {
                category: ErrorCategory::Permanent,
                is_retryable: false,
                suggested_delay_ms: None,
                context: format!("No data available: {}", self),
                error_code: Some("NO_DATA".to_string()),
            },

            YahooError::DataInconsistency => ErrorInfo {
                category: ErrorCategory::Transient,
                is_retryable: true,
                suggested_delay_ms: Some(1000),
                context: "Data inconsistency detected".to_string(),
                error_code: Some("DATA_INCONSISTENCY".to_string()),
            },

            YahooError::FetchFailed(_) => ErrorInfo {
                category: ErrorCategory::Transient,
                is_retryable: true,
                suggested_delay_ms: Some(2000),
                context: format!("Fetch operation failed: {}", self),
                error_code: Some("FETCH_FAILED".to_string()),
            },

            _ => ErrorInfo {
                category: ErrorCategory::Unknown,
                is_retryable: false,
                suggested_delay_ms: None,
                context: format!("Unhandled error: {}", self),
                error_code: Some("UNKNOWN".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categorization() {
        let rate_limit_error = YahooError::TooManyRequests("test".to_string());
        let info = rate_limit_error.categorize_error();

        assert_eq!(info.category, ErrorCategory::RateLimit);
        assert!(info.is_retryable);
        assert_eq!(info.suggested_delay_ms, Some(5000));
        assert_eq!(info.error_code, Some("RATE_LIMIT".to_string()));
    }

    #[test]
    fn test_category_properties() {
        assert!(ErrorCategory::RateLimit.is_retryable());
        assert!(!ErrorCategory::Authentication.is_retryable());
        assert_eq!(ErrorCategory::RateLimit.base_delay_ms(), 5000);
        assert_eq!(ErrorCategory::RateLimit.max_retries(), 5);
    }

    #[test]
    fn test_category_display() {
        assert_eq!(ErrorCategory::RateLimit.to_string(), "rate_limit");
        assert_eq!(ErrorCategory::Authentication.to_string(), "authentication");
    }
}
