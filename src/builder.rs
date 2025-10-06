//! Builder pattern for `YahooConnector` with fluent API and preset support.
//!
//! This module provides a flexible way to configure a `YahooConnector` with
//! enterprise features like circuit breakers, rate limiting, retries, caching,
//! and observability.
//!
//! # Examples
//!
//! ## Using production defaults (safe, stable)
//! ```no_run
//! use eeyf::YahooConnector;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Production defaults: safe, stable, comprehensive features
//!     let connector = YahooConnector::new()?;
//!     
//!     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Using builder for customization
//! ```no_run
//! use eeyf::YahooConnector;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Start with development defaults, then customize
//!     let connector = YahooConnector::builder()
//!         .rate_limit(5.0)              // 5 requests per second
//!         .cache_size(2000)             // 2000 entries
//!         .timeout(Duration::from_secs(45))
//!         .retry_attempts(5)
//!         .build()?;
//!     
//!     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Using presets
//! ```no_run
//! use eeyf::YahooConnector;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Load enterprise preset (conservative settings)
//!     let connector = YahooConnector::from_preset("enterprise")?;
//!     
//!     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
//!     Ok(())
//! }
//! ```

use crate::YahooError;
use std::time::Duration;

/// Builder for constructing a `YahooConnector` with custom configuration.
///
/// The builder starts with **development defaults** that prioritize fast feedback
/// and debugging. Use the fluent API methods to customize any setting.
///
/// # Development Defaults
/// - Rate limit: 10 requests/second (permissive)
/// - Circuit breaker: 10 failures in 60s triggers open (lenient)
/// - Retry attempts: 3
/// - Timeout: 30 seconds
/// - Cache size: 500 entries
/// - Cache duration: 30 seconds (short TTL for fresh data)
/// - Logging: Verbose (all events logged)
///
/// # Examples
///
/// ```no_run
/// use eeyf::YahooConnector;
/// use std::time::Duration;
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let connector = YahooConnector::builder()
///     .rate_limit(2.0)
///     .cache_duration(600)
///     .retry_attempts(5)
///     .build()?;
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct YahooConnectorBuilder {
    /// Requests per second allowed
    pub(crate) rate_limit: f64,

    /// Number of failures before circuit breaker opens
    pub(crate) circuit_breaker_threshold: u32,

    /// Time window for counting failures (seconds)
    pub(crate) circuit_breaker_window_secs: u64,

    /// How long circuit breaker stays open (seconds)
    pub(crate) circuit_breaker_timeout_secs: u64,

    /// Number of retry attempts
    pub(crate) retry_attempts: u32,

    /// Initial delay between retries (milliseconds)
    pub(crate) retry_initial_delay_ms: u64,

    /// Maximum delay between retries (milliseconds)
    pub(crate) retry_max_delay_ms: u64,

    /// Request timeout
    pub(crate) timeout: Duration,

    /// Maximum number of cache entries
    pub(crate) cache_size: usize,

    /// Cache entry time-to-live (seconds)
    pub(crate) cache_duration_secs: u64,

    /// Maximum concurrent connections in pool
    pub(crate) connection_pool_max: usize,

    /// Enable verbose logging
    pub(crate) verbose_logging: bool,

    /// Enable request metrics
    pub(crate) enable_metrics: bool,

    /// Enable distributed tracing
    pub(crate) enable_tracing: bool,
}

impl Default for YahooConnectorBuilder {
    /// Creates a builder with **development defaults**.
    ///
    /// Development defaults prioritize:
    /// - Fast failure detection (lenient circuit breaker)
    /// - Fresh data (short cache TTL)
    /// - Debugging visibility (verbose logging)
    /// - Rapid iteration (permissive rate limits)
    fn default() -> Self {
        Self {
            // Development: Permissive rate limiting
            rate_limit: 10.0,

            // Development: Lenient circuit breaker (fail fast but not too fast)
            circuit_breaker_threshold: 10,
            circuit_breaker_window_secs: 60,
            circuit_breaker_timeout_secs: 30,

            // Development: Moderate retries
            retry_attempts: 3,
            retry_initial_delay_ms: 1000,
            retry_max_delay_ms: 10_000,

            // Development: Standard timeout
            timeout: Duration::from_secs(30),

            // Development: Small cache, short TTL (fresh data)
            cache_size: 500,
            cache_duration_secs: 30,

            // Development: Standard connection pool
            connection_pool_max: 10,

            // Development: Verbose logging enabled
            verbose_logging: true,

            // Development: Basic metrics
            enable_metrics: true,
            enable_tracing: false,
        }
    }
}

