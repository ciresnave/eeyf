//! Preset configuration management for YahooConnector.
//!
//! This module provides functionality to load and save preset configurations,
//! allowing users to define and reuse custom connector settings.

use crate::YahooError;
use crate::builder::YahooConnectorBuilder;
use crate::enterprise::EnterpriseConfig;
use crate::{
    circuit_breaker::CircuitBreakerConfig, connection_pool::ConnectionPoolConfig,
    observability::ObservabilityConfig, rate_limiter::RateLimitConfig,
    request_deduplication::DeduplicationConfig, response_cache::ResponseCacheConfig,
    retry::RetryConfig,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A preset configuration that can be loaded or saved.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresetConfig {
    /// Name of the preset
    pub name: String,

    /// Description of the preset
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Requests per second allowed
    pub rate_limit: f64,

    /// Number of failures before circuit breaker opens
    pub circuit_breaker_threshold: u32,

    /// Time window for counting failures (seconds)
    pub circuit_breaker_window_secs: u64,

    /// How long circuit breaker stays open (seconds)
    pub circuit_breaker_timeout_secs: u64,

    /// Number of retry attempts
    pub retry_attempts: u32,

    /// Initial delay between retries (milliseconds)
    pub retry_initial_delay_ms: u64,

    /// Maximum delay between retries (milliseconds)
    pub retry_max_delay_ms: u64,

    /// Request timeout (seconds)
    pub timeout_secs: u64,

    /// Maximum number of cache entries
    pub cache_size: usize,

    /// Cache entry time-to-live (seconds)
    pub cache_duration_secs: u64,

    /// Maximum concurrent connections in pool
    pub connection_pool_max: usize,

    /// Enable verbose logging
    pub verbose_logging: bool,

    /// Enable request metrics
    pub enable_metrics: bool,

    /// Enable distributed tracing
    pub enable_tracing: bool,
}

impl From<&YahooConnectorBuilder> for PresetConfig {
    fn from(builder: &YahooConnectorBuilder) -> Self {
        Self {
            name: "custom".to_string(),
            description: None,
            rate_limit: builder.rate_limit,
            circuit_breaker_threshold: builder.circuit_breaker_threshold,
            circuit_breaker_window_secs: builder.circuit_breaker_window_secs,
            circuit_breaker_timeout_secs: builder.circuit_breaker_timeout_secs,
            retry_attempts: builder.retry_attempts,
            retry_initial_delay_ms: builder.retry_initial_delay_ms,
            retry_max_delay_ms: builder.retry_max_delay_ms,
            timeout_secs: builder.timeout.as_secs(),
            cache_size: builder.cache_size,
            cache_duration_secs: builder.cache_duration_secs,
            connection_pool_max: builder.connection_pool_max,
            verbose_logging: builder.verbose_logging,
            enable_metrics: builder.enable_metrics,
            enable_tracing: builder.enable_tracing,
        }
    }
}

impl From<PresetConfig> for YahooConnectorBuilder {
    fn from(preset: PresetConfig) -> Self {
        Self {
            rate_limit: preset.rate_limit,
            circuit_breaker_threshold: preset.circuit_breaker_threshold,
            circuit_breaker_window_secs: preset.circuit_breaker_window_secs,
            circuit_breaker_timeout_secs: preset.circuit_breaker_timeout_secs,
            retry_attempts: preset.retry_attempts,
            retry_initial_delay_ms: preset.retry_initial_delay_ms,
            retry_max_delay_ms: preset.retry_max_delay_ms,
            timeout: Duration::from_secs(preset.timeout_secs),
            cache_size: preset.cache_size,
            cache_duration_secs: preset.cache_duration_secs,
            connection_pool_max: preset.connection_pool_max,
            verbose_logging: preset.verbose_logging,
            enable_metrics: preset.enable_metrics,
            enable_tracing: preset.enable_tracing,
        }
    }
}

impl From<PresetConfig> for EnterpriseConfig {
    fn from(preset: PresetConfig) -> Self {
        Self {
            circuit_breaker: CircuitBreakerConfig {
                failure_threshold: preset.circuit_breaker_threshold,
                success_threshold: 3, // reasonable default
                recovery_timeout_ms: preset.circuit_breaker_timeout_secs * 1000, // convert to ms
                half_open_max_requests: 3, // reasonable default
                failure_rate_window_ms: preset.circuit_breaker_window_secs * 1000, // convert to ms
                minimum_request_volume: 10, // minimum 10 calls before circuit breaker activates
                categorize_failures: true, // enable smart error categorization
            },
            retry: RetryConfig {
                max_attempts: preset.retry_attempts,
                base_delay_ms: preset.retry_initial_delay_ms,
                max_delay_ms: preset.retry_max_delay_ms,
                backoff_multiplier: 2.0, // exponential backoff
                jitter_factor: 0.1,      // 10% jitter to prevent thundering herd
                enable_exponential_backoff: true,
                respect_error_categories: true,
            },
            deduplication: DeduplicationConfig {
                cache_ttl_ms: preset.cache_duration_secs * 1000, // convert to ms
                max_cache_entries: preset.cache_size,
                deduplicate_in_flight: true,
                cache_successes: true,
                cache_failures: true,
                failure_cache_ttl_ms: 30_000, // 30 seconds for failures
                max_key_length: 256,
            },
            response_cache: ResponseCacheConfig {
                max_entries: preset.cache_size,
                default_ttl_ms: preset.cache_duration_secs * 1000, // convert to ms
                quote_ttl_ms: (preset.cache_duration_secs / 5) * 1000, // 1/5 of default for live quotes
                search_ttl_ms: preset.cache_duration_secs * 1000,
                history_ttl_ms: preset.cache_duration_secs * 1000,
                max_memory_bytes: 50 * 1024 * 1024, // 50MB default
                enable_size_eviction: true,
                cleanup_interval_ms: 60_000, // 1 minute
                cache_errors: false,         // don't cache errors by default
                error_ttl_ms: 5_000,         // 5 seconds for errors
            },
            observability: ObservabilityConfig {
                enable_logging: true, // always enable logging for presets
                log_level: if preset.verbose_logging {
                    crate::observability::LogLevel::Debug
                } else {
                    crate::observability::LogLevel::Info
                },
                enable_metrics: preset.enable_metrics,
                enable_tracing: preset.enable_tracing,
                enable_health_checks: true,
                metrics_interval_ms: 60_000, // 1 minute
                log_request_details: preset.verbose_logging,
                log_error_details: true, // always log error details
                slow_request_threshold_ms: preset.timeout_secs * 1000, // use timeout as slow request threshold
            },
            connection_pool: ConnectionPoolConfig {
                max_connections_per_host: preset.connection_pool_max / 2, // split between per-host and total
                max_total_connections: preset.connection_pool_max,
                connect_timeout_ms: preset.timeout_secs * 1000, // convert to ms
                request_timeout_ms: preset.timeout_secs * 1000, // convert to ms
                keep_alive_timeout_ms: 90_000,                  // 90 seconds
                idle_timeout_ms: 60_000,                        // 1 minute
                enable_http2: true,
                enable_connection_reuse: true,
                cleanup_interval_ms: 300_000, // 5 minutes
                user_agent: "EEYF/0.1.0 (Enterprise)".to_string(),
                enable_tcp_keepalive: true,
                tcp_keepalive_interval_ms: 30_000, // 30 seconds
            },
            rate_limiter: RateLimitConfig {
                requests_per_hour: (preset.rate_limit * 3600.0) as u32, // convert per-second to per-hour
                burst_limit: (preset.rate_limit * 2.0).max(1.0) as u32, // 2x rate limit as burst
                min_interval: Duration::from_millis((1000.0 / preset.rate_limit.max(1.0)) as u64),
            },
            enable_all_features: true, // presets always enable all features
        }
    }
}

/// Manages preset configurations for YahooConnector.
pub struct PresetManager {
    /// Built-in presets
    builtins: HashMap<String, PresetConfig>,

    /// User config directory (~/.config/eeyf/presets/)
    user_presets_dir: Option<PathBuf>,

    /// Project local directory (./.eeyf/presets/)
    project_presets_dir: Option<PathBuf>,
}

impl PresetManager {
    /// Creates a new preset manager with built-in presets.
    pub fn new() -> Self {
        let mut builtins = HashMap::new();

        // Production preset
        let production = PresetConfig::from(&YahooConnectorBuilder::production());
        builtins.insert("production".to_string(), production);

        // Development preset
        let development = PresetConfig::from(&YahooConnectorBuilder::default());
        builtins.insert("development".to_string(), development);

        // Enterprise preset
        let enterprise = PresetConfig::from(&YahooConnectorBuilder::enterprise());
        builtins.insert("enterprise".to_string(), enterprise);

        // Minimal preset
        let minimal = PresetConfig::from(&YahooConnectorBuilder::minimal());
        builtins.insert("minimal".to_string(), minimal);

        Self {
            builtins,
            user_presets_dir: Self::get_user_presets_dir(),
            project_presets_dir: Self::get_project_presets_dir(),
        }
    }

    /// Gets the user presets directory (~/.config/eeyf/presets/)
    fn get_user_presets_dir() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            std::env::var("APPDATA")
                .ok()
                .map(|appdata| PathBuf::from(appdata).join("eeyf").join("presets"))
        }

        #[cfg(not(target_os = "windows"))]
        {
            std::env::var("HOME").ok().map(|home| {
                PathBuf::from(home)
                    .join(".config")
                    .join("eeyf")
                    .join("presets")
            })
        }
    }

    /// Gets the project presets directory (./.eeyf/presets/)
    fn get_project_presets_dir() -> Option<PathBuf> {
        std::env::current_dir()
            .ok()
            .map(|cwd| cwd.join(".eeyf").join("presets"))
    }

    /// Loads a preset by name.
    ///
    /// Searches in this order:
    /// 1. Built-in presets
    /// 2. Project-local presets (./.eeyf/presets/)
    /// 3. User presets (~/.config/eeyf/presets/)
    ///
    /// # Errors
    ///
    /// Returns an error if the preset is not found or cannot be loaded.
    pub fn load_preset(&self, name: &str) -> Result<PresetConfig, YahooError> {
        // Check built-in presets first
        if let Some(preset) = self.builtins.get(name) {
            return Ok(preset.clone());
        }

        // Check project-local presets
        if let Some(ref dir) = self.project_presets_dir {
            if let Ok(preset) = self.load_preset_from_dir(dir, name) {
                return Ok(preset);
            }
        }

        // Check user presets
        if let Some(ref dir) = self.user_presets_dir {
            if let Ok(preset) = self.load_preset_from_dir(dir, name) {
                return Ok(preset);
            }
        }

        Err(YahooError::InvalidStatusCode(format!(
            "Preset '{}' not found. Available built-in presets: production, development, enterprise, minimal",
            name
        )))
    }

    /// Loads a preset from a specific directory.
    fn load_preset_from_dir(&self, dir: &Path, name: &str) -> Result<PresetConfig, YahooError> {
        // Try .toml first, then .json
        let toml_path = dir.join(format!("{}.toml", name));
        let json_path = dir.join(format!("{}.json", name));

        if toml_path.exists() {
            let content = fs::read_to_string(&toml_path).map_err(|e| {
                YahooError::InvalidStatusCode(format!("Failed to read preset file: {}", e))
            })?;

            let preset: PresetConfig = toml::from_str(&content).map_err(|e| {
                YahooError::InvalidStatusCode(format!("Failed to parse TOML preset: {}", e))
            })?;

            return Ok(preset);
        }

        if json_path.exists() {
            let content = fs::read_to_string(&json_path).map_err(|e| {
                YahooError::InvalidStatusCode(format!("Failed to read preset file: {}", e))
            })?;

            let preset: PresetConfig = serde_json::from_str(&content).map_err(|e| {
                YahooError::InvalidStatusCode(format!("Failed to parse JSON preset: {}", e))
            })?;

            return Ok(preset);
        }

        Err(YahooError::InvalidStatusCode(format!(
            "Preset file not found in directory: {:?}",
            dir
        )))
    }

    /// Saves a preset to the user presets directory.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Attempting to save a built-in preset
    /// - Unable to create the presets directory
    /// - Unable to write the preset file
    pub fn save_preset(
        &self,
        preset: &PresetConfig,
        format: PresetFormat,
    ) -> Result<(), YahooError> {
        // Prevent overwriting built-in presets
        if self.builtins.contains_key(&preset.name) {
            return Err(YahooError::InvalidStatusCode(format!(
                "Cannot overwrite built-in preset '{}'",
                preset.name
            )));
        }

        let dir = self.user_presets_dir.as_ref().ok_or_else(|| {
            YahooError::InvalidStatusCode("Unable to determine user presets directory".into())
        })?;

        // Create directory if it doesn't exist
        fs::create_dir_all(dir).map_err(|e| {
            YahooError::InvalidStatusCode(format!("Failed to create presets directory: {}", e))
        })?;

        let (filename, content) = match format {
            PresetFormat::Toml => {
                let content = toml::to_string_pretty(preset).map_err(|e| {
                    YahooError::InvalidStatusCode(format!(
                        "Failed to serialize preset to TOML: {}",
                        e
                    ))
                })?;
                (format!("{}.toml", preset.name), content)
            }
            PresetFormat::Json => {
                let content = serde_json::to_string_pretty(preset).map_err(|e| {
                    YahooError::InvalidStatusCode(format!(
                        "Failed to serialize preset to JSON: {}",
                        e
                    ))
                })?;
                (format!("{}.json", preset.name), content)
            }
        };

        let path = dir.join(filename);
        fs::write(&path, content).map_err(|e| {
            YahooError::InvalidStatusCode(format!("Failed to write preset file: {}", e))
        })?;

        Ok(())
    }

    /// Lists all available presets (built-in and user-defined).
    pub fn list_presets(&self) -> Vec<String> {
        let mut presets: Vec<String> = self.builtins.keys().cloned().collect();

        // Add user presets
        if let Some(ref dir) = self.user_presets_dir {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.path().file_stem() {
                        if let Some(name_str) = name.to_str() {
                            if !presets.contains(&name_str.to_string()) {
                                presets.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Add project presets
        if let Some(ref dir) = self.project_presets_dir {
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Some(name) = entry.path().file_stem() {
                        if let Some(name_str) = name.to_str() {
                            if !presets.contains(&name_str.to_string()) {
                                presets.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        presets.sort();
        presets
    }
}

impl Default for PresetManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Format for saving preset configurations.
#[derive(Debug, Clone, Copy)]
pub enum PresetFormat {
    /// TOML format
    Toml,
    /// JSON format
    Json,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_manager_builtins() {
        let manager = PresetManager::new();

        assert!(manager.load_preset("production").is_ok());
        assert!(manager.load_preset("development").is_ok());
        assert!(manager.load_preset("enterprise").is_ok());
        assert!(manager.load_preset("minimal").is_ok());
    }

    #[test]
    fn test_preset_manager_not_found() {
        let manager = PresetManager::new();
        assert!(manager.load_preset("nonexistent").is_err());
    }

    #[test]
    fn test_preset_config_from_builder() {
        let builder = YahooConnectorBuilder::production();
        let preset = PresetConfig::from(&builder);

        assert_eq!(preset.rate_limit, 0.5);
        assert_eq!(preset.circuit_breaker_threshold, 5);
        assert!(preset.enable_tracing);
    }

    #[test]
    fn test_builder_from_preset() {
        let mut preset = PresetConfig::from(&YahooConnectorBuilder::production());
        preset.name = "test".to_string();
        preset.rate_limit = 7.5;
        preset.cache_size = 9999;

        let builder = YahooConnectorBuilder::from(preset);
        assert_eq!(builder.rate_limit, 7.5);
        assert_eq!(builder.cache_size, 9999);
    }

    #[test]
    fn test_list_presets() {
        let manager = PresetManager::new();
        let presets = manager.list_presets();

        assert!(presets.contains(&"production".to_string()));
        assert!(presets.contains(&"development".to_string()));
        assert!(presets.contains(&"enterprise".to_string()));
        assert!(presets.contains(&"minimal".to_string()));
    }

    #[test]
    fn test_prevent_overwrite_builtin() {
        let manager = PresetManager::new();
        let mut preset = PresetConfig::from(&YahooConnectorBuilder::production());
        preset.name = "production".to_string(); // Set to a built-in preset name

        let result = manager.save_preset(&preset, PresetFormat::Toml);
        assert!(result.is_err());
    }
}
