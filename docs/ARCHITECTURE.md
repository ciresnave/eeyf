# EEYF Architecture Guide

**Last Updated**: October 3, 2025  
**Version**: 0.1.0

This document explains the internal architecture of EEYF (Efficient Enterprise Yahoo Finance), how its components work together, and the design decisions behind the enterprise-grade features.

---

## 🏗️ System Overview

EEYF is designed as a layered architecture with enterprise reliability features built into every layer:

```
┌─────────────────────────────────────────────────────────────┐
│                    Public API Layer                         │
│  YahooConnector::new() | builder() | from_preset()         │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                 Configuration Layer                         │
│  Builder Pattern | Presets | Validation | Enterprise Config │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                Enterprise Features Layer                    │
│  Rate Limiter | Circuit Breaker | Cache | Retry | Pool     │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                 Observability Layer                         │
│  Metrics | Tracing | Request Context | Error Categories    │
└─────────────────────────┬───────────────────────────────────┘
                          │
┌─────────────────────────▼───────────────────────────────────┐
│                    Network Layer                            │
│  HTTP Client | Connection Pool | Proxy Support | TLS       │
└─────────────────────────────────────────────────────────────┘
```

---

## 🔧 Core Components

### YahooConnector - Main API Entry Point

The `YahooConnector` is the primary interface users interact with. It provides three construction patterns:

```rust
// 1. Production defaults - Safe, stable, no configuration required
let connector = YahooConnector::new()?;

// 2. Builder pattern - Flexible configuration with validation
let connector = YahooConnector::builder()
    .rate_limit(1800.0)  // requests per hour
    .timeout(Duration::from_secs(30))
    .cache_size(1000)
    .enable_metrics(true)
    .build()?;

// 3. Preset-based - Pre-configured for common use cases
let connector = YahooConnector::from_preset("enterprise")?;
```

**Key Design Decisions:**
- `new()` returns production-ready defaults to prevent IP blocks
- Builder pattern provides type safety and build-time validation
- Presets enable team-wide consistent configuration

### Enterprise Features Layer

This layer implements the reliability and performance features:

#### Rate Limiter
```
Request → Rate Limiter → [Allow/Deny] → Yahoo API
    ↓         ↓
  Token     Hourly
  Bucket    Counter
```

