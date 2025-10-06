//! Property-based tests for EEYF
//!
//! Uses proptest to verify properties hold for a wide range of inputs.
//! This helps catch edge cases and unexpected behaviors.

use proptest::prelude::*;
use eeyf::{YahooError, YahooErrorCode, ErrorContext};

// Property: All error codes should have valid string representations
proptest! {
    #[test]
    fn error_code_to_string_never_panics(code in prop_error_code()) {
        let _ = format!("{}", code);
    }
}

// Property: Error codes are consistent (same code always produces same string)
proptest! {
    #[test]
    fn error_code_display_is_consistent(code in prop_error_code()) {
        let str1 = format!("{}", code);
        let str2 = format!("{}", code);
        prop_assert_eq!(str1, str2);
    }
}

// Property: Error codes are deterministic
proptest! {
    #[test]
    fn error_code_equality_is_reflexive(code in prop_error_code()) {
        prop_assert_eq!(code.clone(), code);
    }
}

// Property: is_retryable is deterministic
proptest! {
    #[test]
    fn is_retryable_is_deterministic(error in prop_yahoo_error()) {
        let result1 = error.is_retryable();
        let result2 = error.is_retryable();
        prop_assert_eq!(result1, result2);
    }
}

// Property: error_code() never panics
proptest! {
    #[test]
    fn error_code_method_never_panics(error in prop_yahoo_error()) {
        let _ = error.error_code();
    }
}

// Property: suggested_action() always returns a non-empty string
proptest! {
    #[test]
    fn suggested_action_always_non_empty(error in prop_yahoo_error()) {
        let suggestion = error.suggested_action();
        prop_assert!(!suggestion.is_empty());
        prop_assert!(suggestion.len() > 10); // Meaningful suggestions
    }
}

// Property: ErrorContext builder methods are chainable
proptest! {
    #[test]
    fn error_context_builder_is_chainable(
        symbol in "[A-Z]{1,5}",
        endpoint in "/[a-z/]+",
        request_id in "[a-z0-9-]{10,20}"
    ) {
        let context = ErrorContext::new()
            .with_symbol(symbol.clone())
            .with_endpoint(endpoint.clone())
            .with_request_id(request_id.clone());
        
        prop_assert_eq!(context.symbol, Some(symbol));
        prop_assert_eq!(context.endpoint, Some(endpoint));
        prop_assert_eq!(context.request_id, Some(request_id));
    }
}

// Property: ErrorContext metadata can store arbitrary key-value pairs
proptest! {
    #[test]
    fn error_context_metadata_stores_values(
        key in "[a-z_]{3,20}",
        value in "[a-zA-Z0-9 ]{5,50}"
    ) {
        let context = ErrorContext::new()
            .with_metadata(key.clone(), value.clone());
        
        prop_assert_eq!(context.metadata.get(&key), Some(&value));
    }
}

// Property: with_context never loses the original error
proptest! {
    #[test]
    fn with_context_preserves_error(error in prop_yahoo_error()) {
        let error_code = error.error_code();
        let context = ErrorContext::new();
        let error_with_context = error.with_context(context);
        
        prop_assert_eq!(error_with_context.error.error_code(), error_code);
    }
}

// Property: Error display never panics
proptest! {
    #[test]
    fn error_display_never_panics(error in prop_yahoo_error()) {
        let _ = format!("{}", error);
    }
}

// Property: Error debug never panics
proptest! {
    #[test]
    fn error_debug_never_panics(error in prop_yahoo_error()) {
        let _ = format!("{:?}", error);
    }
}

// Property: Error with context display never panics
proptest! {
    #[test]
    fn error_with_context_display_never_panics(
        error in prop_yahoo_error(),
        symbol in option::of("[A-Z]{1,5}")
    ) {
        let mut context = ErrorContext::new();
        if let Some(s) = symbol {
            context = context.with_symbol(s);
        }
        let error_with_context = error.with_context(context);
        let _ = format!("{}", error_with_context);
    }
}

// Property: Retryable errors should have suggested actions mentioning retry
proptest! {
    #[test]
    fn retryable_errors_mention_retry_in_suggestion(error in prop_yahoo_error()) {
        if error.is_retryable() {
            let suggestion = error.suggested_action().to_lowercase();
            // Retryable errors should mention retry or wait
            let mentions_retry = suggestion.contains("retry") 
                || suggestion.contains("wait") 
                || suggestion.contains("try again")
                || suggestion.contains("seconds");
            
            if !mentions_retry {
                // Some retryable errors may suggest other recovery (like "recreate")
                prop_assert!(
                    suggestion.contains("recreate") || 
                    suggestion.contains("connection") ||
                    suggestion.contains("network"),
                    "Retryable error suggestion should mention retry or recovery: {}",
                    suggestion
                );
            }
        }
    }
}

