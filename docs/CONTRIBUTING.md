# Contributing to EEYF

**Welcome!** We're excited that you're interested in contributing to EEYF (Efficient Enterprise Yahoo Finance). This guide will help you get started.

---

## 🎯 Quick Start for Contributors

### 1. Development Environment Setup

#### Prerequisites
- **Rust**: 1.70.0 or later (`rustup install stable`)
- **Git**: For version control
- **GitHub Account**: For pull requests
- **Editor**: VS Code with rust-analyzer recommended

#### Clone and Setup
```bash
# Fork the repository on GitHub, then clone your fork
git clone https://github.com/YOUR_USERNAME/EEYF.git
cd EEYF

# Install dependencies
cargo build

# Run tests to verify setup
cargo test

# Run examples to verify functionality  
cargo run --example builder_pattern_demo
```

### 2. Understanding the Codebase

```
src/
├── lib.rs              # Main library entry point and YahooConnector
├── builder.rs          # Builder pattern implementation (Phase 1.1)  
├── presets.rs          # Configuration preset management (Phase 1.1)
├── enterprise.rs       # Enterprise features and configurations
├── rate_limiter.rs     # Token bucket rate limiting
├── circuit_breaker.rs  # Circuit breaker implementation
├── response_cache.rs   # LRU response caching  
├── retry.rs            # Exponential backoff retry logic
├── connection_pool.rs  # HTTP connection pooling
├── observability.rs    # Metrics and tracing
├── error_categories.rs # Error classification system
├── async_impl.rs       # Core async API implementations
├── yahoo_error.rs      # Error types and handling
└── quotes.rs          # Data structures for quotes/responses

examples/
├── builder_pattern_demo.rs  # Demonstrates new builder API
└── ...                      # More examples coming in Phase 1.2

docs/
├── ARCHITECTURE.md     # System design and component interaction  
├── TROUBLESHOOTING.md  # Common issues and solutions
├── PERFORMANCE.md      # Performance tuning guide
└── CONTRIBUTING.md     # This file

tests/
└── ...                 # Integration and unit tests
```

---

## 🛠️ Development Workflow

### 1. Before You Start

1. **Check existing issues** to avoid duplicate work
2. **Create an issue** to discuss your proposed changes
3. **Read the roadmap** (`ROADMAP.md`) to understand project direction
4. **Check the current phase** - we implement features in order

### 2. Making Changes

#### Branch Naming
```bash
# Feature branches
git checkout -b feature/phase-2-1-websocket-streaming
git checkout -b feature/add-options-chain-api

# Bug fix branches  
git checkout -b fix/rate-limiter-overflow
git checkout -b fix/circuit-breaker-race-condition

# Documentation branches
git checkout -b docs/improve-api-examples
git checkout -b docs/add-migration-guide
```

#### Commit Messages
Follow conventional commit format:
```bash
# Feature commits
git commit -m "feat(builder): add custom preset validation"
git commit -m "feat(websocket): implement real-time quote streaming"

# Bug fix commits
git commit -m "fix(rate_limiter): prevent integer overflow in token calculation"
git commit -m "fix(cache): resolve race condition in concurrent access"

# Documentation commits  
git commit -m "docs(troubleshooting): add rate limiting section"
git commit -m "docs(examples): add portfolio tracker example"

# Test commits
git commit -m "test(builder): add validation edge case tests"
git commit -m "test(integration): add Yahoo API error simulation"
```

### 3. Code Style and Standards

#### Rust Style Guidelines
```rust
// ✅ Use descriptive names
let rate_limiter_config = RateLimitConfig::default();

// ✅ Document public APIs
/// Creates a new Yahoo Finance connector with production defaults.
/// 
/// This connector includes enterprise features like rate limiting,
/// circuit breaking, response caching, and retry logic.
///
/// # Examples
/// ```rust
/// let connector = YahooConnector::new()?;
/// let quotes = connector.get_latest_quotes("AAPL", "1d").await?;
/// ```
pub fn new() -> Result<YahooConnector, YahooError> {
    // Implementation
}

