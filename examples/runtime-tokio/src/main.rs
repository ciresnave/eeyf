use std::error::Error;

use eeyf::YahooConnector;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("=== EEYF with Tokio Runtime ===\n");

    // Create connector
    let connector = YahooConnector::new()?;

    // Example 1: Get latest quotes
    println!("1. Fetching latest AAPL quote...");
    let response = connector.get_latest_quotes("AAPL", "1d").await?;

    if let Some(quote) = response.last_quote() {
        println!("   Symbol: AAPL");
        println!("   Price: ${:.2}", quote.close);
        println!("   Volume: {}", quote.volume);
    }

    // Example 2: Search for symbols
    println!("\n2. Searching for 'Apple'...");
    let search_results = connector.search_ticker("Apple").await?;

    for (i, result) in search_results.quotes.iter().take(3).enumerate() {
        println!("   {}. {} ({})", i + 1, result.long_name, result.symbol);
    }

    // Example 3: Get multiple symbols
    println!("\n3. Fetching multiple symbols...");
    let symbols = vec!["AAPL", "MSFT", "GOOGL"];

    for symbol in symbols {
        match connector.get_latest_quotes(symbol, "1d").await {
            Ok(response) => {
                if let Some(quote) = response.last_quote() {
                    println!("   {}: ${:.2}", symbol, quote.close);
                }
            },
            Err(e) => println!("   {}: Error - {}", symbol, e),
        }
    }

    // Example 4: Using runtime abstraction
    println!("\n4. Using runtime abstraction...");
    let runtime_name = eeyf::runtime::runtime_name();
    println!("   Current runtime: {}", runtime_name);

    // Spawn a task using runtime abstraction
    let handle = eeyf::runtime::spawn(async {
        println!("   Task running on {} runtime", runtime_name);
        42
    });

    let result = handle.await?;
    println!("   Task result: {}", result);

    // Example 5: Sleep using runtime abstraction
    println!("\n5. Sleeping for 100ms using runtime abstraction...");
    eeyf::runtime::sleep(std::time::Duration::from_millis(100)).await;
    println!("   Done!");

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}
