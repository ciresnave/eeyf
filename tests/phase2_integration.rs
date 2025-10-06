//! Integration tests for Phase 2: Observability & Configuration
//!
//! This test verifies that all Phase 2 components work together correctly.

#[cfg(all(feature = "observability", feature = "config-management"))]
mod phase2_integration_tests {
use eeyf::YahooConnector;
use eeyf::config::{ConfigBuilder, ConfigManager};
use eeyf::runtime_config::{FeatureFlag, RuntimeConfigManager};
use std::collections::HashMap;    #[tokio::test]
    async fn test_phase2_integration() {
        // Test configuration management
        let config_manager = ConfigManager::new();
        
        let custom_config = ConfigBuilder::new("test_config")
            .rate_limit(2.0)
            .cache(1000, 300)
            .circuit_breaker(5, 300, 60)
            .custom("test_setting", "test_value")
            .build()
            .unwrap();

        config_manager.add_profile(custom_config).await.unwrap();
        config_manager.set_active_profile("test_config").await.unwrap();
        
        let active_config = config_manager.get_active_config();
        assert_eq!(active_config.name, "test_config");
        assert_eq!(active_config.rate_limit, 2.0);

        // Test runtime configuration with feature flags
        let runtime_manager = RuntimeConfigManager::new();
        
        let test_flag = FeatureFlag {
            name: "test_feature".to_string(),
            description: "Test feature flag".to_string(),
            enabled: true,
            rollout_percentage: 100.0,
            target_groups: vec![],
            conditions: HashMap::new(),
            metadata: HashMap::new(),
        };

        runtime_manager.add_feature_flag(test_flag).await.unwrap();
        
        let context = HashMap::new();
        let is_enabled = runtime_manager.is_feature_enabled("test_feature", &context);
        assert!(is_enabled);

        // Test Yahoo Finance connector with configuration
        let connector = YahooConnector::from_preset("development")
            .unwrap();

        // Test a simple quote request
        if let Ok(response) = connector.get_latest_quotes("AAPL", "1d").await {
            if let Ok(quote) = response.last_quote() {
                assert!(quote.close > rust_decimal::Decimal::ZERO);
                println!("✅ AAPL quote: ${:.2}", quote.close);
            }
        }
    }

    #[tokio::test]
    async fn test_configuration_profiles() {
        let config_manager = ConfigManager::new();
        
        // Test built-in profiles
        let profiles = config_manager.list_profiles();
        assert!(profiles.contains(&"default".to_string()));
        assert!(profiles.contains(&"development".to_string()));
        assert!(profiles.contains(&"production".to_string()));

        // Test switching between profiles
        config_manager.set_active_profile("development").await.unwrap();
        let dev_config = config_manager.get_active_config();
        assert_eq!(dev_config.name, "development");
        
        config_manager.set_active_profile("production").await.unwrap();
        let prod_config = config_manager.get_active_config();
        assert_eq!(prod_config.name, "production");
        
        // Production should have more conservative rate limit
        assert!(prod_config.rate_limit <= dev_config.rate_limit);
    }

    #[tokio::test]
    async fn test_runtime_config_versioning() {
        use eeyf::config::{ConfigProfile, ConfigSource};
        
        let runtime_manager = RuntimeConfigManager::new();
        
        let test_config = ConfigProfile::development();
        let change_event = runtime_manager.apply_config_change(
            test_config,
            ConfigSource::Memory,
            "Test configuration change".to_string(),
            "test_system".to_string(),
        ).await.unwrap();

        assert!(change_event.event_id.len() > 0);
        
        let history = runtime_manager.get_config_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].version, 2); // First version is 1, this is 2
    }
}