//! Request deduplication to prevent duplicate API calls
//!
//! This module provides intelligent request deduplication by identifying
//! identical requests and sharing their results, reducing unnecessary
//! load on the Yahoo Finance API and improving response times.

use crate::yahoo_error::YahooError;
use dashmap::DashMap;
use log::{debug, warn};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{Mutex, oneshot};

/// Configuration for request deduplication
#[derive(Debug, Clone)]
pub struct DeduplicationConfig {
    /// Maximum time to cache request results (in milliseconds)
    pub cache_ttl_ms: u64,
    /// Maximum number of cached entries
    pub max_cache_entries: usize,
    /// Whether to deduplicate in-flight requests
    pub deduplicate_in_flight: bool,
    /// Whether to cache successful responses
    pub cache_successes: bool,
    /// Whether to cache failed responses (for short periods)
    pub cache_failures: bool,
    /// TTL for cached failures (typically much shorter)
    pub failure_cache_ttl_ms: u64,
    /// Maximum size of request key (for memory efficiency)
    pub max_key_length: usize,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            cache_ttl_ms: 300_000, // 5 minutes
            max_cache_entries: 1000,
            deduplicate_in_flight: true,
            cache_successes: true,
            cache_failures: true,
            failure_cache_ttl_ms: 30_000, // 30 seconds
            max_key_length: 256,
        }
    }
}

impl DeduplicationConfig {
    /// Configuration optimized for aggressive caching
    pub fn aggressive_caching() -> Self {
        Self {
            cache_ttl_ms: 900_000, // 15 minutes
            max_cache_entries: 5000,
            deduplicate_in_flight: true,
            cache_successes: true,
            cache_failures: true,
            failure_cache_ttl_ms: 60_000, // 1 minute
            max_key_length: 512,
        }
    }

    /// Configuration for minimal caching
    pub fn minimal_caching() -> Self {
        Self {
            cache_ttl_ms: 60_000, // 1 minute
            max_cache_entries: 100,
            deduplicate_in_flight: true,
            cache_successes: false,
            cache_failures: false,
            failure_cache_ttl_ms: 5_000, // 5 seconds
            max_key_length: 128,
        }
    }

    /// Configuration for development/testing
    pub fn development() -> Self {
        Self {
            cache_ttl_ms: 10_000, // 10 seconds
            max_cache_entries: 50,
            deduplicate_in_flight: false,
            cache_successes: false,
            cache_failures: false,
            failure_cache_ttl_ms: 1_000, // 1 second
            max_key_length: 256,
        }
    }

    /// Disable all caching and deduplication
    pub fn disabled() -> Self {
        Self {
            cache_ttl_ms: 0,
            max_cache_entries: 0,
            deduplicate_in_flight: false,
            cache_successes: false,
            cache_failures: false,
            failure_cache_ttl_ms: 0,
            max_key_length: 0,
        }
    }
}

/// Cached response entry
#[derive(Debug, Clone)]
struct CacheEntry {
    response: Result<Value, YahooError>,
    timestamp: u64,
    access_count: u64,
    is_failure: bool,
}

impl CacheEntry {
    fn new(response: Result<Value, YahooError>) -> Self {
        let is_failure = response.is_err();
        Self {
            response,
            timestamp: current_time_millis(),
            access_count: 1,
            is_failure,
        }
    }

    fn is_expired(&self, ttl_ms: u64) -> bool {
        if ttl_ms == 0 {
            return true; // Always expired if TTL is 0
        }
        current_time_millis() - self.timestamp > ttl_ms
    }

    fn access(&mut self) -> Result<Value, YahooError> {
        self.access_count += 1;
        self.response.clone()
    }
}

/// Statistics for deduplication monitoring
#[derive(Debug, Clone, Default)]
pub struct DeduplicationStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub deduplicated_requests: u64,
    pub cached_entries: usize,
    pub cache_evictions: u64,
    pub average_response_time_ms: f64,
}

