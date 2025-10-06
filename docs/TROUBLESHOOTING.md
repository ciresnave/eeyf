# EEYF Troubleshooting Guide

**Last Updated**: October 3, 2025  
**Version**: 0.1.0

This guide helps you diagnose and solve common issues when using EEYF. Issues are organized by symptom with step-by-step solutions.

---

## 🚨 Common Issues & Solutions

### Rate Limiting Issues

#### Issue: Getting HTTP 429 (Too Many Requests) Errors
```
YahooError::TooManyRequests("Request limit exceeded")
```

**Causes & Solutions:**

1. **Rate limit too aggressive**
   ```rust
   // ❌ Problem: Too many requests per hour
   let connector = YahooConnector::builder()
       .rate_limit(5000.0)  // WAY too high!
       .build()?;
   
   // ✅ Solution: Use conservative limits
   let connector = YahooConnector::builder()
       .rate_limit(1800.0)  // 90% of Yahoo's 2000/hour limit
       .build()?;
   ```

2. **Multiple instances competing**
   ```rust
   // ❌ Problem: Creating multiple connectors
   let conn1 = YahooConnector::new()?;
   let conn2 = YahooConnector::new()?;  // Separate rate limiters!
   
   // ✅ Solution: Reuse single connector
   let connector = YahooConnector::new()?;
   let connector = Arc::new(connector);  // Share between threads
   ```

3. **Burst requests exceeding limit**
   ```rust
   // ❌ Problem: Too many requests at once
   for symbol in symbols {
       connector.get_latest_quotes(symbol, "1d").await?;  // No delay!
   }
   
   // ✅ Solution: Use built-in rate limiting or add delays
   for symbol in symbols {
       let result = connector.get_latest_quotes(symbol, "1d").await;
       // Rate limiter automatically handles spacing
       match result {
           Ok(quotes) => process_quotes(quotes),
           Err(e) => eprintln!("Failed for {}: {}", symbol, e),
       }
   }
   ```

**Quick Fix:**
```rust
// Emergency rate limiting
let connector = YahooConnector::builder()
    .rate_limit(900.0)  // Half of normal limit
    .build()?;
```

---

#### Issue: Rate Limiter Blocking All Requests
```
YahooError::RateLimitExceeded("Hourly quota exceeded")
```

**Diagnostic Steps:**

1. **Check current rate limiter status** (if metrics enabled):
   ```rust
   // Enable verbose logging to see rate limiter decisions
   let connector = YahooConnector::builder()
       .verbose_logging(true)
       .enable_metrics(true)
       .build()?;
   ```

2. **Reset rate limiter** (nuclear option):
   ```rust
   // Create new connector to reset rate limiter
   drop(old_connector);
   let connector = YahooConnector::new()?;
   ```

3. **Use minimal preset for testing**:
   ```rust
   let connector = YahooConnector::from_preset("minimal")?;
   ```

---

### Network & Timeout Issues

#### Issue: Frequent Timeout Errors
```
YahooError::Timeout("Request timed out after 30s")
```

**Diagnostic Questions:**
- Are you behind a corporate firewall?
- Is your internet connection stable?
- Are you making requests during market hours (higher load)?

**Solutions:**

1. **Increase timeout duration**:
   ```rust
   use std::time::Duration;
   
   let connector = YahooConnector::builder()
       .timeout(Duration::from_secs(60))  // Increase from 30s default
       .build()?;
   ```

2. **Check proxy configuration**:
   ```rust
   use reqwest::Proxy;
   
   let proxy = Proxy::http("http://proxy.company.com:8080")?;
   
   // Note: Proxy configuration requires manual HTTP client setup
   // Use YahooConnectorBuilderLegacy for proxy support
   ```

3. **Use enterprise preset** (has generous timeouts):
   ```rust
   let connector = YahooConnector::from_preset("enterprise")?;
   ```

---

#### Issue: SSL/TLS Connection Errors
```
YahooError::NetworkError("SSL handshake failed")
```

**Common Causes:**
- Corporate firewall intercepting SSL
- Outdated system certificates
- Clock skew (incorrect system time)

**Solutions:**

1. **Update system certificates**:
   ```bash
   # Ubuntu/Debian
   sudo apt update && sudo apt install ca-certificates
   
   # CentOS/RHEL
   sudo yum update ca-certificates
   
   # Windows: Update via Windows Update
   ```

2. **Check system time**:
   ```bash
   # Ensure system clock is accurate
   timedatectl status  # Linux
   # Sync if needed: sudo ntpdate -s time.nist.gov
   ```

3. **Corporate firewall workaround**:
   ```rust
   // May need custom TLS configuration (advanced)
   // Consider using HTTP proxy instead of HTTPS interception
   ```

---

### Circuit Breaker Issues

