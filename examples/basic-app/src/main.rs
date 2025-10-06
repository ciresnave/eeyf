//! Basic EEYF Application Example
//!
//! This example demonstrates how to use the EEYF library to fetch and display
//! Yahoo Finance data.

use eeyf::{YahooConnector, YahooError};
use time::OffsetDateTime;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("EEYF Basic Application Example");
    println!("==============================\n");
    
    // Create a Yahoo Finance connector
    let provider = YahooConnector::new()?;
    
    // Example 1: Get latest quotes
    println!("1. Fetching latest quotes for AAPL...");
    match fetch_latest_quotes(&provider, "AAPL").await {
        Ok(()) => println!("✓ Success\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }
    
    // Example 2: Get historical data
    println!("2. Fetching historical data for MSFT...");
    match fetch_historical_data(&provider, "MSFT").await {
        Ok(()) => println!("✓ Success\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }
    
    // Example 3: Search for symbols
    println!("3. Searching for 'Apple'...");
    match search_symbols(&provider, "Apple").await {
        Ok(()) => println!("✓ Success\n"),
        Err(e) => println!("✗ Error: {}\n", e),
    }
    
    println!("Done!");
    Ok(())
}

/// Fetch and display latest quotes
async fn fetch_latest_quotes(
    provider: &YahooConnector,
    symbol: &str,
) -> Result<(), YahooError> {
    let response = provider.get_latest_quotes(symbol, "1d").await?;
    let quote = response.last_quote()?;
    
    let time = OffsetDateTime::from_unix_timestamp(quote.timestamp)
        .unwrap();
    
    println!("  Symbol: {}", symbol);
    println!("  Date: {}", time.date());
    println!("  Open: {:.2}", quote.open);
    println!("  High: {:.2}", quote.high);
    println!("  Low: {:.2}", quote.low);
    println!("  Close: {:.2}", quote.close);
    println!("  Volume: {}", quote.volume);
    
    Ok(())
}

/// Fetch and display historical data
async fn fetch_historical_data(
    provider: &YahooConnector,
    symbol: &str,
) -> Result<(), YahooError> {
    // Get 5 days of data
    let response = provider.get_quote_range(symbol, "1d", "5d").await?;
    let quotes = response.quotes()?;
    
    println!("  Found {} quotes:", quotes.len());
    for quote in quotes.iter().take(3) {
        let time = OffsetDateTime::from_unix_timestamp(quote.timestamp)
            .unwrap();
        println!("    {} - Close: {:.2}", time.date(), quote.close);
    }
    if quotes.len() > 3 {
        println!("    ... and {} more", quotes.len() - 3);
    }
    
    Ok(())
}

/// Search for symbols
async fn search_symbols(
    provider: &YahooConnector,
    query: &str,
) -> Result<(), YahooError> {
    let response = provider.search_ticker(query).await?;
    
    println!("  Found {} results:", response.quotes.len());
    for quote in response.quotes.iter().take(5) {
        println!("    {} - {}", quote.symbol, quote.long_name);
    }
    
    Ok(())
}
