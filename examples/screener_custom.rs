//! Custom Screener Query Example
//!
//! This example demonstrates building custom screener queries using the DSL
//! to find stocks matching specific criteria.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example screener_custom
//! ```

use eeyf::screener::query::{Field, Query, QueryValue};
use eeyf::screener::Screener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Custom Screener Query Builder Demo\n");
    println!("This example demonstrates building custom queries with the DSL.\n");

    let screener = Screener::new();

    // Example 1: Find large tech stocks with strong gains
    println!("═══════════════════════════════════════════════════════");
    println!("Example 1: Large Tech Stocks with Strong Gains");
    println!("Criteria: US, Technology, Market Cap > $10B, Change > 2%");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::eq(Field::Sector, "Technology"),
        Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
        Query::gt(Field::PercentChange, 2.0),
    ]);

    let request = screener.query(query).limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            print_results_table(&results.quotes);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Example 2: Value stocks with low P/E and decent dividend
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 2: Value Stocks with Dividends");
    println!("Criteria: P/E < 15, Dividend Yield > 2%, Price > $10");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::lte(Field::PERatioTTM, 15.0),
        Query::gte(Field::DividendYield, 0.02),
        Query::gte(Field::IntradayPrice, 10.0),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
    ]);

    let request = screener.query(query).limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            for quote in &results.quotes {
                println!(
                    "{:<8} {:<25} ${:>9.2} P/E:{:>6.2} Yield:{:>5.2}%",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A").get(..25).unwrap_or(quote.short_name.as_deref().unwrap_or("N/A")),
                    quote.regular_market_price.unwrap_or(0.0),
                    quote.trailing_pe.unwrap_or(0.0),
                    quote.dividend_yield.unwrap_or(0.0) * 100.0
                );
            }
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Example 3: Mid-cap growth stocks in specific sectors
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 3: Mid-Cap Growth in Healthcare or Technology");
    println!("Criteria: $2B-$10B cap, Healthcare OR Technology, EPS growth > 20%");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::between(Field::IntradayMarketCap, 2_000_000_000.0, 10_000_000_000.0),
        Query::or(vec![
            Query::eq(Field::Sector, "Healthcare"),
            Query::eq(Field::Sector, "Technology"),
        ]),
        Query::gt(Field::EPSGrowthTTM, 20.0),
    ]);

    let request = screener.query(query).limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            print_results_table(&results.quotes);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Example 4: Momentum stocks with volume spike
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 4: Momentum Stocks with Volume Spike");
    println!("Criteria: Change > 5%, Price $20-$100, Volume > 1M");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gt(Field::PercentChange, 5.0),
        Query::between(Field::IntradayPrice, 20.0, 100.0),
        Query::gt(Field::DayVolume, 1_000_000.0),
    ]);

    let request = screener
        .query(query)
        .limit(10)
        .sort_by(Field::PercentChange, false); // Sort by change descending

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            print_results_table(&results.quotes);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Example 5: Stocks in multiple specific sectors
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 5: Using IN operator for multiple sectors");
    println!("Criteria: Financial Services, Energy, or Basic Materials");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::in_list(
            Field::Sector,
            vec![
                QueryValue::from("Financial Services"),
                QueryValue::from("Energy"),
                QueryValue::from("Basic Materials"),
            ],
        ),
        Query::gte(Field::IntradayMarketCap, 5_000_000_000.0),
        Query::gte(Field::IntradayPrice, 10.0),
    ]);

    let request = screener.query(query).limit(15);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            
            // Group by sector
            println!("{:<8} {:<25} {:>10} {:<20}", "Symbol", "Company", "Price", "Sector");
            println!("{}", "-".repeat(75));
            
            for quote in &results.quotes {
                println!(
                    "{:<8} {:<25} ${:>9.2} {:<20}",
                    quote.symbol,
                    quote.short_name.as_deref().unwrap_or("N/A").get(..25).unwrap_or(quote.short_name.as_deref().unwrap_or("N/A")),
                    quote.regular_market_price.unwrap_or(0.0),
                    "N/A" // Sector not always returned in screener results
                );
            }
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    // Example 6: Complex query with profitability metrics
    println!("\n═══════════════════════════════════════════════════════");
    println!("Example 6: Profitable Growth Stocks");
    println!("Criteria: ROE > 15%, Profit margin > 10%, Revenue growth > 15%");
    println!("═══════════════════════════════════════════════════════\n");

    let query = Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::ReturnOnEquity, 0.15),
        Query::gte(Field::ProfitMargin, 0.10),
        Query::gt(Field::RevenueGrowthTTM, 0.15),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
    ]);

    let request = screener.query(query).limit(10);

    match screener.execute(request).await {
        Ok(results) => {
            println!("Found {} matching stocks, showing top {}:\n", results.total, results.count);
            print_results_table(&results.quotes);
        }
        Err(e) => eprintln!("❌ Error: {}", e),
    }

    println!("\n✅ Custom screener query demo complete!");
    println!("\nTip: Combine operators like AND, OR, GT, LT, BETWEEN, IN");
    println!("     to create powerful custom stock screens!");

    Ok(())
}

/// Print a standard results table
fn print_results_table(quotes: &[eeyf::screener::ScreenerQuote]) {
    println!("{:<8} {:<25} {:>10} {:>12} {:>15}", "Symbol", "Company", "Price", "Change %", "Market Cap");
    println!("{}", "-".repeat(75));

    for quote in quotes {
        println!(
            "{:<8} {:<25} ${:>9.2} {:>11.2}% {:>14}",
            quote.symbol,
            quote.short_name.as_deref().unwrap_or("N/A").get(..25).unwrap_or(quote.short_name.as_deref().unwrap_or("N/A")),
            quote.regular_market_price.unwrap_or(0.0),
            quote.regular_market_change_percent.unwrap_or(0.0),
            format_market_cap(quote.market_cap.unwrap_or(0))
        );
    }
}

/// Format market cap for display
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
