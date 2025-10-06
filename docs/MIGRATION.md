# Migration Guide

**Last Updated**: October 3, 2025  
**Version**: 0.1.0

This guide helps you migrate to EEYF from other Yahoo Finance libraries and upgrade between EEYF versions.

---

## 🔄 Migrating from Other Libraries

### From `yahoo_finance_api` Crate

EEYF is designed as a drop-in replacement with enterprise features. Here's how to migrate:

#### Basic Usage Migration

**Before (yahoo_finance_api):**
```rust
use yahoo_finance_api as yahoo;

// Basic connector
let provider = yahoo::YahooConnector::new();

// Get quotes
let response = provider.get_latest_quotes("AAPL", "1d").await.unwrap();
let quote = response.last_quote().unwrap();
```

**After (EEYF):**
```rust
use eeyf as yahoo;  // Drop-in namespace replacement

// Production-ready connector with enterprise features
let provider = yahoo::YahooConnector::new().unwrap();

// Same API, better reliability
let response = provider.get_latest_quotes("AAPL", "1d").await.unwrap();
let quote = response.last_quote().unwrap();
```

#### Key Differences

| Feature              | yahoo_finance_api       | EEYF                     | Migration Notes                        |
| -------------------- | ----------------------- | ------------------------ | -------------------------------------- |
| **Construction**     | `YahooConnector::new()` | `YahooConnector::new()?` | EEYF returns Result for validation     |
| **Rate Limiting**    | None                    | Built-in token bucket    | Automatic, no code changes needed      |
| **Retries**          | None                    | Exponential backoff      | Automatic, configurable                |
| **Circuit Breaking** | None                    | Built-in protection      | Automatic, prevents cascading failures |
| **Caching**          | None                    | LRU response cache       | Automatic, improves performance        |
| **Error Handling**   | Basic                   | Rich error categories    | Better error classification            |

#### Advanced Configuration

**Before (limited configuration):**
```rust
// Limited configuration options
let provider = yahoo::YahooConnector::new();
```

**After (rich configuration):**
```rust
// Rich configuration with builder pattern
let provider = yahoo::YahooConnector::builder()
    .rate_limit(1800.0)                    // Conservative rate limiting
    .timeout(Duration::from_secs(30))      // Custom timeout
    .cache_size(1000)                      // Response caching
    .retry_attempts(3)                     // Retry on failures
    .enable_metrics(true)                  // Observability
    .build()
    .unwrap();

// Or use presets for common configurations
let provider = yahoo::YahooConnector::from_preset("enterprise").unwrap();
```

#### Error Handling Migration

**Before (basic errors):**
```rust
match provider.get_latest_quotes("AAPL", "1d").await {
    Ok(response) => process_response(response),
    Err(e) => println!("Error: {}", e),  // Generic error handling
}
```

**After (rich error handling):**
```rust
use yahoo::YahooError;

match provider.get_latest_quotes("AAPL", "1d").await {
    Ok(response) => process_response(response),
    Err(YahooError::RateLimitExceeded(_)) => {
        // Handle rate limiting specifically
        println!("Rate limited, waiting before retry...");
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
    Err(YahooError::CircuitBreakerOpen(_)) => {
        // Handle circuit breaker specifically  
        println!("Service unavailable, backing off...");
    }
    Err(YahooError::NetworkError(e)) => {
        // Handle network issues
        println!("Network error: {}", e);
    }
    Err(e) => println!("Other error: {}", e),
}
```

### From `yfinance` (Python)

If you're migrating from Python's yfinance library:

#### Basic Ticker Data

**Before (Python yfinance):**
```python
import yfinance as yf

# Create ticker
ticker = yf.Ticker("AAPL")

# Get historical data
hist = ticker.history(period="1d")
print(hist.tail())
```