impl DeduplicationStats {
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_requests as f64
        }
    }

    pub fn deduplication_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            self.deduplicated_requests as f64 / self.total_requests as f64
        }
    }
}

/// Type for in-flight request tracking
type InFlightRequests =
    Arc<DashMap<String, Arc<Mutex<Option<oneshot::Receiver<Result<Value, YahooError>>>>>>>;

/// Request deduplication manager
pub struct RequestDeduplicator {
    config: DeduplicationConfig,
    cache: Arc<DashMap<String, CacheEntry>>,
    in_flight: InFlightRequests,
    stats: Arc<Mutex<DeduplicationStats>>,
}

impl RequestDeduplicator {
    pub fn new(config: DeduplicationConfig) -> Self {
        Self {
            config,
            cache: Arc::new(DashMap::new()),
            in_flight: Arc::new(DashMap::new()),
            stats: Arc::new(Mutex::new(DeduplicationStats::default())),
        }
    }

    pub fn with_default_config() -> Self {
        Self::new(DeduplicationConfig::default())
    }

    /// Generate a cache key for a request
    fn generate_cache_key(
        &self,
        method: &str,
        url: &str,
        params: Option<&HashMap<String, String>>,
    ) -> String {
        let mut hasher = Sha256::new();

        // Include HTTP method
        hasher.update(method.as_bytes());
        hasher.update(b"|");

        // Include URL
        hasher.update(url.as_bytes());
        hasher.update(b"|");

        // Include sorted parameters
        if let Some(params) = params {
            let mut sorted_params: Vec<_> = params.iter().collect();
            sorted_params.sort_by_key(|(k, _)| *k);

            for (key, value) in sorted_params {
                hasher.update(key.as_bytes());
                hasher.update(b"=");
                hasher.update(value.as_bytes());
                hasher.update(b"&");
            }
        }

        let hash = hasher.finalize();
        let key = format!("{:x}", hash);

        // Truncate if necessary
        if key.len() > self.config.max_key_length {
            key[..self.config.max_key_length].to_string()
        } else {
            key
        }
    }

    /// Execute a request with deduplication
    pub async fn execute<F, Fut>(
        &self,
        method: &str,
        url: &str,
        params: Option<HashMap<String, String>>,
        operation: F,
    ) -> Result<Value, YahooError>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<Value, YahooError>> + Send,
    {
        // Update stats
        {
            let mut stats = self.stats.lock().await;
            stats.total_requests += 1;
        }

        let start_time = current_time_millis();

        // Check if caching/deduplication is disabled
        if self.config.max_cache_entries == 0 {
            let result = operation().await;
            self.update_response_time(start_time).await;
            return result;
        }

        let cache_key = self.generate_cache_key(method, url, params.as_ref());

        // Check cache first
        if let Some(result) = self.check_cache(&cache_key).await {
            debug!("Cache hit for key: {}", &cache_key[..8]);
            self.update_response_time(start_time).await;
            return result;
        }

        // Check for in-flight request
        if self.config.deduplicate_in_flight {
            if let Some(result) = self.check_in_flight(&cache_key).await {
                debug!(
                    "Deduplicated in-flight request for key: {}",
                    &cache_key[..8]
                );
                {
                    let mut stats = self.stats.lock().await;
                    stats.deduplicated_requests += 1;
                }
                self.update_response_time(start_time).await;
                return result;
            }
        }

        // Execute the request
        let (sender, receiver) = oneshot::channel();

        // Register as in-flight
        if self.config.deduplicate_in_flight {
            self.in_flight
                .insert(cache_key.clone(), Arc::new(Mutex::new(Some(receiver))));
        }

        // Execute operation
        let result = operation().await;

        // Cache the result
        self.cache_result(&cache_key, &result).await;

        // Notify waiting requests
        if self.config.deduplicate_in_flight {
            let _ = sender.send(result.clone());
            self.in_flight.remove(&cache_key);
        }

        // Clean up expired entries periodically
        if rand::random::<f32>() < 0.01 {
            // 1% chance
            self.cleanup_expired_entries().await;
        }

        self.update_response_time(start_time).await;
        result
    }

