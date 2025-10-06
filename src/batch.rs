//! Batch operations for fetching multiple symbols in parallel with automatic rate limiting.
//!
//! This module provides efficient parallel processing of multiple symbols while respecting
//! rate limits and handling per-symbol errors gracefully.
//!
//! # Examples
//!
//! ```no_run
//! use eeyf::{YahooConnector, batch::BatchQuoteRequest};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let provider = YahooConnector::new()?;
//!     
//!     let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA"];
//!     let batch = BatchQuoteRequest::new(symbols)
//!         .with_concurrency(10)
//!         .with_continue_on_error(true);
//!     
//!     let results = provider.batch_get_quote(&batch).await?;
//!     
//!     for result in results {
//!         match result {
//!             Ok(quote) => println!("{}: ${:.2}", quote.symbol, quote.regular_market_price),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

use crate::YahooError;
use futures_util::stream::{self, StreamExt};
use std::time::Duration;

/// Configuration for batch quote requests
#[derive(Debug, Clone)]
pub struct BatchQuoteRequest {
    /// List of symbols to fetch
    pub symbols: Vec<String>,
    /// Maximum number of concurrent requests (default: 10)
    pub concurrency: usize,
    /// Whether to continue processing on individual symbol errors (default: true)
    pub continue_on_error: bool,
    /// Timeout per symbol request in seconds (default: 30)
    pub timeout_secs: u64,
}

impl BatchQuoteRequest {
    /// Create a new batch request for the given symbols
    pub fn new<S: AsRef<str>>(symbols: Vec<S>) -> Self {
        Self {
            symbols: symbols.iter().map(|s| s.as_ref().to_string()).collect(),
            concurrency: 10,
            continue_on_error: true,
            timeout_secs: 30,
        }
    }

    /// Set the maximum number of concurrent requests
    pub fn with_concurrency(mut self, concurrency: usize) -> Self {
        self.concurrency = concurrency.max(1).min(50); // Clamp between 1 and 50
        self
    }

    /// Set whether to continue on individual symbol errors
    pub fn with_continue_on_error(mut self, continue_on_error: bool) -> Self {
        self.continue_on_error = continue_on_error;
        self
    }

    /// Set the timeout per symbol request
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout_secs = timeout.as_secs().max(1).min(300); // 1 to 300 seconds
        self
    }
}

/// Result of a batch operation with progress tracking
#[derive(Debug)]
pub struct BatchResult<T> {
    /// Successful results with their symbols
    pub results: Vec<(String, T)>,
    /// Failed symbols with their errors
    pub errors: Vec<(String, YahooError)>,
    /// Total symbols processed
    pub total: usize,
    /// Number of successful fetches
    pub successful: usize,
    /// Number of failed fetches
    pub failed: usize,
}

impl<T> BatchResult<T> {
    /// Create a new empty batch result
    pub fn new(total: usize) -> Self {
        Self {
            results: Vec::with_capacity(total),
            errors: Vec::new(),
            total,
            successful: 0,
            failed: 0,
        }
    }

    /// Add a successful result
    pub fn add_success(&mut self, symbol: String, result: T) {
        self.results.push((symbol, result));
        self.successful += 1;
    }

    /// Add a failed result
    pub fn add_error(&mut self, symbol: String, error: YahooError) {
        self.errors.push((symbol, error));
        self.failed += 1;
    }

    /// Get success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.successful as f64 / self.total as f64) * 100.0
        }
    }

    /// Check if all requests were successful
    pub fn is_complete_success(&self) -> bool {
        self.failed == 0 && self.successful == self.total
    }

    /// Get all successful results, consuming self
    pub fn into_results(self) -> Vec<(String, T)> {
        self.results
    }

    /// Get all errors, consuming self
    pub fn into_errors(self) -> Vec<(String, YahooError)> {
        self.errors
    }
}

/// Progress callback for batch operations
pub type ProgressCallback = Box<dyn Fn(usize, usize) + Send + Sync>;

