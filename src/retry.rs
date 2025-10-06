//! Exponential backoff retry logic for resilient API calls
//!
//! This module provides configurable retry strategies with exponential backoff,
//! jitter, and intelligent error handling based on error categorization.

use crate::error_categories::{ErrorCategorizer, ErrorCategory};
use crate::yahoo_error::YahooError;
use log::{debug, error, warn};
use rand::Rng;
use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;

/// Configuration for retry behavior
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_attempts: u32,
    /// Base delay between retries in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay between retries in milliseconds
    pub max_delay_ms: u64,
    /// Exponential backoff multiplier (e.g., 2.0 doubles delay each time)
    pub backoff_multiplier: f64,
    /// Maximum jitter percentage (0.0 to 1.0) to prevent thundering herd
    pub jitter_factor: f64,
    /// Whether to enable exponential backoff
    pub enable_exponential_backoff: bool,
    /// Whether to respect error-specific retry policies
    pub respect_error_categories: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.1,
            enable_exponential_backoff: true,
            respect_error_categories: true,
        }
    }
}

impl RetryConfig {
    /// Create a conservative retry configuration for production use
    pub fn conservative() -> Self {
        Self {
            max_attempts: 2,
            base_delay_ms: 2000,
            max_delay_ms: 15000,
            backoff_multiplier: 1.5,
            jitter_factor: 0.2,
            enable_exponential_backoff: true,
            respect_error_categories: true,
        }
    }

    /// Create an aggressive retry configuration for development/testing
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 500,
            max_delay_ms: 60000,
            backoff_multiplier: 2.5,
            jitter_factor: 0.05,
            enable_exponential_backoff: true,
            respect_error_categories: true,
        }
    }

    /// Create a configuration optimized for rate-limited APIs
    pub fn rate_limit_optimized() -> Self {
        Self {
            max_attempts: 5,
            base_delay_ms: 5000,
            max_delay_ms: 120000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.25,
            enable_exponential_backoff: true,
            respect_error_categories: true,
        }
    }

    /// Disable retries completely
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 0,
            ..Default::default()
        }
    }
}

/// Retry execution statistics
#[derive(Debug, Clone, Default)]
pub struct RetryStats {
    pub total_attempts: u32,
    pub successful_attempts: u32,
    pub failed_attempts: u32,
    pub total_delay_ms: u64,
    pub last_error_category: Option<ErrorCategory>,
}

/// Retry policy engine that handles exponential backoff and jitter
pub struct RetryPolicy {
    config: RetryConfig,
    stats: RetryStats,
}

impl RetryPolicy {
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            stats: RetryStats::default(),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(RetryConfig::default())
    }

    pub fn stats(&self) -> &RetryStats {
        &self.stats
    }

    /// Execute a future with retry logic
    pub async fn execute<F, Fut, T>(&mut self, operation: F) -> Result<T, YahooError>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, YahooError>>,
    {
        self.stats = RetryStats::default();
        let mut last_error = None;

        for attempt in 0..=self.config.max_attempts {
            self.stats.total_attempts += 1;

            debug!(
                "Retry attempt {} of {}",
                attempt + 1,
                self.config.max_attempts + 1
            );

            match operation().await {
                Ok(result) => {
                    self.stats.successful_attempts += 1;
                    debug!("Operation succeeded on attempt {}", attempt + 1);
                    return Ok(result);
                }
                Err(error) => {
                    self.stats.failed_attempts += 1;
                    let error_info = error.categorize_error();
                    self.stats.last_error_category = Some(error_info.category.clone());

                    warn!(
                        "Operation failed on attempt {}: {} (category: {})",
                        attempt + 1,
                        error,
                        error_info.category
                    );

                    // Check if we should retry based on error category
                    if self.config.respect_error_categories && !error_info.is_retryable {
                        debug!(
                            "Error category {} is not retryable, stopping",
                            error_info.category
                        );
                        return Err(error);
                    }

                    // If this is the last attempt, return the error
                    if attempt >= self.config.max_attempts {
                        error!("All retry attempts exhausted, returning last error");
                        return Err(error);
                    }

                    // Calculate delay for next attempt
                    let delay_ms = self.calculate_delay(attempt, &error_info);
                    self.stats.total_delay_ms += delay_ms;

                    debug!(
                        "Waiting {}ms before retry attempt {}",
                        delay_ms,
                        attempt + 2
                    );
                    sleep(Duration::from_millis(delay_ms)).await;

                    last_error = Some(error);
                }
            }
        }

        // This should never be reached, but just in case
        Err(last_error
            .unwrap_or_else(|| YahooError::FetchFailed("Unexpected retry loop exit".to_string())))
    }

    /// Calculate delay with exponential backoff and jitter
    fn calculate_delay(
        &self,
        attempt: u32,
        error_info: &crate::error_categories::ErrorInfo,
    ) -> u64 {
        let base_delay = if self.config.respect_error_categories {
            error_info
                .suggested_delay_ms
                .unwrap_or(self.config.base_delay_ms)
        } else {
            self.config.base_delay_ms
        };

        let delay = if self.config.enable_exponential_backoff && attempt > 0 {
            let exponential_delay =
                (base_delay as f64) * self.config.backoff_multiplier.powi(attempt as i32);
            exponential_delay as u64
        } else {
            base_delay
        };

        // Apply maximum delay limit
        let capped_delay = delay.min(self.config.max_delay_ms);

        // Apply jitter to prevent thundering herd
        if self.config.jitter_factor > 0.0 {
            let jitter_range = (capped_delay as f64 * self.config.jitter_factor) as u64;
            let mut rng = rand::thread_rng();
            let jitter = rng.gen_range(0..=jitter_range);
            capped_delay.saturating_add(jitter)
        } else {
            capped_delay
        }
    }
}

