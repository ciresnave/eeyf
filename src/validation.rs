//! Symbol validation and lookup using Yahoo Finance search API.
//!
//! This module provides utilities for validating stock symbols before making requests,
//! suggesting corrections for misspelled symbols, and retrieving symbol metadata.
//!
//! # Examples
//!
//! ```no_run
//! use eeyf::{YahooConnector, validation::SymbolValidator};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = YahooConnector::new()?;
//!     let validator = SymbolValidator::new(&provider);
//!     
//!     // Validate a symbol
//!     let result = validator.validate("AAPL").await?;
//!     if result.is_valid {
//!         println!("✓ AAPL is a valid symbol");
//!         println!("  Exchange: {}", result.exchange.unwrap());
//!         println!("  Type: {}", result.quote_type.unwrap());
//!     }
//!     
//!     // Find similar symbols
//!     let suggestions = validator.suggest("APPL").await?; // typo
//!     for symbol in suggestions {
//!         println!("Did you mean: {} ({})", symbol.symbol, symbol.name);
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::{YahooConnector, YahooError};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Result of symbol validation
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// The symbol that was validated
    pub symbol: String,
    /// Whether the symbol is valid
    pub is_valid: bool,
    /// Exchange where the symbol trades (if valid)
    pub exchange: Option<String>,
    /// Type of security (EQUITY, ETF, CRYPTOCURRENCY, etc.)
    pub quote_type: Option<String>,
    /// Full name of the security
    pub name: Option<String>,
    /// Short name of the security
    pub short_name: Option<String>,
    /// When this validation result was cached
    pub cached_at: Instant,
}

impl ValidationResult {
    /// Create a new validation result for an invalid symbol
    pub fn invalid(symbol: String) -> Self {
        Self {
            symbol,
            is_valid: false,
            exchange: None,
            quote_type: None,
            name: None,
            short_name: None,
            cached_at: Instant::now(),
        }
    }

    /// Create a new validation result for a valid symbol
    pub fn valid(
        symbol: String,
        exchange: String,
        quote_type: String,
        name: String,
        short_name: String,
    ) -> Self {
        Self {
            symbol,
            is_valid: true,
            exchange: Some(exchange),
            quote_type: Some(quote_type),
            name: Some(name),
            short_name: Some(short_name),
            cached_at: Instant::now(),
        }
    }

    /// Check if this cached result is still fresh
    pub fn is_fresh(&self, ttl: Duration) -> bool {
        self.cached_at.elapsed() < ttl
    }
}

/// Symbol suggestion from search results
#[derive(Debug, Clone)]
pub struct SymbolSuggestion {
    /// Symbol ticker
    pub symbol: String,
    /// Full name
    pub name: String,
    /// Short name
    pub short_name: String,
    /// Exchange
    pub exchange: String,
    /// Quote type
    pub quote_type: String,
    /// Search score (relevance)
    pub score: f64,
}

/// Configuration for symbol validator
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Cache TTL for validation results (default: 1 hour)
    pub cache_ttl: Duration,
    /// Maximum number of cached entries (default: 10,000)
    pub max_cache_size: usize,
    /// Maximum suggestions to return (default: 5)
    pub max_suggestions: usize,
    /// Minimum score threshold for suggestions (0.0-1.0, default: 0.1)
    pub min_score: f64,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            cache_ttl: Duration::from_secs(3600), // 1 hour
            max_cache_size: 10_000,
            max_suggestions: 5,
            min_score: 0.1,
        }
    }
}

impl ValidatorConfig {
    /// Create a new validator config with custom cache TTL
    pub fn with_cache_ttl(mut self, ttl: Duration) -> Self {
        self.cache_ttl = ttl;
        self
    }

    /// Set maximum cache size
    pub fn with_max_cache_size(mut self, size: usize) -> Self {
        self.max_cache_size = size;
        self
    }

    /// Set maximum number of suggestions
    pub fn with_max_suggestions(mut self, max: usize) -> Self {
        self.max_suggestions = max;
        self
    }

    /// Set minimum score threshold
    pub fn with_min_score(mut self, score: f64) -> Self {
        self.min_score = score.max(0.0).min(1.0);
        self
    }
}

/// Symbol validator with caching
pub struct SymbolValidator<'a> {
    connector: &'a YahooConnector,
    cache: Arc<DashMap<String, ValidationResult>>,
    config: ValidatorConfig,
}

