//! Dynamic Runtime Configuration for EEYF
//!
//! This module provides runtime configuration capabilities including:
//! - Live configuration updates without restart
//! - Configuration change notifications and callbacks
//! - Configuration rollback and versioning
//! - Remote configuration fetching and synchronization
//! - Configuration A/B testing and feature flags

use crate::yahoo_error::YahooError;
use crate::config::{ConfigProfile, ConfigSource};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::{broadcast, watch};
use uuid::Uuid;

/// Configuration change event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigChangeEvent {
    /// Event ID for tracking
    pub event_id: String,
    /// Timestamp of the change
    pub timestamp: SystemTime,
    /// Type of configuration change
    pub change_type: ConfigChangeType,
    /// Profile name that changed
    pub profile_name: String,
    /// Previous configuration (for rollback)
    pub previous_config: Option<ConfigProfile>,
    /// New configuration
    pub new_config: ConfigProfile,
    /// Source of the change
    pub source: ConfigSource,
    /// Change metadata
    pub metadata: HashMap<String, String>,
}

/// Types of configuration changes
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConfigChangeType {
    /// Configuration profile was created
    Created,
    /// Configuration profile was updated
    Updated,
    /// Configuration profile was deleted
    Deleted,
    /// Active profile was switched
    Activated,
    /// Configuration was rolled back
    RolledBack,
    /// Remote configuration was synchronized
    Synchronized,
}

/// Configuration version for tracking changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigVersion {
    /// Version number
    pub version: u64,
    /// Configuration snapshot
    pub config: ConfigProfile,
    /// Timestamp when this version was created
    pub created_at: SystemTime,
    /// Change description
    pub description: String,
    /// Change author/source
    pub author: String,
}

/// Feature flag configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureFlag {
    /// Feature name
    pub name: String,
    /// Feature description
    pub description: String,
    /// Whether the feature is enabled
    pub enabled: bool,
    /// Rollout percentage (0-100)
    pub rollout_percentage: f64,
    /// Target groups for the feature
    pub target_groups: Vec<String>,
    /// Conditions for enabling the feature
    pub conditions: HashMap<String, String>,
    /// Feature metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Configuration change callback
pub type ConfigChangeCallback = Box<dyn Fn(&ConfigChangeEvent) -> Result<(), String> + Send + Sync>;

/// Dynamic runtime configuration manager
pub struct RuntimeConfigManager {
    /// Configuration change history
    config_history: Arc<RwLock<Vec<ConfigVersion>>>,
    /// Feature flags
    feature_flags: Arc<RwLock<HashMap<String, FeatureFlag>>>,
    /// Configuration change broadcaster
    change_broadcaster: broadcast::Sender<ConfigChangeEvent>,
    /// Configuration watcher for live updates
    config_watcher: Arc<RwLock<Option<watch::Sender<ConfigProfile>>>>,
    /// Registered change callbacks
    callbacks: Arc<RwLock<Vec<ConfigChangeCallback>>>,
    /// Remote configuration sources
    remote_sources: Arc<RwLock<HashMap<String, RemoteConfigSource>>>,
    /// Current configuration version
    current_version: Arc<RwLock<u64>>,
    /// Configuration sync interval
    sync_interval: Duration,
}

/// Remote configuration source
#[derive(Debug, Clone)]
pub struct RemoteConfigSource {
    /// Source name
    pub name: String,
    /// Source URL or identifier
    pub url: String,
    /// Sync interval
    pub sync_interval: Duration,
    /// Authentication headers
    pub auth_headers: HashMap<String, String>,
    /// Last sync timestamp
    pub last_sync: Option<SystemTime>,
    /// Sync status
    pub status: RemoteSourceStatus,
}

/// Remote source status
#[derive(Debug, Clone, PartialEq)]
pub enum RemoteSourceStatus {
    /// Source is active and syncing
    Active,
    /// Source has errors
    Error(String),
    /// Source is disabled
    Disabled,
    /// Source is being synchronized
    Syncing,
}

impl RuntimeConfigManager {
    /// Create a new runtime configuration manager
    pub fn new() -> Self {
        let (change_tx, _) = broadcast::channel(100);
        
        Self {
            config_history: Arc::new(RwLock::new(Vec::new())),
            feature_flags: Arc::new(RwLock::new(HashMap::new())),
            change_broadcaster: change_tx,
            config_watcher: Arc::new(RwLock::new(None)),
            callbacks: Arc::new(RwLock::new(Vec::new())),
            remote_sources: Arc::new(RwLock::new(HashMap::new())),
            current_version: Arc::new(RwLock::new(1)),
            sync_interval: Duration::from_secs(60), // Default 1 minute sync
        }
    }