/// Batch operations implementation
pub struct BatchOperations<'a, T> {
    fetch_fn: Box<dyn Fn(String) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, YahooError>> + Send + 'a>> + Send + Sync + 'a>,
    progress_callback: Option<ProgressCallback>,
}

impl<'a, T: Send + 'a> BatchOperations<'a, T> {
    /// Create a new batch operations handler
    pub fn new<F, Fut>(fetch_fn: F) -> Self
    where
        F: Fn(String) -> Fut + Send + Sync + 'a,
        Fut: std::future::Future<Output = Result<T, YahooError>> + Send + 'a,
    {
        Self {
            fetch_fn: Box::new(move |symbol| Box::pin(fetch_fn(symbol))),
            progress_callback: None,
        }
    }

    /// Set a progress callback that will be called after each symbol is processed
    pub fn with_progress<F>(mut self, callback: F) -> Self
    where
        F: Fn(usize, usize) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    /// Execute the batch operation
    pub async fn execute(self, request: BatchQuoteRequest) -> BatchResult<T> {
        let total = request.symbols.len();
        let mut result = BatchResult::new(total);
        let mut completed = 0usize;

        // Create a stream of futures
        let futures = stream::iter(request.symbols.into_iter().map(|symbol| {
            let fetch_fn = &self.fetch_fn;
            async move {
                let symbol_clone = symbol.clone();
                match tokio::time::timeout(
                    Duration::from_secs(request.timeout_secs),
                    (fetch_fn)(symbol.clone()),
                )
                .await
                {
                    Ok(Ok(data)) => Ok((symbol_clone, data)),
                    Ok(Err(e)) => Err((symbol_clone, e)),
                    Err(_) => Err((
                        symbol_clone,
                        YahooError::ConnectionFailed(format!(
                            "Request timeout after {} seconds",
                            request.timeout_secs
                        )),
                    )),
                }
            }
        }));

        // Process with concurrency limit
        let mut stream = futures.buffer_unordered(request.concurrency);

        while let Some(outcome) = stream.next().await {
            completed += 1;

            match outcome {
                Ok((symbol, data)) => {
                    result.add_success(symbol, data);
                }
                Err((symbol, error)) => {
                    result.add_error(symbol, error);
                    if !request.continue_on_error {
                        // Drain remaining futures without processing
                        while let Some(_) = stream.next().await {
                            completed += 1;
                        }
                        break;
                    }
                }
            }

            // Call progress callback if set
            if let Some(ref callback) = self.progress_callback {
                callback(completed, total);
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_request_creation() {
        let symbols = vec!["AAPL", "GOOGL", "MSFT"];
        let request = BatchQuoteRequest::new(symbols);

        assert_eq!(request.symbols.len(), 3);
        assert_eq!(request.concurrency, 10);
        assert!(request.continue_on_error);
        assert_eq!(request.timeout_secs, 30);
    }

    #[test]
    fn test_batch_request_builder() {
        let request = BatchQuoteRequest::new(vec!["AAPL"])
            .with_concurrency(5)
            .with_continue_on_error(false)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(request.concurrency, 5);
        assert!(!request.continue_on_error);
        assert_eq!(request.timeout_secs, 60);
    }

    #[test]
    fn test_concurrency_clamping() {
        let request = BatchQuoteRequest::new(vec!["AAPL"]).with_concurrency(100);
        assert_eq!(request.concurrency, 50); // Should be clamped to max 50

        let request = BatchQuoteRequest::new(vec!["AAPL"]).with_concurrency(0);
        assert_eq!(request.concurrency, 1); // Should be clamped to min 1
    }

    #[test]
    fn test_timeout_clamping() {
        let request = BatchQuoteRequest::new(vec!["AAPL"]).with_timeout(Duration::from_secs(500));
        assert_eq!(request.timeout_secs, 300); // Should be clamped to max 300

        let request = BatchQuoteRequest::new(vec!["AAPL"]).with_timeout(Duration::from_secs(0));
        assert_eq!(request.timeout_secs, 1); // Should be clamped to min 1
    }

    #[test]
    fn test_batch_result_tracking() {
        let mut result = BatchResult::<String>::new(5);

        result.add_success("AAPL".to_string(), "data1".to_string());
        result.add_success("GOOGL".to_string(), "data2".to_string());
        result.add_error(
            "MSFT".to_string(),
            YahooError::FetchFailed("error".to_string()),
        );

        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 1);
        assert_eq!(result.total, 5);
        assert!(!result.is_complete_success());
    }

    #[test]
    fn test_success_rate_calculation() {
        let mut result = BatchResult::<String>::new(10);

        for i in 0..7 {
            result.add_success(format!("SYM{}", i), "data".to_string());
        }
        for i in 7..10 {
            result.add_error(
                format!("SYM{}", i),
                YahooError::FetchFailed("error".to_string()),
            );
        }

        assert_eq!(result.success_rate(), 70.0);
    }

    #[tokio::test]
    async fn test_batch_operations_success() {
        let fetch_fn = |symbol: String| async move {
            Ok::<_, YahooError>(format!("data_{}", symbol))
        };

        let batch_ops = BatchOperations::new(fetch_fn);
        let request = BatchQuoteRequest::new(vec!["AAPL", "GOOGL", "MSFT"]);

        let result = batch_ops.execute(request).await;

        assert_eq!(result.successful, 3);
        assert_eq!(result.failed, 0);
        assert!(result.is_complete_success());
        assert_eq!(result.success_rate(), 100.0);
    }

    #[tokio::test]
    async fn test_batch_operations_with_errors() {
        let fetch_fn = |symbol: String| async move {
            if symbol == "FAIL" {
                Err(YahooError::FetchFailed("error".to_string()))
            } else {
                Ok::<_, YahooError>(format!("data_{}", symbol))
            }
        };

        let batch_ops = BatchOperations::new(fetch_fn);
        let request = BatchQuoteRequest::new(vec!["AAPL", "FAIL", "GOOGL"]);

        let result = batch_ops.execute(request).await;

        assert_eq!(result.successful, 2);
        assert_eq!(result.failed, 1);
        assert!(!result.is_complete_success());
    }

    #[tokio::test]
    async fn test_batch_operations_stop_on_error() {
        let fetch_fn = |symbol: String| async move {
            if symbol == "FAIL" {
                Err(YahooError::FetchFailed("error".to_string()))
            } else {
                Ok::<_, YahooError>(format!("data_{}", symbol))
            }
        };

        let batch_ops = BatchOperations::new(fetch_fn);
        let request = BatchQuoteRequest::new(vec!["AAPL", "FAIL", "GOOGL", "MSFT"])
            .with_continue_on_error(false);

        let result = batch_ops.execute(request).await;

        // Should stop after encountering error
        assert!(result.failed > 0);
        assert!(result.successful + result.failed <= 4);
    }

    #[tokio::test]
    async fn test_batch_operations_with_progress() {
        use std::sync::Arc;
        use std::sync::Mutex;

        let progress_calls = Arc::new(Mutex::new(Vec::new()));
        let progress_calls_clone = progress_calls.clone();

        let fetch_fn = |symbol: String| async move {
            tokio::time::sleep(Duration::from_millis(10)).await;
            Ok::<_, YahooError>(format!("data_{}", symbol))
        };

        let batch_ops = BatchOperations::new(fetch_fn).with_progress(move |completed, total| {
            progress_calls_clone
                .lock()
                .unwrap()
                .push((completed, total));
        });

        let request = BatchQuoteRequest::new(vec!["AAPL", "GOOGL", "MSFT"]);
        let result = batch_ops.execute(request).await;

        assert_eq!(result.successful, 3);

        let calls = progress_calls.lock().unwrap();
        assert_eq!(calls.len(), 3); // Should have 3 progress calls
        assert_eq!(calls.last(), Some(&(3, 3))); // Last call should be (3, 3)
    }
}
