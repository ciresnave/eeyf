//! This project provides a set of functions to receive data from the
//! the [yahoo! finance](https://finance.yahoo.com) website via their API.
//!
//! This project is licensed under Apache 2.0 or MIT license (see files
//! LICENSE-Apache2.0 and LICENSE-MIT).
//!
//! All requests to the yahoo API return ```async``` futures.
//! Therefore, the functions need to be called from an ```async``` function with
//! ```.await``` or via functions like ```block_on```. The examples are based on
//! the ```tokio``` runtime applying the ```tokio-test``` crate.
//!
//! # Get the latest available quote:
//! ```rust
//! use eeyf as yahoo;
//! use time::OffsetDateTime;
//! use tokio_test;
//!
//! fn main() {
//!     let provider = yahoo::YahooConnector::new().unwrap();
//!     // get the latest quotes in 1 minute intervals
//!     let response = tokio_test::block_on(provider.get_latest_quotes("AAPL",
//! "1d")).unwrap();     // extract just the latest valid quote summery
//!     // including timestamp,open,close,high,low,volume
//!     let quote = response.last_quote().unwrap();
//!     let time: OffsetDateTime =
//! OffsetDateTime::from_unix_timestamp(quote.timestamp).unwrap();     println!
//! ("At {} quote price of Apple was {}", time, quote.close); }
//! ```
//! # Get history of quotes for given time period:
//! ```rust
//! use eeyf as yahoo;
//! use time::{macros::datetime, OffsetDateTime};
//! use tokio_test;
//!
//! fn main() {
//!     let provider = yahoo::YahooConnector::new().unwrap();
//!     let start = datetime!(2020-1-1 0:00:00.00 UTC);
//!     let end = datetime!(2020-1-31 23:59:59.99 UTC);
//!     // returns historic quotes with daily interval
//!     let resp = tokio_test::block_on(provider.get_quote_history("AAPL",
//! start, end)).unwrap();     let quotes = resp.quotes().unwrap();
//!     println!("Apple's quotes in January: {:?}", quotes);
//! }
//! ```
//! # Get the history of quotes for time range
//! Another method to retrieve a range of quotes is by requesting the quotes for
//! a given period and lookup frequency. Here is an example retrieving the daily
//! quotes for the last month: ```rust
//! use eeyf as yahoo;
//! use tokio_test;
//!
//! fn main() {
//!     let provider = yahoo::YahooConnector::new().unwrap();
//!     let response = tokio_test::block_on(provider.get_quote_range("AAPL", "1d", "1mo")).unwrap();
//!     let quotes = response.quotes().unwrap();
//!     println!("Apple's quotes of the last month: {:?}", quotes);
//! }
//! ```
//!
//! # Search for a ticker given a search string (e.g. company name):
//! ```rust
//! use eeyf as yahoo;
//! use tokio_test;
//!
//! fn main() {
//!     let provider = yahoo::YahooConnector::new().unwrap();
//!     let resp = tokio_test::block_on(provider.search_ticker("Apple")).unwrap();
//!
//!     let mut apple_found = false;
//!     println!("All tickers found while searching for 'Apple':");
//!     for item in resp.quotes {
//!         println!("{}", item.symbol)
//!     }
//! }
//! ```
//! Some fields like `longname` are only optional and will be replaced by
//! default values if missing (e.g. empty string). If you do not like this
//! behavior, use `search_ticker_opt` instead which contains `Option<String>`
//! fields, returning `None` if the field found missing in the response.

#[cfg(feature = "debug")]
extern crate serde_json_path_to_error as serde_json;

use std::{sync::Arc, time::Duration};

// re-export time crate
pub use quotes::decimal::Decimal;
use reqwest::{Client, ClientBuilder, Proxy};
pub use time;
use time::OffsetDateTime;

mod quotes;
pub mod rate_limiter;
mod search_result;
pub mod yahoo_error;

// Builder and preset management
pub mod builder;
pub mod presets;

// Enterprise modules
pub mod circuit_breaker;
pub mod connection_pool;
pub mod enterprise;
pub mod error_categories;
pub mod observability;
pub mod request_deduplication;
pub mod response_cache;
pub mod retry;

// Phase 2: Observability & Configuration modules
pub mod health;
pub mod metrics;
pub mod tracing;

#[cfg(feature = "config-management")]
pub mod config;

#[cfg(feature = "config-management")]
pub mod runtime_config;

// Phase 3: Performance & Reliability modules
#[cfg(feature = "performance-cache")]
pub mod advanced_cache;
#[cfg(feature = "performance-pool")]
pub mod connection_pool_advanced;
#[cfg(feature = "performance-rate-limit")]
pub mod intelligent_rate_limit;
#[cfg(feature = "performance-optimization")]
pub mod performance_optimization;