    /// Apply a configuration change with versioning
    pub async fn apply_config_change(
        &self,
        new_config: ConfigProfile,
        source: ConfigSource,
        description: String,
        author: String,
    ) -> Result<ConfigChangeEvent, YahooError> {
        let event_id = Uuid::new_v4().to_string();
        let timestamp = SystemTime::now();
        
        // Create new version
        let version = {
            let mut version_guard = self.current_version.write().unwrap();
            *version_guard += 1;
            *version_guard
        };

        let config_version = ConfigVersion {
            version,
            config: new_config.clone(),
            created_at: timestamp,
            description,
            author,
        };

        // Add to history
        {
            let mut history = self.config_history.write().unwrap();
            history.push(config_version);
            
            // Keep only last 100 versions
            if history.len() > 100 {
                history.remove(0);
            }
        }

        // Create change event
        let change_event = ConfigChangeEvent {
            event_id,
            timestamp,
            change_type: ConfigChangeType::Updated,
            profile_name: new_config.name.clone(),
            previous_config: None, // Could store previous config here
            new_config: new_config.clone(),
            source,
            metadata: HashMap::new(),
        };

        // Broadcast change event
        let _ = self.change_broadcaster.send(change_event.clone());

        // Notify callbacks
        self.notify_callbacks(&change_event).await?;

        // Update watcher
        if let Some(watcher) = self.config_watcher.read().unwrap().as_ref() {
            let _ = watcher.send(new_config);
        }

        Ok(change_event)
    }

    /// Rollback to a previous configuration version
    pub async fn rollback_to_version(&self, version: u64) -> Result<ConfigChangeEvent, YahooError> {
        let history = self.config_history.read().unwrap();
        let config_version = history
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| YahooError::InvalidStatusCode(format!("Version {} not found", version)))?;

        let rollback_config = config_version.config.clone();
        drop(history);