    /// Check cache for existing result
    async fn check_cache(&self, cache_key: &str) -> Option<Result<Value, YahooError>> {
        if let Some(mut entry) = self.cache.get_mut(cache_key) {
            let ttl = if entry.is_failure {
                self.config.failure_cache_ttl_ms
            } else {
                self.config.cache_ttl_ms
            };

            if !entry.is_expired(ttl) {
                let mut stats = self.stats.lock().await;
                stats.cache_hits += 1;
                return Some(entry.access());
            } else {
                // Remove expired entry
                drop(entry);
                self.cache.remove(cache_key);
            }
        }

        let mut stats = self.stats.lock().await;
        stats.cache_misses += 1;
        None
    }

    /// Check for in-flight request with same key
    async fn check_in_flight(&self, cache_key: &str) -> Option<Result<Value, YahooError>> {
        if let Some(receiver_arc) = self.in_flight.get(cache_key) {
            let mut receiver_opt = receiver_arc.lock().await;
            if let Some(receiver) = receiver_opt.take() {
                // Wait for the in-flight request to complete
                match receiver.await {
                    Ok(result) => return Some(result),
                    Err(_) => {
                        // The sender was dropped, proceed with new request
                        warn!(
                            "In-flight request sender dropped for key: {}",
                            &cache_key[..8]
                        );
                    }
                }
            }
        }
        None
    }

    /// Cache the result of a request
    async fn cache_result(&self, cache_key: &str, result: &Result<Value, YahooError>) {
        let should_cache = match result {
            Ok(_) => self.config.cache_successes,
            Err(_) => self.config.cache_failures,
        };

        if !should_cache {
            return;
        }

        // Check cache size limit
        if self.cache.len() >= self.config.max_cache_entries {
            self.evict_oldest_entries().await;
        }

        let entry = CacheEntry::new(result.clone());
        self.cache.insert(cache_key.to_string(), entry);

        let mut stats = self.stats.lock().await;
        stats.cached_entries = self.cache.len();

        debug!(
            "Cached result for key: {} (cache size: {})",
            &cache_key[..8],
            self.cache.len()
        );
    }

    /// Evict oldest entries to make room
    async fn evict_oldest_entries(&self) {
        let entries_to_evict = self.config.max_cache_entries / 4; // Evict 25%
        let mut entries: Vec<_> = self
            .cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.timestamp))
            .collect();

        entries.sort_by_key(|(_, timestamp)| *timestamp);

        let mut evicted = 0;
        for (key, _) in entries.into_iter().take(entries_to_evict) {
            self.cache.remove(&key);
            evicted += 1;
        }

        let mut stats = self.stats.lock().await;
        stats.cache_evictions += evicted as u64;
        stats.cached_entries = self.cache.len();

        debug!("Evicted {} cache entries", evicted);
    }

    /// Clean up expired entries
    async fn cleanup_expired_entries(&self) {
        let _current_time = current_time_millis();
        let mut to_remove = Vec::new();

        for entry in self.cache.iter() {
            let ttl = if entry.is_failure {
                self.config.failure_cache_ttl_ms
            } else {
                self.config.cache_ttl_ms
            };

            if entry.is_expired(ttl) {
                to_remove.push(entry.key().clone());
            }
        }

        for key in to_remove {
            self.cache.remove(&key);
        }

        let mut stats = self.stats.lock().await;
        stats.cached_entries = self.cache.len();

        if !self.cache.is_empty() {
            debug!(
                "Cleaned up expired entries, cache size: {}",
                self.cache.len()
            );
        }
    }

    /// Update response time statistics
    async fn update_response_time(&self, start_time: u64) {
        let duration = current_time_millis() - start_time;
        let mut stats = self.stats.lock().await;

        // Simple moving average
        let total_requests = stats.total_requests as f64;
        let current_avg = stats.average_response_time_ms;
        stats.average_response_time_ms =
            ((current_avg * (total_requests - 1.0)) + duration as f64) / total_requests;
    }

    /// Get current statistics
    pub async fn stats(&self) -> DeduplicationStats {
        let mut stats = self.stats.lock().await;
        stats.cached_entries = self.cache.len();
        stats.clone()
    }

    /// Clear all cached entries
    pub async fn clear_cache(&self) {
        self.cache.clear();
        self.in_flight.clear();

        let mut stats = self.stats.lock().await;
        stats.cached_entries = 0;

        debug!("Cache cleared");
    }

    /// Get cache entry count
    pub fn cache_size(&self) -> usize {
        self.cache.len()
    }
}