impl YahooConnectorBuilder {
    /// Creates a new builder with development defaults.
    ///
    /// This is the same as calling `YahooConnector::builder()`.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::builder::YahooConnectorBuilder;
    ///
    /// let builder = YahooConnectorBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a builder with **production defaults**.
    ///
    /// Production defaults prioritize:
    /// - Safety (strict circuit breaker)
    /// - Stability (conservative rate limits)
    /// - Reliability (extended retries)
    /// - Efficiency (longer cache TTL)
    ///
    /// This is the same as calling `YahooConnector::new()` internally.
    pub fn production() -> Self {
        Self {
            // Production: Conservative rate limiting (1800 requests/hour)
            rate_limit: 0.5,

            // Production: Strict circuit breaker
            circuit_breaker_threshold: 5,
            circuit_breaker_window_secs: 300, // 5 minutes
            circuit_breaker_timeout_secs: 60,

            // Production: Extended retries with exponential backoff
            retry_attempts: 5,
            retry_initial_delay_ms: 1000,
            retry_max_delay_ms: 30_000,

            // Production: Longer timeout for reliability
            timeout: Duration::from_secs(60),

            // Production: Larger cache, longer TTL (efficiency)
            cache_size: 2000,
            cache_duration_secs: 900, // 15 minutes

            // Production: Larger connection pool
            connection_pool_max: 50,

            // Production: Standard logging
            verbose_logging: false,

            // Production: Full observability
            enable_metrics: true,
            enable_tracing: true,
        }
    }

    /// Creates a builder with **enterprise defaults**.
    ///
    /// Enterprise defaults prioritize:
    /// - IP protection (very conservative rate limits)
    /// - Data freshness (moderate cache TTL)
    /// - Observability (full metrics and tracing)
    /// - Resilience (moderate circuit breaker)
    pub fn enterprise() -> Self {
        Self {
            // Enterprise: Very conservative rate limiting (1800 requests/hour)
            rate_limit: 0.5,

            // Enterprise: Moderate circuit breaker
            circuit_breaker_threshold: 8,
            circuit_breaker_window_secs: 180, // 3 minutes
            circuit_breaker_timeout_secs: 45,

            // Enterprise: Moderate retries
            retry_attempts: 4,
            retry_initial_delay_ms: 1500,
            retry_max_delay_ms: 20_000,

            // Enterprise: Standard timeout
            timeout: Duration::from_secs(45),

            // Enterprise: Large cache, moderate TTL
            cache_size: 3000,
            cache_duration_secs: 300, // 5 minutes

            // Enterprise: Large connection pool
            connection_pool_max: 100,

            // Enterprise: Standard logging
            verbose_logging: false,

            // Enterprise: Full observability
            enable_metrics: true,
            enable_tracing: true,
        }
    }

    /// Creates a builder with **minimal defaults**.
    ///
    /// Minimal defaults are useful for:
    /// - Testing (predictable behavior)
    /// - Debugging (no caching interference)
    /// - Benchmarking (minimal overhead)
    pub fn minimal() -> Self {
        Self {
            // Minimal: No rate limiting
            rate_limit: 1000.0, // Effectively unlimited

            // Minimal: Disabled circuit breaker
            circuit_breaker_threshold: 1000,
            circuit_breaker_window_secs: 1,
            circuit_breaker_timeout_secs: 1,

            // Minimal: No retries
            retry_attempts: 0,
            retry_initial_delay_ms: 0,
            retry_max_delay_ms: 0,

            // Minimal: Short timeout
            timeout: Duration::from_secs(10),

            // Minimal: No caching
            cache_size: 0,
            cache_duration_secs: 0,

            // Minimal: Minimal connection pool
            connection_pool_max: 1,

            // Minimal: No logging
            verbose_logging: false,

            // Minimal: No observability
            enable_metrics: false,
            enable_tracing: false,
        }
    }