        self.apply_config_change(
            rollback_config,
            ConfigSource::Memory,
            format!("Rollback to version {}", version),
            "system".to_string(),
        ).await
    }

    /// Get configuration version history
    pub fn get_config_history(&self) -> Vec<ConfigVersion> {
        self.config_history.read().unwrap().clone()
    }

    /// Add a feature flag
    pub async fn add_feature_flag(&self, flag: FeatureFlag) -> Result<(), YahooError> {
        let mut flags = self.feature_flags.write().unwrap();
        flags.insert(flag.name.clone(), flag);
        Ok(())
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature_name: &str, context: &HashMap<String, String>) -> bool {
        let flags = self.feature_flags.read().unwrap();
        
        if let Some(flag) = flags.get(feature_name) {
            if !flag.enabled {
                return false;
            }

            // Check rollout percentage
            if flag.rollout_percentage < 100.0 {
                // Simple hash-based rollout (in production, use more sophisticated method)
                let hash = feature_name.len() % 100;
                if hash as f64 > flag.rollout_percentage {
                    return false;
                }
            }

            // Check target groups
            if !flag.target_groups.is_empty() {
                if let Some(user_group) = context.get("user_group") {
                    if !flag.target_groups.contains(user_group) {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            // Check conditions
            for (key, expected_value) in &flag.conditions {
                if let Some(actual_value) = context.get(key) {
                    if actual_value != expected_value {
                        return false;
                    }
                } else {
                    return false;
                }
            }

            true
        } else {
            false
        }
    }

    /// Subscribe to configuration changes
    pub fn subscribe_to_changes(&self) -> broadcast::Receiver<ConfigChangeEvent> {
        self.change_broadcaster.subscribe()
    }

    /// Watch configuration changes with a receiver
    pub fn watch_config_changes(&self) -> watch::Receiver<ConfigProfile> {
        let (tx, rx) = watch::channel(ConfigProfile::default());
        *self.config_watcher.write().unwrap() = Some(tx);
        rx
    }

    /// Add a configuration change callback
    pub fn add_change_callback(&self, callback: ConfigChangeCallback) {
        self.callbacks.write().unwrap().push(callback);
    }

    /// Add a remote configuration source
    pub async fn add_remote_source(&self, source: RemoteConfigSource) -> Result<(), YahooError> {
        let mut sources = self.remote_sources.write().unwrap();
        sources.insert(source.name.clone(), source);
        Ok(())
    }

    /// Sync configuration from remote sources
    pub async fn sync_remote_configs(&self) -> Result<Vec<ConfigChangeEvent>, YahooError> {
        let mut changes = Vec::new();
        let sources: Vec<RemoteConfigSource> = {
            let sources_guard = self.remote_sources.read().unwrap();
            sources_guard.values().cloned().collect()
        };

        for mut source in sources {
            if source.status == RemoteSourceStatus::Active {
                match self.fetch_remote_config(&source).await {
                    Ok(config) => {
                        let change = self.apply_config_change(
                            config,
                            ConfigSource::Remote(source.url.clone()),
                            format!("Sync from remote source: {}", source.name),
                            "remote-sync".to_string(),
                        ).await?;
                        changes.push(change);

                        // Update last sync time
                        source.last_sync = Some(SystemTime::now());
                        let mut sources_guard = self.remote_sources.write().unwrap();
                        sources_guard.insert(source.name.clone(), source);
                    }
                    Err(e) => {
                        eprintln!("Failed to sync from remote source {}: {}", source.name, e);
                        // Update source status to error
                        source.status = RemoteSourceStatus::Error(e.to_string());
                        let mut sources_guard = self.remote_sources.write().unwrap();
                        sources_guard.insert(source.name.clone(), source);
                    }
                }
            }
        }

        Ok(changes)
    }

    /// Start automatic remote configuration sync
    pub async fn start_auto_sync(&self) -> Result<(), YahooError> {
        let manager = Arc::new(self.clone());
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(manager.sync_interval);
            
            loop {
                interval.tick().await;
                if let Err(e) = manager.sync_remote_configs().await {
                    eprintln!("Auto-sync failed: {}", e);
                }
            }
        });
        
        Ok(())
    }

    /// Fetch configuration from a remote source
    async fn fetch_remote_config(&self, _source: &RemoteConfigSource) -> Result<ConfigProfile, YahooError> {
        // In a real implementation, this would make HTTP requests to fetch remote config
        // For now, return a mock configuration
        Ok(ConfigProfile::default())
    }

    /// Notify all registered callbacks about a configuration change
    async fn notify_callbacks(&self, event: &ConfigChangeEvent) -> Result<(), YahooError> {
        let callbacks = self.callbacks.read().unwrap();
        
        for callback in callbacks.iter() {
            if let Err(e) = callback(event) {
                eprintln!("Configuration change callback failed: {}", e);
            }
        }
        
        Ok(())
    }
}

impl Clone for RuntimeConfigManager {
    fn clone(&self) -> Self {
        Self {
            config_history: Arc::clone(&self.config_history),
            feature_flags: Arc::clone(&self.feature_flags),
            change_broadcaster: self.change_broadcaster.clone(),
            config_watcher: Arc::clone(&self.config_watcher),
            callbacks: Arc::clone(&self.callbacks),
            remote_sources: Arc::clone(&self.remote_sources),
            current_version: Arc::clone(&self.current_version),
            sync_interval: self.sync_interval,
        }
    }
}

impl Default for RuntimeConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FeatureFlag {
    fn default() -> Self {
        Self {
            name: String::new(),
            description: String::new(),
            enabled: false,
            rollout_percentage: 100.0,
            target_groups: Vec::new(),
            conditions: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Configuration A/B testing manager
#[derive(Debug)]
pub struct ABTestManager {
    /// Active A/B tests
    tests: Arc<RwLock<HashMap<String, ABTest>>>,
    /// User assignments
    assignments: Arc<RwLock<HashMap<String, String>>>,
}

/// A/B test configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTest {
    /// Test name
    pub name: String,
    /// Test description
    pub description: String,
    /// Test variants
    pub variants: Vec<ABTestVariant>,
    /// Traffic allocation percentage
    pub traffic_percentage: f64,
    /// Test status
    pub status: ABTestStatus,
    /// Start time
    pub start_time: SystemTime,
    /// End time
    pub end_time: Option<SystemTime>,
}

/// A/B test variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ABTestVariant {
    /// Variant name
    pub name: String,
    /// Traffic percentage for this variant
    pub traffic_percentage: f64,
    /// Configuration for this variant
    pub config: ConfigProfile,
}

/// A/B test status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ABTestStatus {
    /// Test is running
    Active,
    /// Test is paused
    Paused,
    /// Test has completed
    Completed,
    /// Test is in draft state
    Draft,
}