/// Get current time in milliseconds since Unix epoch
fn current_time_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{Duration, sleep};

    #[tokio::test]
    async fn test_cache_key_generation() {
        let dedup = RequestDeduplicator::with_default_config();

        let key1 = dedup.generate_cache_key("GET", "https://example.com", None);
        let key2 = dedup.generate_cache_key("GET", "https://example.com", None);
        assert_eq!(key1, key2);

        let mut params = HashMap::new();
        params.insert("param1".to_string(), "value1".to_string());
        let key3 = dedup.generate_cache_key("GET", "https://example.com", Some(&params));
        assert_ne!(key1, key3);
    }

    #[tokio::test]
    async fn test_successful_caching() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let config = DeduplicationConfig {
            cache_successes: true,
            cache_ttl_ms: 1000,
            ..Default::default()
        };
        let dedup = RequestDeduplicator::new(config);

        let call_count = Arc::new(AtomicUsize::new(0));

        // First call should execute
        let call_count_1 = call_count.clone();
        let result1 = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_1.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"data": "test"}))
                }
            })
            .await;
        assert!(result1.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call should use cache
        let call_count_2 = call_count.clone();
        let result2 = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_2.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"data": "test"}))
                }
            })
            .await;
        assert!(result2.is_ok());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not increment

        let stats = dedup.stats().await;
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let config = DeduplicationConfig {
            cache_successes: true,
            cache_ttl_ms: 50, // Short TTL for testing
            ..Default::default()
        };
        let dedup = RequestDeduplicator::new(config);

        let call_count = Arc::new(AtomicUsize::new(0));

        // First call
        let call_count_1 = call_count.clone();
        let _ = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_1.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"data": "test"}))
                }
            })
            .await;
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Wait for expiration
        sleep(Duration::from_millis(60)).await;

        // Second call should execute again
        let call_count_2 = call_count.clone();
        let _ = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_2.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Ok(serde_json::json!({"data": "test"}))
                }
            })
            .await;
        assert_eq!(call_count.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_failure_caching() {
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};

        let config = DeduplicationConfig {
            cache_failures: true,
            failure_cache_ttl_ms: 100,
            ..Default::default()
        };
        let dedup = RequestDeduplicator::new(config);

        let call_count = Arc::new(AtomicUsize::new(0));

        // First call fails
        let call_count_1 = call_count.clone();
        let result1 = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_1.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err(YahooError::FetchFailed("test error".to_string()))
                }
            })
            .await;
        assert!(result1.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1);

        // Second call should use cached failure
        let call_count_2 = call_count.clone();
        let result2 = dedup
            .execute("GET", "https://test.com", None, move || {
                let count = call_count_2.clone();
                async move {
                    count.fetch_add(1, Ordering::SeqCst);
                    Err(YahooError::FetchFailed("test error".to_string()))
                }
            })
            .await;
        assert!(result2.is_err());
        assert_eq!(call_count.load(Ordering::SeqCst), 1); // Should not increment
    }
}
