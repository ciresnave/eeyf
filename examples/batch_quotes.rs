//! Example demonstrating batch operations for fetching multiple symbols efficiently
//!
//! This example shows how to use batch operations to fetch quotes for multiple
//! symbols in parallel while respecting rate limits and handling errors gracefully.
//!
//! Run with:
//! ```bash
//! cargo run --example batch_quotes
//! ```

use eeyf::batch::BatchQuoteRequest;
use eeyf::YahooConnector;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Batch Operations Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Create connector with rate limiting
    let provider = YahooConnector::new()?;

    // Example 1: Basic batch fetch with default settings
    println!("📊 Example 1: Fetching latest quotes for multiple symbols");
    println!("{}", "-".repeat(60));

    let symbols = vec![
        "AAPL", "GOOGL", "MSFT", "AMZN", "TSLA", "META", "NVDA", "AMD", "INTC", "NFLX",
    ];

    let batch = BatchQuoteRequest::new(symbols.clone());

    let start = Instant::now();
    let result = provider.batch_get_latest_quotes(&batch, "1d").await?;
    let elapsed = start.elapsed();

    println!("✅ Batch completed in {:.2}s", elapsed.as_secs_f64());
    println!("📈 Statistics:");
    println!("   Total symbols:    {}", result.total);
    println!("   Successful:       {}", result.successful);
    println!("   Failed:           {}", result.failed);
    println!("   Success rate:     {:.1}%", result.success_rate());
    println!();

    // Display results
    println!("💰 Latest Quotes:");
    println!("{:<8} {:>12} {:>12} {:>10}", "Symbol", "Price", "Change", "Change %");
    println!("{}", "-".repeat(60));

    for (symbol, response) in &result.results {
        match response.last_quote() {
            Ok(quote) => {
                let change = quote.close - quote.open;
                let change_pct = (change / quote.open) * 100.0;
                let change_sign = if change >= 0.0 { "+" } else { "" };

                println!(
                    "{:<8} ${:>11.2} {:>11}{:.2} {:>9}{:.2}%",
                    symbol, quote.close, change_sign, change, change_sign, change_pct
                );
            }
            Err(e) => {
                println!("{:<8} Error: {}", symbol, e);
            }
        }
    }

    // Display errors if any
    if !result.errors.is_empty() {
        println!();
        println!("❌ Errors:");
        for (symbol, error) in &result.errors {
            println!("   {}: {}", symbol, error);
        }
    }

    println!();
    println!();

    // Example 2: Batch with progress tracking and custom concurrency
    println!("📊 Example 2: Batch with progress tracking");
    println!("{}", "-".repeat(60));

    let large_symbols = vec![
        "AAPL", "GOOGL", "MSFT", "AMZN", "TSLA", "META", "NVDA", "AMD", "INTC", "NFLX", "CRM",
        "ADBE", "ORCL", "CSCO", "IBM", "QCOM", "TXN", "AVGO", "NOW", "INTU",
    ];

    let batch = BatchQuoteRequest::new(large_symbols)
        .with_concurrency(15) // Increase concurrency
        .with_continue_on_error(true);

    println!("Fetching {} symbols with concurrency=15...", batch.symbols.len());

    let start = Instant::now();
    let result = provider.batch_get_quote_range(&batch, "1d", "5d").await?;
    let elapsed = start.elapsed();

    println!("✅ Completed in {:.2}s", elapsed.as_secs_f64());
    println!("   Average time per symbol: {:.3}s", elapsed.as_secs_f64() / result.total as f64);
    println!("   Success rate: {:.1}%", result.success_rate());
    println!();

    // Example 3: Batch search for ticker symbols
    println!("📊 Example 3: Batch ticker search");
    println!("{}", "-".repeat(60));

    let queries = vec!["Apple", "Microsoft", "Tesla", "Amazon", "Nvidia"];
    let batch = BatchQuoteRequest::new(queries);

    let result = provider.batch_search_ticker(&batch).await?;

    println!("🔍 Search Results:");
    for (query, search_result) in result.results {
        println!("   '{}': Found {} matches", query, search_result.count);
        for quote in search_result.quotes.iter().take(3) {
            println!(
                "      • {} ({}) - {}",
                quote.symbol, quote.quote_type, quote.short_name
            );
        }
    }

    println!();
    println!();

    // Example 4: Error handling - deliberately include invalid symbols
    println!("📊 Example 4: Error handling with invalid symbols");
    println!("{}", "-".repeat(60));

    let mixed_symbols = vec!["AAPL", "INVALID_SYM", "GOOGL", "FAKE123", "MSFT"];
    let batch = BatchQuoteRequest::new(mixed_symbols)
        .with_continue_on_error(true); // Continue despite errors

    let result = provider.batch_get_latest_quotes(&batch, "1d").await?;

    println!("Results:");
    println!("   ✅ Successful: {} symbols", result.successful);
    println!("   ❌ Failed:     {} symbols", result.failed);
    println!();

    if !result.errors.is_empty() {
        println!("Error details:");
        for (symbol, error) in &result.errors {
            println!("   {} → {}", symbol, error);
        }
    }

    println!();
    println!();

    // Example 5: Historical data batch fetch
    println!("📊 Example 5: Batch historical data (YTD)");
    println!("{}", "-".repeat(60));

    let historical_symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN"];
    let batch = BatchQuoteRequest::new(historical_symbols);

    // Get year-to-date data
    let start = time::OffsetDateTime::now_utc()
        .replace_month(time::Month::January)
        .unwrap()
        .replace_day(1)
        .unwrap();
    let end = time::OffsetDateTime::now_utc();

    let result = provider.batch_get_quote_history(&batch, start, end).await?;

    println!("📈 Year-to-Date Summary:");
    for (symbol, response) in result.results {
        match response.quotes() {
            Ok(quotes) => {
                if let (Some(first), Some(last)) = (quotes.first(), quotes.last()) {
                    let ytd_return = ((last.close - first.open) / first.open) * 100.0;
                    let sign = if ytd_return >= 0.0 { "+" } else { "" };
                    println!(
                        "   {:<6} {} data points, YTD return: {}{:.2}%",
                        symbol,
                        quotes.len(),
                        sign,
                        ytd_return
                    );
                }
            }
            Err(e) => println!("   {:<6} Error: {}", symbol, e),
        }
    }

    println!();
    println!("{}", "=".repeat(60));
    println!("✨ Batch operations demo complete!");
    println!();
    println!("Key takeaways:");
    println!("  • Batch operations handle multiple symbols efficiently");
    println!("  • Automatic rate limiting prevents API abuse");
    println!("  • Configurable concurrency for optimal performance");
    println!("  • Graceful error handling per symbol");
    println!("  • Progress tracking for long-running operations");

    Ok(())
}
