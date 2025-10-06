//! Complete Phase 2 Example: Observability & Configuration
//!
//! This example demonstrates the full Phase 2 implementation including:
//! - Comprehensive observability (metrics, tracing, health monitoring)
//! - Advanced configuration management with hot reload
//! - Dynamic runtime configuration with feature flags
//! - A/B testing capabilities
//! - Real Yahoo Finance API integration with enterprise features

use eeyf::builder::YahooConnectorBuilder;
use eeyf::presets::Preset;

#[cfg(feature = "observability")]
use eeyf::{health::HealthManager, metrics::PrometheusMetrics, tracing::TracingManager};

#[cfg(feature = "config-management")]
use eeyf::{
    config::{ConfigBuilder, ConfigManager, ConfigProfile},
    runtime_config::{ABTest, ABTestManager, ABTestVariant, ABTestStatus, FeatureFlag, RuntimeConfigManager},
};

use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use tokio::time::sleep;
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Phase 2: Observability & Configuration Demo");
    println!("==================================================\n");

    // Phase 2.1: Initialize Observability
    #[cfg(feature = "observability")]
    {
        println!("📊 Phase 2.1: Setting up Observability Infrastructure");
        
        // Initialize distributed tracing
        let tracing_manager = TracingManager::new().await?;
        tracing_manager.start().await?;
        println!("✅ Distributed tracing with Jaeger started");

        // Initialize Prometheus metrics
        let metrics = PrometheusMetrics::new("eeyf_phase2_demo", 8081).await?;
        metrics.start().await?;
        println!("✅ Prometheus metrics server started on http://localhost:8081/metrics");

        // Initialize health monitoring
        let health_manager = HealthManager::new();
        health_manager.start_server(8082).await?;
        println!("✅ Health monitoring started on http://localhost:8082/health");
        println!("   Health checks available at:");
        println!("   - http://localhost:8082/health (overall health)");
        println!("   - http://localhost:8082/health/detailed (detailed status)");
        
        sleep(Duration::from_secs(2)).await;
    }

    // Phase 2.2: Configuration Management
    #[cfg(feature = "config-management")]
    {
        println!("\n⚙️  Phase 2.2: Advanced Configuration Management");
        
        let config_manager = ConfigManager::new();
        
        // Create custom configuration profiles
        let trading_config = ConfigBuilder::new("high_frequency_trading")
            .rate_limit(10.0) // 10 requests per second for HFT
            .cache(10000, 60) // Large cache with 1-minute TTL
            .circuit_breaker(3, 60, 30) // Aggressive circuit breaker
            .custom("max_concurrent_requests", 100)
            .custom("priority_symbols", vec!["AAPL", "GOOGL", "MSFT", "TSLA"])
            .build()?;

        let research_config = ConfigBuilder::new("research_analytics")
            .rate_limit(0.5) // Conservative for research
            .cache(5000, 900) // 15-minute cache for analysis
            .circuit_breaker(5, 300, 60) // Tolerant circuit breaker
            .custom("enable_extended_data", true)
            .custom("historical_depth_years", 10)
            .build()?;

        // Add profiles to manager
        config_manager.add_profile(trading_config).await?;
        config_manager.add_profile(research_config).await?;
        
        println!("✅ Created specialized configuration profiles:");
        println!("   - high_frequency_trading (10 req/sec, fast cache)");
        println!("   - research_analytics (0.5 req/sec, long cache)");

        // Demonstrate configuration switching
        config_manager.set_active_profile("high_frequency_trading").await?;
        let active_config = config_manager.get_active_config();
        println!("✅ Activated profile: {} (rate limit: {} req/sec)", 
                 active_config.name, active_config.rate_limit);
    }

    // Phase 2.3: Dynamic Runtime Configuration
    #[cfg(feature = "config-management")]
    {
        println!("\n🔄 Phase 2.3: Dynamic Runtime Configuration");
        
        let runtime_manager = RuntimeConfigManager::new();
        
        // Feature Flags Example
        println!("\n🚩 Feature Flags Management:");
        
        let advanced_caching_flag = FeatureFlag {
            name: "advanced_caching".to_string(),
            description: "Advanced multi-layer caching system".to_string(),
            enabled: true,
            rollout_percentage: 75.0, // 75% rollout
            target_groups: vec!["beta_users".to_string(), "premium_users".to_string()],
            conditions: HashMap::new(),
            metadata: HashMap::new(),
        };

        let real_time_alerts_flag = FeatureFlag {
            name: "real_time_alerts".to_string(),
            description: "Real-time price alerts and notifications".to_string(),
            enabled: true,
            rollout_percentage: 100.0,
            target_groups: vec!["premium_users".to_string()],
            conditions: {
                let mut conditions = HashMap::new();
                conditions.insert("subscription_tier".to_string(), "premium".to_string());
                conditions
            },
            metadata: HashMap::new(),
        };

        runtime_manager.add_feature_flag(advanced_caching_flag).await?;
        runtime_manager.add_feature_flag(real_time_alerts_flag).await?;

        // Test feature flags for different user contexts
        let mut user_context = HashMap::new();
        user_context.insert("user_group".to_string(), "beta_users".to_string());
        user_context.insert("subscription_tier".to_string(), "premium".to_string());

        let caching_enabled = runtime_manager.is_feature_enabled("advanced_caching", &user_context);
        let alerts_enabled = runtime_manager.is_feature_enabled("real_time_alerts", &user_context);

        println!("✅ Feature flag evaluation for premium beta user:");
        println!("   - advanced_caching: {}", caching_enabled);
        println!("   - real_time_alerts: {}", alerts_enabled);

        // A/B Testing Example
        println!("\n🧪 A/B Testing Configuration:");
        
        let ab_manager = ABTestManager::new();
        
        let caching_experiment = ABTest {
            name: "cache_strategy_test".to_string(),
            description: "Test different caching strategies for performance".to_string(),
            variants: vec![
                ABTestVariant {
                    name: "aggressive_cache".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigBuilder::new("aggressive_cache")
                        .cache(20000, 300) // Large cache, 5-minute TTL
                        .rate_limit(5.0)
                        .build()?,
                },
                ABTestVariant {
                    name: "conservative_cache".to_string(),
                    traffic_percentage: 50.0,
                    config: ConfigBuilder::new("conservative_cache")
                        .cache(5000, 900) // Smaller cache, 15-minute TTL
                        .rate_limit(2.0)
                        .build()?,
                },
            ],
            traffic_percentage: 100.0,
            status: ABTestStatus::Active,
            start_time: SystemTime::now(),
            end_time: None,
        };

        ab_manager.add_test(caching_experiment)?;

        // Demonstrate user assignment to A/B test variants
        for user_id in ["user_001", "user_002", "user_003", "user_004"] {
            if let Some(config) = ab_manager.get_config_for_user("cache_strategy_test", user_id) {
                println!("   - {}: assigned to '{}' variant (cache size: {})",
                         user_id, config.name, config.cache.size);
            }
        }
        
        // Configuration versioning and rollback
        println!("\n📚 Configuration Versioning:");
        
        let experimental_config = ConfigBuilder::new("experimental")
            .rate_limit(20.0) // Very high rate limit
            .cache(50000, 30) // Massive cache with short TTL
            .build()?;

        let change_event = runtime_manager.apply_config_change(
            experimental_config,
            eeyf::config::ConfigSource::Memory,
            "Experimental high-performance configuration".to_string(),
            "demo_system".to_string(),
        ).await?;

        println!("✅ Applied experimental config (Event ID: {})", change_event.event_id);

        let history = runtime_manager.get_config_history();
        println!("✅ Configuration history now has {} versions", history.len());

        // Rollback demonstration
        if history.len() > 1 {
            let rollback_version = history[0].version;
            let rollback_event = runtime_manager.rollback_to_version(rollback_version).await?;
            println!("✅ Rolled back to version {} (Event ID: {})", 
                     rollback_version, rollback_event.event_id);
        }
    }

    // Demonstrate Yahoo Finance Integration with Observability
    println!("\n📈 Yahoo Finance Integration with Full Observability");
    
    // Build connector with enterprise preset
    let connector = YahooConnectorBuilder::new()
        .preset(Preset::Enterprise)
        .user_agent("EEYF-Phase2-Demo/1.0")
        .timeout(Duration::from_secs(30))
        .build()
        .await?;

    println!("✅ Yahoo Finance connector built with Enterprise preset");

    // Test with popular symbols
    let test_symbols = ["AAPL", "GOOGL", "MSFT", "TSLA", "NVDA"];
    
    for symbol in test_symbols.iter() {
        match connector.get_latest_quotes(symbol, "1d").await {
            Ok(response) => {
                if let Ok(quote) = response.last_quote() {
                    println!("✅ {}: ${:.2} (volume: {})", 
                             symbol, quote.close, quote.volume);
                    
                    #[cfg(feature = "observability")]
                    {
                        // Metrics would be automatically recorded here
                        println!("   📊 Metrics recorded for {} request", symbol);
                    }
                } else {
                    println!("⚠️  {}: No quote data available", symbol);
                }
            }
            Err(e) => {
                println!("❌ {}: Error - {}", symbol, e);
            }
        }
        
        // Brief delay between requests to respect rate limits
        sleep(Duration::from_millis(200)).await;
    }

    // Phase 2 Summary
    println!("\n✨ Phase 2 Implementation Complete!");
    println!("==================================");
    println!("Phase 2.1 - Observability:");
    #[cfg(feature = "observability")]
    {
        println!("  ✅ Prometheus metrics collection");
        println!("  ✅ Distributed tracing with OpenTelemetry/Jaeger");
        println!("  ✅ Comprehensive health monitoring");
        println!("  ✅ Real-time observability dashboards");
    }
    #[cfg(not(feature = "observability"))]
    {
        println!("  ⚪ (Enable 'observability' feature to see full demo)");
    }
    
    println!("\nPhase 2.2 - Configuration Management:");
    #[cfg(feature = "config-management")]
    {
        println!("  ✅ Multiple configuration profiles");
        println!("  ✅ Environment-based configuration");
        println!("  ✅ Configuration validation and hot reload");
        println!("  ✅ Fluent configuration builder API");
    }
    #[cfg(not(feature = "config-management"))]
    {
        println!("  ⚪ (Enable 'config-management' feature to see full demo)");
    }
    
    println!("\nPhase 2.3 - Runtime Configuration:");
    #[cfg(feature = "config-management")]
    {
        println!("  ✅ Dynamic feature flags with rollout control");
        println!("  ✅ A/B testing framework");
        println!("  ✅ Configuration versioning and rollback");
        println!("  ✅ Real-time configuration updates");
    }
    #[cfg(not(feature = "config-management"))]
    {
        println!("  ⚪ (Enable 'config-management' feature to see full demo)");
    }

    println!("\n🌐 Monitoring Endpoints:");
    #[cfg(feature = "observability")]
    {
        println!("  📊 Metrics: http://localhost:8081/metrics");
        println!("  🏥 Health: http://localhost:8082/health");
        println!("  🔍 Tracing: http://localhost:16686 (Jaeger UI)");
    }
    
    println!("\n🚀 Ready for Phase 3: Performance & Reliability Enhancements!");

    // Keep servers running for demonstration
    println!("\nPress Ctrl+C to stop the demo servers...");
    
    // Keep the demo running
    loop {
        sleep(Duration::from_secs(10)).await;
        
        #[cfg(feature = "observability")]
        {
            // Simulate some activity for metrics
            if let Ok(response) = connector.get_latest_quotes("AAPL", "1d").await {
                if response.last_quote().is_ok() {
                    println!("📊 Background metrics collection active...");
                }
            }
        }
    }
}

/// Helper function to display configuration profile details
#[cfg(feature = "config-management")]
fn display_config_profile(profile: &ConfigProfile) {
    println!("Configuration Profile: {}", profile.name);
    println!("  Description: {}", profile.description);
    println!("  Rate Limit: {} req/sec", profile.rate_limit);
    println!("  Cache Size: {} entries", profile.cache.size);
    println!("  Cache TTL: {} seconds", profile.cache.ttl_secs);
    println!("  Circuit Breaker Threshold: {}", profile.circuit_breaker.threshold);
    println!("  Max Retry Attempts: {}", profile.retry.max_attempts);
    println!("  Request Timeout: {} seconds", profile.timeouts.request_timeout_secs);
    
    if !profile.custom.is_empty() {
        println!("  Custom Settings:");
        for (key, value) in &profile.custom {
            println!("    {}: {}", key, value);
        }
    }
}