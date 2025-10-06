use eeyf::YahooConnector;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== EEYF with smol Runtime ===\n");

    // smol requires explicit executor
    smol::block_on(async {
        run_examples().await
    })
}

async fn run_examples() -> Result<(), Box<dyn Error>> {
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
    println!("\n2. Searching for 'Tesla'...");
    let search_results = connector.search_ticker("Tesla").await?;
    
    for (i, result) in search_results.quotes.iter().take(3).enumerate() {
        println!("   {}. {} ({})", i + 1, result.long_name, result.symbol);
    }

    // Example 3: Get multiple symbols with smol::spawn
    println!("\n3. Fetching multiple symbols with smol tasks...");
    let symbols = vec!["AAPL", "MSFT", "GOOGL"];
    
    let mut handles = vec![];
    for symbol in symbols {
        let conn = YahooConnector::new()?;
        let handle = smol::spawn(async move {
            match conn.get_latest_quotes(symbol, "1d").await {
                Ok(response) => {
                    if let Some(quote) = response.last_quote() {
                        format!("{}: ${:.2}", symbol, quote.close)
                    } else {
                        format!("{}: No data", symbol)
                    }
                }
                Err(e) => format!("{}: Error - {}", symbol, e),
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await;
        println!("   {}", result);
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

    // Example 6: Demonstrating smol's lightweight nature
    println!("\n6. Spawning many lightweight tasks...");
    let mut handles = vec![];
    
    for i in 0..10 {
        let handle = smol::spawn(async move {
            smol::Timer::after(std::time::Duration::from_millis(10 * i)).await;
            format!("Task {} completed", i)
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let result = handle.await;
        println!("   {}", result);
    }

    // Example 7: Using smol::unblock for blocking operations
    println!("\n7. Using smol::unblock for blocking operations...");
    let result = smol::unblock(|| {
        // Simulate blocking operation
        std::thread::sleep(std::time::Duration::from_millis(50));
        "Blocking operation completed!"
    }).await;
    println!("   {}", result);

    println!("\n=== All examples completed successfully! ===");
    Ok(())
}