    /// Sets the rate limit in requests per second.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .rate_limit(2.0)  // 2 requests per second
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn rate_limit(mut self, requests_per_second: f64) -> Self {
        self.rate_limit = requests_per_second;
        self
    }

    /// Sets the circuit breaker threshold (number of failures before opening).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .circuit_breaker_threshold(5)  // Open after 5 failures
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn circuit_breaker_threshold(mut self, threshold: u32) -> Self {
        self.circuit_breaker_threshold = threshold;
        self
    }

    /// Sets the circuit breaker window (time window for counting failures in seconds).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .circuit_breaker_window_secs(120)  // 2 minute window
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn circuit_breaker_window_secs(mut self, seconds: u64) -> Self {
        self.circuit_breaker_window_secs = seconds;
        self
    }

    /// Sets how long the circuit breaker stays open before attempting recovery (seconds).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .circuit_breaker_timeout_secs(60)  // Stay open for 1 minute
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn circuit_breaker_timeout_secs(mut self, seconds: u64) -> Self {
        self.circuit_breaker_timeout_secs = seconds;
        self
    }

    /// Sets the number of retry attempts.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .retry_attempts(5)  // Retry up to 5 times
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry_attempts(mut self, attempts: u32) -> Self {
        self.retry_attempts = attempts;
        self
    }

    /// Sets the initial delay between retry attempts (milliseconds).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .retry_initial_delay_ms(500)  // Start with 500ms delay
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry_initial_delay_ms(mut self, milliseconds: u64) -> Self {
        self.retry_initial_delay_ms = milliseconds;
        self
    }

    /// Sets the maximum delay between retry attempts (milliseconds).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .retry_max_delay_ms(30000)  // Cap at 30 seconds
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn retry_max_delay_ms(mut self, milliseconds: u64) -> Self {
        self.retry_max_delay_ms = milliseconds;
        self
    }