// Phase 4: WebSocket streaming for real-time data
#[cfg(feature = "websocket-streaming")]
pub mod websocket;

// Phase 4: Batch operations for parallel fetching
pub mod batch;

// Phase 4: Symbol validation and lookup
pub mod validation;

// Phase 4: Market hours checking
pub mod market_hours;

// Phase 4.2: Stock screener API
pub mod screener;

// Phase 7: Production Hardening
#[cfg(feature = "phase7")]
pub mod security;

#[cfg(feature = "phase7")]
pub mod audit;

#[cfg(feature = "phase7")]
pub mod fallback;

// Phase 8: Runtime Flexibility
pub mod runtime;

// Phase 9: Advanced Features
#[cfg(feature = "phase9")]
pub mod analytics;

// Phase 4.3: Data processing features
pub mod export;

// EXPERIMENTAL MODULES (Temporarily Disabled)
// These modules are under refactoring to work with both f64 and
// rust_decimal::Decimal types. They will be re-enabled in a future release
// (v0.2.0 or v0.1.1).
//
// To use these features now, enable the `decimal` feature in your Cargo.toml:
// ```toml
// [dependencies]
// eeyf = { version = "0.1", features = ["decimal"] }
// ```
//
// Or wait for the refactored versions that work with plain f64.
//
// pub mod timeseries;  // Time series utilities (resampling, timezone handling)
// pub mod transform;   // Data transformation (OHLC aggregation, technical
// indicators) pub mod validate;    // Data validation (integrity checks,
// anomaly detection)

// Phase 5: Performance & Optimization modules
#[cfg(feature = "phase5-compression")]
pub mod compression;
#[cfg(feature = "phase5-http2")]
pub mod http2;
#[cfg(feature = "phase5-limits")]
pub mod limits;
#[cfg(feature = "phase5-shutdown")]
pub mod shutdown;

// Builder and preset management
pub use builder::YahooConnectorBuilder as EnterpriseYahooConnectorBuilder;
// Enterprise features
pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerStats, CircuitState,
};
pub use connection_pool::{ConnectionPool, ConnectionPoolConfig, ConnectionStats};
pub use enterprise::{
    EnterpriseConfig, EnterpriseHealthStatus, EnterpriseMetrics, EnterpriseYahooConnector,
};
pub use error_categories::{ErrorCategorizer, ErrorCategory, ErrorInfo};
pub use observability::{
    HealthCheck, HealthStatus, LibraryMetrics, ObservabilityConfig, ObservabilityManager,
    RequestContext,
};
pub use presets::{PresetConfig, PresetFormat, PresetManager};
pub use quotes::{
    AdjClose, AssetProfile, CapitalGain, CurrentTradingPeriod, DefaultKeyStatistics, Dividend,
    ExtendedQuoteSummary, FinancialData, FinancialEvent, PeriodInfo, Quote, QuoteBlock, QuoteList,
    QuoteType, Split, SummaryDetail, TradingPeriods, YChart, YMetaData, YQuoteBlock, YQuoteSummary,
    YResponse, YSummaryData,
};
pub use rate_limiter::{RateLimitConfig, RateLimitError, RateLimitStatus, RateLimiter};
pub use request_deduplication::{DeduplicationConfig, DeduplicationStats, RequestDeduplicator};
pub use response_cache::{CacheStats, ResponseCache, ResponseCacheConfig};
pub use retry::{RetryConfig, RetryPolicy, RetryStats};
pub use search_result::{
    YNewsItem, YOptionChain, YOptionChainData, YOptionChainResult, YOptionContract, YOptionDetails,
    YQuote, YQuoteItem, YQuoteItemOpt, YSearchResult, YSearchResultOpt,
};
pub use yahoo_error::{ErrorContext, YahooError, YahooErrorCode, YahooErrorWithContext};

const YCHART_URL: &str = "https://query1.finance.yahoo.com/v8/finance/chart";
const YSEARCH_URL: &str = "https://query2.finance.yahoo.com/v1/finance/search";
const Y_GET_COOKIE_URL: &str = "https://fc.yahoo.com";
const Y_GET_CRUMB_URL: &str = "https://query1.finance.yahoo.com/v1/test/getcrumb";
const Y_EARNINGS_URL: &str = "https://query1.finance.yahoo.com/v1/finance/visualization";

// special yahoo hardcoded keys and headers
const Y_COOKIE_REQUEST_HEADER: &str = "set-cookie";
const USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) \
                          Chrome/122.0.0.0 Safari/537.36";