#### Issue: Circuit Breaker Stuck in Open State
```
YahooError::CircuitBreakerOpen("Circuit breaker is open, failing fast")
```

**Understanding Circuit Breaker States:**
- **Closed**: Normal operation
- **Open**: Too many failures, rejecting requests
- **Half-Open**: Testing if service recovered

**Solutions:**

1. **Wait for automatic recovery**:
   ```rust
   // Circuit breaker opens for 60 seconds by default
   // It will automatically transition to Half-Open
   tokio::time::sleep(Duration::from_secs(65)).await;
   
   // Try a single request to test recovery
   let result = connector.get_latest_quotes("AAPL", "1d").await;
   ```

2. **Adjust circuit breaker sensitivity**:
   ```rust
   let connector = YahooConnector::builder()
       .circuit_breaker_threshold(10)        // Allow more failures (default: 5)
       .circuit_breaker_window_secs(600)     // Longer window (default: 300)
       .circuit_breaker_timeout_secs(30)     // Shorter recovery time (default: 60)
       .build()?;
   ```

3. **Create new connector** (resets circuit breaker):
   ```rust
   // Nuclear option: fresh circuit breaker state
   let connector = YahooConnector::new()?;
   ```

---

### Caching Issues

#### Issue: Stale Data from Cache
```
// Getting old quotes when market is active
```

**Solutions:**

1. **Reduce cache TTL**:
   ```rust
   use std::time::Duration;
   
   let connector = YahooConnector::builder()
       .cache_duration(Duration::from_secs(60))  // 1 minute instead of 15
       .build()?;
   ```

2. **Disable caching for real-time data**:
   ```rust
   let connector = YahooConnector::builder()
       .cache_size(0)  // Disable cache entirely
       .build()?;
   ```

3. **Use minimal preset** (no caching):
   ```rust
   let connector = YahooConnector::from_preset("minimal")?;
   ```

---

#### Issue: High Memory Usage from Cache
```
// Application using too much memory
```

**Diagnostic Steps:**

1. **Check cache configuration**:
   ```rust
   // Current cache holds 1000 entries by default
   // Each entry ~1KB, so ~1MB total cache
   ```

2. **Reduce cache size**:
   ```rust
   let connector = YahooConnector::builder()
       .cache_size(100)  // Reduce from 1000 default
       .build()?;
   ```

3. **Monitor cache hit rates** (if metrics enabled):
   ```rust
   let connector = YahooConnector::builder()
       .enable_metrics(true)
       .verbose_logging(true)
       .build()?;
   
   // Check logs for cache hit/miss ratios
   ```

---

### Data Quality Issues

#### Issue: Unexpected Data Format from Yahoo API
```
YahooError::DataParsingError("Failed to parse response JSON")
```

**This usually indicates Yahoo changed their API format.**

**Solutions:**

1. **Enable verbose logging** to see raw responses:
   ```rust
   let connector = YahooConnector::builder()
       .verbose_logging(true)
       .build()?;
   ```

2. **Check for library updates**:
   ```bash
   cargo update eeyf
   ```

3. **Report the issue** with raw response data:
   ```rust
   // Include the symbol and time period that failed
   // Include verbose log output
   // Include your EEYF version: cargo tree | grep eeyf
   ```

---

#### Issue: Missing Data Fields
```rust
// Some quotes missing expected fields like volume, high, low
let quote = response.last_quote().unwrap();
// quote.volume might be None for some symbols
```

**This is normal for certain symbols/time periods.**

**Handling:**

```rust
let quote = response.last_quote().unwrap();

// ✅ Always check for None values
match quote.volume {
    Some(vol) => println!("Volume: {}", vol),
    None => println!("Volume data not available"),
}

// ✅ Use unwrap_or for defaults
let volume = quote.volume.unwrap_or(0);
let high = quote.high.unwrap_or(quote.close);
```

---

### Performance Issues

#### Issue: Slow Response Times
```
// Requests taking >5 seconds consistently
```

**Diagnostic Steps:**

1. **Check if it's caching related**:
   ```rust
   // Test with cache disabled
   let connector = YahooConnector::builder()
       .cache_size(0)
       .build()?;
   ```

2. **Test with minimal configuration**:
   ```rust
   let connector = YahooConnector::from_preset("minimal")?;
   ```

3. **Enable metrics** to see timing breakdown:
   ```rust
   let connector = YahooConnector::builder()
       .enable_metrics(true)
       .verbose_logging(true)
       .build()?;
   ```

**Common Solutions:**

- **Network latency**: Use enterprise preset (has connection pooling optimizations)
- **Yahoo API slowness**: Enable retry with exponential backoff
- **DNS issues**: Consider using IP addresses or different DNS servers

---

### Configuration Issues

#### Issue: Builder Validation Errors
```
YahooError::InvalidConfiguration("Rate limit must be positive")
```