// ✅ Use Result types for fallible operations
pub fn validate_configuration(&self) -> Result<(), YahooError> {
    if self.rate_limit <= 0.0 {
        return Err(YahooError::InvalidConfiguration(
            "Rate limit must be positive".into()
        ));
    }
    Ok(())
}

// ✅ Use structured error handling
match result {
    Ok(data) => process_data(data),
    Err(YahooError::RateLimitExceeded(_)) => {
        // Handle rate limiting specifically
        tokio::time::sleep(Duration::from_secs(60)).await;
        retry_request()
    }
    Err(e) => return Err(e),
}
```

#### Error Handling Standards
```rust
// ✅ Provide context in error messages
Err(YahooError::InvalidStatusCode(format!(
    "Failed to fetch quotes for symbol '{}': HTTP {}", 
    symbol, 
    status_code
)))

// ✅ Use appropriate error categories
YahooError::NetworkError(_)        // Network/connection issues
YahooError::RateLimitExceeded(_)   // Rate limiting 
YahooError::InvalidConfiguration(_) // Configuration errors
YahooError::DataParsingError(_)    // JSON/data parsing issues

// ✅ Include recovery suggestions
Err(YahooError::CircuitBreakerOpen(
    "Circuit breaker is open. Wait 60 seconds before retrying.".into()
))
```

#### Documentation Standards
```rust
// ✅ Include examples in all public APIs
/// Configures the rate limiting for this connector.
///
/// # Arguments
/// * `requests_per_hour` - Maximum requests per hour (must be > 0)
///
/// # Examples
/// ```rust
/// let connector = YahooConnector::builder()
///     .rate_limit(1800.0)  // Conservative rate limiting
///     .build()?;
/// ```
///
/// # Panics
/// Never panics. Returns validation error if rate limit is invalid.
///
/// # Errors
/// Returns `YahooError::InvalidConfiguration` if rate limit is <= 0.
pub fn rate_limit(mut self, requests_per_hour: f64) -> Self {
    // Implementation
}
```

---

## 🧪 Testing Guidelines

### 1. Test Categories

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_rate_limiter_allows_initial_requests() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);
        
        // Should allow initial burst
        for _ in 0..10 {
            assert!(limiter.try_acquire().is_ok());
        }
    }

    #[test]
    fn test_builder_validation_rejects_invalid_rate_limit() {
        let result = YahooConnectorBuilder::new()
            .rate_limit(0.0)  // Invalid
            .build();
            
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), YahooError::InvalidConfiguration(_)));
    }
}
```

#### Integration Tests
```rust
#[tokio::test]
async fn test_real_yahoo_api_request() {
    let connector = YahooConnector::new().unwrap();
    
    let result = connector.get_latest_quotes("AAPL", "1d").await;
    
    match result {
        Ok(response) => {
            assert!(!response.quotes().unwrap().is_empty());
        }
        Err(YahooError::RateLimitExceeded(_)) => {
            // Expected if we hit rate limits during testing
            println!("Rate limited - this is expected in CI");
        }
        Err(e) => panic!("Unexpected error: {}", e),
    }
}
```

#### Benchmark Tests
```rust
#[cfg(test)]
mod bench {
    use super::*;
    use std::time::Instant;

    #[test]
    fn bench_rate_limiter_performance() {
        let config = RateLimitConfig::default();
        let mut limiter = RateLimiter::new(config);
        
        let start = Instant::now();
        for _ in 0..1000 {
            let _ = limiter.try_acquire();
        }
        let duration = start.elapsed();
        
        // Should be very fast (< 1ms for 1000 operations)
        assert!(duration < Duration::from_millis(1));
    }
}
```

### 2. Running Tests

```bash
# Run all tests
cargo test

# Run specific test module
cargo test rate_limiter

# Run tests with output
cargo test -- --nocapture

# Run integration tests (may hit real Yahoo API)
cargo test --test integration

# Run benchmarks
cargo test bench -- --nocapture

# Check test coverage (requires cargo-tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out Html
```

### 3. Test Data and Mocking

