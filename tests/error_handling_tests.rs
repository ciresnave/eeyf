//! Tests for enhanced error handling features (Phase 1.3)

use eeyf::{ErrorCategorizer, ErrorCategory, ErrorContext, YahooError, YahooErrorCode, YahooErrorWithContext};
use std::time::SystemTime;

#[test]
fn test_error_codes() {
    // Test error code mapping
    let rate_limit = YahooError::TooManyRequests("test".to_string());
    assert_eq!(rate_limit.error_code(), YahooErrorCode::RateLimit);

    let connection_failed = YahooError::ConnectionFailed("timeout".to_string());
    assert_eq!(connection_failed.error_code(), YahooErrorCode::ConnectionFailed);

    let unauthorized = YahooError::Unauthorized;
    assert_eq!(unauthorized.error_code(), YahooErrorCode::Unauthorized);

    let no_result = YahooError::NoResult;
    assert_eq!(no_result.error_code(), YahooErrorCode::NoResult);

    let no_quotes = YahooError::NoQuotes;
    assert_eq!(no_quotes.error_code(), YahooErrorCode::NoResult);

    let deserialize_failed = YahooError::DeserializeFailed("test".to_string());
    assert_eq!(deserialize_failed.error_code(), YahooErrorCode::DeserializeFailed);
}

#[test]
fn test_error_code_display() {
    assert_eq!(YahooErrorCode::RateLimit.to_string(), "RATE_LIMIT");
    assert_eq!(YahooErrorCode::Unauthorized.to_string(), "UNAUTHORIZED");
    assert_eq!(YahooErrorCode::ConnectionFailed.to_string(), "CONNECTION_FAILED");
    assert_eq!(YahooErrorCode::NoResult.to_string(), "NO_RESULT");
    assert_eq!(YahooErrorCode::InvalidUrl.to_string(), "INVALID_URL");
}

#[test]
fn test_is_retryable() {
    // Retryable errors
    assert!(YahooError::ConnectionFailed("test".to_string()).is_retryable());
    assert!(YahooError::FetchFailed("test".to_string()).is_retryable());
    assert!(YahooError::DeserializeFailed("test".to_string()).is_retryable());
    assert!(YahooError::TooManyRequests("test".to_string()).is_retryable());
    assert!(YahooError::DataInconsistency.is_retryable());
    assert!(YahooError::NoResponse.is_retryable());

    // Non-retryable errors
    assert!(!YahooError::Unauthorized.is_retryable());
    assert!(!YahooError::NoResult.is_retryable());
    assert!(!YahooError::NoQuotes.is_retryable());
    assert!(!YahooError::BuilderFailed.is_retryable());
    assert!(!YahooError::InvalidUrl.is_retryable());
    assert!(!YahooError::InvalidDateFormat.is_retryable());
    assert!(!YahooError::MissingField("test".to_string()).is_retryable());
}

#[test]
fn test_suggested_action() {
    let rate_limit = YahooError::TooManyRequests("test".to_string());
    let suggestion = rate_limit.suggested_action();
    assert!(suggestion.contains("Rate limit"));
    assert!(suggestion.contains("60 seconds"));

    let unauthorized = YahooError::Unauthorized;
    let suggestion = unauthorized.suggested_action();
    assert!(suggestion.contains("Authentication"));
    assert!(suggestion.contains("YahooConnector"));

    let no_result = YahooError::NoResult;
    let suggestion = no_result.suggested_action();
    assert!(suggestion.contains("symbol"));
    assert!(suggestion.contains("correct"));

    let connection_failed = YahooError::ConnectionFailed("timeout".to_string());
    let suggestion = connection_failed.suggested_action();
    assert!(suggestion.contains("Network"));
    assert!(suggestion.contains("connection"));
}

#[test]
fn test_error_context_builder() {
    let context = ErrorContext::new()
        .with_symbol("AAPL")
        .with_endpoint("/v8/finance/chart")
        .with_request_id("req-12345")
        .with_metadata("user_id", "user-789")
        .with_metadata("region", "us-east-1");

    assert_eq!(context.symbol, Some("AAPL".to_string()));
    assert_eq!(context.endpoint, Some("/v8/finance/chart".to_string()));
    assert_eq!(context.request_id, Some("req-12345".to_string()));
    assert_eq!(context.metadata.get("user_id"), Some(&"user-789".to_string()));
    assert_eq!(context.metadata.get("region"), Some(&"us-east-1".to_string()));
    
    // Timestamp should be recent (within last second)
    let elapsed = context.timestamp.elapsed().unwrap();
    assert!(elapsed.as_secs() < 1);
}

#[test]
fn test_error_context_default() {
    let context = ErrorContext::default();
    assert!(context.symbol.is_none());
    assert!(context.endpoint.is_none());
    assert!(context.request_id.is_none());
    assert!(context.metadata.is_empty());
}

#[test]
fn test_error_with_context() {
    let error = YahooError::ConnectionFailed("timeout".to_string());
    let context = ErrorContext::new()
        .with_symbol("AAPL")
        .with_endpoint("/v8/finance/chart")
        .with_request_id("req-12345");

    let error_with_context = error.with_context(context);

    assert!(matches!(error_with_context.error, YahooError::ConnectionFailed(_)));
    assert_eq!(error_with_context.context.symbol, Some("AAPL".to_string()));
    assert_eq!(error_with_context.context.endpoint, Some("/v8/finance/chart".to_string()));
    assert_eq!(error_with_context.context.request_id, Some("req-12345".to_string()));
}