**Common Validation Failures:**

```rust
// ❌ Invalid configurations
YahooConnector::builder()
    .rate_limit(0.0)           // Must be > 0
    .timeout(Duration::ZERO)   // Must be > 0
    .cache_size(usize::MAX)    // Reasonable limits
    .retry_attempts(0)         // Must be > 0
    .build()?;  // Will fail validation

// ✅ Valid configurations
YahooConnector::builder()
    .rate_limit(1800.0)
    .timeout(Duration::from_secs(30))
    .cache_size(1000)
    .retry_attempts(3)
    .build()?;  // Will succeed
```

---

#### Issue: Preset Not Found
```
YahooError::PresetNotFound("my-preset")
```

**Preset Loading Order:**
1. Built-in presets: `production`, `development`, `enterprise`, `minimal`
2. Project presets: `./.eeyf/presets/my-preset.toml`
3. User presets: `~/.config/eeyf/presets/my-preset.toml` (Linux/macOS)
4. User presets: `%APPDATA%\eeyf\presets\my-preset.toml` (Windows)

**Solutions:**

1. **Check preset name spelling**:
   ```rust
   // ✅ Correct built-in preset names
   YahooConnector::from_preset("production")?;
   YahooConnector::from_preset("development")?;
   YahooConnector::from_preset("enterprise")?;
   YahooConnector::from_preset("minimal")?;
   ```

2. **List available presets**:
   ```rust
   use eeyf::presets::PresetManager;
   
   let manager = PresetManager::new();
   let presets = manager.list_presets()?;
   println!("Available presets: {:?}", presets);
   ```

3. **Create missing preset file**:
   ```bash
   # Create directory
   mkdir -p .eeyf/presets
   
   # Create preset file
   cat > .eeyf/presets/my-preset.toml << EOF
   name = "my-preset"
   description = "My custom configuration"
   rate_limit = 1800.0
   timeout_secs = 30
   cache_size = 1000
   EOF
   ```

---

## 🔧 Diagnostic Tools

### Enable Verbose Logging
```rust
let connector = YahooConnector::builder()
    .verbose_logging(true)
    .build()?;

// This will log:
// - Rate limiter decisions
// - Circuit breaker state changes
// - Cache hit/miss events
// - Retry attempts
// - Raw HTTP requests/responses (if debug level)
```

### Enable Metrics Collection
```rust
let connector = YahooConnector::builder()
    .enable_metrics(true)
    .build()?;

// Metrics available (if Prometheus endpoint enabled):
// - eeyf_requests_total{symbol, endpoint, status}
// - eeyf_request_duration_seconds{symbol, endpoint}
// - eeyf_rate_limit_tokens_available
// - eeyf_circuit_breaker_state{endpoint}
// - eeyf_cache_hits_total
// - eeyf_cache_misses_total
```

### Test with Minimal Configuration
```rust
// Bare minimum for testing - bypasses most enterprise features
let connector = YahooConnector::from_preset("minimal")?;

// Or manually:
let connector = YahooConnector::builder()
    .rate_limit(f64::MAX)      // No rate limiting
    .cache_size(0)             // No caching
    .retry_attempts(1)         // No retries
    .circuit_breaker_threshold(u32::MAX)  // No circuit breaking
    .timeout(Duration::from_secs(5))       // Short timeout
    .build()?;
```

---

## 📞 Getting Help

### Before Reporting Issues

1. **Check this troubleshooting guide**
2. **Test with minimal preset** to isolate the issue
3. **Enable verbose logging** to capture detailed information
4. **Check for library updates**: `cargo update eeyf`

### Information to Include in Bug Reports

```rust
// Include this information:
println!("EEYF Version: {}", env!("CARGO_PKG_VERSION"));
println!("Rust Version: {}", env!("RUSTC_VERSION"));
println!("OS: {}", env::consts::OS);

// Configuration used:
let config = YahooConnector::builder()
    /* your configuration */
    .build()?;

// Exact error message and stack trace
// Steps to reproduce
// Expected vs actual behavior
```

### Common Support Channels

- **GitHub Issues**: For bugs and feature requests
- **Documentation**: Check `docs/` folder for guides
- **Examples**: See `examples/` folder for usage patterns
- **Tests**: Look at test files for expected behavior patterns

---

### Performance Benchmarking

If reporting performance issues, include benchmark data:

```rust
use std::time::Instant;

let start = Instant::now();
let result = connector.get_latest_quotes("AAPL", "1d").await?;
let duration = start.elapsed();

println!("Request took: {:?}", duration);
println!("Response size: {} bytes", serde_json::to_string(&result)?.len());
```

---

This guide covers the most common issues. For complex problems, create a minimal reproduction case and file an issue with detailed logs and configuration information.