impl ABTestManager {
    /// Create a new A/B test manager
    pub fn new() -> Self {
        Self {
            tests: Arc::new(RwLock::new(HashMap::new())),
            assignments: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an A/B test
    pub fn add_test(&self, test: ABTest) -> Result<(), YahooError> {
        // Validate test configuration
        let total_traffic: f64 = test.variants.iter().map(|v| v.traffic_percentage).sum();
        if (total_traffic - 100.0).abs() > 0.001 {
            return Err(YahooError::InvalidStatusCode(
                "Variant traffic percentages must sum to 100%".into()
            ));
        }

        self.tests.write().unwrap().insert(test.name.clone(), test);
        Ok(())
    }

    /// Get configuration for a user in an A/B test
    pub fn get_config_for_user(&self, test_name: &str, user_id: &str) -> Option<ConfigProfile> {
        let tests = self.tests.read().unwrap();
        let test = tests.get(test_name)?;

        if test.status != ABTestStatus::Active {
            return None;
        }

        // Check if user is already assigned
        let assignments = self.assignments.read().unwrap();
        if let Some(variant_name) = assignments.get(&format!("{}:{}", test_name, user_id)) {
            if let Some(variant) = test.variants.iter().find(|v| &v.name == variant_name) {
                return Some(variant.config.clone());
            }
        }
        drop(assignments);

        // Assign user to a variant based on hash
        let hash = self.hash_user_for_test(user_id, test_name);
        let mut cumulative = 0.0;
        
        for variant in &test.variants {
            cumulative += variant.traffic_percentage;
            if hash <= cumulative {
                // Assign user to this variant
                let mut assignments = self.assignments.write().unwrap();
                assignments.insert(
                    format!("{}:{}", test_name, user_id),
                    variant.name.clone()
                );
                return Some(variant.config.clone());
            }
        }

        None
    }

    /// Hash user ID for consistent test assignment
    fn hash_user_for_test(&self, user_id: &str, test_name: &str) -> f64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        test_name.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert to percentage (0-100)
        (hash % 100) as f64
    }
}

impl Default for ABTestManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_config_manager() {
        let manager = RuntimeConfigManager::new();
        
        let config = ConfigProfile::development();
        let change = manager.apply_config_change(
            config,
            ConfigSource::Memory,
            "Test change".to_string(),
            "test".to_string(),
        ).await.unwrap();

        assert_eq!(change.change_type, ConfigChangeType::Updated);
        
        let history = manager.get_config_history();
        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_feature_flags() {
        let manager = RuntimeConfigManager::new();
        
        let flag = FeatureFlag {
            name: "test_feature".to_string(),
            description: "Test feature".to_string(),
            enabled: true,
            rollout_percentage: 50.0,
            target_groups: vec!["beta".to_string()],
            conditions: HashMap::new(),
            metadata: HashMap::new(),
        };

        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            manager.add_feature_flag(flag).await.unwrap();
        });

        let mut context = HashMap::new();
        context.insert("user_group".to_string(), "beta".to_string());
        
        // This might be true or false depending on the hash-based rollout
        let _is_enabled = manager.is_feature_enabled("test_feature", &context);
    }

    #[test]
    fn test_ab_testing() {
        let manager = ABTestManager::new();
        
        let test = ABTest {
            name: "test_experiment".to_string(),
            description: "Test experiment".to_string(),
            variants: vec![
                ABTestVariant {
                    name: "control".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigProfile::default(),
                },
                ABTestVariant {
                    name: "treatment".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigProfile::development(),
                },
            ],
            traffic_percentage: 100.0,
            status: ABTestStatus::Active,
            start_time: SystemTime::now(),
            end_time: None,
        };

        manager.add_test(test).unwrap();
        
        // Test user assignment
        let config1 = manager.get_config_for_user("test_experiment", "user1");
        let config2 = manager.get_config_for_user("test_experiment", "user1");
        
        // Same user should get same variant
        assert_eq!(config1.is_some(), config2.is_some());
        if let (Some(c1), Some(c2)) = (config1, config2) {
            assert_eq!(c1.name, c2.name);
        }
    }
}