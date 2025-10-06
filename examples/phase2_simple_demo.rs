//! Simple Phase 2 Configuration Demo
//!
//! This example demonstrates the configuration management capabilities
//! implemented in Phase 2.2 and 2.3:
//! - Configuration profiles and management
//! - Feature flags and A/B testing
//! - Runtime configuration changes

use eeyf::YahooConnector;
use std::time::Duration;

#[cfg(feature = "config-management")]
use eeyf::{
    config::{ConfigBuilder, ConfigManager},
    runtime_config::{ABTest, ABTestManager, ABTestVariant, ABTestStatus, FeatureFlag, RuntimeConfigManager},
};

use std::collections::HashMap;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Phase 2: Configuration Management Demo");
    println!("================================================\n");

    #[cfg(feature = "config-management")]
    {
        // Phase 2.2: Configuration Management
        println!("⚙️  Phase 2.2: Configuration Management");
        
        let config_manager = ConfigManager::new();
        
        // List built-in profiles
        let profiles = config_manager.list_profiles();
        println!("✅ Available built-in profiles: {:?}", profiles);
        
        // Create custom configuration profiles
        println!("\n📝 Creating Custom Configuration Profiles:");
        
        let high_performance = ConfigBuilder::new("high_performance")
            .rate_limit(10.0) // 10 requests per second
            .cache(20000, 180) // Large cache, 3-minute TTL
            .circuit_breaker(2, 60, 15) // Aggressive circuit breaker
            .custom("parallel_requests", 50)
            .custom("priority_symbols", vec!["AAPL", "GOOGL", "MSFT"])
            .build()?;
            
        let conservative = ConfigBuilder::new("conservative")
            .rate_limit(0.2) // 1 request per 5 seconds
            .cache(1000, 1800) // Small cache, 30-minute TTL
            .circuit_breaker(10, 600, 120) // Tolerant circuit breaker
            .custom("enable_validation", true)
            .custom("max_retries", 5)
            .build()?;

        config_manager.add_profile(high_performance).await?;
        config_manager.add_profile(conservative).await?;
        
        println!("✅ Created 'high_performance' profile (10 req/sec)");
        println!("✅ Created 'conservative' profile (0.2 req/sec)");
        
        // Switch between profiles
        println!("\n🔄 Configuration Profile Switching:");
        
        config_manager.set_active_profile("high_performance").await?;
        let active = config_manager.get_active_config();
        println!("✅ Active: {} (rate: {} req/sec, cache: {} entries)", 
                 active.name, active.rate_limit, active.cache.size);
        
        config_manager.set_active_profile("conservative").await?;
        let active = config_manager.get_active_config();
        println!("✅ Active: {} (rate: {} req/sec, cache: {} entries)", 
                 active.name, active.rate_limit, active.cache.size);

        // Phase 2.3: Runtime Configuration and Feature Flags
        println!("\n🚩 Phase 2.3: Dynamic Feature Flags");
        
        let runtime_manager = RuntimeConfigManager::new();
        
        // Add feature flags
        let caching_flag = FeatureFlag {
            name: "advanced_caching".to_string(),
            description: "Multi-layer caching with smart invalidation".to_string(),
            enabled: true,
            rollout_percentage: 80.0, // 80% rollout
            target_groups: vec!["beta_testers".to_string()],
            conditions: HashMap::new(),
            metadata: HashMap::new(),
        };
        
        let real_time_flag = FeatureFlag {
            name: "real_time_quotes".to_string(),
            description: "Real-time quote streaming".to_string(),
            enabled: true,
            rollout_percentage: 100.0,
            target_groups: vec![],
            conditions: {
                let mut conditions = HashMap::new();
                conditions.insert("subscription".to_string(), "premium".to_string());
                conditions
            },
            metadata: HashMap::new(),
        };

        runtime_manager.add_feature_flag(caching_flag).await?;
        runtime_manager.add_feature_flag(real_time_flag).await?;
        
        // Test feature flags for different user contexts
        println!("\n🧪 Feature Flag Testing:");
        
        let mut beta_user = HashMap::new();
        beta_user.insert("user_group".to_string(), "beta_testers".to_string());
        beta_user.insert("subscription".to_string(), "premium".to_string());
        
        let mut regular_user = HashMap::new();
        regular_user.insert("subscription".to_string(), "basic".to_string());
        
        println!("Beta user feature access:");
        println!("  - advanced_caching: {}", 
                 runtime_manager.is_feature_enabled("advanced_caching", &beta_user));
        println!("  - real_time_quotes: {}", 
                 runtime_manager.is_feature_enabled("real_time_quotes", &beta_user));
                 
        println!("Regular user feature access:");
        println!("  - advanced_caching: {}", 
                 runtime_manager.is_feature_enabled("advanced_caching", &regular_user));
        println!("  - real_time_quotes: {}", 
                 runtime_manager.is_feature_enabled("real_time_quotes", &regular_user));

        // A/B Testing
        println!("\n🔬 A/B Testing Framework:");
        
        let ab_manager = ABTestManager::new();
        
        let cache_test = ABTest {
            name: "cache_optimization".to_string(),
            description: "Test different caching strategies".to_string(),
            variants: vec![
                ABTestVariant {
                    name: "aggressive".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigBuilder::new("aggressive_cache")
                        .cache(50000, 120) // Very large cache, 2-minute TTL
                        .rate_limit(15.0)
                        .build()?,
                },
                ABTestVariant {
                    name: "balanced".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigBuilder::new("balanced_cache")
                        .cache(10000, 300) // Moderate cache, 5-minute TTL
                        .rate_limit(8.0)
                        .build()?,
                },
            ],
            traffic_percentage: 100.0,
            status: ABTestStatus::Active,
            start_time: SystemTime::now(),
            end_time: None,
        };

        ab_manager.add_test(cache_test)?;
        
        // Show user assignments
        println!("A/B Test 'cache_optimization' user assignments:");
        for i in 1..=6 {
            let user_id = format!("user_{:03}", i);
            if let Some(config) = ab_manager.get_config_for_user("cache_optimization", &user_id) {
                println!("  - {} → {} variant (cache: {})", 
                         user_id, config.name, config.cache.size);
            }
        }

        // Configuration versioning
        println!("\n📚 Configuration Versioning:");
        
        let experimental = ConfigBuilder::new("experimental_v2")
            .rate_limit(25.0) // Very aggressive
            .cache(100000, 60) // Huge cache, 1-minute TTL
            .circuit_breaker(1, 30, 5) // Hair-trigger circuit breaker
            .build()?;

        let change_event = runtime_manager.apply_config_change(
            experimental,
            eeyf::config::ConfigSource::Memory,
            "Experimental high-throughput configuration".to_string(),
            "demo_system".to_string(),
        ).await?;

        println!("✅ Applied experimental config");
        println!("   Event ID: {}", change_event.event_id);
        println!("   Change type: {:?}", change_event.change_type);
        
        let history = runtime_manager.get_config_history();
        println!("✅ Configuration history: {} versions", history.len());
        
        for (_i, version) in history.iter().enumerate() {
            println!("   v{}: {} ({})", 
                     version.version, 
                     version.config.name,
                     version.description);
        }
    }
    
    #[cfg(not(feature = "config-management"))]
    {
        println!("ℹ️  Configuration management features not enabled.");
        println!("   Run with: cargo run --example phase2_simple_demo --features config-management");
    }

    // Test Yahoo Finance integration with default connector
    println!("\n📈 Yahoo Finance Integration Test");
    
    let connector = YahooConnector::new()?;
    println!("✅ Yahoo Finance connector created");

    // Test a few popular symbols
    let test_symbols = ["AAPL", "MSFT", "GOOGL"];
    
    for symbol in test_symbols.iter() {
        match connector.get_latest_quotes(symbol, "1d").await {
            Ok(response) => {
                if let Ok(quote) = response.last_quote() {
                    println!("✅ {}: ${:.2} (volume: {})", 
                             symbol, quote.close, quote.volume);
                } else {
                    println!("⚠️  {}: Quote available but no data", symbol);
                }
            }
            Err(e) => {
                println!("❌ {}: {}", symbol, e);
            }
        }
        
        // Brief delay between requests
        tokio::time::sleep(Duration::from_millis(300)).await;
    }

    println!("\n✨ Phase 2 Configuration Demo Complete!");
    println!("======================================");
    
    #[cfg(feature = "config-management")]
    {
        println!("✅ Configuration Management:");
        println!("   - Multiple configuration profiles");
        println!("   - Dynamic profile switching");
        println!("   - Configuration validation");
        println!("   - Fluent builder API");
        
        println!("✅ Runtime Configuration:");
        println!("   - Feature flags with rollout control");
        println!("   - User group and condition targeting");
        println!("   - A/B testing framework");
        println!("   - Configuration versioning and history");
    }
    
    println!("✅ Yahoo Finance Integration:");
    println!("   - Real-time quote fetching");
    println!("   - Multiple symbol support");
    println!("   - Error handling and resilience");
    
    println!("\n🚀 Phase 2 successfully demonstrates:");
    println!("   • Advanced configuration management with profiles");
    println!("   • Dynamic feature flags and A/B testing");
    println!("   • Real-time configuration updates");
    println!("   • Robust Yahoo Finance API integration");
    
    println!("\n🎯 Ready for Phase 3: Performance & Reliability!");

    Ok(())
}

/// Helper to display a configuration profile nicely
#[cfg(feature = "config-management")]
#[allow(dead_code)]
fn display_profile(profile: &eeyf::config::ConfigProfile) {
    println!("  Profile: {}", profile.name);
    println!("    Rate limit: {} req/sec", profile.rate_limit);
    println!("    Cache: {} entries, {} sec TTL", profile.cache.size, profile.cache.ttl_secs);
    println!("    Circuit breaker: {} failures / {} sec window", 
             profile.circuit_breaker.threshold, profile.circuit_breaker.window_secs);
    if !profile.custom.is_empty() {
        println!("    Custom settings: {} entries", profile.custom.len());
    }
}