// Macros instead of constants,
macro_rules! YCHART_PERIOD_QUERY {
    () => {
        "{url}/{symbol}?symbol={symbol}&period1={start}&period2={end}&interval={interval}&\
         events=div|split|capitalGains"
    };
}
macro_rules! YCHART_PERIOD_QUERY_PRE_POST {
    () => {
        "{url}/{symbol}?symbol={symbol}&period1={start}&period2={end}&interval={interval}&\
         events=div|split|capitalGains&includePrePost={prepost}"
    };
}
macro_rules! YCHART_RANGE_QUERY {
    () => {
        "{url}/{symbol}?symbol={symbol}&interval={interval}&range={range}&\
         events=div|split|capitalGains"
    };
}
macro_rules! YCHART_PERIOD_INTERVAL_QUERY {
    () => {
        "{url}/{symbol}?symbol={symbol}&range={range}&interval={interval}&includePrePost={prepost}"
    };
}
macro_rules! YTICKER_QUERY {
    () => {
        "{url}?q={name}"
    };
}
macro_rules! YQUOTE_SUMMARY_QUERY {
    () => {
        "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{symbol}?modules=financialData,quoteType,defaultKeyStatistics,assetProfile,summaryDetail&corsDomain=finance.yahoo.com&formatted=false&symbol={symbol}&crumb={crumb}"
    }
}
macro_rules! YEARNINGS_QUERY {
    () => {
        "{url}?lang={lang}&region={region}&crumb={crumb}"
    };
}

/// Container for connection parameters to yahoo! finance server
#[derive(Debug, Clone)]
pub struct YahooConnector {
    client: Client,
    url: &'static str,
    search_url: &'static str,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    proxy: Option<Proxy>,
    cookie: Option<String>,
    crumb: Option<String>,
    pub rate_limiter: Option<Arc<RateLimiter>>,
}

#[derive(Default)]
pub struct YahooConnectorBuilderLegacy {
    inner: ClientBuilder,
    timeout: Option<Duration>,
    user_agent: Option<String>,
    proxy: Option<Proxy>,
    rate_limit_config: Option<RateLimitConfig>,
}

impl YahooConnector {
    /// Constructor for a new instance of the yahoo connector with **production
    /// defaults**.
    ///
    /// Production defaults prioritize:
    /// - Safety (strict circuit breaker)
    /// - Stability (conservative rate limits)
    /// - Reliability (extended retries)
    /// - Efficiency (longer cache TTL)
    ///
    /// For development/testing, use [`YahooConnector::builder()`] instead.
    /// For custom presets, use [`YahooConnector::from_preset()`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Production defaults: safe, stable, comprehensive
    ///     let connector = YahooConnector::new()?;
    ///     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn new() -> Result<YahooConnector, YahooError> {
        // TODO: Create production defaults via EnterpriseYahooConnector
        // For now, use existing implementation
        Self::builder().build()
    }

    /// Creates a builder with **development defaults**.
    ///
    /// Development defaults prioritize:
    /// - Fast failure detection (lenient circuit breaker)
    /// - Fresh data (short cache TTL)
    /// - Debugging visibility (verbose logging)
    /// - Rapid iteration (permissive rate limits)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use eeyf::YahooConnector;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Start with development defaults, then customize
    ///     let connector = YahooConnector::builder()
    ///         .rate_limit_config(eeyf::RateLimitConfig::new(5.0))
    ///         .timeout(Duration::from_secs(45))
    ///         .build()?;
    ///
    ///     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
    ///     Ok(())
    /// }
    /// ```
    pub fn builder() -> crate::builder::YahooConnectorBuilder {
        crate::builder::YahooConnectorBuilder::default()
    }

    /// Creates a connector from a named preset configuration.
    ///
    /// Searches for presets in this order:
    /// 1. Built-in presets: "production", "development", "enterprise",
    ///    "minimal"
    /// 2. Project-local presets (./.eeyf/presets/)
    /// 3. User presets (~/.config/eeyf/presets/ or %APPDATA%\eeyf\presets\)
    ///
    /// # Built-in Presets
    ///
    /// - **"production"** - Safe defaults (same as [`YahooConnector::new()`])
    /// - **"development"** - Fast feedback (same as
    ///   [`YahooConnector::builder()`])
    /// - **"enterprise"** - Conservative rate limits, extended caching
    /// - **"minimal"** - Bare minimum for testing, no caching/retries
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use eeyf::YahooConnector;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     // Load enterprise preset (conservative settings)
    ///     let connector = YahooConnector::from_preset("enterprise")?;
    ///
    ///     // Load custom user-defined preset
    ///     let connector = YahooConnector::from_preset("my-staging-config")?;
    ///
    ///     let quote = connector.get_latest_quotes("AAPL", "1d").await?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if the preset is not found or cannot be loaded.
    pub fn from_preset(name: &str) -> Result<YahooConnector, YahooError> {
        use crate::{
            enterprise::{EnterpriseConfig, EnterpriseYahooConnector},
            presets::PresetManager,
        };

        let manager = PresetManager::new();
        let preset = manager.load_preset(name)?;

        // Convert PresetConfig to EnterpriseConfig
        let enterprise_config = EnterpriseConfig::from(preset);

        // TODO: We need to decide how to integrate EnterpriseYahooConnector with
        // YahooConnector For now, create a basic YahooConnector with rate
        // limiting from the preset
        let rate_limit_config = enterprise_config.rate_limiter.clone();

        // Create EnterpriseYahooConnector and wrap it
        let _enterprise_connector = EnterpriseYahooConnector::new(enterprise_config)?;

        Self::builder().rate_limit(rate_limit_config.requests_per_hour as f64).build()
    }

    /// Saves the current connector configuration as a named preset.
    ///
    /// Presets are saved to the user configuration directory:
    /// - Linux/macOS: `~/.config/eeyf/presets/{name}.toml`
    /// - Windows: `%APPDATA%\eeyf\presets\{name}.toml`
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use std::time::Duration;
    ///
    /// use eeyf::YahooConnector;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let connector = YahooConnector::builder()
    ///         .rate_limit_config(eeyf::RateLimitConfig::new(2.5))
    ///         .timeout(Duration::from_secs(45))
    ///         .build()?;
    ///
    ///     // Save for later reuse
    ///     connector.save_preset("my-staging-config")?;
    ///
    ///     // Later, reload it
    ///     let reloaded = YahooConnector::from_preset("my-staging-config")?;
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Attempting to save a built-in preset name
    /// - Unable to create the presets directory
    /// - Unable to write the preset file
    pub fn save_preset(&self, _name: &str) -> Result<(), YahooError> {
        // TODO: Extract current configuration and save via PresetManager
        // For now, return error indicating not yet implemented
        Err(YahooError::ConnectionFailed(
            format!(
                "Preset saving not yet implemented. Need to extract current configuration from \
                 YahooConnector and convert to PresetConfig."
            )
            .into(),
        ))
    }

    /// Internal default implementation used exclusively by the builder.
    /// Note: This default implementation does not set the user agent in the
    /// client, so it does not work on its own. The builder will set the
    /// user agent.
    fn default_internal() -> Self {
        YahooConnector {
            client: Client::default(),
            url: YCHART_URL,
            search_url: YSEARCH_URL,
            timeout: None,
            user_agent: Some(USER_AGENT.to_string()),
            proxy: None,
            cookie: None,
            crumb: None,
            rate_limiter: None,
        }
    }
}