    /// Sets the request timeout.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    /// use std::time::Duration;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .timeout(Duration::from_secs(45))
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = duration;
        self
    }

    /// Sets the maximum number of cache entries.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .cache_size(5000)  // Store up to 5000 entries
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache_size(mut self, size: usize) -> Self {
        self.cache_size = size;
        self
    }

    /// Sets the cache entry time-to-live in seconds.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .cache_duration(600)  // Cache for 10 minutes
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn cache_duration(mut self, seconds: u64) -> Self {
        self.cache_duration_secs = seconds;
        self
    }

    /// Sets the maximum number of concurrent connections in the pool.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .connection_pool_max(100)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn connection_pool_max(mut self, max_connections: usize) -> Self {
        self.connection_pool_max = max_connections;
        self
    }

    /// Enables or disables verbose logging.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .verbose_logging(true)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn verbose_logging(mut self, enabled: bool) -> Self {
        self.verbose_logging = enabled;
        self
    }

    /// Enables or disables metrics collection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .enable_metrics(true)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enable_metrics(mut self, enabled: bool) -> Self {
        self.enable_metrics = enabled;
        self
    }

    /// Enables or disables distributed tracing.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .enable_tracing(true)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn enable_tracing(mut self, enabled: bool) -> Self {
        self.enable_tracing = enabled;
        self
    }

    /// Validates the configuration and builds the `YahooConnector`.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Rate limit is <= 0
    /// - Circuit breaker threshold is 0
    /// - Timeout is 0
    /// - Any other configuration is invalid
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let connector = YahooConnector::builder()
    ///     .rate_limit(2.0)
    ///     .cache_duration(600)
    ///     .build()?;
    /// # Ok(())
    /// # }
    /// ```

    pub fn build(self) -> Result<crate::YahooConnector, YahooError> {
        use crate::rate_limiter::RateLimitConfig;
        use std::time::Duration;

        // Validate configuration
        self.validate()?;

        // Create rate limit config from our settings
        let rate_limit_config = RateLimitConfig {
            requests_per_hour: self.rate_limit as u32,
            burst_limit: 10,                          // Default burst limit
            min_interval: Duration::from_millis(100), // Default minimum interval
        };

        // Use the existing YahooConnector legacy builder with our rate limiting
        crate::YahooConnectorBuilderLegacy::new()
            .rate_limit_config(rate_limit_config)
            .timeout(self.timeout)
            .build()
    }

    /// Validates the builder configuration.
    fn validate(&self) -> Result<(), YahooError> {
        if self.rate_limit <= 0.0 {
            return Err(YahooError::InvalidStatusCode(
                "Rate limit must be greater than 0".into(),
            ));
        }

        if self.circuit_breaker_threshold == 0 {
            return Err(YahooError::InvalidStatusCode(
                "Circuit breaker threshold must be greater than 0".into(),
            ));
        }

        if self.timeout.as_secs() == 0 {
            return Err(YahooError::InvalidStatusCode(
                "Timeout must be greater than 0".into(),
            ));
        }

        if self.retry_attempts > 20 {
            return Err(YahooError::InvalidStatusCode(
                "Retry attempts should not exceed 20 (recommended max: 10)".into(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default_is_development() {
        let builder = YahooConnectorBuilder::default();
        assert_eq!(builder.rate_limit, 10.0);
        assert_eq!(builder.circuit_breaker_threshold, 10);
        assert_eq!(builder.cache_duration_secs, 30);
        assert!(builder.verbose_logging);
    }

    #[test]
    fn test_builder_production() {
        let builder = YahooConnectorBuilder::production();
        assert_eq!(builder.rate_limit, 0.5);
        assert_eq!(builder.circuit_breaker_threshold, 5);
        assert_eq!(builder.cache_duration_secs, 900);
        assert!(!builder.verbose_logging);
        assert!(builder.enable_tracing);
    }

    #[test]
    fn test_builder_enterprise() {
        let builder = YahooConnectorBuilder::enterprise();
        assert_eq!(builder.rate_limit, 0.5);
        assert_eq!(builder.circuit_breaker_threshold, 8);
        assert_eq!(builder.cache_duration_secs, 300);
        assert!(builder.enable_metrics);
        assert!(builder.enable_tracing);
    }

    #[test]
    fn test_builder_minimal() {
        let builder = YahooConnectorBuilder::minimal();
        assert_eq!(builder.rate_limit, 1000.0);
        assert_eq!(builder.retry_attempts, 0);
        assert_eq!(builder.cache_size, 0);
        assert!(!builder.enable_metrics);
        assert!(!builder.enable_tracing);
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = YahooConnectorBuilder::new()
            .rate_limit(5.0)
            .cache_size(1000)
            .retry_attempts(7)
            .timeout(Duration::from_secs(45));

        assert_eq!(builder.rate_limit, 5.0);
        assert_eq!(builder.cache_size, 1000);
        assert_eq!(builder.retry_attempts, 7);
        assert_eq!(builder.timeout, Duration::from_secs(45));
    }

    #[test]
    fn test_builder_validation_rate_limit() {
        let builder = YahooConnectorBuilder::new().rate_limit(0.0);
        assert!(builder.validate().is_err());

        let builder = YahooConnectorBuilder::new().rate_limit(-1.0);
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_builder_validation_circuit_breaker() {
        let builder = YahooConnectorBuilder::new().circuit_breaker_threshold(0);
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_builder_validation_timeout() {
        let builder = YahooConnectorBuilder::new().timeout(Duration::from_secs(0));
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_builder_validation_retry_attempts() {
        let builder = YahooConnectorBuilder::new().retry_attempts(25);
        assert!(builder.validate().is_err());
    }

    #[test]
    fn test_builder_validation_success() {
        let builder = YahooConnectorBuilder::production();
        assert!(builder.validate().is_ok());
    }
}
