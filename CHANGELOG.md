# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed
- Replaced language bindings directory with comprehensive FFI documentation suite
- Enhanced decimal module with compatibility helpers for f64 type
- Improved type system documentation and error messages

### Fixed
- Fixed f64 comparison methods throughout codebase (`.cmp()` → `.partial_cmp()`)
- Cleaned up all compiler warnings
- Added proper `#[allow]` attributes for intentionally unused code

### Removed
- Removed `bindings/` directory containing Python, JavaScript, Go, and Ruby bindings (Phase 10.2)
  - Replaced with FFI documentation guide for community-maintained bindings
  - See `docs/BINDINGS_REMOVAL_SUMMARY.md` for migration details

## [0.1.0] - TBD

### Added
- Complete Yahoo Finance API client with async/await support
- Historical data fetching with configurable date ranges
- Real-time quote streaming capabilities
- Advanced search functionality (ticker, news, trending symbols)
- Options data retrieval and analysis
- Financial statements and company information
- Dividend and split history
- Market hours checking for global exchanges
- Screener functionality with customizable filters
- Comprehensive error handling with `YahooError` type
- Rate limiting and retry logic with exponential backoff
- Circuit breaker pattern for fault tolerance
- Response caching with configurable TTL
- Request deduplication to prevent duplicate API calls
- Tracing and observability with OpenTelemetry support
- Prometheus metrics integration (optional feature)
- Builder pattern for flexible client configuration
- Connection pooling for optimal performance
- Middleware support for custom request/response processing
- WebSocket support for real-time data streaming
- TLS/SSL support for secure connections
- Proxy configuration support
- Cookie persistence across requests
- User agent customization
- Timeout configuration (connect, read, write)
- Request/response logging
- Data validation and integrity checks
- Export functionality (CSV, JSON formats)
- Comprehensive documentation (20,000+ lines)
- 145 unit and integration tests
- 20+ working examples
- FFI integration guide (2,400+ lines)

### Features
- `decimal` - Enable rust_decimal for high-precision decimal arithmetic (optional)
- `metrics` - Enable Prometheus metrics collection (optional)
- `ml` - Enable machine learning analytics features (optional, experimental)

### Documentation
- Complete API documentation with examples
- Getting started guide for contributors
- FFI integration guide for language bindings
- Architecture documentation covering all 10 development phases
- Migration guides and historical context
- Quick reference guides

### License
- Dual licensed under MIT OR Apache-2.0

[Unreleased]: https://github.com/yahoofinancelive/yliveticker/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/yahoofinancelive/yliveticker/releases/tag/v0.1.0
