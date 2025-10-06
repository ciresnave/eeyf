# Reliability Guide

This guide covers reliability features and best practices for building resilient applications with the EEYF library.

## Table of Contents

- [Fallback Strategies](#fallback-strategies)
- [Circuit Breakers](#circuit-breakers)
- [Retry Logic](#retry-logic)
- [Timeout Configuration](#timeout-configuration)
- [Request Deduplication](#request-deduplication)
- [Connection Pooling](#connection-pooling)
- [Chaos Engineering](#chaos-engineering)
- [Monitoring & Alerting](#monitoring--alerting)

---

## Fallback Strategies

Fallback strategies ensure your application continues functioning even when the primary Yahoo Finance API is unavailable.

### Cached Data Fallback

```rust
use eeyf::fallback::{FallbackConfig, FallbackStrategy};
use std::time::Duration;

let fallback = FallbackConfig::new()
    .with_cached_fallback(true)
    .with_max_cache_age(Duration::from_secs(3600)) // 1 hour
    .build();
```

### Degraded Mode

```rust
let fallback = FallbackConfig::new()
    .with_degraded_mode(true)
    .build();

// Returns limited data with warnings when primary service fails
```

### Alternative Data Sources

```rust
use eeyf::fallback::{AlternativeSource, DataTransform};

let fallback = FallbackConfig::new()
    .add_alternative_source(
        AlternativeSource::new("Backup API", "https://backup.api.com", 1)
            .with_transform(DataTransform::AlphaVantage)
    )
    .build();
```

### Automatic Strategy Selection

```rust
use eeyf::fallback::ErrorType;

// Automatically selects the best fallback based on error type
let strategy = fallback.select_strategy(ErrorType::NetworkError);

match strategy {
    FallbackStrategy::CachedData => {
        // Use cached data
    }
    FallbackStrategy::AlternativeSource => {
        // Try alternative API
    }
    FallbackStrategy::DegradedMode => {
        // Return limited data
    }
    FallbackStrategy::Fail => {
        // No fallback available
    }
}
```

---

## Circuit Breakers

Circuit breakers prevent cascading failures by temporarily stopping requests to failing services.

### Basic Circuit Breaker

```rust
use eeyf::circuit_breaker::CircuitBreaker;
use std::time::Duration;

let circuit_breaker = CircuitBreaker::new()
    .with_failure_threshold(5)           // Open after 5 failures
    .with_timeout(Duration::from_secs(30)) // Try again after 30 seconds
    .build();
```

### States

- **Closed**: Normal operation, requests pass through
- **Open**: Service is failing, requests are blocked
- **Half-Open**: Testing if service has recovered

### Monitoring State

```rust
// Check circuit breaker state
if circuit_breaker.is_open() {
    println!("Circuit breaker is OPEN - service unavailable");
    // Use fallback strategy
}

// Get statistics
let stats = circuit_breaker.stats();
println!("Success rate: {:.1}%", stats.success_rate * 100.0);
```

### Manual Control

```rust
// Manually trip the circuit breaker
circuit_breaker.trip();

// Reset the circuit breaker
circuit_breaker.reset();
```

---

## Retry Logic

Retry logic automatically retries failed requests with exponential backoff.

### Basic Retry

```rust
use eeyf::retry::RetryPolicy;
use std::time::Duration;

let retry_policy = RetryPolicy::new()
    .with_max_retries(3)
    .with_initial_backoff(Duration::from_millis(100))
    .with_max_backoff(Duration::from_secs(10))
    .build();
```

### Exponential Backoff

```rust
let retry_policy = RetryPolicy::exponential()
    .with_multiplier(2.0)  // Double wait time each retry
    .with_jitter(true);    // Add randomness
```

### Retry Conditions

```rust
let retry_policy = RetryPolicy::new()
    .retry_on_status_codes(vec![500, 502, 503, 504])
    .retry_on_timeout(true)
    .retry_on_network_error(true);
```

---

## Timeout Configuration

### Global Timeout

```rust
use std::time::Duration;

let connector = YahooConnector::new()
    .with_timeout(Duration::from_secs(30))
    .build()?;
```

### Per-Request Timeout

```rust
let connector = YahooConnector::new()
    .with_request_timeout(Duration::from_secs(10))
    .build()?;
```

### Adaptive Timeouts

```rust
use eeyf::timeout::AdaptiveTimeout;

// Automatically adjusts timeout based on response times
let adaptive = AdaptiveTimeout::new()
    .with_percentile(0.95)  // 95th percentile
    .with_min_timeout(Duration::from_secs(5))
    .with_max_timeout(Duration::from_secs(60));
```

---

## Request Deduplication

Prevents duplicate requests for the same data within a time window.

### Basic Deduplication

```rust
use eeyf::request_deduplication::RequestDeduplicator;
use std::time::Duration;

let deduplicator = RequestDeduplicator::new()
    .with_window(Duration::from_millis(100))
    .build();
```

### Statistics

```rust
let stats = deduplicator.stats();
println!("Deduplication rate: {:.1}%", stats.dedup_rate * 100.0);
println!("Requests saved: {}", stats.requests_deduplicated);
```

---

## Connection Pooling

Efficient connection management for high-throughput applications.

### Basic Pool

```rust
use eeyf::connection_pool::ConnectionPool;

let pool = ConnectionPool::new()
    .with_max_idle(10)
    .with_max_active(100)
    .build()?;
```

### Pool Statistics

```rust
let stats = pool.stats();
println!("Active connections: {}", stats.active);
println!("Idle connections: {}", stats.idle);
println!("Pool utilization: {:.1}%", stats.utilization * 100.0);
```

### Pool Health Checks

```rust
let pool = ConnectionPool::new()
    .with_health_check_interval(Duration::from_secs(30))
    .with_connection_timeout(Duration::from_secs(5))
    .build()?;
```

---

## Chaos Engineering

Test your application's resilience with chaos engineering.

### Basic Chaos Test

```rust
use eeyf_tests::chaos::{ChaosConfig, ChaosScenario};
use std::time::Duration;

let config = ChaosConfig::new()
    .with_network_failure_rate(0.1)  // 10% failure rate
    .with_latency_ms(100, 1000)       // 100-1000ms latency
    .with_error_injection_rate(0.05)  // 5% error rate
    .with_duration(Duration::from_secs(60));

let scenario = ChaosScenario::new(config);
let report = scenario.run().await;

report.print();
```

### Interpreting Results

```
📊 Chaos Engineering Test Report
═══════════════════════════════════════
Test Duration:        60s
Total Requests:       600
Successful Requests:  540 (90.0%)

Chaos Injections:
  Network Failures:   60
  Error Responses:    30
  Latency Injections: 300 (avg 550ms)

Resilience:
  Failures Handled:   85/90
  Resilience Score:   94.4/100

✅ Excellent resilience!
```

### Custom Chaos Scenarios

```rust
// Simulate load spike
let config = ChaosConfig::new()
    .with_load_spike(5.0);  // 5x normal load

// Simulate network partition
let config = ChaosConfig::new()
    .with_network_failure_rate(1.0)  // 100% failure
    .with_duration(Duration::from_secs(30));
```

---

## Monitoring & Alerting

### Metrics

```rust
use eeyf::metrics::Metrics;

let metrics = Metrics::new();

// Track request metrics
metrics.record_request_duration(Duration::from_millis(123));
metrics.increment_error_count();
metrics.increment_cache_hits();

// Get current stats
let stats = metrics.stats();
println!("Average latency: {}ms", stats.avg_latency_ms);
println!("Error rate: {:.2}%", stats.error_rate * 100.0);
println!("Cache hit rate: {:.2}%", stats.cache_hit_rate * 100.0);
```

### Health Checks

```rust
use eeyf::health::HealthCheck;

let health = HealthCheck::new()
    .add_check("yahoo_api", check_yahoo_api)
    .add_check("cache", check_cache)
    .add_check("circuit_breaker", check_circuit_breaker);

let status = health.check_all().await;

if status.is_healthy() {
    println!("✅ All systems healthy");
} else {
    println!("❌ Health check failed: {:?}", status.failing_checks);
}
```

### Alerting

```rust
use eeyf::observability::AlertManager;

let alerts = AlertManager::new()
    .alert_on_error_rate(0.05)        // Alert if error rate > 5%
    .alert_on_latency_p95(1000)        // Alert if p95 > 1000ms
    .alert_on_circuit_breaker_open()  // Alert when circuit opens
    .with_webhook("https://alerts.example.com/webhook");

alerts.start().await;
```

---

## Best Practices

### 1. Use Circuit Breakers

```rust
let connector = YahooConnector::new()
    .with_circuit_breaker(
        CircuitBreaker::new()
            .with_failure_threshold(5)
            .with_timeout(Duration::from_secs(30))
    )
    .build()?;
```

### 2. Enable Retry Logic

```rust
let connector = YahooConnector::new()
    .with_retry_policy(
        RetryPolicy::exponential()
            .with_max_retries(3)
    )
    .build()?;
```

### 3. Configure Timeouts

```rust
let connector = YahooConnector::new()
    .with_timeout(Duration::from_secs(30))
    .build()?;
```

### 4. Use Fallback Strategies

```rust
let fallback = FallbackConfig::new()
    .with_cached_fallback(true)
    .with_degraded_mode(true)
    .build();
```

### 5. Enable Connection Pooling

```rust
let pool = ConnectionPool::new()
    .with_max_active(100)
    .build()?;
```

### 6. Monitor Metrics

```rust
// Log metrics regularly
tokio::spawn(async move {
    loop {
        let stats = metrics.stats();
        log::info!("Metrics: {:?}", stats);
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
});
```

### 7. Test with Chaos Engineering

Run chaos tests regularly in staging:

```rust
#[tokio::test]
async fn chaos_test() {
    let config = ChaosConfig::new()
        .with_duration(Duration::from_secs(300));
    
    let scenario = ChaosScenario::new(config);
    let report = scenario.run().await;
    
    assert!(report.resilience_score >= 90.0);
}
```

---

## Failure Scenarios

### Network Outage

```rust
// Automatic handling
let connector = YahooConnector::new()
    .with_fallback(
        FallbackConfig::new()
            .with_cached_fallback(true)
    )
    .build()?;

// Will use cached data during outage
```

### API Rate Limiting

```rust
// Automatic backoff and retry
let connector = YahooConnector::new()
    .with_rate_limiter(RateLimiter::new(100, Duration::from_secs(60)))
    .with_retry_policy(RetryPolicy::exponential())
    .build()?;
```

### Service Degradation

```rust
// Circuit breaker prevents cascading failures
let connector = YahooConnector::new()
    .with_circuit_breaker(CircuitBreaker::default())
    .build()?;
```

### Timeout Issues

```rust
// Adaptive timeouts adjust automatically
let connector = YahooConnector::new()
    .with_adaptive_timeout(AdaptiveTimeout::default())
    .build()?;
```

---

## Graceful Degradation

### Level 1: Normal Operation

- All features available
- Full data quality
- Low latency

### Level 2: Cached Data

- Use recent cached data
- Add "stale data" warnings
- Monitor for recovery

### Level 3: Degraded Mode

- Limited functionality
- Minimum viable data
- Clear user messaging

### Level 4: Fail Safe

- Disable non-critical features
- Return error messages
- Log for investigation

---

## Recovery Procedures

### After Circuit Breaker Opens

1. Wait for timeout period
2. Circuit enters half-open state
3. Test with limited traffic
4. Gradually increase load
5. Reset to closed state

### After Rate Limit

1. Automatic exponential backoff
2. Reduce request rate
3. Use cached data if available
4. Monitor recovery

### After Timeout

1. Retry with exponential backoff
2. Increase timeout if needed
3. Check network conditions
4. Consider fallback strategies

---

## Support

For reliability questions:

- See [documentation](../docs/)
- Check [examples](../examples/)
- Open a discussion on GitHub
- Report issues with detailed logs