**After (EEYF Rust):**
```rust
use eeyf::YahooConnector;

// Create connector
let connector = YahooConnector::new().unwrap();

// Get latest quotes (equivalent to 1-day history)
let response = connector.get_latest_quotes("AAPL", "1d").await.unwrap();
let quotes = response.quotes().unwrap();

for quote in quotes.iter().rev().take(5) {  // Last 5 quotes
    println!("Date: {}, Close: {}", quote.timestamp, quote.close);
}
```

#### Historical Data Range

**Before (Python yfinance):**
```python
import yfinance as yf
from datetime import datetime

ticker = yf.Ticker("AAPL")
hist = ticker.history(
    start="2023-01-01", 
    end="2023-12-31",
    interval="1d"
)
```

**After (EEYF Rust):**
```rust
use eeyf::YahooConnector;
use time::macros::datetime;

let connector = YahooConnector::new().unwrap();
let start = datetime!(2023-01-01 0:00:00.00 UTC);
let end = datetime!(2023-12-31 23:59:59.99 UTC);

let response = connector.get_quote_history("AAPL", start, end).await.unwrap();
let quotes = response.quotes().unwrap();
```

### From `alphavantage` or other APIs

#### Rate Limiting Comparison

| Library               | Rate Limits               | EEYF Advantage           |
| --------------------- | ------------------------- | ------------------------ |
| **alphavantage**      | 5 calls/min (free)        | No API key required      |
| **finnhub**           | 60 calls/min (free)       | Higher free limits       |
| **yahoo_finance_api** | ~2000/hour (unofficial)   | Built-in protection      |
| **EEYF**              | ~1800/hour (conservative) | **Automatic management** |

#### API Key Migration

**Before (alphavantage):**
```rust
// Requires API key management
let api_key = env::var("ALPHA_VANTAGE_API_KEY").unwrap();
let client = alphavantage::Client::new(&api_key);
```

**After (EEYF):**
```rust
// No API key required - Yahoo Finance is public
let connector = YahooConnector::new().unwrap();
```

---

## 🔄 EEYF Version Upgrades

### From 0.1.0 to Future Versions

#### Breaking Changes Policy

EEYF follows semantic versioning:
- **Patch versions** (0.1.x): Bug fixes only, no breaking changes
- **Minor versions** (0.x.0): New features, backward compatible
- **Major versions** (x.0.0): Breaking changes, migration required

#### Deprecation Process

1. **Deprecation warning** in documentation and `#[deprecated]` attributes
2. **Migration guide** provided in changelog
3. **Minimum 2 minor versions** before removal
4. **Clear upgrade path** documented

Example future deprecation:
```rust
#[deprecated(since = "0.3.0", note = "Use `YahooConnector::builder()` instead")]
pub fn new_with_config(config: Config) -> Result<YahooConnector, YahooError> {
    // Legacy implementation
}
```

### Configuration Format Changes

If preset formats change in future versions, we'll provide migration tools:

```bash
# Hypothetical migration tool for future versions
cargo install eeyf-migrate
eeyf-migrate --from 0.1.0 --to 0.2.0 ./presets/
```

---

## 🏗️ Architecture Migration Patterns

### Single Connector vs Multiple Connectors

**Anti-pattern (creates multiple rate limiters):**
```rust
// ❌ Don't do this - creates separate rate limiters
async fn get_multiple_quotes(symbols: Vec<&str>) -> Result<Vec<QuoteResponse>, YahooError> {
    let mut results = Vec::new();
    
    for symbol in symbols {
        let connector = YahooConnector::new()?;  // New connector each time!
        let response = connector.get_latest_quotes(symbol, "1d").await?;
        results.push(response);
    }
    
    Ok(results)
}
```

**Correct pattern (shared connector):**
```rust
// ✅ Do this - shared rate limiter and connection pool
use std::sync::Arc;

async fn get_multiple_quotes(
    connector: Arc<YahooConnector>,
    symbols: Vec<&str>
) -> Result<Vec<QuoteResponse>, YahooError> {
    let mut results = Vec::new();
    
    for symbol in symbols {
        let response = connector.get_latest_quotes(symbol, "1d").await?;
        results.push(response);
    }
    
    Ok(results)
}

// Usage:
let connector = Arc::new(YahooConnector::new()?);
let quotes = get_multiple_quotes(Arc::clone(&connector), symbols).await?;
```