#[test]
fn test_error_with_context_display() {
    let error = YahooError::TooManyRequests("test endpoint".to_string());
    let context = ErrorContext::new()
        .with_symbol("AAPL")
        .with_endpoint("/v8/finance/chart")
        .with_request_id("req-12345");

    let error_with_context = error.with_context(context);
    let display = format!("{}", error_with_context);

    // Should contain the error message
    assert!(display.contains("Too many requests"));
    assert!(display.contains("test endpoint"));

    // Should contain context information
    assert!(display.contains("symbol: AAPL"));
    assert!(display.contains("endpoint: /v8/finance/chart"));
    assert!(display.contains("request_id: req-12345"));
    assert!(display.contains("occurred:"));
}

#[test]
fn test_error_categorization_rate_limit() {
    let error = YahooError::TooManyRequests("test".to_string());
    let info = error.categorize_error();

    assert_eq!(info.category, ErrorCategory::RateLimit);
    assert!(info.is_retryable);
    assert_eq!(info.suggested_delay_ms, Some(5000));
    assert_eq!(info.error_code, Some("RATE_LIMIT".to_string()));
}

#[test]
fn test_error_categorization_authentication() {
    let error = YahooError::Unauthorized;
    let info = error.categorize_error();

    assert_eq!(info.category, ErrorCategory::Authentication);
    assert!(!info.is_retryable);
    assert_eq!(info.suggested_delay_ms, None);
    assert_eq!(info.error_code, Some("UNAUTHORIZED".to_string()));
}

#[test]
fn test_error_categorization_transient() {
    let error = YahooError::ConnectionFailed("timeout".to_string());
    let info = error.categorize_error();

    assert_eq!(info.category, ErrorCategory::Transient);
    assert!(info.is_retryable);
    assert!(info.suggested_delay_ms.is_some());
}

#[test]
fn test_error_categorization_permanent() {
    let error = YahooError::NoResult;
    let info = error.categorize_error();

    assert_eq!(info.category, ErrorCategory::Permanent);
    assert!(!info.is_retryable);
    assert_eq!(info.suggested_delay_ms, None);
}

#[test]
fn test_error_categorization_configuration() {
    let error = YahooError::InvalidUrl;
    let info = error.categorize_error();

    assert_eq!(info.category, ErrorCategory::Configuration);
    assert!(!info.is_retryable);
    assert_eq!(info.suggested_delay_ms, None);
}

#[test]
fn test_category_properties() {
    // Test retryable categories
    assert!(ErrorCategory::Transient.is_retryable());
    assert!(ErrorCategory::RateLimit.is_retryable());
    assert!(ErrorCategory::ServerError.is_retryable());

    // Test non-retryable categories
    assert!(!ErrorCategory::Authentication.is_retryable());
    assert!(!ErrorCategory::ClientError.is_retryable());
    assert!(!ErrorCategory::Configuration.is_retryable());
    assert!(!ErrorCategory::Permanent.is_retryable());
    assert!(!ErrorCategory::Unknown.is_retryable());
}

#[test]
fn test_category_delays() {
    assert_eq!(ErrorCategory::Transient.base_delay_ms(), 1000);
    assert_eq!(ErrorCategory::RateLimit.base_delay_ms(), 5000);
    assert_eq!(ErrorCategory::ServerError.base_delay_ms(), 2000);
    assert_eq!(ErrorCategory::Authentication.base_delay_ms(), 0);
}

#[test]
fn test_category_max_retries() {
    assert_eq!(ErrorCategory::Transient.max_retries(), 3);
    assert_eq!(ErrorCategory::RateLimit.max_retries(), 5);
    assert_eq!(ErrorCategory::ServerError.max_retries(), 2);
    assert_eq!(ErrorCategory::Authentication.max_retries(), 0);
}

#[test]
fn test_category_display() {
    assert_eq!(ErrorCategory::Transient.to_string(), "transient");
    assert_eq!(ErrorCategory::RateLimit.to_string(), "rate_limit");
    assert_eq!(ErrorCategory::Authentication.to_string(), "authentication");
    assert_eq!(ErrorCategory::ClientError.to_string(), "client_error");
    assert_eq!(ErrorCategory::ServerError.to_string(), "server_error");
    assert_eq!(ErrorCategory::Configuration.to_string(), "configuration");
    assert_eq!(ErrorCategory::Permanent.to_string(), "permanent");
    assert_eq!(ErrorCategory::Unknown.to_string(), "unknown");
}

#[test]
fn test_error_code_equality() {
    let code1 = YahooErrorCode::RateLimit;
    let code2 = YahooErrorCode::RateLimit;
    let code3 = YahooErrorCode::Unauthorized;

    assert_eq!(code1, code2);
    assert_ne!(code1, code3);
}

#[test]
fn test_error_categorization_consistency() {
    // Ensure is_retryable() and categorize_error() are consistent
    let errors = vec![
        YahooError::TooManyRequests("test".to_string()),
        YahooError::ConnectionFailed("test".to_string()),
        YahooError::Unauthorized,
        YahooError::NoResult,
        YahooError::InvalidUrl,
        YahooError::DataInconsistency,
    ];

    for error in errors {
        let is_retryable = error.is_retryable();
        let info = error.categorize_error();

        assert_eq!(
            is_retryable,
            info.is_retryable,
            "Inconsistency for error: {:?}",
            error
        );
    }
}