impl<'a> SymbolValidator<'a> {
    /// Create a new symbol validator with default configuration
    pub fn new(connector: &'a YahooConnector) -> Self {
        Self::with_config(connector, ValidatorConfig::default())
    }

    /// Create a new symbol validator with custom configuration
    pub fn with_config(connector: &'a YahooConnector, config: ValidatorConfig) -> Self {
        Self {
            connector,
            cache: Arc::new(DashMap::new()),
            config,
        }
    }

    /// Validate a single symbol
    ///
    /// Returns cached result if available and fresh, otherwise performs a search.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use eeyf::{YahooConnector, validation::SymbolValidator};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = YahooConnector::new()?;
    /// let validator = SymbolValidator::new(&provider);
    ///
    /// let result = validator.validate("AAPL").await?;
    /// if result.is_valid {
    ///     println!("Valid symbol: {}", result.symbol);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn validate(&self, symbol: &str) -> Result<ValidationResult, YahooError> {
        let symbol_upper = symbol.to_uppercase();

        // Check cache first
        if let Some(cached) = self.cache.get(&symbol_upper) {
            if cached.is_fresh(self.config.cache_ttl) {
                return Ok(cached.clone());
            }
        }

        // Perform search
        let search_result = self.connector.search_ticker(&symbol_upper).await?;

        // Look for exact match
        for quote in &search_result.quotes {
            if quote.symbol.to_uppercase() == symbol_upper {
                let result = ValidationResult::valid(
                    quote.symbol.clone(),
                    quote.exchange.clone(),
                    quote.quote_type.clone(),
                    quote.long_name.clone(),
                    quote.short_name.clone(),
                );

                self.add_to_cache(result.clone());
                return Ok(result);
            }
        }

        // No exact match found
        let result = ValidationResult::invalid(symbol_upper);
        self.add_to_cache(result.clone());
        Ok(result)
    }

    /// Validate multiple symbols in batch
    ///
    /// Returns a map of symbol -> ValidationResult
    pub async fn validate_many(
        &self,
        symbols: &[&str],
    ) -> Result<Vec<(String, ValidationResult)>, YahooError> {
        let mut results = Vec::with_capacity(symbols.len());

        for symbol in symbols {
            let result = self.validate(symbol).await?;
            results.push((symbol.to_string(), result));
        }

        Ok(results)
    }

    /// Suggest similar symbols for a query (useful for typos)
    ///
    /// Returns suggestions sorted by relevance score.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use eeyf::{YahooConnector, validation::SymbolValidator};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = YahooConnector::new()?;
    /// let validator = SymbolValidator::new(&provider);
    ///
    /// // User typed "APPL" instead of "AAPL"
    /// let suggestions = validator.suggest("APPL").await?;
    /// for symbol in suggestions {
    ///     println!("Did you mean: {} ({})", symbol.symbol, symbol.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn suggest(&self, query: &str) -> Result<Vec<SymbolSuggestion>, YahooError> {
        let search_result = self.connector.search_ticker(query).await?;

        let mut suggestions: Vec<SymbolSuggestion> = search_result
            .quotes
            .into_iter()
            .filter(|quote| quote.score >= self.config.min_score)
            .map(|quote| SymbolSuggestion {
                symbol: quote.symbol,
                name: quote.long_name,
                short_name: quote.short_name,
                exchange: quote.exchange,
                quote_type: quote.quote_type,
                score: quote.score,
            })
            .collect();

        // Sort by score (highest first)
        suggestions.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        // Limit to max_suggestions
        suggestions.truncate(self.config.max_suggestions);

        Ok(suggestions)
    }

    /// Search for symbols by company name
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use eeyf::{YahooConnector, validation::SymbolValidator};
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let provider = YahooConnector::new()?;
    /// let validator = SymbolValidator::new(&provider);
    ///
    /// let results = validator.search_by_name("Apple").await?;
    /// for symbol in results {
    ///     println!("{}: {}", symbol.symbol, symbol.name);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn search_by_name(&self, name: &str) -> Result<Vec<SymbolSuggestion>, YahooError> {
        self.suggest(name).await
    }

    /// Get metadata for a symbol without validating
    ///
    /// This is useful when you know the symbol is valid but want additional info.
    pub async fn get_metadata(&self, symbol: &str) -> Result<Option<ValidationResult>, YahooError> {
        let result = self.validate(symbol).await?;
        Ok(if result.is_valid {
            Some(result)
        } else {
            None
        })
    }

    /// Check if a symbol is valid (boolean check without metadata)
    pub async fn is_valid(&self, symbol: &str) -> Result<bool, YahooError> {
        let result = self.validate(symbol).await?;
        Ok(result.is_valid)
    }

    /// Clear the validation cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let size = self.cache.len();
        let valid_count = self.cache.iter().filter(|entry| entry.is_valid).count();
        let invalid_count = size - valid_count;

        CacheStats {
            total_entries: size,
            valid_entries: valid_count,
            invalid_entries: invalid_count,
            max_size: self.config.max_cache_size,
        }
    }

    /// Add result to cache with eviction if needed
    fn add_to_cache(&self, result: ValidationResult) {
        // Evict oldest entries if cache is full
        if self.cache.len() >= self.config.max_cache_size {
            // Find oldest entry
            if let Some(oldest) = self
                .cache
                .iter()
                .min_by_key(|entry| entry.cached_at)
                .map(|entry| entry.symbol.clone())
            {
                self.cache.remove(&oldest);
            }
        }

        self.cache.insert(result.symbol.clone(), result);
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub valid_entries: usize,
    pub invalid_entries: usize,
    pub max_size: usize,
}