- **Algorithm**: Token bucket with hourly quotas
- **Default**: 1800 req/hour (90% of Yahoo's 2000/hour limit)
- **Burst handling**: Allows 10 rapid requests, then enforces minimum intervals
- **Thread safety**: Atomic counters for concurrent access

#### Circuit Breaker
```
Closed → [Failures] → Half-Open → [Success] → Closed
   ↑                      ↓            ↓
   └── [Timeout] ← Open ←┘         [Failure]
```

- **States**: Closed (normal), Open (failing), Half-Open (testing)
- **Default**: 5 failures in 5 minutes triggers circuit opening
- **Recovery**: 60-second timeout before retry attempts
- **Fast failure**: Prevents cascading failures in distributed systems

#### Response Cache
```
Request → Cache Check → [Hit: Return] | [Miss: Fetch → Store → Return]
              ↓
         LRU Eviction
```

- **Algorithm**: LRU (Least Recently Used) eviction
- **Default**: 1000 entries, 15-minute TTL
- **Thread safety**: DashMap for concurrent access
- **Size estimation**: JSON content size tracking

#### Retry System
```
Request → [Fail] → Exponential Backoff → Retry → [Success/Give Up]
             ↓           ↓
       Error Analysis  Jitter Added
```

- **Strategy**: Exponential backoff with jitter
- **Default**: 3 attempts, 1s → 2s → 4s delays
- **Smart retries**: Only retries network/timeout errors, not 4xx client errors
- **Jitter**: ±25% randomization prevents thundering herd

### Connection Pool
```
Request → Pool → [Available Connection] → Yahoo API
            ↓            ↓
      Connection    Connection
       Creation      Reuse
```

- **Pool size**: Default 10 connections per host
- **Keep-alive**: Reuses connections for efficiency
- **TLS session resumption**: Reduces handshake overhead
- **Proxy support**: HTTP/HTTPS proxy configuration

---

## 🔄 Request Flow

Here's how a typical request flows through the system:

### 1. Request Initiation
```rust
let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
```

### 2. Flow Through Enterprise Layers
```
User Request
     ↓
┌────────────────────┐
│   Rate Limiter     │ ← Check if request is allowed
│  (Token Bucket)    │
└────────┬───────────┘
         ↓ [Allowed]
┌────────────────────┐
│   Circuit Breaker  │ ← Check if Yahoo API is healthy
│   (State Machine)  │
└────────┬───────────┘
         ↓ [Closed/Half-Open]
┌────────────────────┐
│  Response Cache    │ ← Check for cached response
│     (LRU Map)      │
└────────┬───────────┘
         ↓ [Cache Miss]
┌────────────────────┐
│  HTTP Client       │ ← Make actual request
│ (Connection Pool)  │
└────────┬───────────┘
         ↓
┌────────────────────┐
│   Retry Logic      │ ← Handle failures with backoff
│ (Exponential)      │
└────────┬───────────┘
         ↓ [Success]
┌────────────────────┐
│  Response Cache    │ ← Store result for future use
│    (Store)         │
└────────┬───────────┘
         ↓
┌────────────────────┐
│ Observability      │ ← Record metrics and traces
│ (Metrics/Tracing)  │
└────────┬───────────┘
         ↓
    Return Result
```

### 3. Error Handling Flow
```
Error Occurs
     ↓
┌────────────────────┐
│ Error Categories   │ ← Classify error type
└────────┬───────────┘
         ↓
    [Retryable?] ──→ [No] ──→ Return Error
         ↓ [Yes]
┌────────────────────┐
│  Circuit Breaker   │ ← Record failure
│   (Increment)      │
└────────┬───────────┘
         ↓
┌────────────────────┐
│   Retry Logic      │ ← Wait and retry
│ (Backoff + Jitter) │
└────────┬───────────┘
         ↓
    [Max Retries?] ──→ [Yes] ──→ Return Error
         ↓ [No]
    Retry Request
```

---

## 📊 Configuration Management

### Builder Pattern Architecture
```rust
YahooConnectorBuilder {
    // Core settings
    rate_limit: f64,                    // Requests per hour
    timeout: Duration,                  // Request timeout
    
    // Enterprise features
    circuit_breaker_threshold: u32,     // Failures before opening
    circuit_breaker_window_secs: u64,   // Failure counting window
    circuit_breaker_timeout_secs: u64,  // Recovery timeout
    
    // Retry configuration
    retry_attempts: u32,                // Max retry attempts
    retry_initial_delay_ms: u64,        // First retry delay
    retry_max_delay_ms: u64,           // Maximum retry delay
    
    // Caching
    cache_size: usize,                  // Max cached entries
    cache_duration_secs: u64,          // Cache TTL
    
    // Observability
    enable_metrics: bool,               // Prometheus metrics
    enable_tracing: bool,              // Distributed tracing
    verbose_logging: bool,             // Debug logging
}
```

### Preset System Architecture
```
Built-in Presets (Embedded)
├── production.toml    ← Safe defaults, conservative limits
├── development.toml   ← Fast feedback, verbose logging  
├── enterprise.toml    ← Maximum reliability, extended caching
└── minimal.toml       ← Bare minimum for testing

User Presets (File System)
├── Project: ./.eeyf/presets/
│   ├── staging.toml
│   └── integration.toml
└── Global: ~/.config/eeyf/presets/
    ├── personal.toml
    └── team-shared.toml
```

**Preset Loading Priority:**
1. Built-in presets (always available)
2. Project-local presets (`./.eeyf/presets/`)
3. User global presets (`~/.config/eeyf/presets/`)

### Validation System
```rust
impl YahooConnectorBuilder {
    fn validate(&self) -> Result<(), YahooError> {
        // Rate limiting validation
        if self.rate_limit <= 0.0 {
            return Err(YahooError::InvalidConfiguration(
                "Rate limit must be positive".into()
            ));
        }
        
        // Circuit breaker validation
        if self.circuit_breaker_threshold == 0 {
            return Err(YahooError::InvalidConfiguration(
                "Circuit breaker threshold must be > 0".into()
            ));
        }
        
        // Timeout validation
        if self.timeout.is_zero() {
            return Err(YahooError::InvalidConfiguration(
                "Timeout must be positive".into()
            ));
        }
        
        Ok(())
    }
}
```

---

## 🔍 Observability Architecture

### Metrics Collection
```
Request Events → Prometheus Counters/Histograms → Grafana Dashboard
     ↓
Rate Limiter → Token Bucket Metrics → Alerts
     ↓
Circuit Breaker → State Change Events → Notifications  
     ↓
Cache → Hit/Miss Ratios → Performance Tuning
```

### Distributed Tracing
```
Request Span (Root)
├── Rate Limiter Span
├── Circuit Breaker Span  
├── Cache Lookup Span
└── HTTP Request Span
    ├── Connection Pool Span
    ├── TLS Handshake Span
    └── Response Processing Span
```

### Request Context
```rust
pub struct RequestContext {
    pub request_id: String,           // Unique request identifier
    pub symbol: String,              // Stock symbol
    pub endpoint: String,            // API endpoint
    pub timestamp: OffsetDateTime,   // Request start time
    pub user_agent: String,          // Client identification
    pub timeout: Duration,           // Request timeout
}
```

---

## 🚀 Performance Characteristics

### Memory Usage
- **Base overhead**: ~50KB per YahooConnector instance
- **Cache overhead**: ~1KB per cached response (configurable)
- **Connection pool**: ~10KB per connection (default 10 connections)
- **Total typical usage**: ~100KB for production configuration

### Network Efficiency
- **Connection reuse**: HTTP/1.1 keep-alive reduces handshake overhead
- **TLS session resumption**: Reuses TLS sessions where possible
- **Compression**: Automatic gzip/deflate response decompression
- **Proxy support**: Corporate proxy environments supported

### Concurrency Model
- **Async/await**: Non-blocking I/O using tokio runtime
- **Thread safety**: All components safe for concurrent use
- **Lock-free rate limiting**: Atomic counters for high performance
- **Concurrent caching**: DashMap allows simultaneous cache operations

### Latency Breakdown (Typical Request)
```
Rate Limiter Check:     < 1μs
Circuit Breaker Check:  < 1μs  
Cache Lookup:          10-50μs
Network Request:       50-500ms (depends on Yahoo API)
Response Processing:   1-10ms
Cache Storage:         10-50μs
Total Overhead:        < 1ms (excluding network)
```

---

## 🔒 Security Considerations

### Network Security
- **TLS 1.2+**: Enforced for all Yahoo API communications
- **Certificate validation**: Full certificate chain validation
- **Proxy support**: HTTP/HTTPS proxy with authentication
- **No credentials storage**: Yahoo Finance API is public, no API keys required

### Data Privacy
- **No PII storage**: Only caches public market data
- **Configurable cache TTL**: Automatic data expiration
- **Request logging**: Minimal logging of request metadata only
- **User agent**: Identifies as EEYF client for transparency

### Rate Limiting Protection
- **Conservative defaults**: 90% of Yahoo's published limits
- **Burst protection**: Token bucket prevents rapid-fire requests
- **Circuit breaker**: Automatic backing off on API errors
- **Jitter**: Randomized retry delays prevent coordinated load

---

## 🧪 Testing Strategy

### Unit Testing
- Each enterprise feature has comprehensive unit tests
- Mock HTTP responses for predictable testing
- Property-based testing for edge cases
- Performance regression tests

### Integration Testing  
- Real Yahoo API calls with rate limiting
- Circuit breaker behavior under load
- Cache behavior across multiple requests
- Error handling with various Yahoo API responses

### Load Testing
- Concurrent request handling
- Rate limiter accuracy under high load
- Circuit breaker performance impact
- Memory usage under sustained load

---

## 🔄 Future Architecture Plans

### Phase 2 Enhancements
- **Real-time streaming**: WebSocket support for live quotes
- **Batch operations**: Multi-symbol request optimization
- **Advanced caching**: Intelligent cache warming and prefetching

### Phase 3 Expansions
- **Data enrichment**: Fundamental data integration
- **Market analysis**: Technical indicator calculations
- **Portfolio tracking**: Multi-symbol portfolio management

### Phase 4 Advanced Features
- **ML integration**: Predictive analytics capabilities  
- **Custom indicators**: User-defined technical indicators
- **Risk management**: Portfolio risk analysis tools

---

This architecture provides a solid foundation for reliable, enterprise-grade Yahoo Finance API access while maintaining simplicity for basic use cases. Each component is designed to fail gracefully and provide clear feedback for troubleshooting.