// Property: Non-retryable errors should suggest fixing the problem
proptest! {
    #[test]
    fn non_retryable_errors_suggest_fixes(error in prop_yahoo_error()) {
        if !error.is_retryable() {
            let suggestion = error.suggested_action();
            // Non-retryable errors should provide actionable guidance
            prop_assert!(!suggestion.is_empty());
            prop_assert!(suggestion.len() > 20); // Substantial advice
        }
    }
}

// Property: Error categorization is consistent with is_retryable
proptest! {
    #[test]
    fn categorization_matches_retryable(error in prop_yahoo_error()) {
        use eeyf::ErrorCategorizer;
        let is_retryable = error.is_retryable();
        let info = error.categorize_error();
        prop_assert_eq!(is_retryable, info.is_retryable);
    }
}

// Property: Symbol strings in context should be uppercase (convention)
proptest! {
    #[test]
    fn context_preserves_symbol_case(symbol in "[A-Za-z]{1,10}") {
        let context = ErrorContext::new().with_symbol(symbol.clone());
        prop_assert_eq!(context.symbol, Some(symbol));
    }
}

// Helper: Generate arbitrary YahooErrorCode
fn prop_error_code() -> impl Strategy<Value = YahooErrorCode> {
    prop_oneof![
        Just(YahooErrorCode::FetchFailed),
        Just(YahooErrorCode::DeserializeFailed),
        Just(YahooErrorCode::ConnectionFailed),
        Just(YahooErrorCode::ApiError),
        Just(YahooErrorCode::NoResult),
        Just(YahooErrorCode::DataInconsistency),
        Just(YahooErrorCode::BuilderFailed),
        Just(YahooErrorCode::NoCookies),
        Just(YahooErrorCode::InvalidCookie),
        Just(YahooErrorCode::Unauthorized),
        Just(YahooErrorCode::InvalidCrumb),
        Just(YahooErrorCode::RateLimit),
        Just(YahooErrorCode::InvalidUrl),
        Just(YahooErrorCode::InvalidDateFormat),
        Just(YahooErrorCode::MissingField),
        Just(YahooErrorCode::InvalidStatusCode),
    ]
}

// Helper: Generate arbitrary YahooError
fn prop_yahoo_error() -> impl Strategy<Value = YahooError> {
    prop_oneof![
        "[a-z ]{10,50}".prop_map(YahooError::FetchFailed),
        "[a-z ]{10,50}".prop_map(YahooError::DeserializeFailed),
        "[a-z ]{10,50}".prop_map(YahooError::ConnectionFailed),
        Just(YahooError::NoResult),
        Just(YahooError::NoQuotes),
        Just(YahooError::DataInconsistency),
        Just(YahooError::BuilderFailed),
        Just(YahooError::NoCookies),
        Just(YahooError::InvalidCookie),
        Just(YahooError::Unauthorized),
        Just(YahooError::InvalidCrumb),
        "[a-z/]{5,20}".prop_map(YahooError::TooManyRequests),
        Just(YahooError::InvalidUrl),
        Just(YahooError::InvalidDateFormat),
        "[a-z_]{3,20}".prop_map(YahooError::MissingField),
        "[0-9]{3}".prop_map(|s| YahooError::InvalidStatusCode(format!("Status {}", s))),
    ]
}

#[cfg(test)]
mod timestamp_tests {
    use super::*;

    // Property: Timestamps in ErrorContext are always in the past or present
    proptest! {
        #[test]
        fn error_context_timestamp_is_not_future(_i in 0..100u32) {
            let context = ErrorContext::new();
            let now = std::time::SystemTime::now();
            
            // Allow for small clock drift (1 second)
            match context.timestamp.duration_since(now) {
                Ok(duration) => {
                    prop_assert!(duration.as_secs() <= 1);
                }
                Err(_) => {
                    // Timestamp is in the past, which is expected
                }
            }
        }
    }
}

#[cfg(test)]
mod edge_cases {
    use super::*;

    // Property: Empty strings in errors don't cause panics
    proptest! {
        #[test]
        fn empty_string_errors_dont_panic(msg in "[ ]*") {
            let error = YahooError::FetchFailed(msg);
            let _ = format!("{}", error);
            let _ = error.error_code();
            let _ = error.is_retryable();
            let _ = error.suggested_action();
        }
    }

    // Property: Very long error messages don't cause issues
    proptest! {
        #[test]
        fn long_error_messages_are_handled(msg in "[a-z ]{1000,2000}") {
            let error = YahooError::FetchFailed(msg);
            let display = format!("{}", error);
            prop_assert!(display.len() > 0);
        }
    }

    // Property: Special characters in metadata don't break anything
    proptest! {
        #[test]
        fn special_chars_in_metadata_work(
            key in "[a-z!@#$%]{5,20}",
            value in "[a-zA-Z0-9 !@#$%^&*()]{10,50}"
        ) {
            let context = ErrorContext::new().with_metadata(key.clone(), value.clone());
            prop_assert_eq!(context.metadata.get(&key), Some(&value));
        }
    }
}
