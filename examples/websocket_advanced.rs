//! Advanced WebSocket Streaming Example
//!
//! This example demonstrates the enhanced WebSocket features:
//! - Custom configuration with WebSocketConfig builder
//! - Automatic reconnection with exponential backoff
//! - Message handler callbacks for event-driven processing
//! - Backpressure handling for high-frequency updates
//! - Statistics monitoring
//!
//! # Requirements
//!
//! - Build with: `cargo run --example websocket_advanced --features websocket-streaming`
//! - Requires protoc installed (see README.md)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example websocket_advanced --features websocket-streaming
//! ```

use eeyf::websocket::{WebSocketConfig, WebSocketStream};
use std::error::Error;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::{interval, sleep};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging to see reconnection attempts
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🚀 Advanced WebSocket Features Demo\n");
    println!("This example demonstrates:");
    println!("  • Custom configuration");
    println!("  • Automatic reconnection");
    println!("  • Message handler callbacks");
    println!("  • Backpressure handling");
    println!("  • Statistics monitoring\n");

    // Example 1: Custom Configuration
    println!("═══════════════════════════════════════════════════════");
    println!("Example 1: Custom WebSocket Configuration");
    println!("═══════════════════════════════════════════════════════\n");
    
    let config = WebSocketConfig::default()
        .with_initial_reconnect_delay(Duration::from_secs(2))
        .with_max_reconnect_attempts(5)
        .with_heartbeat_interval(Duration::from_secs(10))
        .with_backpressure_buffer_size(2000)
        .with_auto_reconnect(true);
    
    println!("✅ Created config with:");
    println!("   - Initial reconnect delay: 2s");
    println!("   - Max reconnect attempts: 5");
    println!("   - Heartbeat interval: 10s");
    println!("   - Backpressure buffer: 2000 messages");
    println!("   - Auto-reconnect: enabled\n");

    // Example 2: Message Handler Callbacks
    println!("═══════════════════════════════════════════════════════");
    println!("Example 2: Message Handler Callbacks");
    println!("═══════════════════════════════════════════════════════\n");
    
    let mut stream = WebSocketStream::connect_with_config(config.clone()).await?;
    println!("✅ Connected to Yahoo Finance WebSocket\n");

    // Add a handler that tracks price changes
    let price_change_counter = Arc::new(AtomicU64::new(0));
    let counter_clone = price_change_counter.clone();
    
    stream.add_handler(move |ticker| {
        if ticker.change.abs() > 0.5 {
            counter_clone.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    });
    
    // Add a handler that tracks high-volume tickers
    let high_volume_counter = Arc::new(AtomicU64::new(0));
    let volume_clone = high_volume_counter.clone();
    
    stream.add_handler(move |ticker| {
        if ticker.day_volume > 10_000_000 {
            volume_clone.fetch_add(1, Ordering::Relaxed);
        }
        Ok(())
    });
    
    println!("✅ Registered 2 message handlers:");
    println!("   - Handler 1: Tracks large price changes (> $0.50)");
    println!("   - Handler 2: Tracks high-volume tickers (> 10M)\n");

    // Subscribe to multiple symbols
    let symbols = ["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"];
    println!("📊 Subscribing to: {}\n", symbols.join(", "));
    stream.subscribe(&symbols).await?;

    // Example 3: Statistics Monitoring
    println!("═══════════════════════════════════════════════════════");
    println!("Example 3: Statistics Monitoring");
    println!("═══════════════════════════════════════════════════════\n");
    
    println!("📈 Receiving updates for 30 seconds...\n");
    
    // Display table header
    println!("{:<10} {:<10} {:<12} {:<12} {:<15}", "Symbol", "Price", "Change", "Change %", "Volume");
    println!("{}", "-".repeat(70));

    // Note: We'll check statistics periodically in the main loop rather than spawning
    // a separate task, to avoid borrowing issues

    // Receive updates for 30 seconds
    let start = std::time::Instant::now();
    let mut count = 0;
    let mut last_stats_time = std::time::Instant::now();
    
    while start.elapsed() < Duration::from_secs(30) {
        // Show statistics every 10 seconds
        if last_stats_time.elapsed() >= Duration::from_secs(10) {
            let stats = stream.stats().await;
            println!("\n📊 Statistics:");
            println!("   Messages received: {}", stats.messages_received);
            println!("   Messages dropped: {}", stats.messages_dropped);
            println!("   Reconnect attempts: {}", stats.reconnect_attempts);
            println!("   Successful reconnects: {}", stats.successful_reconnects);
            println!("   Heartbeats sent: {}", stats.heartbeats_sent);
            println!();
            last_stats_time = std::time::Instant::now();
        }
        
        if let Some(result) = stream.next().await {
            match result {
                Ok(ticker) => {
                    println!(
                        "{:<10} ${:<9.2} ${:<11.2} {:<11.2}% {:>14}",
                        ticker.id,
                        ticker.price,
                        ticker.change,
                        ticker.change_percent,
                        format_volume(ticker.day_volume)
                    );
                    count += 1;
                }
                Err(e) => {
                    eprintln!("❌ Error: {}", e);
                }
            }
        }
    }

    println!("\n✅ Received {} updates in 30 seconds", count);
    println!("   Large price changes: {}", price_change_counter.load(Ordering::Relaxed));
    println!("   High-volume tickers: {}", high_volume_counter.load(Ordering::Relaxed));

    // Example 4: Backpressure Handling
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 4: Backpressure Handling");
    println!("═══════════════════════════════════════════════════════\n");
    
    stream.enable_backpressure();
    println!("✅ Enabled backpressure with 2000-message buffer");
    println!("📈 Messages are now buffered, processing at controlled rate...\n");

    // Simulate slow processing
    for i in 0..20 {
        if let Some(ticker) = stream.next_buffered().await {
            println!(
                "[Buffered {}] {:<10} ${:<9.2}",
                i + 1,
                ticker.id,
                ticker.price
            );
            
            // Simulate slow processing (100ms per message)
            sleep(Duration::from_millis(100)).await;
        }
    }

    // Final statistics
    println!("\n═══════════════════════════════════════════════════════");
    println!("Final Statistics");
    println!("═══════════════════════════════════════════════════════\n");
    
    let stats = stream.stats().await;
    
    println!("Connection State: {:?}", stream.state());
    println!("Messages Received: {}", stats.messages_received);
    println!("Messages Dropped: {}", stats.messages_dropped);
    println!("Reconnect Attempts: {}", stats.reconnect_attempts);
    println!("Successful Reconnects: {}", stats.successful_reconnects);
    println!("Heartbeats Sent: {}", stats.heartbeats_sent);
    
    let drop_rate = if stats.messages_received > 0 {
        (stats.messages_dropped as f64 / stats.messages_received as f64) * 100.0
    } else {
        0.0
    };
    println!("Drop Rate: {:.2}%", drop_rate);

    // Close connection
    stream.close().await?;
    println!("\n👋 Connection closed gracefully");

    Ok(())
}

/// Format volume for display (e.g., 1000000 -> "1.00M")
fn format_volume(volume: i64) -> String {
    if volume >= 1_000_000_000 {
        format!("{:.2}B", volume as f64 / 1_000_000_000.0)
    } else if volume >= 1_000_000 {
        format!("{:.2}M", volume as f64 / 1_000_000.0)
    } else if volume >= 1_000 {
        format!("{:.2}K", volume as f64 / 1_000.0)
    } else {
        volume.to_string()
    }
}
