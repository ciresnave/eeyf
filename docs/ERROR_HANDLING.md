# Error Handling Guide

This guide provides comprehensive information about error handling in EEYF, including error types, recovery strategies, and best practices.

## Table of Contents

- [Overview](#overview)
- [Error Types](#error-types)
- [Error Codes](#error-codes)
- [Error Context](#error-context)
- [Error Categorization](#error-categorization)
- [Retry Strategies](#retry-strategies)
- [Best Practices](#best-practices)
- [Examples](#examples)
- [Troubleshooting](#troubleshooting)

## Overview

EEYF provides rich error handling capabilities to help you build robust applications. All errors are structured to provide:

- **Clear error messages** - Human-readable descriptions
- **Error codes** - Programmatic error identification
- **Context information** - Symbol, endpoint, timestamp, request ID
- **Retryability detection** - Automatic detection of transient errors
- **Suggested actions** - User-friendly guidance on resolution

## Error Types

### YahooError

The main error type returned by all EEYF operations:

```rust
use eeyf::{YahooError, YahooErrorCode};

fn handle_error(error: YahooError) {
    // Get error code for programmatic handling
    let code = error.error_code();
    
    // Check if error is retryable
    if error.is_retryable() {
        println!("This error can be retried");
    }
    
    // Get suggested action
    println!("Suggestion: {}", error.suggested_action());
}
```

### Common Error Variants

| Variant             | Description              | Retryable | Typical Cause                        |
| ------------------- | ------------------------ | --------- | ------------------------------------ |
| `TooManyRequests`   | Rate limit exceeded      | ✅ Yes     | Too many requests in short period    |
| `ConnectionFailed`  | Network connection error | ✅ Yes     | Network issues, timeout              |
| `Unauthorized`      | Authentication failed    | ❌ No      | Invalid/expired session              |
| `NoResult`          | No data available        | ❌ No      | Invalid symbol or no data for period |
| `DeserializeFailed` | JSON parsing failed      | ✅ Yes     | API change or data corruption        |
| `InvalidUrl`        | Malformed URL            | ❌ No      | Invalid parameters                   |
| `InvalidDateFormat` | Bad date format          | ❌ No      | Incorrect date format                |
| `DataInconsistency` | Inconsistent data        | ✅ Yes     | Temporary Yahoo issue                |
| `ApiError`          | Yahoo API error          | ❌ No      | API-level rejection                  |

## Error Codes

Error codes enable programmatic error handling without string matching:

```rust
use eeyf::{YahooError, YahooErrorCode};

match error.error_code() {
    YahooErrorCode::RateLimit => {
        // Handle rate limiting
        tokio::time::sleep(Duration::from_secs(60)).await;
        retry_request().await?;
    }
    YahooErrorCode::Unauthorized => {
        // Recreate client with fresh credentials
        let connector = YahooConnector::new();
        retry_with_new_client(connector).await?;
    }
    YahooErrorCode::NoResult => {
        // Symbol doesn't exist or no data
        log::warn!("No data available for symbol");
        return Ok(None);
    }
    _ => {
        // Handle other errors
        log::error!("Unhandled error: {}", error);
    }
}
```

### Available Error Codes

- `FETCH_FAILED` - Data fetch operation failed
- `DESERIALIZE_FAILED` - JSON deserialization failed
- `CONNECTION_FAILED` - Network connection error
- `API_ERROR` - Yahoo API returned error
- `NO_RESULT` - No data available
- `DATA_INCONSISTENCY` - Inconsistent data detected
- `BUILDER_FAILED` - Client builder error
- `NO_COOKIES` - Missing cookies
- `INVALID_COOKIE` - Invalid cookie format
- `UNAUTHORIZED` - Authentication failed
- `INVALID_CRUMB` - Invalid crumb token
- `RATE_LIMIT` - Rate limit exceeded
- `INVALID_URL` - Malformed URL
- `INVALID_DATE_FORMAT` - Bad date format
- `MISSING_FIELD` - Required field missing
- `INVALID_STATUS_CODE` - Unexpected HTTP status

## Error Context

Add rich context to errors for better debugging and observability:

```rust
use eeyf::{ErrorContext, YahooError};

// Create error context
let context = ErrorContext::new()
    .with_symbol("AAPL")
    .with_endpoint("/v8/finance/chart")
    .with_request_id("req-12345")
    .with_metadata("user_id", "user-789")
    .with_metadata("region", "us-east-1");

// Attach context to error
let error_with_context = error.with_context(context);

// Display includes all context
println!("{}", error_with_context);
// Output: "connection to yahoo! finance server failed: timeout [symbol: AAPL] 
//          [endpoint: /v8/finance/chart] [request_id: req-12345] [occurred: 2s ago]"
```

### ErrorContext Fields

- **symbol** - The stock symbol being requested
- **endpoint** - The API endpoint being called
- **timestamp** - When the error occurred (automatically set)
- **request_id** - Unique request identifier for tracing
- **metadata** - Additional key-value context data

## Error Categorization

EEYF automatically categorizes errors for intelligent handling:

```rust
use eeyf::{ErrorCategorizer, ErrorCategory};

let error_info = error.categorize_error();

match error_info.category {
    ErrorCategory::Transient => {
        // Temporary issue - retry immediately
        println!("Transient error, retrying...");
    }
    ErrorCategory::RateLimit => {
        // Rate limited - use exponential backoff
        let delay = error_info.suggested_delay_ms.unwrap_or(5000);
        tokio::time::sleep(Duration::from_millis(delay)).await;
    }
    ErrorCategory::Authentication => {
        // Auth failed - recreate client
        println!("Authentication failed, refreshing credentials");
    }
    ErrorCategory::Configuration => {
        // Bad input - fix configuration
        println!("Configuration error: {}", error);
        return Err(error);
    }
    _ => {
        // Other categories
        println!("Error category: {}", error_info.category);
    }
}
```

### Error Categories

| Category           | Description                      | Retry? | Base Delay | Max Retries |
| ------------------ | -------------------------------- | ------ | ---------- | ----------- |
| **Transient**      | Temporary network/service issues | ✅ Yes  | 1s         | 3           |
| **RateLimit**      | Rate limit exceeded              | ✅ Yes  | 5s         | 5           |
| **ServerError**    | 5xx server errors                | ✅ Yes  | 2s         | 2           |
| **Authentication** | Auth/authorization failures      | ❌ No   | -          | 0           |
| **ClientError**    | 4xx client errors                | ❌ No   | -          | 0           |
| **Configuration**  | Invalid configuration/input      | ❌ No   | -          | 0           |
| **Permanent**      | Permanent failures               | ❌ No   | -          | 0           |
| **Unknown**        | Unclassified errors              | ❌ No   | -          | 0           |

## Retry Strategies

### Simple Retry with is_retryable()

```rust
async fn fetch_with_retry(
    connector: &YahooConnector,
    symbol: &str,
) -> Result<YResponse, YahooError> {
    let max_retries = 3;
    let mut retries = 0;
    
    loop {
        match connector.get_quote_history(symbol).await {
            Ok(response) => return Ok(response),
            Err(error) if error.is_retryable() && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_secs(2_u64.pow(retries)); // Exponential backoff
                log::warn!("Retrying after error (attempt {}/{}): {}", 
                          retries, max_retries, error);
                tokio::time::sleep(delay).await;
            }
            Err(error) => return Err(error),
        }
    }
}
```

### Category-Based Retry Strategy

```rust
use eeyf::{ErrorCategorizer, ErrorCategory};

async fn intelligent_retry(
    connector: &YahooConnector,
    symbol: &str,
) -> Result<YResponse, YahooError> {
    let mut retries = 0;
    
    loop {
        match connector.get_quote_history(symbol).await {
            Ok(response) => return Ok(response),
            Err(error) => {
                let info = error.categorize_error();
                
                if !info.is_retryable {
                    log::error!("Non-retryable error: {}", error);
                    return Err(error);
                }
                
                let max_retries = info.category.max_retries();
                if retries >= max_retries {
                    log::error!("Max retries exceeded for {}: {}", symbol, error);
                    return Err(error);
                }
                
                retries += 1;
                let base_delay = info.suggested_delay_ms.unwrap_or(1000);
                let delay = base_delay * 2_u64.pow(retries - 1); // Exponential backoff
                
                log::warn!(
                    "Retrying {} after {} (attempt {}/{}, waiting {}ms)", 
                    symbol, info.category, retries, max_retries, delay
                );
                
                tokio::time::sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}
```

### Rate Limit Aware Retry

```rust
async fn rate_limit_aware_fetch(
    connector: &YahooConnector,
    symbol: &str,
) -> Result<YResponse, YahooError> {
    const MAX_RETRIES: u32 = 5;
    let mut retries = 0;
    
    loop {
        match connector.get_quote_history(symbol).await {
            Ok(response) => return Ok(response),
            Err(error) => {
                match error.error_code() {
                    YahooErrorCode::RateLimit => {
                        if retries >= MAX_RETRIES {
                            return Err(error);
                        }
                        retries += 1;
                        
                        // Exponential backoff starting at 60 seconds for rate limits
                        let delay = Duration::from_secs(60 * 2_u64.pow(retries - 1));
                        log::warn!(
                            "Rate limited, waiting {:?} before retry {}/{}",
                            delay, retries, MAX_RETRIES
                        );
                        tokio::time::sleep(delay).await;
                    }
                    _ if error.is_retryable() && retries < 3 => {
                        retries += 1;
                        let delay = Duration::from_secs(2);
                        log::warn!("Transient error, retrying: {}", error);
                        tokio::time::sleep(delay).await;
                    }
                    _ => return Err(error),
                }
            }
        }
    }
}
```

## Best Practices

### 1. Always Check is_retryable()

```rust
if error.is_retryable() {
    // Safe to retry
} else {
    // Don't retry - log and handle appropriately
    log::error!("Permanent error: {}", error);
}
```

### 2. Use Suggested Actions

```rust
match result {
    Err(error) => {
        log::error!("Error: {}", error);
        log::info!("Suggestion: {}", error.suggested_action());
        
        // Show suggestion to user in production
        eprintln!("💡 {}", error.suggested_action());
    }
    Ok(data) => { /* ... */ }
}
```

### 3. Add Context for Debugging

```rust
let context = ErrorContext::new()
    .with_symbol(symbol)
    .with_endpoint("/v8/finance/chart")
    .with_request_id(&request_id);

match fetch_data(symbol).await {
    Err(error) => {
        let error_with_context = error.with_context(context);
        log::error!("{}", error_with_context);
        return Err(error);
    }
    Ok(data) => { /* ... */ }
}
```

### 4. Implement Circuit Breaking

```rust
use std::sync::atomic::{AtomicU32, Ordering};

struct CircuitBreaker {
    failures: AtomicU32,
    threshold: u32,
}

impl CircuitBreaker {
    fn new(threshold: u32) -> Self {
        Self {
            failures: AtomicU32::new(0),
            threshold,
        }
    }
    
    fn record_success(&self) {
        self.failures.store(0, Ordering::Relaxed);
    }
    
    fn record_failure(&self) -> bool {
        let failures = self.failures.fetch_add(1, Ordering::Relaxed) + 1;
        failures >= self.threshold
    }
    
    fn is_open(&self) -> bool {
        self.failures.load(Ordering::Relaxed) >= self.threshold
    }
}

async fn fetch_with_circuit_breaker(
    connector: &YahooConnector,
    breaker: &CircuitBreaker,
    symbol: &str,
) -> Result<YResponse, YahooError> {
    if breaker.is_open() {
        return Err(YahooError::FetchFailed(
            "Circuit breaker open".to_string()
        ));
    }
    
    match connector.get_quote_history(symbol).await {
        Ok(response) => {
            breaker.record_success();
            Ok(response)
        }
        Err(error) => {
            if error.is_retryable() {
                if breaker.record_failure() {
                    log::error!("Circuit breaker opened after repeated failures");
                }
            }
            Err(error)
        }
    }
}
```

### 5. Log with Error Codes

```rust
match result {
    Err(error) => {
        log::error!(
            target: "eeyf::errors",
            error_code = %error.error_code(),
            error_message = %error,
            "Request failed"
        );
    }
    Ok(_) => { /* ... */ }
}
```

### 6. Group Errors by Category

```rust
use std::collections::HashMap;

struct ErrorMetrics {
    counts: HashMap<String, u64>,
}

impl ErrorMetrics {
    fn record_error(&mut self, error: &YahooError) {
        let info = error.categorize_error();
        let counter = self.counts.entry(info.category.to_string()).or_insert(0);
        *counter += 1;
    }
    
    fn report(&self) {
        for (category, count) in &self.counts {
            log::info!("Error category {}: {} occurrences", category, count);
        }
    }
}
```

## Examples

### Example 1: Basic Error Handling

```rust
use eeyf::{YahooConnector, YahooError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connector = YahooConnector::new();
    
    match connector.get_latest_quotes("AAPL", "1d").await {
        Ok(quotes) => {
            println!("Successfully fetched {} quotes", quotes.quotes.len());
        }
        Err(error) => {
            eprintln!("❌ Error: {}", error);
            eprintln!("💡 Suggestion: {}", error.suggested_action());
            
            if error.is_retryable() {
                eprintln!("🔄 This error is retryable");
            }
        }
    }
    
    Ok(())
}
```

### Example 2: Robust Retry Logic

```rust
use eeyf::{YahooConnector, YahooError, ErrorCategorizer};
use tokio::time::{sleep, Duration};

async fn fetch_with_robust_retry(
    connector: &YahooConnector,
    symbol: &str,
) -> Result<eeyf::YResponse, YahooError> {
    let mut attempt = 0;
    const MAX_ATTEMPTS: u32 = 5;
    
    loop {
        attempt += 1;
        
        match connector.get_quote_history(symbol).await {
            Ok(response) => {
                if attempt > 1 {
                    log::info!("Succeeded on attempt {}", attempt);
                }
                return Ok(response);
            }
            Err(error) => {
                let info = error.categorize_error();
                
                log::warn!(
                    "Attempt {}/{} failed: {} (category: {})",
                    attempt, MAX_ATTEMPTS, error, info.category
                );
                
                if attempt >= MAX_ATTEMPTS {
                    log::error!("All retry attempts exhausted");
                    return Err(error);
                }
                
                if !info.is_retryable {
                    log::error!("Non-retryable error: {}", error);
                    eprintln!("💡 {}", error.suggested_action());
                    return Err(error);
                }
                
                // Calculate delay with exponential backoff
                let base_delay = info.suggested_delay_ms.unwrap_or(1000);
                let delay = base_delay * 2_u64.pow(attempt - 1);
                let delay = delay.min(60_000); // Cap at 60 seconds
                
                log::info!("Waiting {}ms before retry...", delay);
                sleep(Duration::from_millis(delay)).await;
            }
        }
    }
}
```

### Example 3: Error Recovery with Context

```rust
use eeyf::{YahooConnector, YahooError, ErrorContext};
use uuid::Uuid;

async fn fetch_with_context(
    connector: &YahooConnector,
    symbol: &str,
    user_id: &str,
) -> Result<eeyf::YResponse, YahooError> {
    let request_id = Uuid::new_v4().to_string();
    
    let context = ErrorContext::new()
        .with_symbol(symbol)
        .with_endpoint("/v8/finance/chart")
        .with_request_id(&request_id)
        .with_metadata("user_id", user_id)
        .with_metadata("version", env!("CARGO_PKG_VERSION"));
    
    match connector.get_quote_history(symbol).await {
        Ok(response) => Ok(response),
        Err(error) => {
            let error_with_context = error.with_context(context);
            
            // Log with full context
            log::error!("{}", error_with_context);
            
            // Send to error tracking service
            // error_tracker::report(error_with_context);
            
            Err(error_with_context.error)
        }
    }
}
```

### Example 4: Fallback Strategy

```rust
use eeyf::{YahooConnector, YahooError, YahooErrorCode};

async fn fetch_with_fallback(
    primary: &YahooConnector,
    symbol: &str,
) -> Result<Vec<eeyf::Quote>, YahooError> {
    match primary.get_latest_quotes(symbol, "1d").await {
        Ok(response) => Ok(response.quotes),
        Err(error) => {
            match error.error_code() {
                YahooErrorCode::RateLimit => {
                    log::warn!("Rate limited, using cached data");
                    // Return cached data if available
                    if let Some(cached) = check_cache(symbol) {
                        return Ok(cached);
                    }
                }
                YahooErrorCode::NoResult => {
                    log::warn!("No data for symbol {}", symbol);
                    return Ok(Vec::new());
                }
                _ => {
                    log::error!("Error fetching {}: {}", symbol, error);
                }
            }
            Err(error)
        }
    }
}

fn check_cache(symbol: &str) -> Option<Vec<eeyf::Quote>> {
    // Implement cache lookup
    None
}
```

## Troubleshooting

### Rate Limiting Errors

**Error**: `TooManyRequests` or `RATE_LIMIT`

**Solutions**:
1. Use the built-in rate limiter:
   ```rust
   let connector = YahooConnector::builder()
       .rate_limit(30, 60) // 30 requests per 60 seconds
       .build();
   ```
2. Implement exponential backoff
3. Reduce request frequency
4. Use caching to avoid repeated requests

### Connection Failures

**Error**: `ConnectionFailed` or `CONNECTION_FAILED`

**Solutions**:
1. Check internet connectivity
2. Verify firewall/proxy settings
3. Implement retry logic with backoff
4. Check if Yahoo Finance is accessible in your region

### Authentication Errors

**Error**: `Unauthorized`, `InvalidCrumb`, `InvalidCookie`

**Solutions**:
1. Recreate the `YahooConnector` client:
   ```rust
   let connector = YahooConnector::new();
   ```
2. Check for VPN/proxy interference
3. Ensure you're not reusing stale connectors

### No Data Available

**Error**: `NoResult`, `NoQuotes`, `NO_RESULT`

**Solutions**:
1. Verify the symbol is correct
2. Check if the symbol is delisted or suspended
3. Ensure the date range is valid
4. Try a different time period

### Deserialization Failures

**Error**: `DeserializeFailed`, `DESERIALIZE_FAILED`

**Solutions**:
1. Update to the latest version of EEYF
2. Report the issue with the response body
3. Check if Yahoo changed their API format
4. Implement fallback to cached data

## See Also

- [TROUBLESHOOTING.md](./TROUBLESHOOTING.md) - Comprehensive troubleshooting guide
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture and error flow
- [Examples](../examples/) - Working code examples
- [API Documentation](https://docs.rs/eeyf) - Full API reference
