//! Error handling and recovery example
//!
//! This example demonstrates:
//! - Error categorization and handling
//! - Intelligent retry strategies
//! - Using error codes for programmatic handling
//! - Adding context to errors
//! - Suggested actions
//!
//! Run with: cargo run --example error_recovery

use eeyf::{
    ErrorCategorizer, ErrorCategory, ErrorContext, YahooConnector, YahooError, YahooErrorCode,
};
use std::time::Duration;
use time::OffsetDateTime;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== EEYF Error Handling and Recovery Demo ===\n");

    // Example 1: Error codes
    println!("1️⃣  Error Codes:");
    demonstrate_error_codes();

    println!("\n{}", "=".repeat(60));

    // Example 2: Retryability
    println!("\n2️⃣  Error Retryability:");
    demonstrate_retryability();

    println!("\n{}", "=".repeat(60));

    // Example 3: Suggested actions
    println!("\n3️⃣  Suggested Actions:");
    demonstrate_suggested_actions();

    println!("\n{}", "=".repeat(60));

    // Example 4: Error categorization
    println!("\n4️⃣  Error Categorization:");
    demonstrate_categorization();

    println!("\n{}", "=".repeat(60));

    // Example 5: Error context
    println!("\n5️⃣  Error Context:");
    demonstrate_context();

    println!("\n{}", "=".repeat(60));

    // Example 6: Real API error handling
    println!("\n6️⃣  Real API Error Handling:");
    if let Err(e) = demonstrate_real_api_error().await {
        eprintln!("API error: {}", e);
    }

    Ok(())
}

/// Demonstrate error code usage
fn demonstrate_error_codes() {
    let errors = vec![
        ("Rate Limit", YahooError::TooManyRequests("endpoint".to_string())),
        ("Connection Failed", YahooError::ConnectionFailed("timeout".to_string())),
        ("Unauthorized", YahooError::Unauthorized),
        ("No Result", YahooError::NoResult),
        ("Invalid URL", YahooError::InvalidUrl),
    ];

    for (name, error) in errors {
        let code = error.error_code();
        println!("  {} -> Error Code: {}", name, code);
    }
}

/// Demonstrate retryability detection
fn demonstrate_retryability() {
    let errors = vec![
        ("Rate Limit", YahooError::TooManyRequests("test".to_string()), true),
        ("Connection Failed", YahooError::ConnectionFailed("timeout".to_string()), true),
        ("Unauthorized", YahooError::Unauthorized, false),
        ("No Result", YahooError::NoResult, false),
        ("Data Inconsistency", YahooError::DataInconsistency, true),
    ];

    for (name, error, expected) in errors {
        let is_retryable = error.is_retryable();
        let status = if is_retryable == expected { "✅" } else { "❌" };
        println!("  {} {} -> Retryable: {}", status, name, is_retryable);
    }
}

/// Demonstrate suggested actions
fn demonstrate_suggested_actions() {
    let errors = vec![
        YahooError::TooManyRequests("test".to_string()),
        YahooError::Unauthorized,
        YahooError::NoResult,
        YahooError::ConnectionFailed("timeout".to_string()),
    ];

    for error in errors {
        println!("\n  Error: {}", error);
        println!("  💡 {}", error.suggested_action());
    }
}

/// Demonstrate error categorization
fn demonstrate_categorization() {
    let errors = vec![
        YahooError::TooManyRequests("test".to_string()),
        YahooError::ConnectionFailed("timeout".to_string()),
        YahooError::Unauthorized,
        YahooError::InvalidUrl,
        YahooError::DataInconsistency,
    ];

    println!("\n  Category Retry Base Delay Max Retries");
    println!("  {}", "-".repeat(50));

    for error in errors {
        let info = error.categorize_error();
        let delay = info.suggested_delay_ms.unwrap_or(0);
        println!(
            "  {:15} {:5} {:9}ms {:11}",
            info.category.to_string(),
            info.is_retryable,
            delay,
            info.category.max_retries()
        );
    }
}

/// Demonstrate error context
fn demonstrate_context() {
    let context = ErrorContext::new()
        .with_symbol("AAPL")
        .with_endpoint("/v8/finance/chart")
        .with_request_id("req-12345")
        .with_metadata("user_id", "demo-user-123")
        .with_metadata("region", "us-east-1");

    let error = YahooError::ConnectionFailed("timeout".to_string());
    let error_with_context = error.with_context(context);

    println!("\n  Error with context:");
    println!("  {}", error_with_context);
}

/// Demonstrate real API error handling
async fn demonstrate_real_api_error() -> Result<(), YahooError> {
    println!("\n  Attempting to fetch data with error handling...");
    
    let connector = match YahooConnector::new() {
        Ok(c) => c,
        Err(e) => {
            println!("  ❌ Failed to create connector: {}", e);
            println!("  💡 {}", e.suggested_action());
            return Err(e);
        }
    };

    let symbol = "AAPL";
    let start = OffsetDateTime::now_utc() - time::Duration::days(30);
    let end = OffsetDateTime::now_utc();

    match fetch_with_intelligent_retry(&connector, symbol, start, end, 3).await {
        Ok(_) => {
            println!("  ✅ Successfully fetched data for {}", symbol);
            Ok(())
        }
        Err(error) => {
            println!("  ❌ Final error: {}", error);
            println!("  � Error Code: {}", error.error_code());
            println!("  💡 {}", error.suggested_action());
            Err(error)
        }
    }
}

/// Intelligent retry with exponential backoff
async fn fetch_with_intelligent_retry(
    connector: &YahooConnector,
    symbol: &str,
    start: OffsetDateTime,
    end: OffsetDateTime,
    max_attempts: u32,
) -> Result<YResponse, YahooError> {
    let mut attempt = 0;

    loop {
        attempt += 1;
        println!("    🔄 Attempt {}/{}", attempt, max_attempts);

        match connector.get_quote_history(symbol, start, end).await {
            Ok(response) => {
                if attempt > 1 {
                    println!("    ✅ Success on attempt {}", attempt);
                }
                return Ok(response);
            }
            Err(error) => {
                let info = error.categorize_error();
                println!("    ❌ Error ({}): {}", info.category, error);

                if attempt >= max_attempts {
                    println!("    ⛔ Max attempts reached");
                    return Err(error);
                }

                if !info.is_retryable {
                    println!("    ⛔ Error is not retryable");
                    return Err(error);
                }

                // Calculate exponential backoff
                let base_delay = info.suggested_delay_ms.unwrap_or(1000);
                let delay = base_delay * 2_u64.pow(attempt - 1);
                let delay = delay.min(10_000); // Cap at 10 seconds

                println!("    ⏰ Waiting {}ms before retry...", delay);
                sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}
