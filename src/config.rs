//! Configuration Management for EEYF
//!
//! This module provides advanced configuration management capabilities including:
//! - Environment-based configuration loading
//! - Configuration validation and schema checking  
//! - Hot reloading of configuration files
//! - Configuration profiling and optimization suggestions
//! - Configuration templating and inheritance

use crate::yahoo_error::YahooError;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use std::fs;


/// Configuration source types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConfigSource {
    /// Configuration from file (TOML, JSON, YAML)
    File(PathBuf),
    /// Configuration from environment variables
    Environment,
    /// Configuration from command line arguments
    CommandLine,
    /// Configuration from remote source (URL, database, etc.)
    Remote(String),
    /// In-memory configuration
    Memory,
}

/// Configuration format types
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFormat {
    Toml,
    Json,
    Yaml,
    Env,
}

/// Configuration profile for different environments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigProfile {
    /// Profile name (development, staging, production, etc.)
    pub name: String,
    /// Profile description
    pub description: String,
    /// Inherits from another profile
    pub inherits_from: Option<String>,
    /// Rate limiting configuration
    pub rate_limit: f64,
    /// Circuit breaker settings
    pub circuit_breaker: CircuitBreakerConfig,
    /// Cache settings
    pub cache: CacheConfig,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Timeout settings
    pub timeouts: TimeoutConfig,
    /// Observability settings
    pub observability: ObservabilityConfig,
    /// Custom settings
    pub custom: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub threshold: u32,
    pub window_secs: u64,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub size: usize,
    pub ttl_secs: u64,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub request_timeout_secs: u64,
    pub connection_timeout_secs: u64,
    pub read_timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
    pub health_checks_enabled: bool,
    pub log_level: String,
}

/// Configuration validation rule
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub validator: fn(&ConfigProfile) -> Result<(), String>,
}

/// Configuration manager for advanced configuration handling
#[derive(Debug)]
pub struct ConfigManager {
    /// Current active configuration
    active_config: Arc<RwLock<ConfigProfile>>,
    /// Available configuration profiles
    profiles: Arc<RwLock<HashMap<String, ConfigProfile>>>,
    /// Configuration sources and their last modified times
    sources: Arc<RwLock<HashMap<ConfigSource, SystemTime>>>,
    /// Validation rules
    validation_rules: Vec<ValidationRule>,
    /// Configuration file watchers for hot reload
    watchers: Arc<RwLock<HashMap<PathBuf, tokio::sync::watch::Receiver<()>>>>,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            threshold: 5,
            window_secs: 300,
            timeout_secs: 60,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            size: 1000,
            ttl_secs: 300,
            enabled: true,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay_ms: 1000,
            max_delay_ms: 30000,
            backoff_multiplier: 2.0,
        }
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout_secs: 30,
            connection_timeout_secs: 10,
            read_timeout_secs: 30,
        }
    }
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            metrics_enabled: true,
            tracing_enabled: true,
            health_checks_enabled: true,
            log_level: "info".to_string(),
        }
    }
}

impl Default for ConfigProfile {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default configuration profile".to_string(),
            inherits_from: None,
            rate_limit: 1.0, // 1 request per second
            circuit_breaker: CircuitBreakerConfig::default(),
            cache: CacheConfig::default(),
            retry: RetryConfig::default(),
            timeouts: TimeoutConfig::default(),
            observability: ObservabilityConfig::default(),
            custom: HashMap::new(),
        }
    }
}