/// Convenience macro for retrying operations
#[macro_export]
macro_rules! retry_operation {
    ($retry_policy:expr, $operation:expr) => {
        $retry_policy.execute(|| async { $operation }).await
    };
}

/// Helper function to create and execute a retry operation in one call
#[allow(dead_code)]
pub async fn retry_with_config<F, Fut, T>(
    config: RetryConfig,
    operation: F,
) -> Result<T, YahooError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, YahooError>>,
{
    let mut policy = RetryPolicy::new(config);
    policy.execute(operation).await
}

/// Helper function to retry with default configuration
#[allow(dead_code)]
pub async fn retry_with_default<F, Fut, T>(operation: F) -> Result<T, YahooError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, YahooError>>,
{
    retry_with_config(RetryConfig::default(), operation).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_retry_success_on_second_attempt() {
        let attempt_counter = Arc::new(AtomicU32::new(0));
        let counter_clone = attempt_counter.clone();

        let config = RetryConfig {
            max_attempts: 3,
            base_delay_ms: 10, // Fast for testing
            ..Default::default()
        };

        let mut policy = RetryPolicy::new(config);

        let result = policy
            .execute(|| {
                let counter = counter_clone.clone();
                async move {
                    let count = counter.fetch_add(1, Ordering::SeqCst);
                    if count == 0 {
                        Err(YahooError::FetchFailed("First attempt fails".to_string()))
                    } else {
                        Ok("Success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Success");
        assert_eq!(policy.stats().total_attempts, 2);
        assert_eq!(policy.stats().successful_attempts, 1);
    }

    #[tokio::test]
    async fn test_non_retryable_error() {
        let config = RetryConfig::default();
        let mut policy = RetryPolicy::new(config);

        let result: Result<(), YahooError> = policy
            .execute(|| async { Err(YahooError::InvalidUrl) })
            .await;

        assert!(result.is_err());
        assert_eq!(policy.stats().total_attempts, 1);
        assert_eq!(policy.stats().successful_attempts, 0);
    }

    #[tokio::test]
    async fn test_delay_calculation() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            backoff_multiplier: 2.0,
            jitter_factor: 0.0, // No jitter for predictable testing
            enable_exponential_backoff: true,
            respect_error_categories: false, // Use base delay for predictable testing
            ..Default::default()
        };

        let policy = RetryPolicy::new(config);
        let error_info = YahooError::FetchFailed("test".to_string()).categorize_error();

        assert_eq!(policy.calculate_delay(0, &error_info), 1000);
        assert_eq!(policy.calculate_delay(1, &error_info), 2000);
        assert_eq!(policy.calculate_delay(2, &error_info), 4000);
    }

    #[tokio::test]
    async fn test_max_delay_cap() {
        let config = RetryConfig {
            base_delay_ms: 1000,
            max_delay_ms: 3000,
            backoff_multiplier: 10.0,
            jitter_factor: 0.0,
            enable_exponential_backoff: true,
            ..Default::default()
        };

        let policy = RetryPolicy::new(config);
        let error_info = YahooError::FetchFailed("test".to_string()).categorize_error();

        // Should be capped at max_delay_ms
        assert_eq!(policy.calculate_delay(5, &error_info), 3000);
    }
}
