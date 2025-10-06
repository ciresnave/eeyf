//! Predefined Screener Example
//!
//! This example demonstrates using Yahoo Finance's built-in screeners to find
//! stocks matching predefined criteria like "day gainers", "most actives", etc.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example screener_presets
//! ```

use eeyf::screener::{PredefinedScreener, Screener};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Yahoo Finance Predefined Screeners Demo\n");
    println!("This example demonstrates using Yahoo's built-in stock screeners.\n");

    let screener = Screener::new();

    // Example 1: Day Gainers
    println!("═══════════════════════════════════════════════════════");
    println!("Example 1: Day Gainers (stocks up >3%)");
    println!("═══════════════════════════════════════════════════════\n");

    let request = screener
        .predefined(PredefinedScreener::DayGainers)
        .limit(10); // Get top 10

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} total gainers, showing top {}:\n", results.total, results.count);
            println!("{:<8} {:<30} {:>10} {:>12} {:>15}", "Symbol", "Company", "Price", "Change %", "Volume");
            println!("{}", "-".repeat(80));

            for quote in results.quotes {
                println!(
                    "{:<8} {:<30} ${:>9.2} {:>11.2}% {:>14}",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A"),
                    quote.regular_market_price.unwrap_or(0.0),
                    quote.regular_market_change_percent.unwrap_or(0.0),
                    format_volume(quote.regular_market_volume.unwrap_or(0))
                );
            }
        }
        Err(e) => eprintln!("❌ Error fetching day gainers: {}", e),
    }

    // Example 2: Most Active Stocks
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 2: Most Active Stocks (highest volume)");
    println!("═══════════════════════════════════════════════════════\n");

    let request = screener
        .predefined(PredefinedScreener::MostActives)
        .limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} total active stocks, showing top {}:\n", results.total, results.count);
            println!("{:<8} {:<30} {:>10} {:>15}", "Symbol", "Company", "Price", "Volume");
            println!("{}", "-".repeat(70));

            for quote in results.quotes {
                println!(
                    "{:<8} {:<30} ${:>9.2} {:>14}",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A"),
                    quote.regular_market_price.unwrap_or(0.0),
                    format_volume(quote.regular_market_volume.unwrap_or(0))
                );
            }
        }
        Err(e) => eprintln!("❌ Error fetching most actives: {}", e),
    }

    // Example 3: Growth Technology Stocks
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 3: Growth Technology Stocks");
    println!("═══════════════════════════════════════════════════════\n");

    let request = screener
        .predefined(PredefinedScreener::GrowthTechnologyStocks)
        .limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} technology growth stocks, showing top {}:\n", results.total, results.count);
            println!("{:<8} {:<25} {:>10} {:>10} {:>12}", "Symbol", "Company", "Price", "P/E", "Market Cap");
            println!("{}", "-".repeat(75));

            for quote in results.quotes {
                println!(
                    "{:<8} {:<25} ${:>9.2} {:>10.2} {:>11}",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A").get(..25).unwrap_or(quote.short_name.as_deref().unwrap_or("N/A")),
                    quote.regular_market_price.unwrap_or(0.0),
                    quote.trailing_pe.unwrap_or(0.0),
                    format_market_cap(quote.market_cap.unwrap_or(0))
                );
            }
        }
        Err(e) => eprintln!("❌ Error fetching growth tech stocks: {}", e),
    }

    // Example 4: High Yield Dividend Stocks
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 4: High Yield Dividend Stocks (yield >3%)");
    println!("═══════════════════════════════════════════════════════\n");

    let request = screener
        .predefined(PredefinedScreener::HighYieldStocks)
        .limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} high yield stocks, showing top {}:\n", results.total, results.count);
            println!("{:<8} {:<30} {:>10} {:>12}", "Symbol", "Company", "Price", "Yield");
            println!("{}", "-".repeat(70));

            for quote in results.quotes {
                let yield_pct = quote.dividend_yield.unwrap_or(0.0) * 100.0;
                println!(
                    "{:<8} {:<30} ${:>9.2} {:>10.2}%",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A"),
                    quote.regular_market_price.unwrap_or(0.0),
                    yield_pct
                );
            }
        }
        Err(e) => eprintln!("❌ Error fetching high yield stocks: {}", e),
    }

    // Example 5: List all available screeners
    println!("\n═══════════════════════════════════════════════════════");
    println!("All Available Predefined Screeners");
    println!("═══════════════════════════════════════════════════════\n");

    for screener_type in PredefinedScreener::all() {
        println!("• {} - {}", screener_type.id(), screener_type.description());
    }

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

/// Format market cap for display (e.g., 1000000000 -> "1.00B")
fn format_market_cap(market_cap: i64) -> String {
    if market_cap >= 1_000_000_000_000 {
        format!("{:.2}T", market_cap as f64 / 1_000_000_000_000.0)
    } else if market_cap >= 1_000_000_000 {
        format!("{:.2}B", market_cap as f64 / 1_000_000_000.0)
    } else if market_cap >= 1_000_000 {
        format!("{:.2}M", market_cap as f64 / 1_000_000.0)
    } else {
        market_cap.to_string()
    }
}
