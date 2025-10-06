# Getting Started with EEYF: A Comprehensive Tutorial

Welcome to EEYF (Extensible Exchange & Yahoo Finance)! This tutorial will guide you through everything you need to know to get started, from installation to building production-ready applications.

## Table of Contents

1. [Introduction](#introduction)
2. [Installation](#installation)
3. [Your First Request](#your-first-request)
4. [Understanding the Builder Pattern](#understanding-the-builder-pattern)
5. [Working with Presets](#working-with-presets)
6. [Error Handling](#error-handling)
7. [Caching for Performance](#caching-for-performance)
8. [Rate Limiting](#rate-limiting)
9. [Real-Time Data with WebSockets](#real-time-data-with-websockets)
10. [Batch Operations](#batch-operations)
11. [Market Hours & Scheduling](#market-hours--scheduling)
12. [Advanced Features](#advanced-features)
13. [Production Best Practices](#production-best-practices)
14. [Next Steps](#next-steps)

---

## Introduction

EEYF is a high-performance, production-ready Rust library for accessing financial data from Yahoo Finance. It's designed with:

- **Reliability**: Circuit breakers, retry logic, fallback strategies
- **Performance**: Intelligent caching, HTTP/2, connection pooling
- **Observability**: Metrics, tracing, structured logging
- **Flexibility**: Multiple async runtimes, customizable configurations
- **Type Safety**: Strongly-typed responses, comprehensive error handling

### Who This Tutorial Is For

- Rust developers building financial applications
- Traders building algorithmic trading systems
- Data scientists needing market data in Rust
- Anyone interested in high-performance data fetching

### Prerequisites

- Rust 1.75 or later
- Basic understanding of async/await in Rust
- Familiarity with tokio (or async-std/smol)

---

## Installation

Add EEYF to your `Cargo.toml`:

```toml
[dependencies]
eeyf = { version = "0.1", features = ["default"] }
tokio = { version = "1", features = ["full"] }
```

### Feature Flags

EEYF uses feature flags to enable functionality:

| Feature            | Description                 | Recommended   |
| ------------------ | --------------------------- | ------------- |
| `default`          | Basic functionality         | ✅ Always      |
| `decimal`          | Decimal number support      | ✅ For finance |
| `performance-full` | All performance features    | ✅ Production  |
| `observability`    | Metrics & tracing           | ✅ Production  |
| `phase5`           | HTTP/2, compression, limits | ✅ Production  |
| `phase6`           | Developer tools (CLI, REPL) | 🔧 Development |
| `phase7`           | Security & reliability      | ✅ Production  |
| `phase8`           | Multi-runtime support       | 🔧 If needed   |
| `phase9`           | Advanced analytics          | 📊 Optional    |

**Recommended production setup**:

```toml
[dependencies]
eeyf = { version = "0.1", features = [
    "default",
    "decimal",
    "performance-full",
    "observability",
    "phase5",
    "phase7"
] }
```

---

## Your First Request

Let's fetch a stock quote:

```rust
use eeyf::{YahooFinanceClient, prelude::*};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with default settings
    let client = YahooFinanceClient::builder()
        .build()
        .await?;
    
    // Fetch a quote for Apple
    let quote = client.get_quote("AAPL").await?;
    
    println!("Apple Stock:");
    println!("  Price: ${}", quote.regular_market_price);
    println!("  Change: {:.2}%", quote.regular_market_change_percent);
    println!("  Volume: {}", quote.regular_market_volume);
    
    Ok(())
}
```

### Understanding the Response

The `Quote` struct contains comprehensive data:

```rust
pub struct Quote {
    pub symbol: String,
    pub regular_market_price: f64,
    pub regular_market_change: f64,
    pub regular_market_change_percent: f64,
    pub regular_market_volume: u64,
    pub regular_market_open: Option<f64>,
    pub regular_market_day_high: Option<f64>,
    pub regular_market_day_low: Option<f64>,
    pub regular_market_previous_close: Option<f64>,
    pub market_state: String,
    // ... and many more fields
}
```

---

## Understanding the Builder Pattern

EEYF uses the builder pattern for flexible configuration:

### Basic Builder

```rust
let client = YahooFinanceClient::builder()
    .build()
    .await?;
```

### Customized Builder

```rust
use std::time::Duration;

let client = YahooFinanceClient::builder()
    .timeout(Duration::from_secs(10))
    .user_agent("MyApp/1.0")
    .max_retries(5)
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(300)) // 5 minutes
    .build()
    .await?;
```

### All Builder Options

```rust
let client = YahooFinanceClient::builder()
    // Network settings
    .timeout(Duration::from_secs(30))
    .connect_timeout(Duration::from_secs(10))
    .user_agent("MyApp/1.0")
    .proxy("http://proxy:8080")
    
    // Retry logic
    .max_retries(3)
    .retry_delay(Duration::from_millis(500))
    .exponential_backoff(true)
    
    // Caching
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(60))
    .cache_max_entries(1000)
    
    // Rate limiting
    .enable_rate_limiting(true)
    .requests_per_second(5)
    .burst_size(10)
    
    // Connection pooling
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    
    // HTTP/2 & compression
    .http_version_pref(HttpVersion::Http2)
    .enable_compression(true)
    
    .build()
    .await?;
```

---

## Working with Presets

Presets provide pre-configured settings for common use cases:

### Available Presets

```rust
use eeyf::presets::*;

// Development: verbose logging, no caching
let dev_client = apply_development_preset(
    YahooFinanceClient::builder()
).build().await?;

// Production: optimized for reliability and performance
let prod_client = apply_production_preset(
    YahooFinanceClient::builder()
).build().await?;

// High-frequency: optimized for many requests
let hft_client = apply_high_frequency_preset(
    YahooFinanceClient::builder()
).build().await?;

// Research: longer timeouts, larger caches
let research_client = apply_research_preset(
    YahooFinanceClient::builder()
).build().await?;
```

### Custom Presets

Create your own preset function:

```rust
use eeyf::YahooFinanceClientBuilder;
use std::time::Duration;

fn my_trading_preset(builder: YahooFinanceClientBuilder) -> YahooFinanceClientBuilder {
    builder
        .timeout(Duration::from_secs(5))
        .max_retries(5)
        .enable_caching(true)
        .cache_ttl(Duration::from_secs(10)) // Fresh data
        .requests_per_second(10)
        .enable_rate_limiting(true)
}

let client = my_trading_preset(YahooFinanceClient::builder())
    .build()
    .await?;
```

---

## Error Handling

EEYF provides comprehensive error types:

### Error Types

```rust
pub enum YahooFinanceError {
    Network(String),           // Network failures
    Http(StatusCode, String),  // HTTP errors
    Parse(String),             // JSON parsing errors
    RateLimit(String),         // Rate limit exceeded
    Timeout(String),           // Request timeout
    InvalidSymbol(String),     // Invalid ticker
    ServiceUnavailable(String),// Yahoo Finance down
    Unknown(String),           // Other errors
}
```

### Handling Errors

```rust
use eeyf::{YahooFinanceClient, YahooFinanceError};

async fn fetch_with_error_handling(
    client: &YahooFinanceClient,
    symbol: &str
) -> Result<(), Box<dyn std::error::Error>> {
    match client.get_quote(symbol).await {
        Ok(quote) => {
            println!("Success: {} = ${}", symbol, quote.regular_market_price);
        }
        Err(YahooFinanceError::RateLimit(msg)) => {
            eprintln!("Rate limited: {}", msg);
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
        Err(YahooFinanceError::InvalidSymbol(msg)) => {
            eprintln!("Invalid symbol: {}", msg);
        }
        Err(YahooFinanceError::Timeout(msg)) => {
            eprintln!("Timeout: {}", msg);
            // Retry with exponential backoff
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
    Ok(())
}
```

### Using the ? Operator

```rust
async fn fetch_multiple_quotes(
    client: &YahooFinanceClient
) -> Result<Vec<Quote>, YahooFinanceError> {
    let mut quotes = Vec::new();
    
    quotes.push(client.get_quote("AAPL").await?);
    quotes.push(client.get_quote("GOOGL").await?);
    quotes.push(client.get_quote("MSFT").await?);
    
    Ok(quotes)
}
```

---

## Caching for Performance

Caching dramatically improves performance and reduces API calls:

### Enabling Cache

```rust
let client = YahooFinanceClient::builder()
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(300)) // 5 minutes
    .cache_max_entries(1000)
    .build()
    .await?;
```

### Cache Benefits

- **Speed**: 100x faster for cached responses
- **Cost**: Reduces API calls
- **Reliability**: Works during network issues
- **Rate Limits**: Avoids hitting limits

### Cache Strategies

```rust
// Short TTL for real-time data
let realtime_client = YahooFinanceClient::builder()
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(10))
    .build()
    .await?;

// Long TTL for historical data
let historical_client = YahooFinanceClient::builder()
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(3600)) // 1 hour
    .build()
    .await?;

// Persistent cache (Phase 9)
#[cfg(feature = "phase9")]
let persistent_client = YahooFinanceClient::builder()
    .enable_persistent_cache(true)
    .persistent_cache_path("./cache")
    .build()
    .await?;
```

---

## Rate Limiting

Protect your application from rate limits:

### Intelligent Rate Limiting

```rust
let client = YahooFinanceClient::builder()
    .enable_rate_limiting(true)
    .requests_per_second(5)
    .burst_size(10)
    .build()
    .await?;

// Requests are automatically throttled
for symbol in &["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"] {
    let quote = client.get_quote(symbol).await?;
    // No manual delays needed!
}
```

### Adaptive Rate Limiting (Phase 7)

```rust
#[cfg(feature = "phase7")]
let client = YahooFinanceClient::builder()
    .enable_adaptive_rate_limiting(true)
    .build()
    .await?;

// Automatically adjusts based on 429 responses
```

---

## Real-Time Data with WebSockets

Subscribe to live price updates:

```rust
use eeyf::websocket::WebSocketManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ws_manager = WebSocketManager::new().await?;
    
    // Subscribe to symbols
    ws_manager.subscribe(vec!["AAPL", "GOOGL", "MSFT"]).await?;
    
    // Receive updates
    let mut receiver = ws_manager.subscribe_to_updates();
    
    while let Some(update) = receiver.recv().await {
        println!("Update: {} = ${}", update.symbol, update.price);
    }
    
    Ok(())
}
```

### WebSocket Features

- Real-time price updates
- Automatic reconnection
- Backpressure handling
- Message batching

---

## Batch Operations

Fetch multiple quotes efficiently:

```rust
let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"];

// Sequential (slower)
for symbol in &symbols {
    let quote = client.get_quote(symbol).await?;
    println!("{}: ${}", symbol, quote.regular_market_price);
}

// Batch (faster)
let quotes = client.get_quotes_batch(&symbols).await?;
for quote in quotes {
    println!("{}: ${}", quote.symbol, quote.regular_market_price);
}

// Concurrent (fastest)
use futures::future::join_all;

let futures: Vec<_> = symbols.iter()
    .map(|s| client.get_quote(s))
    .collect();
    
let quotes = join_all(futures).await;
for result in quotes {
    if let Ok(quote) = result {
        println!("{}: ${}", quote.symbol, quote.regular_market_price);
    }
}
```

---

## Market Hours & Scheduling

Respect market hours and schedule operations:

```rust
use eeyf::market_hours::{MarketHoursManager, Exchange};
use chrono::Utc;

let market_hours = MarketHoursManager::new();

// Check if market is open
if market_hours.is_market_open(Exchange::NYSE, Utc::now()) {
    println!("NYSE is open, fetching data...");
    let quote = client.get_quote("AAPL").await?;
}

// Get next market open time
let next_open = market_hours.next_market_open(Exchange::NYSE, Utc::now());
println!("Next NYSE open: {}", next_open);

// Get trading hours
let hours = market_hours.get_trading_hours(Exchange::NYSE, Utc::now().date_naive());
println!("Trading: {} to {}", hours.open, hours.close);
```

### Scheduled Tasks

```rust
use tokio::time::{interval, Duration};

// Fetch quotes every 5 minutes during market hours
let mut ticker = interval(Duration::from_secs(300));

loop {
    ticker.tick().await;
    
    if market_hours.is_market_open(Exchange::NYSE, Utc::now()) {
        let quote = client.get_quote("AAPL").await?;
        println!("Apple: ${}", quote.regular_market_price);
    }
}
```

---

## Advanced Features

### Historical Data

```rust
use chrono::{Utc, Duration};

let end = Utc::now();
let start = end - Duration::days(30);

let history = client.get_historical_data(
    "AAPL",
    start,
    end,
    "1d" // daily
).await?;

for record in history {
    println!("{}: Open=${}, Close=${}, Volume={}",
        record.date, record.open, record.close, record.volume);
}
```

### Options Data

```rust
let options = client.get_options("AAPL").await?;

println!("Expiration dates: {:?}", options.expiration_dates);

for call in options.calls {
    println!("Call Strike ${}: Last ${}", call.strike, call.last_price);
}
```

### Stock Screener

```rust
use eeyf::screener::{ScreenerCriteria, Comparison};

let criteria = ScreenerCriteria::builder()
    .market_cap_min(1_000_000_000) // $1B min
    .pe_ratio_max(20.0)
    .volume_min(1_000_000)
    .build();

let results = client.screen_stocks(criteria).await?;
for stock in results {
    println!("{}: ${} (P/E: {})", stock.symbol, stock.price, stock.pe_ratio);
}
```

### Analytics (Phase 9)

```rust
#[cfg(feature = "phase9")]
use eeyf::analytics::Analytics;

let analytics = Analytics::new();

// Track requests
analytics.record_request("AAPL", Duration::from_millis(150)).await;

// Get insights
let insights = analytics.get_insights().await;
println!("Average latency: {:?}", insights.average_latency);
println!("P95 latency: {:?}", insights.p95_latency);
println!("Cache hit rate: {:.2}%", insights.cache_hit_rate * 100.0);

// Detect anomalies
if let Some(anomalies) = analytics.detect_anomalies().await {
    for anomaly in anomalies {
        println!("Anomaly: {:?} (severity: {})", anomaly.anomaly_type, anomaly.severity);
    }
}
```

---

## Production Best Practices

### 1. Use Presets

```rust
use eeyf::presets::apply_production_preset;

let client = apply_production_preset(YahooFinanceClient::builder())
    .build()
    .await?;
```

### 2. Enable All Safety Features

```rust
let client = YahooFinanceClient::builder()
    // Reliability
    .max_retries(5)
    .exponential_backoff(true)
    .enable_circuit_breaker(true)
    
    // Performance
    .enable_caching(true)
    .enable_rate_limiting(true)
    .enable_compression(true)
    
    // Observability
    .enable_metrics(true)
    .enable_tracing(true)
    
    .build()
    .await?;
```

### 3. Handle All Errors

```rust
match client.get_quote("AAPL").await {
    Ok(quote) => { /* handle success */ }
    Err(e) => {
        error!("Failed to fetch quote: {}", e);
        // Log to monitoring system
        // Trigger alert if critical
        // Use fallback data if available
    }
}
```

### 4. Monitor Performance

```rust
#[cfg(feature = "observability")]
{
    let metrics = client.get_metrics().await;
    println!("Total requests: {}", metrics.total_requests);
    println!("Cache hit rate: {:.2}%", metrics.cache_hit_rate * 100.0);
    println!("Average latency: {:?}", metrics.average_latency);
}
```

### 5. Graceful Shutdown

```rust
use tokio::signal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = YahooFinanceClient::builder().build().await?;
    
    // Spawn worker tasks
    let handle = tokio::spawn(async move {
        // Your application logic
    });
    
    // Wait for shutdown signal
    signal::ctrl_c().await?;
    
    // Graceful shutdown
    handle.abort();
    client.shutdown().await?;
    
    Ok(())
}
```

### 6. Resource Management

```rust
// Use Arc for sharing client across tasks
use std::sync::Arc;

let client = Arc::new(
    YahooFinanceClient::builder().build().await?
);

let handles: Vec<_> = (0..10).map(|i| {
    let client = Arc::clone(&client);
    tokio::spawn(async move {
        let quote = client.get_quote("AAPL").await.unwrap();
        println!("Task {}: ${}", i, quote.regular_market_price);
    })
}).collect();

for handle in handles {
    handle.await?;
}
```

---

## Next Steps

### Learn More

- **[API Documentation](https://docs.rs/eeyf)**: Complete API reference
- **[Examples](../examples/)**: Working code examples
- **[Architecture Guide](ARCHITECTURE.md)**: Deep dive into internals
- **[Performance Guide](PERFORMANCE.md)**: Optimization techniques
- **[Analytics Guide](ANALYTICS.md)**: Advanced analytics features

### Explore Features

- **Reliability**: Circuit breakers, fallback strategies
- **Security**: API key rotation, secrets management
- **Observability**: Prometheus metrics, Jaeger tracing
- **Developer Tools**: CLI tool, REPL interface

### Join the Community

- **GitHub**: Report issues, request features
- **Discord**: Get help, share projects
- **Blog**: Technical deep dives, tutorials

### Build Something Amazing

Now that you know the basics, build:
- Real-time trading dashboard
- Portfolio tracker
- Market analysis tool
- Algorithmic trading bot
- Research platform

---

## Troubleshooting

### Common Issues

**Rate Limited?**
```rust
// Increase delays, enable caching
let client = YahooFinanceClient::builder()
    .requests_per_second(2)
    .enable_caching(true)
    .cache_ttl(Duration::from_secs(60))
    .build()
    .await?;
```

**Timeout Errors?**
```rust
// Increase timeout, enable retries
let client = YahooFinanceClient::builder()
    .timeout(Duration::from_secs(30))
    .max_retries(5)
    .build()
    .await?;
```

**Parse Errors?**
```rust
// Enable debug logging
env_logger::init();
// Check symbol validity
// Update to latest version
```

### Getting Help

- Check [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- Search existing GitHub issues
- Ask in Discord channel
- Open a new issue with:
  - EEYF version
  - Rust version
  - Code example
  - Error message
  - Expected behavior

---

**Happy coding! 🚀**
