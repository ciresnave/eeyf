//! WebSocket Streaming Example
//!
//! This example demonstrates real-time streaming of market data from Yahoo Finance
//! using WebSocket connections.
//!
//! # Requirements
//!
//! - Build with: `cargo run --example websocket_streaming --features websocket-streaming`
//! - Requires protoc installed (see README.md)
//!
//! # Usage
//!
//! ```bash
//! cargo run --example websocket_streaming --features websocket-streaming
//! ```
//!
//! The example will connect to Yahoo Finance WebSocket and stream real-time updates
//! for AAPL, GOOGL, and MSFT.

use eeyf::websocket::WebSocketStream;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    println!("🚀 Connecting to Yahoo Finance WebSocket...\n");

    // Connect to Yahoo Finance WebSocket
    let mut stream = WebSocketStream::connect().await?;
    println!("✅ Connected!\n");

    // Subscribe to multiple symbols
    let symbols = ["AAPL", "GOOGL", "MSFT"];
    println!("📊 Subscribing to: {}\n", symbols.join(", "));
    stream.subscribe(&symbols).await?;

    println!("📈 Streaming real-time quotes (press Ctrl+C to stop)...\n");
    println!("{:<10} {:<10} {:<12} {:<12} {:<15}", "Symbol", "Price", "Change", "Change %", "Volume");
    println!("{}", "-".repeat(70));

    // Counter for limiting output in demo
    let mut count = 0;
    let max_updates = 50; // Show 50 updates then exit

    // Receive and display updates
    while let Some(ticker) = stream.next().await {
        match ticker {
            Ok(data) => {
                // Display ticker information
                println!(
                    "{:<10} ${:<9.2} ${:<11.2} {:<11.2}% {:>14}",
                    data.id,
                    data.price,
                    data.change,
                    data.change_percent,
                    format_volume(data.day_volume)
                );

                count += 1;
                if count >= max_updates {
                    println!("\n✅ Received {} updates. Closing connection...", max_updates);
                    break;
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);
                // Continue on error to show resilience
            }
        }
    }

    // Close the connection gracefully
    stream.close().await?;
    println!("\n👋 Connection closed.");

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
