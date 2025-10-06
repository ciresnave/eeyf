# Code Coverage Setup for EEYF

This document provides instructions for measuring and improving code coverage for the EEYF library.

## Tools

We recommend using **cargo-llvm-cov** for code coverage analysis, as it provides accurate coverage data for Rust projects.

### Installation

```powershell
cargo install cargo-llvm-cov
```

## Basic Usage

### Run all tests with coverage

```powershell
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

### Generate HTML report

```powershell
cargo llvm-cov --all-features --workspace --html
```

This will create an HTML report in `target/llvm-cov/html/index.html`.

### Run specific test suites

```powershell
# Library tests only
cargo llvm-cov --lib

# Integration tests
cargo llvm-cov --test integration_tests

# All tests
cargo llvm-cov --all-targets
```

## Coverage Goals

**Target: >90% code coverage**

### Current Status (as of Phase 3.1)

- **Library tests**: 110 tests covering core functionality
- **Property tests**: 16+ properties verified
- **Mock server tests**: 11 HTTP mock scenarios
- **Load tests**: 7 load test scenarios
- **Benchmarks**: 7 benchmark groups

### Priority Areas for Coverage

1. **Error Handling Paths**
   - All YahooError variants
   - Error context creation and display
   - Error recovery scenarios

2. **Rate Limiting**
   - Token bucket algorithm
   - Hourly limits
   - Burst token management
   - Status reporting

3. **Circuit Breaker**
   - State transitions (Closed → Open → HalfOpen → Closed)
   - Failure threshold detection
   - Recovery timeout logic
   - Statistics tracking

4. **Connection Pool**
   - Connection acquisition
   - Connection release
   - Pool statistics
   - Timeout handling

5. **Enterprise Features**
   - Caching (L1/L2/L3)
   - Request deduplication
   - Retry logic
   - Observability integration

## Continuous Integration

### GitHub Actions Example

```yaml
name: Code Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          profile: minimal
          override: true
      
      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov
      
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      
      - name: Upload to codecov.io
        uses: codecov/codecov-action@v3
        with:
          files: lcov.info
          fail_ci_if_error: true
```

## Analyzing Coverage Reports

### Identifying Gaps

1. **Open HTML report**: `target/llvm-cov/html/index.html`
2. **Look for red/yellow highlighted code**: These lines are not covered
3. **Focus on**:
   - Error handling paths
   - Edge cases
   - Concurrent operations
   - Configuration combinations

### Adding Tests for Uncovered Code

For each uncovered code path:

1. **Identify the condition**: What triggers this path?
2. **Create a test**: Write a focused test that exercises this path
3. **Verify coverage**: Re-run coverage to confirm the path is now covered

Example:

```rust
// If you find this uncovered:
if error.is_retryable() {
    // retry logic
} else {
    // permanent failure
}

// Add tests like:
#[test]
fn test_non_retryable_error_handling() {
    let error = YahooError::Unauthorized;
    assert!(!error.is_retryable());
    // Test the else branch behavior
}
```

## Alternative: Tarpaulin

If cargo-llvm-cov doesn't work, use **tarpaulin**:

```powershell
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --all-features --workspace --out Html --output-dir coverage/
```

## Coverage Metrics

### Understanding the numbers

- **Line Coverage**: Percentage of executed lines
- **Branch Coverage**: Percentage of executed decision branches
- **Function Coverage**: Percentage of called functions

### Target Breakdown

- **Critical paths**: 100% (error handling, rate limiting)
- **Core functionality**: >95% (connection pool, circuit breaker)
- **Enterprise features**: >90% (caching, observability)
- **Examples/utilities**: >80%

## Best Practices

1. **Run coverage regularly**: Before each commit
2. **Track trends**: Monitor coverage over time
3. **Don't game the metrics**: Write meaningful tests, not just coverage tests
4. **Focus on critical paths**: Prioritize high-risk code
5. **Document untestable code**: Use `#[cfg(not(tarpaulin_include))]` if needed

## Excluding Code from Coverage

For code that can't be meaningfully tested:

```rust
#[cfg(not(tarpaulin_include))]
fn debug_only_function() {
    // This won't be counted in coverage
}
```

## Viewing Results

### Terminal Summary

```powershell
cargo llvm-cov --all-features --workspace --summary-only
```

Output:
```
Filename                      Regions    Missed Regions     Cover   Functions  Missed Functions  Executed       Lines      Missed Lines     Cover    Branches   Missed Branches     Cover
-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
src/yahoo_error.rs                234                15    93.59%          45                 2    95.56%         456                12    97.37%           0                 0         -
src/rate_limiter.rs               187                 8    95.72%          23                 1    95.65%         312                 5    98.40%           0                 0         -
...
-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------
TOTAL                            2145               123    94.26%         389                14    96.40%        4523               178    96.07%           0                 0         -
```

## Next Steps

1. **Run initial coverage**: `cargo llvm-cov --all-features --workspace --html`
2. **Review HTML report**: Identify uncovered code
3. **Add tests**: Write tests for uncovered paths
4. **Iterate**: Repeat until >90% coverage achieved
5. **Automate**: Add coverage checks to CI/CD pipeline

## Resources

- [cargo-llvm-cov documentation](https://github.com/taiki-e/cargo-llvm-cov)
- [Rust Code Coverage Book](https://doc.rust-lang.org/rustc/instrument-coverage.html)
- [Codecov Rust Guide](https://docs.codecov.com/docs/supported-languages#rust)