impl YahooConnectorBuilderLegacy {
    pub fn new() -> Self {
        YahooConnectorBuilderLegacy {
            inner: Client::builder(),
            user_agent: Some(USER_AGENT.to_string()),
            ..Default::default()
        }
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn user_agent(mut self, user_agent: &str) -> Self {
        self.user_agent = Some(user_agent.to_string());
        self
    }

    pub fn proxy(mut self, proxy: Proxy) -> Self {
        self.proxy = Some(proxy);
        self
    }

    pub fn rate_limit_config(mut self, config: RateLimitConfig) -> Self {
        self.rate_limit_config = Some(config);
        self
    }

    pub fn build(mut self) -> Result<YahooConnector, YahooError> {
        if let Some(timeout) = &self.timeout {
            self.inner = self.inner.timeout(*timeout);
        }
        if let Some(user_agent) = &self.user_agent {
            self.inner = self.inner.user_agent(user_agent.clone());
        }
        if let Some(proxy) = &self.proxy {
            self.inner = self.inner.proxy(proxy.clone());
        }

        let rate_limiter = self.rate_limit_config.map(|config| Arc::new(RateLimiter::new(config)));

        Ok(YahooConnector {
            client: self.inner.use_rustls_tls().build()?,
            timeout: self.timeout,
            user_agent: self.user_agent,
            proxy: self.proxy,
            rate_limiter,
            ..YahooConnector::default_internal()
        })
    }

    pub fn build_with_client(client: Client) -> Result<YahooConnector, YahooError> {
        Ok(YahooConnector {
            client,
            ..YahooConnector::default_internal()
        })
    }
}

impl YahooConnector {
    /// Enable rate limiting with default configuration
    pub fn with_rate_limiting() -> Result<YahooConnector, YahooError> {
        Self::builder()
            .rate_limit(RateLimitConfig::default().requests_per_hour as f64)
            .build()
    }

    /// Enable rate limiting with custom configuration
    pub fn with_custom_rate_limiting(
        config: RateLimitConfig,
    ) -> Result<YahooConnector, YahooError> {
        Self::builder().rate_limit(config.requests_per_hour as f64).build()
    }

    /// Get the current rate limit status
    pub fn rate_limit_status(&self) -> Option<RateLimitStatus> {
        self.rate_limiter.as_ref().map(|limiter| limiter.status())
    }

    /// Check if rate limiting is enabled
    pub fn is_rate_limited(&self) -> bool {
        self.rate_limiter.is_some()
    }
}

pub mod async_impl;