### Async Migration Patterns

#### Blocking to Async Migration

**Before (blocking code):**
```rust
// Synchronous code that blocks the thread
fn get_stock_price(symbol: &str) -> f64 {
    let connector = YahooConnector::new().unwrap();
    
    // This would block in synchronous context
    let response = connector.get_latest_quotes(symbol, "1d").await.unwrap();
    response.last_quote().unwrap().close
}
```

**After (proper async):**
```rust
// Proper async function
async fn get_stock_price(symbol: &str) -> Result<f64, YahooError> {
    let connector = YahooConnector::new()?;
    
    let response = connector.get_latest_quotes(symbol, "1d").await?;
    let quote = response.last_quote()
        .ok_or_else(|| YahooError::DataParsingError("No quotes available".into()))?;
    
    Ok(quote.close)
}

// Usage in async context:
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let price = get_stock_price("AAPL").await?;
    println!("AAPL price: ${:.2}", price);
    Ok(())
}
```

#### Concurrent Request Migration

**Before (sequential requests):**
```rust
// Sequential requests (slow)
async fn get_portfolio_values(symbols: Vec<&str>) -> Result<Vec<f64>, YahooError> {
    let connector = YahooConnector::new()?;
    let mut values = Vec::new();
    
    for symbol in symbols {  // Sequential, slow
        let response = connector.get_latest_quotes(symbol, "1d").await?;
        let quote = response.last_quote().unwrap();
        values.push(quote.close);
    }
    
    Ok(values)
}
```

**After (concurrent requests):**
```rust
// Concurrent requests (fast, respects rate limits)
use futures::future::try_join_all;
use std::sync::Arc;

async fn get_portfolio_values(symbols: Vec<&str>) -> Result<Vec<f64>, YahooError> {
    let connector = Arc::new(YahooConnector::new()?);
    
    // Create futures for all requests
    let futures = symbols.into_iter().map(|symbol| {
        let connector = Arc::clone(&connector);
        async move {
            let response = connector.get_latest_quotes(symbol, "1d").await?;
            let quote = response.last_quote()
                .ok_or_else(|| YahooError::DataParsingError("No quotes".into()))?;
            Ok(quote.close)
        }
    });
    
    // Execute all requests concurrently (rate limiter handles spacing)
    try_join_all(futures).await
}
```

---

## 🔧 Common Migration Issues

### Issue 1: Rate Limiting Surprises

**Problem:** Code that worked with unlimited requests now gets rate limited.

**Solution:** Use appropriate presets or configure rate limits:
```rust
// For testing/development (higher limits, shorter cache)
let connector = YahooConnector::from_preset("development")?;

// For production (conservative, reliable)
let connector = YahooConnector::from_preset("production")?;

// For high-volume applications (optimized configuration)
let connector = YahooConnector::builder()
    .rate_limit(1800.0)
    .cache_size(5000)
    .cache_duration(Duration::from_secs(300))
    .build()?;
```

### Issue 2: Error Handling Changes

**Problem:** Generic errors are now specific error types.

**Before:**
```rust
// Generic error handling
match result {
    Ok(data) => process(data),
    Err(e) => log_error(e),  // All errors treated the same
}
```

**After:**
```rust
// Specific error handling enables better recovery
match result {
    Ok(data) => process(data),
    Err(YahooError::RateLimitExceeded(_)) => {
        // Wait and retry for rate limiting
        tokio::time::sleep(Duration::from_secs(60)).await;
        retry_request().await
    }
    Err(YahooError::CircuitBreakerOpen(_)) => {
        // Circuit breaker open, back off longer
        tokio::time::sleep(Duration::from_secs(300)).await;
        retry_request().await
    }
    Err(YahooError::NetworkError(_)) => {
        // Network issue, retry with exponential backoff
        retry_with_backoff().await
    }
    Err(e) => log_error(e),  // Other errors
}
```