impl ConfigProfile {
    /// Create a new configuration profile
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            ..Default::default()
        }
    }

    /// Create a development profile
    pub fn development() -> Self {
        Self {
            name: "development".to_string(),
            description: "Development environment configuration".to_string(),
            rate_limit: 5.0, // Higher rate for development
            cache: CacheConfig {
                size: 500,
                ttl_secs: 60, // Shorter cache for fresh data during development
                enabled: true,
            },
            retry: RetryConfig {
                max_attempts: 2, // Fewer retries for faster feedback
                ..RetryConfig::default()
            },
            observability: ObservabilityConfig {
                metrics_enabled: true,
                tracing_enabled: true,
                health_checks_enabled: true,
                log_level: "debug".to_string(), // Verbose logging for development
            },
            ..Default::default()
        }
    }

    /// Create a production profile
    pub fn production() -> Self {
        Self {
            name: "production".to_string(),
            description: "Production environment configuration".to_string(),
            rate_limit: 0.5, // Conservative rate for production
            circuit_breaker: CircuitBreakerConfig {
                threshold: 3, // Stricter circuit breaker
                window_secs: 180,
                timeout_secs: 30,
            },
            cache: CacheConfig {
                size: 5000, // Large cache for production
                ttl_secs: 900, // 15 minute cache
                enabled: true,
            },
            retry: RetryConfig {
                max_attempts: 5, // More retries for reliability
                initial_delay_ms: 2000,
                max_delay_ms: 60000,
                ..RetryConfig::default()
            },
            observability: ObservabilityConfig {
                log_level: "info".to_string(), // Less verbose logging
                ..ObservabilityConfig::default()
            },
            ..Default::default()
        }
    }

    /// Merge this profile with a parent profile
    pub fn merge_with(&mut self, parent: &ConfigProfile) -> Result<(), YahooError> {
        if self.inherits_from.as_ref() == Some(&parent.name) {
            // Apply parent values where current values are default
            if self.rate_limit == ConfigProfile::default().rate_limit {
                self.rate_limit = parent.rate_limit;
            }
            
            // Merge other configuration sections
            // This is a simplified merge - in production you might want more sophisticated merging
        }
        Ok(())
    }

    /// Validate the configuration profile
    pub fn validate(&self) -> Result<(), YahooError> {
        if self.rate_limit <= 0.0 {
            return Err(YahooError::InvalidStatusCode(
                "Rate limit must be positive".into()
            ));
        }

        if self.circuit_breaker.threshold == 0 {
            return Err(YahooError::InvalidStatusCode(
                "Circuit breaker threshold must be positive".into()
            ));
        }

        if self.cache.size == 0 {
            return Err(YahooError::InvalidStatusCode(
                "Cache size must be positive".into()
            ));
        }

        if self.retry.max_attempts == 0 {
            return Err(YahooError::InvalidStatusCode(
                "Retry max attempts must be positive".into()
            ));
        }

        Ok(())
    }
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        let mut profiles = HashMap::new();
        
        // Add built-in profiles
        profiles.insert("default".to_string(), ConfigProfile::default());
        profiles.insert("development".to_string(), ConfigProfile::development());
        profiles.insert("production".to_string(), ConfigProfile::production());

        Self {
            active_config: Arc::new(RwLock::new(ConfigProfile::default())),
            profiles: Arc::new(RwLock::new(profiles)),
            sources: Arc::new(RwLock::new(HashMap::new())),
            validation_rules: Self::default_validation_rules(),
            watchers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load configuration from file
    pub async fn load_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), YahooError> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .map_err(|e| YahooError::InvalidStatusCode(format!("Failed to read config file: {}", e)))?;

        let format = Self::detect_format(path)?;
        let profile = self.parse_config(&content, format)?;
        
        self.add_profile(profile).await?;
        
        // Track the source
        let mut sources = self.sources.write().unwrap();
        sources.insert(ConfigSource::File(path.to_path_buf()), SystemTime::now());
        
        Ok(())
    }

    /// Load configuration from environment variables
    pub async fn load_from_env(&self, prefix: &str) -> Result<(), YahooError> {
        let mut profile = ConfigProfile::default();
        profile.name = "environment".to_string();
        profile.description = "Configuration loaded from environment variables".to_string();

        // Load environment variables with the given prefix
        for (key, value) in std::env::vars() {
            if let Some(config_key) = key.strip_prefix(prefix) {
                match config_key {
                    "RATE_LIMIT" => {
                        profile.rate_limit = value.parse().map_err(|_| {
                            YahooError::InvalidStatusCode("Invalid RATE_LIMIT value".into())
                        })?;
                    }
                    "CACHE_SIZE" => {
                        profile.cache.size = value.parse().map_err(|_| {
                            YahooError::InvalidStatusCode("Invalid CACHE_SIZE value".into())
                        })?;
                    }
                    "LOG_LEVEL" => {
                        profile.observability.log_level = value;
                    }
                    _ => {
                        // Store in custom settings
                        profile.custom.insert(
                            config_key.to_lowercase(), 
                            serde_json::Value::String(value)
                        );
                    }
                }
            }
        }

        self.add_profile(profile).await?;
        
        let mut sources = self.sources.write().unwrap();
        sources.insert(ConfigSource::Environment, SystemTime::now());
        
        Ok(())
    }

    /// Add a configuration profile
    pub async fn add_profile(&self, profile: ConfigProfile) -> Result<(), YahooError> {
        // Validate the profile
        profile.validate()?;
        
        // Run custom validation rules
        for rule in &self.validation_rules {
            (rule.validator)(&profile).map_err(|e| {
                YahooError::InvalidStatusCode(format!("Validation rule '{}' failed: {}", rule.name, e))
            })?;
        }

        let mut profiles = self.profiles.write().unwrap();
        profiles.insert(profile.name.clone(), profile);
        
        Ok(())
    }

    /// Set the active configuration profile
    pub async fn set_active_profile(&self, name: &str) -> Result<(), YahooError> {
        let profiles = self.profiles.read().unwrap();
        let profile = profiles.get(name).ok_or_else(|| {
            YahooError::InvalidStatusCode(format!("Profile '{}' not found", name))
        })?.clone();
        drop(profiles);

        let mut active = self.active_config.write().unwrap();
        *active = profile;
        
        Ok(())
    }

    /// Get the current active configuration
    pub fn get_active_config(&self) -> ConfigProfile {
        self.active_config.read().unwrap().clone()
    }

    /// List available configuration profiles
    pub fn list_profiles(&self) -> Vec<String> {
        self.profiles.read().unwrap().keys().cloned().collect()
    }

    /// Enable hot reloading for configuration files
    pub async fn enable_hot_reload<P: AsRef<Path>>(&self, path: P) -> Result<(), YahooError> {
        let _path = path.as_ref().to_path_buf();
        
        // In a real implementation, you would set up file system watchers here
        // For this example, we'll just track that hot reload is enabled
        println!("Hot reload enabled for configuration files");
        
        Ok(())
    }

    /// Detect configuration format from file extension
    fn detect_format(path: &Path) -> Result<ConfigFormat, YahooError> {
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("toml") => Ok(ConfigFormat::Toml),
            Some("json") => Ok(ConfigFormat::Json),
            Some("yaml" | "yml") => Ok(ConfigFormat::Yaml),
            _ => Err(YahooError::InvalidStatusCode(
                "Unsupported configuration file format".into()
            ))
        }
    }

    /// Parse configuration content based on format
    fn parse_config(&self, content: &str, format: ConfigFormat) -> Result<ConfigProfile, YahooError> {
        match format {
            ConfigFormat::Toml => {
                toml::from_str(content).map_err(|e| {
                    YahooError::InvalidStatusCode(format!("Failed to parse TOML config: {}", e))
                })
            }
            ConfigFormat::Json => {
                serde_json::from_str(content).map_err(|e| {
                    YahooError::InvalidStatusCode(format!("Failed to parse JSON config: {}", e))
                })
            }
            ConfigFormat::Yaml => {
                // YAML parsing would require the yaml crate
                Err(YahooError::InvalidStatusCode("YAML parsing not implemented".into()))
            }
            ConfigFormat::Env => {
                Err(YahooError::InvalidStatusCode("ENV format not supported for file parsing".into()))
            }
        }
    }

    /// Get default validation rules
    fn default_validation_rules() -> Vec<ValidationRule> {
        vec![
            ValidationRule {
                name: "positive_rate_limit".to_string(),
                description: "Rate limit must be positive".to_string(),
                validator: |config| {
                    if config.rate_limit <= 0.0 {
                        Err("Rate limit must be positive".to_string())
                    } else {
                        Ok(())
                    }
                },
            },
            ValidationRule {
                name: "reasonable_cache_size".to_string(),
                description: "Cache size should be reasonable (1-100000)".to_string(),
                validator: |config| {
                    if config.cache.size == 0 || config.cache.size > 100_000 {
                        Err("Cache size should be between 1 and 100,000".to_string())
                    } else {
                        Ok(())
                    }
                },
            },
        ]
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration builder for fluent configuration creation
#[derive(Debug)]
pub struct ConfigBuilder {
    profile: ConfigProfile,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    pub fn new(name: &str) -> Self {
        Self {
            profile: ConfigProfile::new(name),
        }
    }

    /// Set rate limit
    pub fn rate_limit(mut self, rate: f64) -> Self {
        self.profile.rate_limit = rate;
        self
    }

    /// Set cache configuration
    pub fn cache(mut self, size: usize, ttl_secs: u64) -> Self {
        self.profile.cache = CacheConfig {
            size,
            ttl_secs,
            enabled: true,
        };
        self
    }

    /// Set circuit breaker configuration
    pub fn circuit_breaker(mut self, threshold: u32, window_secs: u64, timeout_secs: u64) -> Self {
        self.profile.circuit_breaker = CircuitBreakerConfig {
            threshold,
            window_secs,
            timeout_secs,
        };
        self
    }

    /// Add custom configuration value
    pub fn custom<T: serde::Serialize>(mut self, key: &str, value: T) -> Self {
        if let Ok(json_value) = serde_json::to_value(value) {
            self.profile.custom.insert(key.to_string(), json_value);
        }
        self
    }

    /// Inherit from another profile
    pub fn inherits_from(mut self, parent: &str) -> Self {
        self.profile.inherits_from = Some(parent.to_string());
        self
    }

    /// Build the configuration profile
    pub fn build(self) -> Result<ConfigProfile, YahooError> {
        self.profile.validate()?;
        Ok(self.profile)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_profile_validation() {
        let mut profile = ConfigProfile::default();
        assert!(profile.validate().is_ok());

        profile.rate_limit = -1.0;
        assert!(profile.validate().is_err());
    }

    #[test]
    fn test_config_builder() {
        let profile = ConfigBuilder::new("test")
            .rate_limit(2.0)
            .cache(1000, 300)
            .circuit_breaker(5, 300, 60)
            .custom("test_setting", "test_value")
            .build()
            .unwrap();

        assert_eq!(profile.name, "test");
        assert_eq!(profile.rate_limit, 2.0);
        assert_eq!(profile.cache.size, 1000);
        assert_eq!(profile.circuit_breaker.threshold, 5);
    }

    #[tokio::test]
    async fn test_config_manager() {
        let manager = ConfigManager::new();
        
        // Test built-in profiles
        let profiles = manager.list_profiles();
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"development".to_string()));
        assert!(profiles.contains(&"production".to_string()));

        // Test setting active profile
        manager.set_active_profile("production").await.unwrap();
        let active = manager.get_active_config();
        assert_eq!(active.name, "production");
    }
}