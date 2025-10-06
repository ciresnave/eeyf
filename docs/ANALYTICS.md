# Advanced Analytics Guide

This guide covers EEYF's advanced analytics capabilities for comprehensive performance monitoring, predictive analytics, anomaly detection, and usage optimization.

## Table of Contents

- [Overview](#overview)
- [Getting Started](#getting-started)
- [Request Profiling](#request-profiling)
- [Performance Insights](#performance-insights)
- [Anomaly Detection](#anomaly-detection)
- [Predictive Analytics](#predictive-analytics)
- [Usage Analytics](#usage-analytics)
- [Best Practices](#best-practices)
- [Performance Impact](#performance-impact)
- [Integration Examples](#integration-examples)

---

## Overview

The analytics module provides four core capabilities:

1. **Request Profiling**: Detailed timing breakdowns for each request
2. **Predictive Analytics**: Forecast potential issues before they occur
3. **Anomaly Detection**: Automatically identify unusual patterns
4. **Usage Analytics**: Track and optimize API usage patterns

### Key Features

- **Zero-Cost Abstraction**: Minimal performance overhead
- **Async-First Design**: Non-blocking operations
- **Flexible Configuration**: Customize retention, thresholds, and features
- **Production-Ready**: Designed for high-throughput environments
- **Actionable Insights**: Provides specific recommendations

---

## Getting Started

### Basic Setup

```rust
use eeyf::analytics::{Analytics, AnalyticsConfig};
use std::time::Duration;

// Create analytics with default configuration
let analytics = Analytics::new(AnalyticsConfig::default());

// Record a simple request
analytics.record_request("AAPL", Duration::from_millis(150)).await;

// Get performance insights
let insights = analytics.get_insights().await;
println!("Average latency: {:?}", insights.average_latency);
```

### Custom Configuration

```rust
let config = AnalyticsConfig::builder()
    .enable_profiling(true)
    .enable_predictions(true)
    .enable_anomaly_detection(true)
    .enable_usage_analytics(true)
    .retention_period(Duration::from_secs(3600)) // 1 hour
    .max_data_points(10000)
    .anomaly_threshold(3.0) // 3 standard deviations
    .prediction_window(Duration::from_secs(300)) // 5 minutes
    .build();

let analytics = Analytics::new(config);
```

### Configuration Options

| Option                     | Default   | Description                     |
| -------------------------- | --------- | ------------------------------- |
| `enable_profiling`         | `true`    | Enable request profiling        |
| `enable_predictions`       | `true`    | Enable predictive analytics     |
| `enable_anomaly_detection` | `true`    | Enable anomaly detection        |
| `enable_usage_analytics`   | `true`    | Enable usage tracking           |
| `retention_period`         | 1 hour    | How long to keep data           |
| `max_data_points`          | 10,000    | Maximum data points to store    |
| `anomaly_threshold`        | 3.0       | Z-score threshold for anomalies |
| `prediction_window`        | 5 minutes | Prediction time window          |

---

## Request Profiling

Request profiling provides detailed timing breakdowns for each stage of a request.

### Simple Profiling

```rust
use std::time::Instant;

let start = Instant::now();
let result = client.get_quotes(&["AAPL"]).await?;
let duration = start.elapsed();

analytics.record_request("AAPL", duration).await;
```

### Detailed Profiling

For more granular insights, record detailed profiles:

```rust
use eeyf::analytics::RequestProfile;
use std::time::{Duration, SystemTime};

let profile = RequestProfile {
    symbol: "AAPL".to_string(),
    total_duration: Duration::from_millis(250),
    cache_lookup_duration: Some(Duration::from_millis(5)),
    rate_limit_duration: Some(Duration::from_millis(10)),
    network_duration: Some(Duration::from_millis(200)),
    parse_duration: Some(Duration::from_millis(35)),
    cache_hit: true,
    rate_limited: false,
    timestamp: SystemTime::now(),
};

analytics.record_profile(profile).await;
```

### Profile Breakdown

Each profile captures:

- **Total Duration**: End-to-end request time
- **Cache Lookup**: Time spent checking cache
- **Rate Limiting**: Time spent in rate limiter
- **Network Duration**: Actual network request time
- **Parse Duration**: Response parsing time
- **Cache Hit**: Whether request hit cache
- **Rate Limited**: Whether request was rate limited

---

## Performance Insights

Performance insights provide statistical analysis of your request data.

### Getting Insights

```rust
let insights = analytics.get_insights().await;

println!("Performance Metrics:");
println!("  Total requests: {}", insights.total_requests);
println!("  Requests/sec: {:.2}", insights.requests_per_second);
println!("  Average latency: {:?}", insights.average_latency);
println!("  P50 latency: {:?}", insights.p50_latency);
println!("  P95 latency: {:?}", insights.p95_latency);
println!("  P99 latency: {:?}", insights.p99_latency);
println!("  Cache hit rate: {:.1}%", insights.cache_hit_rate * 100.0);
println!("  Rate limit rate: {:.1}%", insights.rate_limit_rate * 100.0);
```

### Available Metrics

| Metric                 | Description                           |
| ---------------------- | ------------------------------------- |
| `total_requests`       | Total number of requests analyzed     |
| `requests_per_second`  | Current request rate                  |
| `average_latency`      | Mean request latency                  |
| `p50_latency`          | Median latency (50th percentile)      |
| `p95_latency`          | 95th percentile latency               |
| `p99_latency`          | 99th percentile latency               |
| `cache_hit_rate`       | Percentage of cache hits (0.0 to 1.0) |
| `rate_limit_rate`      | Percentage of rate-limited requests   |
| `average_network_time` | Average network request time          |
| `average_parse_time`   | Average parsing time                  |

### Interpreting Insights

**High Cache Hit Rate** (>70%):
- Good: Cache is working effectively
- Consider: Increasing cache TTL for even better hit rates

**Low Cache Hit Rate** (<30%):
- Problem: Cache not being used effectively
- Solutions: Increase cache size, adjust TTL, check cache warming

**High Rate Limit Rate** (>10%):
- Problem: Hitting rate limits frequently
- Solutions: Implement request batching, increase rate limits, add backoff

**High P95-P99 Gap**:
- Problem: Inconsistent latency
- Solutions: Investigate network issues, check for outliers

---

## Anomaly Detection

Anomaly detection automatically identifies unusual patterns in your data.

### Detecting Anomalies

```rust
if let Some(anomalies) = analytics.detect_anomalies().await {
    println!("Detected {} anomalies", anomalies.len());
    
    for anomaly in anomalies {
        println!("Type: {:?}", anomaly.anomaly_type);
        println!("Severity: {:.2}", anomaly.severity);
        println!("Description: {}", anomaly.description);
        
        if let Some(mitigation) = anomaly.mitigation {
            println!("Mitigation: {}", mitigation);
        }
    }
}
```

### Anomaly Types

**High Latency**
- Trigger: Response time exceeds mean by 3+ standard deviations
- Severity: Based on how many σ above the mean
- Mitigation: Investigate network, check API status, increase timeouts

**Low Cache Hit Rate**
- Trigger: Recent cache hit rate drops below 50% of historical average
- Severity: Proportional to the drop
- Mitigation: Check cache configuration, verify cache warming

**High Rate Limiting**
- Trigger: More than 20% of recent requests are rate-limited
- Severity: Proportional to rate limiting percentage
- Mitigation: Implement request spacing, add batching, upgrade tier

**High Error Rate**
- Trigger: Error rate exceeds 5%
- Severity: Proportional to error percentage
- Mitigation: Review logs, check API status, implement retries

**Unusual Pattern**
- Trigger: Request patterns deviate from historical norms
- Severity: Based on deviation magnitude
- Mitigation: Review recent changes, check for anomalous traffic

**Traffic Spike**
- Trigger: Sudden increase in request rate
- Severity: Proportional to increase magnitude
- Mitigation: Scale infrastructure, check for DDoS, verify legitimate traffic

### Anomaly Response

```rust
use eeyf::analytics::AnomalyType;

if let Some(anomalies) = analytics.detect_anomalies().await {
    for anomaly in anomalies {
        match anomaly.anomaly_type {
            AnomalyType::HighLatency => {
                // Alert operations team
                alert_ops("High latency detected", &anomaly.description);
            }
            AnomalyType::LowCacheHitRate => {
                // Check cache configuration
                verify_cache_config().await;
            }
            AnomalyType::HighRateLimiting => {
                // Slow down requests
                adjust_rate_limit().await;
            }
            AnomalyType::HighErrorRate => {
                // Review error logs
                investigate_errors().await;
            }
            _ => {
                // Log for review
                log::warn!("Anomaly detected: {:?}", anomaly);
            }
        }
    }
}
```

### Configuring Detection

Adjust anomaly detection sensitivity:

```rust
let config = AnalyticsConfig::builder()
    .anomaly_threshold(2.5) // More sensitive (2.5σ vs default 3.0σ)
    .build();
```

Lower threshold = More sensitive = More false positives
Higher threshold = Less sensitive = May miss anomalies

Recommended: 2.5-3.5 for most applications

---

## Predictive Analytics

Predictive analytics forecasts potential issues before they occur.

### Getting Predictions

```rust
let predictions = analytics.predict_issues().await;

// Check for rate limit exhaustion
if let Some(exhaustion_time) = predictions.rate_limit_exhaustion {
    println!("⚠ Rate limit may be exhausted in: {:?}", exhaustion_time);
    
    // Take preventive action
    if exhaustion_time < Duration::from_secs(60) {
        // Critical: Exhaustion within 1 minute
        emergency_throttle().await;
    } else if exhaustion_time < Duration::from_secs(300) {
        // Warning: Exhaustion within 5 minutes
        gradual_throttle().await;
    }
}

// Review configuration suggestions
for suggestion in predictions.config_suggestions {
    println!("Suggestion: Change {} from {} to {}",
             suggestion.setting,
             suggestion.current_value,
             suggestion.suggested_value);
    println!("Reason: {}", suggestion.reason);
    println!("Expected impact: {}", suggestion.expected_impact);
}

// Capacity recommendations
for recommendation in predictions.capacity_recommendations {
    println!("Recommendation: {}", recommendation);
}
```

### Prediction Types

**Rate Limit Exhaustion**
- Predicts when rate limits will be exhausted
- Based on recent request rate and rate limiting frequency
- Provides estimated time until exhaustion

**Circuit Breaker Trip** (Coming Soon)
- Predicts when circuit breaker will trip
- Based on error rates and thresholds
- Provides estimated time until trip

**Configuration Suggestions**
- Analyzes performance data
- Suggests configuration changes
- Provides expected impact

**Capacity Recommendations**
- Identifies scaling needs
- Suggests infrastructure changes
- Provides growth forecasts

### Configuration Suggestions Example

```rust
ConfigSuggestion {
    setting: "cache_ttl",
    current_value: "300s",
    suggested_value: "600s",
    reason: "Low cache hit rate (25%)",
    expected_impact: "Increase cache hit rate by 15-20%",
}

ConfigSuggestion {
    setting: "connection_pool_size",
    current_value: "10",
    suggested_value: "20",
    reason: "High average latency (520ms)",
    expected_impact: "Reduce latency by 20-30%",
}
```

---

## Usage Analytics

Usage analytics tracks patterns and provides optimization opportunities.

### Getting Usage Analytics

```rust
let usage = analytics.get_usage_analytics().await;

// Most popular symbols
println!("Popular symbols:");
for (i, (symbol, count)) in usage.popular_symbols.iter().enumerate() {
    println!("{}. {} ({} requests)", i + 1, symbol, count);
}

// Query patterns
for pattern in usage.query_patterns {
    println!("Pattern: {}", pattern.description);
    println!("Frequency: {}", pattern.frequency);
    
    if let Some(optimization) = pattern.optimization {
        println!("Optimization: {}", optimization);
    }
}

// Recommendations
for recommendation in usage.recommendations {
    println!("Recommendation: {}", recommendation);
}

// Resource utilization
let resources = usage.resource_utilization;
println!("Resource Utilization:");
println!("  Memory: {:.2} MB", resources.memory_usage_mb);
println!("  Cache: {:.1}%", resources.cache_utilization * 100.0);
println!("  Connection pool: {:.1}%", resources.connection_pool_utilization * 100.0);
println!("  API quota: {:.1}%", resources.api_quota_utilization * 100.0);
```

### Popular Symbols

The top 10 most requested symbols with request counts:

```rust
popular_symbols: vec![
    ("AAPL".to_string(), 1250),
    ("GOOGL".to_string(), 890),
    ("MSFT".to_string(), 765),
    // ...
]
```

Use this to:
- Identify hot paths for optimization
- Implement dedicated caching for popular symbols
- Pre-warm cache for frequently accessed data

### Query Patterns

Detected patterns with optimization suggestions:

```rust
QueryPattern {
    description: "High frequency symbol: AAPL",
    frequency: 1250,
    optimization: Some("Consider dedicated caching for AAPL (47% of requests)"),
}
```

Common patterns:
- **High Frequency Symbols**: Single symbol gets >20% of traffic
- **Batch Queries**: Multiple symbols requested together
- **Periodic Polling**: Regular request patterns
- **Time-Based Access**: Requests clustered around market hours

### Optimization Recommendations

Based on usage patterns:

```rust
recommendations: vec![
    "Consider implementing symbol-specific cache tiers for frequently accessed symbols",
    "High-frequency queries detected. Consider implementing cache warming for popular symbols",
    "Batch query pattern detected. Consider implementing multi-get optimization",
]
```

### Resource Utilization

Track resource consumption:

```rust
ResourceUtilization {
    memory_usage_mb: 2.5,
    cache_utilization: 0.75,  // 75% cache hit rate
    connection_pool_utilization: 0.45,  // 45% of connections in use
    api_quota_utilization: 0.30,  // 30% of API quota used
}
```

---

## Best Practices

### 1. Enable All Features for Comprehensive Insights

```rust
let config = AnalyticsConfig::builder()
    .enable_profiling(true)
    .enable_predictions(true)
    .enable_anomaly_detection(true)
    .enable_usage_analytics(true)
    .build();
```

### 2. Balance Retention with Memory

```rust
// For high-volume applications
let config = AnalyticsConfig::builder()
    .retention_period(Duration::from_secs(1800))  // 30 minutes
    .max_data_points(5000)
    .build();

// For low-volume applications
let config = AnalyticsConfig::builder()
    .retention_period(Duration::from_secs(7200))  // 2 hours
    .max_data_points(20000)
    .build();
```

### 3. Regular Monitoring

```rust
// Periodic monitoring task
tokio::spawn(async move {
    loop {
        sleep(Duration::from_secs(300)).await;  // Every 5 minutes
        
        let insights = analytics.get_insights().await;
        log_metrics(&insights);
        
        if let Some(anomalies) = analytics.detect_anomalies().await {
            handle_anomalies(anomalies).await;
        }
        
        let predictions = analytics.predict_issues().await;
        if predictions.rate_limit_exhaustion.is_some() {
            alert_team().await;
        }
    }
});
```

### 4. Act on Predictions

```rust
let predictions = analytics.predict_issues().await;

for suggestion in predictions.config_suggestions {
    match suggestion.setting.as_str() {
        "cache_ttl" => {
            config.update_cache_ttl(suggestion.suggested_value).await;
        }
        "connection_pool_size" => {
            pool.resize(suggestion.suggested_value.parse()?).await;
        }
        _ => {}
    }
}
```

### 5. Track All Requests

```rust
// Middleware pattern for automatic tracking
async fn make_request(symbol: &str) -> Result<Quote> {
    let start = Instant::now();
    let result = client.get_quotes(&[symbol]).await;
    let duration = start.elapsed();
    
    analytics.record_request(symbol, duration).await;
    
    if result.is_err() {
        analytics.record_error().await;
    }
    
    result
}
```

### 6. Configure Appropriate Thresholds

```rust
// Production: More conservative
let prod_config = AnalyticsConfig::builder()
    .anomaly_threshold(3.5)  // Fewer false positives
    .prediction_window(Duration::from_secs(600))  // 10 minute window
    .build();

// Development: More sensitive
let dev_config = AnalyticsConfig::builder()
    .anomaly_threshold(2.0)  // Catch more issues
    .prediction_window(Duration::from_secs(60))  // 1 minute window
    .build();
```

### 7. Monitor Resource Usage

```rust
let usage = analytics.get_usage_analytics().await;
let resources = usage.resource_utilization;

if resources.memory_usage_mb > 10.0 {
    // Analytics using too much memory
    log::warn!("High analytics memory usage: {:.2} MB", resources.memory_usage_mb);
    
    // Reduce retention or data points
    analytics.configure(
        AnalyticsConfig::builder()
            .max_data_points(5000)
            .build()
    );
}
```

---

## Performance Impact

### Overhead Metrics

| Operation               | Latency | Memory           |
| ----------------------- | ------- | ---------------- |
| `record_request()`      | <100µs  | 200 bytes/record |
| `record_profile()`      | <150µs  | 300 bytes/record |
| `get_insights()`        | <1ms    | Read-only        |
| `detect_anomalies()`    | <5ms    | Temporary        |
| `predict_issues()`      | <2ms    | Temporary        |
| `get_usage_analytics()` | <1ms    | Read-only        |

### Memory Usage

```
Base overhead: ~100 KB
Per data point: ~300 bytes
10,000 data points: ~3 MB
```

### CPU Usage

- Recording: <1% overhead
- Analysis (insights/anomalies): <2% when called
- Background: None (all operations are on-demand)

### Optimization Tips

**High Volume (>1000 req/s)**:
```rust
let config = AnalyticsConfig::builder()
    .max_data_points(5000)  // Limit memory
    .retention_period(Duration::from_secs(1800))  // 30 minutes
    .enable_usage_analytics(false)  // Disable if not needed
    .build();
```

**Low Volume (<100 req/s)**:
```rust
let config = AnalyticsConfig::builder()
    .max_data_points(20000)  // More history
    .retention_period(Duration::from_secs(7200))  // 2 hours
    .build();
```

---

## Integration Examples

### Basic Integration

```rust
use eeyf::{Client, analytics::{Analytics, AnalyticsConfig}};
use std::sync::Arc;

struct MonitoredClient {
    client: Client,
    analytics: Arc<Analytics>,
}

impl MonitoredClient {
    pub fn new(api_key: &str) -> Self {
        let client = Client::new(api_key);
        let analytics = Arc::new(Analytics::new(AnalyticsConfig::default()));
        
        Self { client, analytics }
    }
    
    pub async fn get_quote(&self, symbol: &str) -> Result<Quote> {
        let start = Instant::now();
        let result = self.client.get_quotes(&[symbol]).await;
        let duration = start.elapsed();
        
        self.analytics.record_request(symbol, duration).await;
        
        if result.is_err() {
            self.analytics.record_error().await;
        }
        
        result.map(|mut v| v.remove(0))
    }
    
    pub async fn get_insights(&self) -> PerformanceInsights {
        self.analytics.get_insights().await
    }
}
```

### Production Monitoring

```rust
use eeyf::analytics::{Analytics, AnalyticsConfig, AnomalyType};
use tokio::time::sleep;

async fn monitor_performance(analytics: Arc<Analytics>) {
    loop {
        sleep(Duration::from_secs(60)).await;  // Every minute
        
        // Get insights
        let insights = analytics.get_insights().await;
        
        // Log metrics
        log::info!(
            "Performance: avg={:?} p95={:?} cache={:.1}% rps={:.2}",
            insights.average_latency,
            insights.p95_latency,
            insights.cache_hit_rate * 100.0,
            insights.requests_per_second
        );
        
        // Check for anomalies
        if let Some(anomalies) = analytics.detect_anomalies().await {
            for anomaly in anomalies {
                log::warn!(
                    "Anomaly detected: {:?} severity={:.2} - {}",
                    anomaly.anomaly_type,
                    anomaly.severity,
                    anomaly.description
                );
                
                // Alert on high severity
                if anomaly.severity > 0.7 {
                    alert_ops_team(&anomaly).await;
                }
            }
        }
        
        // Check predictions
        let predictions = analytics.predict_issues().await;
        if let Some(exhaustion) = predictions.rate_limit_exhaustion {
            log::warn!("Rate limit exhaustion predicted in {:?}", exhaustion);
            
            if exhaustion < Duration::from_secs(120) {
                emergency_throttle().await;
            }
        }
    }
}
```

### Adaptive Configuration

```rust
async fn adaptive_configuration(analytics: Arc<Analytics>, config: Arc<RwLock<AppConfig>>) {
    loop {
        sleep(Duration::from_secs(600)).await;  // Every 10 minutes
        
        let predictions = analytics.predict_issues().await;
        
        for suggestion in predictions.config_suggestions {
            log::info!(
                "Config suggestion: {} {} → {} ({})",
                suggestion.setting,
                suggestion.current_value,
                suggestion.suggested_value,
                suggestion.reason
            );
            
            // Apply suggestions automatically
            let mut config = config.write().await;
            match suggestion.setting.as_str() {
                "cache_ttl" => {
                    if let Ok(ttl) = suggestion.suggested_value.parse::<u64>() {
                        config.cache_ttl = Duration::from_secs(ttl);
                        log::info!("Updated cache_ttl to {}s", ttl);
                    }
                }
                "connection_pool_size" => {
                    if let Ok(size) = suggestion.suggested_value.parse::<usize>() {
                        config.pool_size = size;
                        log::info!("Updated pool size to {}", size);
                    }
                }
                _ => {}
            }
        }
    }
}
```

### Dashboard Integration

```rust
use axum::{Router, Json, routing::get};

async fn metrics_handler(
    analytics: Arc<Analytics>
) -> Json<serde_json::Value> {
    let insights = analytics.get_insights().await;
    let usage = analytics.get_usage_analytics().await;
    let predictions = analytics.predict_issues().await;
    
    Json(serde_json::json!({
        "performance": {
            "total_requests": insights.total_requests,
            "requests_per_second": insights.requests_per_second,
            "average_latency_ms": insights.average_latency.as_millis(),
            "p95_latency_ms": insights.p95_latency.as_millis(),
            "p99_latency_ms": insights.p99_latency.as_millis(),
            "cache_hit_rate": insights.cache_hit_rate,
        },
        "popular_symbols": usage.popular_symbols,
        "anomalies": analytics.detect_anomalies().await,
        "predictions": {
            "rate_limit_exhaustion": predictions.rate_limit_exhaustion,
            "config_suggestions": predictions.config_suggestions,
        }
    }))
}

let app = Router::new()
    .route("/metrics", get(metrics_handler));
```

---

## Further Reading

- [Analytics API Documentation](../src/analytics.rs)
- [Performance Optimization Guide](PERFORMANCE.md)
- [Phase 9 Completion Report](PHASE9_COMPLETION.md)
- [Analytics Example](../examples/analytics/)

---

**Last Updated**: Phase 9 Implementation
**Status**: Production Ready