#### Mock HTTP Responses
```rust
#[cfg(test)]
mod tests {
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn test_handles_yahoo_api_error() {
        let _mock = mock("GET", "/v8/finance/chart/INVALID")
            .with_status(404)
            .with_body(r#"{"chart":{"error":"Not Found"}}"#)
            .create();

        let connector = YahooConnector::new().unwrap();
        let result = connector.get_latest_quotes("INVALID", "1d").await;
        
        assert!(result.is_err());
    }
}
```

#### Test Configuration
```rust
// Use minimal preset for faster tests
let connector = YahooConnector::builder()
    .rate_limit(f64::MAX)      // No rate limiting in tests
    .cache_size(0)             // No caching in tests  
    .timeout(Duration::from_secs(5))  // Short timeout
    .retry_attempts(1)         // No retries in tests
    .build()
    .unwrap();
```

---

## 📋 Pull Request Process

### 1. Before Submitting

- [ ] **Tests pass**: `cargo test`
- [ ] **Code compiles**: `cargo check`
- [ ] **Examples work**: `cargo run --example builder_pattern_demo`  
- [ ] **Documentation updated**: If you added new public APIs
- [ ] **CHANGELOG.md updated**: Add entry for your changes
- [ ] **No new clippy warnings**: `cargo clippy`

### 2. PR Template

When creating a pull request, include:

```markdown
## Description
Brief description of what this PR does and why.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)  
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated  
- [ ] Manual testing performed
- [ ] Examples updated if needed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review performed
- [ ] Documentation updated
- [ ] Tests pass locally
- [ ] CHANGELOG.md updated

## Related Issues
Closes #123
Related to #456
```

### 3. Review Process

1. **Automated checks** run (CI/CD pipeline)
2. **Maintainer review** (usually within 48 hours)
3. **Address feedback** if requested
4. **Approval and merge** after all checks pass

---

## 🎯 Areas Where We Need Help

### Phase 1.2 Documentation (Current Priority)
- [ ] **Real-world examples**: Portfolio tracker, price alerts, trading bot
- [ ] **Migration guide**: From other Yahoo Finance libraries
- [ ] **Video tutorials**: YouTube tutorials for common use cases
- [ ] **Blog posts**: Technical deep-dives on enterprise features

### Phase 2+ Future Features
- [ ] **WebSocket streaming**: Real-time quote updates
- [ ] **Options chain data**: Options pricing and Greeks
- [ ] **Fundamental data**: Earnings, balance sheets, cash flow
- [ ] **Technical indicators**: Moving averages, RSI, MACD
- [ ] **Screener API**: Find stocks by criteria

### Infrastructure Improvements
- [ ] **CI/CD enhancements**: Automated benchmarking, security scanning
- [ ] **Documentation hosting**: GitHub Pages or docs.rs improvements
- [ ] **Performance monitoring**: Automated performance regression detection
- [ ] **Error tracking**: Better error reporting and categorization

---

## 🏆 Recognition

### Contributors Wall
We maintain a contributors section in our README.md to recognize everyone who helps make EEYF better.

### Contribution Types We Value
- **Code contributions**: Bug fixes, new features, performance improvements
- **Documentation**: Guides, examples, API docs, blog posts
- **Testing**: Bug reports, test cases, performance benchmarks
- **Community**: Answering questions, helping other users
- **Design**: UI/UX for tools, architecture proposals

---

## ❓ Getting Help

### Communication Channels
- **GitHub Issues**: Bug reports, feature requests, questions
- **GitHub Discussions**: General questions, architecture discussions
- **Email**: maintainers@eeyf-project.org (for security issues)

### What to Include When Asking for Help
1. **EEYF version**: `cargo tree | grep eeyf`
2. **Rust version**: `rustc --version`
3. **Operating system**: Linux/macOS/Windows version
4. **Minimal reproduction case**: Smallest possible code that demonstrates the issue
5. **Error messages**: Full error output with stack traces
6. **Expected vs actual behavior**: What you expected vs what happened

### Response Time Expectations
- **Bug reports**: Within 24 hours (acknowledgment)
- **Feature requests**: Within 48 hours (initial review)
- **Pull requests**: Within 48 hours (initial review)
- **Questions**: Within 24 hours (best effort)

---

Thank you for contributing to EEYF! Your help makes this project better for everyone in the Rust financial data community. 🚀