impl CacheStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_entries == 0 {
            0.0
        } else {
            (self.valid_entries as f64 / self.total_entries as f64) * 100.0
        }
    }

    pub fn usage_percent(&self) -> f64 {
        if self.max_size == 0 {
            0.0
        } else {
            (self.total_entries as f64 / self.max_size as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_result_fresh() {
        let result = ValidationResult::invalid("TEST".to_string());
        assert!(result.is_fresh(Duration::from_secs(3600)));

        let old_result = ValidationResult {
            cached_at: Instant::now() - Duration::from_secs(7200), // 2 hours ago
            ..ValidationResult::invalid("TEST".to_string())
        };
        assert!(!old_result.is_fresh(Duration::from_secs(3600))); // 1 hour TTL
    }

    #[test]
    fn test_validator_config_defaults() {
        let config = ValidatorConfig::default();
        assert_eq!(config.cache_ttl, Duration::from_secs(3600));
        assert_eq!(config.max_cache_size, 10_000);
        assert_eq!(config.max_suggestions, 5);
        assert_eq!(config.min_score, 0.1);
    }

    #[test]
    fn test_validator_config_builder() {
        let config = ValidatorConfig::default()
            .with_cache_ttl(Duration::from_secs(1800))
            .with_max_cache_size(5000)
            .with_max_suggestions(10)
            .with_min_score(0.5);

        assert_eq!(config.cache_ttl, Duration::from_secs(1800));
        assert_eq!(config.max_cache_size, 5000);
        assert_eq!(config.max_suggestions, 10);
        assert_eq!(config.min_score, 0.5);
    }

    #[test]
    fn test_min_score_clamping() {
        let config = ValidatorConfig::default().with_min_score(1.5);
        assert_eq!(config.min_score, 1.0); // Should clamp to max 1.0

        let config = ValidatorConfig::default().with_min_score(-0.5);
        assert_eq!(config.min_score, 0.0); // Should clamp to min 0.0
    }

    #[test]
    fn test_cache_stats() {
        let stats = CacheStats {
            total_entries: 50,
            valid_entries: 40,
            invalid_entries: 10,
            max_size: 100,
        };

        assert_eq!(stats.hit_rate(), 80.0); // 40/50 = 80%
        assert_eq!(stats.usage_percent(), 50.0); // 50/100 = 50%
    }

    #[test]
    fn test_cache_stats_edge_cases() {
        let empty_stats = CacheStats {
            total_entries: 0,
            valid_entries: 0,
            invalid_entries: 0,
            max_size: 100,
        };

        assert_eq!(empty_stats.hit_rate(), 0.0);
        assert_eq!(empty_stats.usage_percent(), 0.0);
    }

    #[test]
    fn test_validation_result_creation() {
        let valid = ValidationResult::valid(
            "AAPL".to_string(),
            "NASDAQ".to_string(),
            "EQUITY".to_string(),
            "Apple Inc.".to_string(),
            "Apple".to_string(),
        );

        assert!(valid.is_valid);
        assert_eq!(valid.symbol, "AAPL");
        assert_eq!(valid.exchange, Some("NASDAQ".to_string()));
        assert_eq!(valid.quote_type, Some("EQUITY".to_string()));

        let invalid = ValidationResult::invalid("INVALID".to_string());
        assert!(!invalid.is_valid);
        assert_eq!(invalid.symbol, "INVALID");
        assert!(invalid.exchange.is_none());
    }
}
