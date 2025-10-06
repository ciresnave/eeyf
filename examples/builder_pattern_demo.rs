//! Example: Using the new builder pattern with presets
//! 
//! This example demonstrates the three ways to create a YahooConnector:
//! 1. YahooConnector::new() - Production defaults
//! 2. YahooConnector::builder() - Development defaults with customization
//! 3. YahooConnector::from_preset() - Named presets

use eeyf::YahooConnector;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Builder Pattern & Presets Demo\n");

    // Method 1: Production defaults (safe, stable, comprehensive)
    println!("1️⃣  Creating connector with production defaults...");
    match YahooConnector::new() {
        Ok(_connector) => {
            println!("   ✅ Production connector created successfully");
            println!("   📊 Features: Conservative rate limits, strict circuit breaker, 15min cache");
            // let quote = connector.get_latest_quotes("AAPL", "1d").await?;
            // println!("   📈 AAPL quote: ${}", quote.last_quote().unwrap().close);
        }
        Err(e) => {
            println!("   ❌ Error: {} (Expected - not fully implemented yet)", e);
        }
    }
    
    println!();

    // Method 2: Builder with development defaults + customization
    println!("2️⃣  Creating connector with builder (development defaults + custom)...");
    match YahooConnector::builder()
        .timeout(Duration::from_secs(45))
        .build() 
    {
        Ok(_connector) => {
            println!("   ✅ Custom connector created successfully");
            println!("   📊 Features: 45s timeout, development defaults (verbose logs, short cache)");
            // let quote = connector.get_latest_quotes("GOOGL", "1d").await?;
            // println!("   📈 GOOGL quote: ${}", quote.last_quote().unwrap().close);
        }
        Err(e) => {
            println!("   ❌ Error: {}", e);
        }
    }
    
    println!();

    // Method 3: Named presets
    println!("3️⃣  Loading built-in presets...");
    
    let presets = ["production", "development", "enterprise", "minimal"];
    
    for preset_name in &presets {
        match YahooConnector::from_preset(preset_name) {
            Ok(_connector) => {
                println!("   ✅ '{}' preset loaded successfully", preset_name);
            }
            Err(e) => {
                println!("   ❌ '{}' preset error: {} (Expected - not fully implemented yet)", preset_name, e);
            }
        }
    }
    
    println!();

    // Method 4: Demonstrate preset management
    println!("4️⃣  Preset management...");
    
    use eeyf::PresetManager;
    let manager = PresetManager::new();
    
    println!("   📋 Available presets:");
    for preset in manager.list_presets() {
        println!("      • {}", preset);
    }
    
    // Load a preset configuration to inspect it
    match manager.load_preset("enterprise") {
        Ok(preset) => {
            println!("\n   🔧 Enterprise preset configuration:");
            println!("      • Rate limit: {} req/sec", preset.rate_limit);
            println!("      • Circuit breaker: {} failures in {}s", 
                preset.circuit_breaker_threshold, preset.circuit_breaker_window_secs);
            println!("      • Cache: {} entries, {}s TTL", 
                preset.cache_size, preset.cache_duration_secs);
            println!("      • Retries: {} attempts", preset.retry_attempts);
            println!("      • Metrics: {}, Tracing: {}", 
                preset.enable_metrics, preset.enable_tracing);
        }
        Err(e) => {
            println!("   ❌ Failed to load preset: {}", e);
        }
    }
    
    // Test from_preset functionality
    println!("\n5️⃣  Testing from_preset functionality...");
    match YahooConnector::from_preset("enterprise") {
        Ok(_connector) => {
            println!("   ✅ YahooConnector::from_preset('enterprise') created successfully");
            println!("   📊 Features: Integrated enterprise configuration via new builder");
        }
        Err(e) => {
            println!("   ❌ Failed to create connector from preset: {}", e);
        }
    }

    println!("\n✨ Demo complete!");
    println!("\n📝 Next Steps:");
    println!("   1. Wire builder to EnterpriseYahooConnector (Phase 1.1 completion)");
    println!("   2. Implement WebSocket streaming (Phase 4.1)");
    println!("   3. Add screener API (Phase 4.2)");
    println!("   4. Create comprehensive examples");

    Ok(())
}