use std::sync::Arc;
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Rate limiter configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    /// Maximum requests per hour
    pub requests_per_hour: u32,
    /// Maximum burst requests (requests allowed in quick succession)
    pub burst_limit: u32,
    /// Minimum interval between requests
    pub min_interval: Duration,
}

impl RateLimitConfig {
    pub fn development() -> Self {
        Self::default()
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            // Conservative: 90% of Yahoo's estimated 2000 requests/hour limit
            requests_per_hour: 1800,
            // Allow small bursts but not too aggressive
            burst_limit: 10,
            // Minimum 100ms between requests
            min_interval: Duration::from_millis(100),
        }
    }
}

/// Token bucket rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    config: RateLimitConfig,
    // Atomic counters for thread-safe access
    hourly_count: AtomicU32,
    burst_tokens: AtomicU32,
    last_request_time: Arc<Mutex<Instant>>,
    hour_start: AtomicU64,
}

impl RateLimiter {
    pub fn new(config: RateLimitConfig) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            burst_tokens: AtomicU32::new(config.burst_limit),
            hourly_count: AtomicU32::new(0),
            last_request_time: Arc::new(Mutex::new(Instant::now())),
            hour_start: AtomicU64::new(now / 3600 * 3600), // Start of current hour
            config,
        }
    }

    /// Acquire a permit to make a request, blocking if necessary
    pub async fn acquire_permit(&self) -> Result<(), RateLimitError> {
        // Check if we've entered a new hour
        self.reset_hourly_counter_if_needed();

        // Check hourly limit
        let current_hourly = self.hourly_count.load(Ordering::Relaxed);
        if current_hourly >= self.config.requests_per_hour {
            return Err(RateLimitError::HourlyLimitExceeded {
                limit: self.config.requests_per_hour,
                used: current_hourly,
            });
        }

        // Wait for minimum interval between requests
        self.enforce_minimum_interval().await;

        // Try to consume a burst token
        loop {
            let current_tokens = self.burst_tokens.load(Ordering::Relaxed);
            if current_tokens == 0 {
                // No burst tokens available, wait and refill
                self.refill_burst_tokens().await;
                continue;
            }

            // Try to consume a token
            if self
                .burst_tokens
                .compare_exchange_weak(
                    current_tokens,
                    current_tokens - 1,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }

        // Increment hourly counter
        self.hourly_count.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Check if we can make a request without blocking
    pub fn try_acquire_permit(&self) -> Result<(), RateLimitError> {
        self.reset_hourly_counter_if_needed();

        let current_hourly = self.hourly_count.load(Ordering::Relaxed);
        if current_hourly >= self.config.requests_per_hour {
            return Err(RateLimitError::HourlyLimitExceeded {
                limit: self.config.requests_per_hour,
                used: current_hourly,
            });
        }

        let burst_tokens = self.burst_tokens.load(Ordering::Relaxed);
        if burst_tokens == 0 {
            return Err(RateLimitError::BurstLimitExceeded {
                limit: self.config.burst_limit,
            });
        }

        // Try to consume a token without waiting
        if self
            .burst_tokens
            .compare_exchange(
                burst_tokens,
                burst_tokens - 1,
                Ordering::Relaxed,
                Ordering::Relaxed,
            )
            .is_ok()
        {
            self.hourly_count.fetch_add(1, Ordering::Relaxed);
            Ok(())
        } else {
            Err(RateLimitError::BurstLimitExceeded {
                limit: self.config.burst_limit,
            })
        }
    }

    /// Get current rate limit status
    pub fn status(&self) -> RateLimitStatus {
        self.reset_hourly_counter_if_needed();

        RateLimitStatus {
            hourly_limit: self.config.requests_per_hour,
            hourly_used: self.hourly_count.load(Ordering::Relaxed),
            burst_limit: self.config.burst_limit,
            burst_available: self.burst_tokens.load(Ordering::Relaxed),
            min_interval: self.config.min_interval,
        }
    }

    async fn enforce_minimum_interval(&self) {
        let mut last_time = self.last_request_time.lock().await;
        let elapsed = last_time.elapsed();

        if elapsed < self.config.min_interval {
            let wait_time = self.config.min_interval - elapsed;
            drop(last_time); // Release lock before sleeping
            sleep(wait_time).await;

            // Update the last request time
            let mut last_time = self.last_request_time.lock().await;
            *last_time = Instant::now();
        } else {
            *last_time = Instant::now();
        }
    }

    async fn refill_burst_tokens(&self) {
        // Simple refill strategy: wait a bit and add one token
        // In a production system, this could be more sophisticated
        sleep(Duration::from_millis(200)).await;

        let current = self.burst_tokens.load(Ordering::Relaxed);
        if current < self.config.burst_limit {
            self.burst_tokens.store(
                (current + 1).min(self.config.burst_limit),
                Ordering::Relaxed,
            );
        }
    }

    fn reset_hourly_counter_if_needed(&self) {
        let current_hour = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            / 3600
            * 3600;

        let stored_hour = self.hour_start.load(Ordering::Relaxed);

        if current_hour > stored_hour {
            // We've moved to a new hour, reset the counter
            if self
                .hour_start
                .compare_exchange(
                    stored_hour,
                    current_hour,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                self.hourly_count.store(0, Ordering::Relaxed);
                self.burst_tokens
                    .store(self.config.burst_limit, Ordering::Relaxed);
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RateLimitStatus {
    pub hourly_limit: u32,
    pub hourly_used: u32,
    pub burst_limit: u32,
    pub burst_available: u32,
    pub min_interval: Duration,
}

impl RateLimitStatus {
    pub fn hourly_remaining(&self) -> u32 {
        self.hourly_limit.saturating_sub(self.hourly_used)
    }

    pub fn hourly_percent_used(&self) -> f64 {
        (self.hourly_used as f64 / self.hourly_limit as f64) * 100.0
    }

    pub fn is_near_limit(&self) -> bool {
        self.hourly_percent_used() > 90.0
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Hourly rate limit exceeded: {used}/{limit} requests used")]
    HourlyLimitExceeded { limit: u32, used: u32 },

    #[error("Burst rate limit exceeded: no tokens available (limit: {limit})")]
    BurstLimitExceeded { limit: u32 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let config = RateLimitConfig {
            requests_per_hour: 100,
            burst_limit: 5,
            min_interval: Duration::from_millis(50),
        };

        let limiter = RateLimiter::new(config);

        // Should be able to make a few requests quickly
        for _ in 0..5 {
            limiter.acquire_permit().await.unwrap();
        }

        let status = limiter.status();
        assert_eq!(status.hourly_used, 5);
        assert!(status.burst_available < 5);
    }

    #[tokio::test]
    async fn test_rate_limiter_interval() {
        let config = RateLimitConfig {
            requests_per_hour: 100,
            burst_limit: 1,
            min_interval: Duration::from_millis(100),
        };

        let limiter = RateLimiter::new(config);

        let start = Instant::now();

        // First request should be immediate
        limiter.acquire_permit().await.unwrap();

        // Second request should wait for min_interval
        limiter.acquire_permit().await.unwrap();

        let elapsed = start.elapsed();
        assert!(elapsed >= Duration::from_millis(100));
    }

    #[tokio::test]
    async fn test_try_acquire_no_wait() {
        let config = RateLimitConfig {
            requests_per_hour: 100,
            burst_limit: 2,
            min_interval: Duration::from_millis(10),
        };

        let limiter = RateLimiter::new(config);

        // Should succeed for burst_limit requests
        limiter.try_acquire_permit().unwrap();
        limiter.try_acquire_permit().unwrap();

        // Should fail when burst tokens exhausted
        let result = limiter.try_acquire_permit();
        assert!(matches!(
            result,
            Err(RateLimitError::BurstLimitExceeded { .. })
        ));
    }
}