### Issue 3: Configuration Complexity

**Problem:** Simple use cases now require configuration understanding.

**Solution:** Use sensible defaults and presets:
```rust
// ✅ Simplest usage - just works
let connector = YahooConnector::new()?;

// ✅ Preset for common configurations
let connector = YahooConnector::from_preset("enterprise")?;

// ✅ Only configure what you need to change
let connector = YahooConnector::builder()
    .timeout(Duration::from_secs(10))  // Only change timeout
    .build()?;
```

### Issue 4: Memory Usage Increases

**Problem:** Caching and connection pooling use more memory.

**Solution:** Configure for your memory constraints:
```rust
// Low-memory configuration
let connector = YahooConnector::builder()
    .cache_size(100)           // Small cache
    .connection_pool_max(2)    // Fewer connections
    .build()?;

// Or use minimal preset (no caching)
let connector = YahooConnector::from_preset("minimal")?;
```

---

## 📊 Performance Migration

### Expected Performance Changes

| Metric                  | Before Migration  | After Migration | Notes                                      |
| ----------------------- | ----------------- | --------------- | ------------------------------------------ |
| **First Request**       | 100-500ms         | 150-600ms       | Slightly slower due to enterprise features |
| **Cached Requests**     | N/A               | 1-10ms          | Much faster with caching enabled           |
| **Error Recovery**      | Manual            | Automatic       | Built-in retry and circuit breaking        |
| **Memory Usage**        | ~1MB              | ~5-15MB         | Configurable based on cache size           |
| **Concurrent Requests** | Limited by client | Managed         | Rate limiter coordinates requests          |

### Performance Tuning After Migration

```rust
// High-performance configuration
let connector = YahooConnector::builder()
    .rate_limit(1800.0)                    // Respect Yahoo's limits
    .cache_size(10000)                     // Large cache for frequently accessed data
    .cache_duration(Duration::from_secs(300)) // 5-minute cache
    .connection_pool_max(20)               // More concurrent connections
    .timeout(Duration::from_secs(10))      // Faster timeout
    .retry_attempts(2)                     // Fewer retries for speed
    .circuit_breaker_threshold(10)         // Less aggressive circuit breaking
    .enable_metrics(true)                  // Monitor performance
    .build()?;
```

---

## ✅ Migration Checklist

### Pre-Migration
- [ ] **Backup existing code** and configuration
- [ ] **Review current usage patterns** (request frequency, error handling)
- [ ] **Identify performance requirements** (latency, throughput, memory)
- [ ] **Plan for testing** in staging environment

### During Migration
- [ ] **Update dependencies** in Cargo.toml
- [ ] **Replace import statements** (`use eeyf as yahoo;`)
- [ ] **Add error handling** for `YahooConnector::new()?`
- [ ] **Choose appropriate preset** or configuration
- [ ] **Update error handling** for specific error types
- [ ] **Test with realistic workloads**

### Post-Migration
- [ ] **Monitor performance** in production
- [ ] **Tune configuration** based on observed metrics
- [ ] **Update documentation** and team knowledge
- [ ] **Plan for future EEYF updates**

---

## 🆘 Migration Support

### Getting Help
- **GitHub Issues**: Tag with `migration` label
- **Documentation**: Check `docs/TROUBLESHOOTING.md`
- **Examples**: See `examples/` for migration patterns

### Common Support Requests
1. **Rate limiting configuration** for specific workloads
2. **Error handling patterns** for enterprise environments  
3. **Performance tuning** for high-volume applications
4. **Memory optimization** for resource-constrained environments

---

This migration guide will be updated as new migration patterns emerge and as EEYF evolves. For specific migration questions not covered here, please create a GitHub issue with the `migration` label.