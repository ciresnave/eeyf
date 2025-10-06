# EEYF - Eric Evans' Yahoo Finance API

[![CI](https://github.com/YOUR_USERNAME/EEYF/workflows/Rust%20CI/badge.svg)](https://github.com/YOUR_USERNAME/EEYF/actions)
[![codecov](https://codecov.io/gh/YOUR_USERNAME/EEYF/branch/main/graph/badge.svg)](https://codecov.io/gh/YOUR_USERNAME/EEYF)
[![Crates.io](https://img.shields.io/crates/v/eeyf.svg)](https://crates.io/crates/eeyf)
[![Documentation](https://docs.rs/eeyf/badge.svg)](https://docs.rs/eeyf)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](LICENSE-MIT)
[![MSRV](https://img.shields.io/badge/MSRV-1.70.0-blue)](https://blog.rust-lang.org/2023/06/01/Rust-1.70.0.html)

A rate-limited, reliable Rust adapter for the Yahoo Finance API designed for enterprise use.

> **⚠️ IMPORTANT**: This library implements proper rate limiting to respect Yahoo Finance's API limits and prevent your IP from being blocked. Unlike other libraries, EEYF includes built-in protection against API abuse.

## Key Features

- 🛡️ **Built-in Rate Limiting** - Prevents API abuse and IP blocking
- 🔄 **Automatic Retry Logic** - Handles transient failures gracefully  
- 📊 **Request Monitoring** - Track your API usage in real-time
- 🏢 **Enterprise Ready** - Reliable for production workloads
- 🚀 **Drop-in Replacement** - Compatible with existing `yahoo_finance_api` code

## Background and Thanks

This project started out as a fork of the [yahoo! finance API crate](<https://github.com/xemwebe/yahoo_finance_api>).  I would like to formally thank Mark Beinker and all of the many contributors to that crate as this crate wouldn't have been possible without their hard work as its base.  On that note, if any of the maintainers of that crate would like to copy any/all code from this crate back to that one, feel free!  I only split it so I could work without needing approval to push changes.  I hope you approve of the ones I make.

I would also like to thank [Yahoo!](https://yahoo.com) as without them we wouldn't have their wonderful [Yahoo! Finance](https://finance.yahoo.com) website that this crate retrieves data from.

## License

This project is licensed under Apache 2.0 or MIT license (see files LICENSE-Apache2.0 and LICENSE-MIT).

## 📚 Documentation

- **[Quick Start](#quick-start)** - Get up and running in 5 minutes
- **[ROADMAP.md](ROADMAP.md)** - Full feature roadmap and development plan
- **[GETTING_STARTED.md](GETTING_STARTED.md)** - Contributor guide for implementing roadmap items
- **[FFI Guide](docs/FFI_GUIDE.md)** - Create language bindings for Python, Node.js, Go, Ruby, etc.
- **[Architecture Change](docs/BINDINGS_ARCHITECTURE_CHANGE.md)** - Language bindings transition to separate repos
- **[BLOCKING_REMOVAL.md](BLOCKING_REMOVAL.md)** - Recent changes and migration guide

## 🌍 Language Bindings

EEYF provides a comprehensive FFI (Foreign Function Interface) layer for creating language bindings. Language bindings are maintained in **separate repositories** for better ecosystem integration:

### Creating Bindings

See **[docs/FFI_GUIDE.md](docs/FFI_GUIDE.md)** for complete instructions on creating bindings for any language. The guide includes:

- Complete FFI layer design and implementation
- Reference implementations for Python, Node.js, Go, and Ruby
- Memory management and error handling patterns
- Distribution strategies and best practices
- CI/CD pipeline setup

### Official Binding Repositories (Community-Maintained)

Community members can create and maintain language bindings. High-quality bindings that follow the FFI guide will be listed here:

- **Python** (`eeyf-python`): *Coming soon* - PyPI package with ctypes/cffi bindings
- **Node.js** (`eeyf-node`): *Coming soon* - npm package with TypeScript support
- **Go** (`eeyf-go`): *Coming soon* - Go modules with CGO bindings
- **Ruby** (`eeyf-ruby`): *Coming soon* - RubyGems with FFI gem

**Want to create a binding?** Follow the [FFI Integration Guide](docs/FFI_GUIDE.md) and open an issue to have your repository listed here.

### Why Separate Repositories?

Language bindings are maintained separately to:
- ✅ Enable proper publishing to language package managers (PyPI, npm, crates.io, etc.)
- ✅ Follow language-specific best practices and conventions
- ✅ Allow independent versioning and release cycles
- ✅ Simplify contribution (no Rust knowledge required for binding improvements)
- ✅ Provide language-specific documentation and examples

This follows the proven patterns used by major projects like SQLite, protobuf, and tree-sitter.

## ⚠️ Module Status

### Core Features (Fully Operational)
All core Yahoo Finance API functionality is **100% operational and production-ready**:
- ✅ Historical data fetching
- ✅ Real-time quote streaming  
- ✅ Options data retrieval
- ✅ Company information and financial statements
- ✅ Market hours checking
- ✅ Screener functionality
- ✅ Search capabilities (ticker, news, trending)
- ✅ Rate limiting and retry logic
- ✅ Circuit breaker pattern
- ✅ Response caching
- ✅ Data export (CSV, JSON)

### Experimental Modules (Temporarily Disabled)
Three advanced data processing modules are temporarily disabled in v0.1.0 pending refactoring:
- ⚠️ `timeseries` - Time series utilities (resampling, timezone handling)
- ⚠️ `transform` - Data transformation (OHLC aggregation, technical indicators)
- ⚠️ `validate` - Data validation (integrity checks, anomaly detection)

**Status**: These modules are under refactoring to work with both `f64` and `rust_decimal::Decimal` types.

**Workaround**: To use these features now, enable the `decimal` feature:
```toml
[dependencies]
eeyf = { version = "0.1", features = ["decimal"] }
```

**Timeline**: These modules will be re-enabled in v0.1.1 or v0.2.0 after refactoring is complete.

See [CHANGELOG.md](CHANGELOG.md) for detailed information about changes between versions.

## Quick Start

### Recommended Usage (with Rate Limiting)

For production use, always enable rate limiting to prevent API violations:

```rust
use eeyf::YahooConnector;
use tokio_test;

fn main() {
    // Create connector with default rate limiting (1800 requests/hour)
    let provider = YahooConnector::with_rate_limiting().unwrap();
    
    // Get latest quote - automatically rate limited
    let response = tokio_test::block_on(
        provider.get_latest_quotes("AAPL", "1d")
    ).unwrap();
    
    let quote = response.last_quote().unwrap();
    println!("Apple's latest price: ${}", quote.close);
}
```

### Custom Rate Limiting Configuration

```rust
use eeyf::{YahooConnector, RateLimitConfig};
use std::time::Duration;

fn main() {
    let config = RateLimitConfig {
        requests_per_hour: 1000,                    // Custom hourly limit
        burst_limit: 5,                            // Allow 5 rapid requests
        min_interval: Duration::from_millis(200),  // 200ms between requests
    };
    
    let provider = YahooConnector::with_custom_rate_limiting(config).unwrap();
    // Your requests are now automatically rate-limited
}
```

## Basic Usage

All request functions return ```async``` futures and are designed for modern async/await workflows. They need to be called from within an ```async``` function with ```.await``` or via functions like ```block_on```. All examples use the ```tokio``` runtime and tests use the ```tokio-test``` crate.

### Get the Most Recent Quote

```rust
use eeyf as yahoo;
use time::OffsetDateTime;
use tokio_test;

fn main() {
    let provider = yahoo::YahooConnector::with_rate_limiting().unwrap();
    // get the latest quotes in 1 minute intervals
    let response = tokio_test::block_on(provider.get_latest_quotes("AAPL", "1d")).unwrap();
    // extract the latest valid quote summary
    // including timestamp,open,close,high,low,volume
    let quote = response.last_quote().unwrap();
    let time: OffsetDateTime =
        OffsetDateTime::from_unix_timestamp(quote.timestamp).unwrap();
    println!("At {} quote price of Apple was {}", time, quote.close);
}
```

### Get Quotes from a Time Period

```rust
use eeyf as yahoo;
use time::{macros::datetime, OffsetDateTime};
use tokio_test;

fn main() {
    let provider = yahoo::YahooConnector::with_rate_limiting().unwrap();
    let start = datetime!(2020-1-1 0:00:00.00 UTC);
    let end = datetime!(2020-1-31 23:59:59.99 UTC);
    // returns historic quotes with daily interval
    let resp = tokio_test::block_on(provider.get_quote_history("AAPL", start, end)).unwrap();
    let quotes = resp.quotes().unwrap();
    println!("Apple's quotes in January: {:?}", quotes);
}
```

Another method to retrieve a range of quotes is by requesting the quotes for a given period and lookup frequency. Here is an example retrieving the daily quotes for the last month:

```rust
use eeyf as yahoo;
use tokio_test;

fn main() {
    let provider = yahoo::YahooConnector::with_rate_limiting().unwrap();
    let response = tokio_test::block_on(provider.get_quote_range("AAPL", "1d", "1mo")).unwrap();
    let quotes = response.quotes().unwrap();
    println!("Apple's quotes of the last month: {:?}", quotes);
}
```

*Note: See the [Time Period Labels](#time-period-labels) and [Valid Parameter Combinations](#valid-parameter-combinations) sections for what can and can't be used in the above code*

### Search Tickers for a String (e.g. company name)

```rust
use eeyf as yahoo;
use tokio_test;

fn main() {
    let provider = yahoo::YahooConnector::with_rate_limiting().unwrap();
    let resp = tokio_test::block_on(provider.search_ticker("Apple")).unwrap();

    let mut apple_found = false;
    println!("All tickers found while searching for 'Apple':");
    for item in resp.quotes
    {
        println!("{}", item.symbol)
    }
}
```

Some fields like `longname` are optional and will be replaced by default values if missing (e.g. empty string). If you do not like this behavior, use `search_ticker_opt` instead which contains `Option<String>` fields, returning `None` if the field found missing in the response.

## Rate Limiting Features

### Monitoring Rate Limits

```rust
if let Some(status) = provider.rate_limit_status() {
    println!("Used: {}/{} requests this hour", status.hourly_used, status.hourly_limit);
    println!("Remaining: {} requests", status.hourly_remaining());
    println!("Usage: {:.1}%", status.hourly_percent_used());
    
    if status.is_near_limit() {
        println!("⚠️ Warning: Approaching rate limit!");
    }
}
```

### Why Rate Limiting Matters

Yahoo Finance has API limits that unprotected libraries don't respect:

- **~2000 requests per hour per IP**
- **Burst limits of ~10-20 requests per minute**
- **Stricter limits during market hours**
- **IP blocking for violations**

Without rate limiting, you risk:

- Getting your IP temporarily or permanently blocked
- Service disruptions during critical market periods  
- Inconsistent data availability
- Potential legal issues from API abuse

### Default Rate Limit Settings

EEYF uses conservative defaults (90% of Yahoo's limits):

- **1800 requests per hour** (leaves 200 request buffer)
- **10 burst requests** (allows small batches)
- **100ms minimum interval** (prevents rapid-fire requests)

These settings are safe for production use while maximizing throughput.

### Migration from yahoo_finance_api

EEYF is designed as a drop-in replacement. Simply change your dependency:

```toml
# Before
# yahoo_finance_api = "4.1.0"

# After  
eeyf = "0.1.0"
```

Then update your imports and add rate limiting:

```rust
// Before (UNSAFE - no rate limiting)
use yahoo_finance_api as yahoo;
let provider = yahoo::YahooConnector::new().unwrap();

// After (SAFE - with protection) 
use eeyf as yahoo;  // Drop-in replacement
let provider = yahoo::YahooConnector::with_rate_limiting().unwrap();
```

## Advanced Features

### Bulk Requests with Automatic Pacing

```rust
let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA"];

for symbol in symbols {
    // Each request automatically waits for rate limit clearance
    let response = provider.get_latest_quotes(symbol, "1d").await?;
    println!("{}: ${}", symbol, response.last_quote()?.close);
}
```

### Error Handling

EEYF provides detailed error information:

```rust
match provider.get_latest_quotes("INVALID", "1d").await {
    Err(eeyf::YahooError::TooManyRequests(details)) => {
        println!("Rate limited: {}", details);
        // Maybe implement exponential backoff
    },
    Err(eeyf::YahooError::ConnectionFailed(err)) => {
        println!("Network error: {}", err);
        // Maybe retry with backoff
    },
    Err(other) => {
        println!("Other error: {}", other);
    },
    Ok(response) => {
        // Success
    }
}
```

### Best Practices

1. **Always use rate limiting** in production
2. **Monitor your usage** with `rate_limit_status()`
3. **Handle rate limit errors** gracefully
4. **Use conservative limits** for critical applications
5. **Implement exponential backoff** for retries
6. **Cache responses** when appropriate to reduce API calls

## Time Period Labels

Time periods are given as strings, combined from the number of periods (except for "ytd" and "max") and a string label specifying a single period. The following period labels are supported:

| label | description  |
| :---: | :----------: |
|   m   |    minute    |
|   h   |     hour     |
|   d   |     day      |
|  wk   |     week     |
|  mo   |    month     |
|   y   |     year     |
|  ytd  | year-to-date |
|  max  |   maximum    |

## Valid Parameter Combinations

| range |                       interval                       |
| :---: | :--------------------------------------------------: |
|  1d   | 1m, 2m, 5m, 15m, 30m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo |
|  1mo  | 2m, 3m, 5m, 15m, 30m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo |
|  3mo  |                1h, 1d, 1wk, 1mo, 3mo                 |
|  6mo  |                1h, 1d, 1wk, 1mo, 3mo                 |
|  1y   |                1h, 1d, 1wk, 1mo, 3mo                 |
|  2y   |                1h, 1d, 1wk, 1mo, 3mo                 |
|  5y   |                  1d, 1wk, 1mo, 3mo                   |
|  10y  |                  1d, 1wk, 1mo, 3mo                   |
|  ytd  | 1m, 2m, 5m, 15m, 30m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo |
|  max  | 1m, 2m, 5m, 15m, 30m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo |
