//! EEYF CLI Tool
//!
//! Command-line interface for interacting with the EEYF Yahoo Finance library.
//! Provides utilities for fetching quotes, testing rate limiting, inspecting cache,
//! and debugging various components.

use clap::{Parser, Subcommand};
use eeyf::{YahooConnector, YahooError};
use std::path::PathBuf;
use time::OffsetDateTime;

#[derive(Parser)]
#[command(name = "eeyf")]
#[command(author = "Eric Evans <ciresnave@gmail.com>")]
#[command(version = "0.1.0")]
#[command(about = "EEYF Yahoo Finance CLI Tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Fetch quote data for a symbol
    Quote {
        /// Stock symbol (e.g., AAPL, MSFT)
        symbol: String,
        
        /// Interval (e.g., 1d, 1h, 5m)
        #[arg(short, long, default_value = "1d")]
        interval: String,
        
        /// Range (e.g., 1d, 5d, 1mo, 1y)
        #[arg(short, long, default_value = "5d")]
        range: String,
        
        /// Output format (json, csv, table)
        #[arg(short, long, default_value = "table")]
        format: String,
        
        /// Output file (optional)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    
    /// Search for ticker symbols
    Search {
        /// Search query (e.g., "Apple", "Microsoft")
        query: String,
        
        /// Maximum results
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
    
    /// Test rate limiting
    RateLimit {
        /// Number of requests to make
        #[arg(short, long, default_value = "10")]
        count: usize,
        
        /// Symbol to fetch
        #[arg(short, long, default_value = "AAPL")]
        symbol: String,
    },
    
    /// Get current cache statistics
    CacheStats,
    
    /// Clear cache
    CacheClear,
    
    /// Get circuit breaker status
    CircuitStatus {
        /// Service name (optional)
        service: Option<String>,
    },
    
    /// Export data to file
    Export {
        /// Stock symbol
        symbol: String,
        
        /// Start date (YYYY-MM-DD)
        #[arg(short, long)]
        start: String,
        
        /// End date (YYYY-MM-DD)
        #[arg(short, long)]
        end: String,
        
        /// Output format (csv, json)
        #[arg(short, long, default_value = "csv")]
        format: String,
        
        /// Output file
        #[arg(short, long)]
        output: PathBuf,
    },
    
    /// Get ticker information
    Info {
        /// Stock symbol
        symbol: String,
    },
    
    /// Interactive mode
    Interactive,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Quote { symbol, interval, range, format, output } => {
            handle_quote(&symbol, &interval, &range, &format, output.as_deref()).await?;
        }
        Commands::Search { query, limit } => {
            handle_search(&query, limit).await?;
        }
        Commands::RateLimit { count, symbol } => {
            handle_rate_limit(count, &symbol).await?;
        }
        Commands::CacheStats => {
            handle_cache_stats().await?;
        }
        Commands::CacheClear => {
            handle_cache_clear().await?;
        }
        Commands::CircuitStatus { service } => {
            handle_circuit_status(service.as_deref()).await?;
        }
        Commands::Export { symbol, start, end, format, output } => {
            handle_export(&symbol, &start, &end, &format, &output).await?;
        }
        Commands::Info { symbol } => {
            handle_info(&symbol).await?;
        }
        Commands::Interactive => {
            handle_interactive().await?;
        }
    }
    
    Ok(())
}

async fn handle_quote(
    symbol: &str,
    interval: &str,
    range: &str,
    format: &str,
    output: Option<&std::path::Path>,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching quotes for {} (interval: {}, range: {})...", symbol, interval, range);
    
    let provider = YahooConnector::new()?;
    let response = provider.get_quote_range(symbol, interval, range).await?;
    let quotes = response.quotes()?;
    
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&quotes)?;
            if let Some(path) = output {
                std::fs::write(path, json)?;
                println!("Exported to {}", path.display());
            } else {
                println!("{}", json);
            }
        }
        "csv" => {
            let csv = format_quotes_csv(&quotes);
            if let Some(path) = output {
                std::fs::write(path, csv)?;
                println!("Exported to {}", path.display());
            } else {
                println!("{}", csv);
            }
        }
        "table" => {
            print_quotes_table(&quotes);
        }
        _ => {
            eprintln!("Unknown format: {}", format);
        }
    }
    
    Ok(())
}

async fn handle_search(query: &str, limit: usize) -> Result<(), Box<dyn std::error::Error>> {
    println!("Searching for: {}", query);
    
    let provider = YahooConnector::new()?;
    let response = provider.search_ticker(query).await?;
    
    println!("\nResults:");
    println!("{:<10} {:<40} {:<15} {:<10}", "Symbol", "Name", "Exchange", "Type");
    println!("{}", "-".repeat(80));
    
    for (i, quote) in response.quotes.iter().take(limit).enumerate() {
        println!("{:<10} {:<40} {:<15} {:<10}", 
            quote.symbol,
            quote.long_name.chars().take(38).collect::<String>(),
            quote.exchange.chars().take(13).collect::<String>(),
            quote.quote_type.chars().take(8).collect::<String>()
        );
        if i >= limit - 1 {
            break;
        }
    }
    
    Ok(())
}

async fn handle_rate_limit(count: usize, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing rate limiting with {} requests for {}...", count, symbol);
    
    let provider = YahooConnector::new()?;
    let mut successful = 0;
    let mut rate_limited = 0;
    let mut errors = 0;
    
    let start = std::time::Instant::now();
    
    for i in 0..count {
        match provider.get_latest_quotes(symbol, "1d").await {
            Ok(_) => {
                successful += 1;
                print!(".");
            }
            Err(YahooError::TooManyRequests(_)) => {
                rate_limited += 1;
                print!("R");
            }
            Err(_) => {
                errors += 1;
                print!("E");
            }
        }
        
        if (i + 1) % 50 == 0 {
            println!();
        }
    }
    
    let elapsed = start.elapsed();
    
    println!("\n\nResults:");
    println!("  Total requests: {}", count);
    println!("  Successful: {} ({:.1}%)", successful, (successful as f64 / count as f64) * 100.0);
    println!("  Rate limited: {} ({:.1}%)", rate_limited, (rate_limited as f64 / count as f64) * 100.0);
    println!("  Errors: {} ({:.1}%)", errors, (errors as f64 / count as f64) * 100.0);
    println!("  Time elapsed: {:.2}s", elapsed.as_secs_f64());
    println!("  Requests/sec: {:.2}", count as f64 / elapsed.as_secs_f64());
    
    Ok(())
}

async fn handle_cache_stats() -> Result<(), Box<dyn std::error::Error>> {
    println!("Cache statistics:");
    println!("  Note: Cache stats require cache feature to be enabled");
    println!("  Build with: cargo build --features performance-cache");
    Ok(())
}

async fn handle_cache_clear() -> Result<(), Box<dyn std::error::Error>> {
    println!("Clearing cache...");
    println!("  Note: Cache clearing requires cache feature to be enabled");
    println!("  Build with: cargo build --features performance-cache");
    Ok(())
}

async fn handle_circuit_status(service: Option<&str>) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(svc) = service {
        println!("Circuit breaker status for: {}", svc);
    } else {
        println!("Circuit breaker status (all services):");
    }
    println!("  Note: Circuit breaker stats require enterprise features");
    Ok(())
}

async fn handle_export(
    symbol: &str,
    start: &str,
    end: &str,
    format: &str,
    output: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Exporting {} data from {} to {}...", symbol, start, end);
    
    let provider = YahooConnector::new()?;
    
    // Parse dates
    let start_date = time::Date::parse(start, &time::format_description::parse("[year]-[month]-[day]")?)?;
    let end_date = time::Date::parse(end, &time::format_description::parse("[year]-[month]-[day]")?)?;
    
    let start_time = start_date.with_hms(0, 0, 0)?;
    let end_time = end_date.with_hms(23, 59, 59)?;
    
    let start_dt = start_time.assume_utc();
    let end_dt = end_time.assume_utc();
    
    let response = provider.get_quote_history(symbol, start_dt, end_dt).await?;
    let quotes = response.quotes()?;
    
    match format {
        "csv" => {
            let csv = format_quotes_csv(&quotes);
            std::fs::write(output, csv)?;
            println!("Exported {} quotes to {}", quotes.len(), output.display());
        }
        "json" => {
            let json = serde_json::to_string_pretty(&quotes)?;
            std::fs::write(output, json)?;
            println!("Exported {} quotes to {}", quotes.len(), output.display());
        }
        _ => {
            eprintln!("Unknown format: {}", format);
        }
    }
    
    Ok(())
}

async fn handle_info(symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
    println!("Fetching information for {}...", symbol);
    
    let provider = YahooConnector::new()?;
    let response = provider.search_ticker(symbol).await?;
    
    if let Some(quote) = response.quotes.first() {
        println!("\nCompany Information:");
        println!("  Symbol: {}", quote.symbol);
        println!("  Name: {}", quote.long_name);
        println!("  Exchange: {}", quote.exchange);
        println!("  Type: {}", quote.quote_type);
        println!("  Type Display: {}", quote.type_display);
        println!("  Score: {:.2}", quote.score);
    } else {
        println!("No information found for {}", symbol);
    }
    
    Ok(())
}

async fn handle_interactive() -> Result<(), Box<dyn std::error::Error>> {
    println!("EEYF Interactive Mode");
    println!("Type 'help' for available commands, 'exit' to quit\n");
    
    let provider = YahooConnector::new()?;
    
    loop {
        print!("> ");
        use std::io::Write;
        std::io::stdout().flush()?;
        
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            continue;
        }
        
        let parts: Vec<&str> = input.split_whitespace().collect();
        
        match parts[0] {
            "help" => {
                println!("Available commands:");
                println!("  quote <SYMBOL> [interval] [range] - Fetch quotes");
                println!("  search <QUERY> - Search for symbols");
                println!("  info <SYMBOL> - Get symbol information");
                println!("  help - Show this help");
                println!("  exit - Exit interactive mode");
            }
            "quote" => {
                if parts.len() < 2 {
                    println!("Usage: quote <SYMBOL> [interval] [range]");
                    continue;
                }
                let symbol = parts[1];
                let interval = parts.get(2).unwrap_or(&"1d");
                let range = parts.get(3).unwrap_or(&"5d");
                
                match provider.get_quote_range(symbol, interval, range).await {
                    Ok(response) => {
                        match response.quotes() {
                            Ok(quotes) => print_quotes_table(&quotes),
                            Err(e) => println!("Error parsing quotes: {}", e),
                        }
                    }
                    Err(e) => println!("Error fetching quotes: {}", e),
                }
            }
            "search" => {
                if parts.len() < 2 {
                    println!("Usage: search <QUERY>");
                    continue;
                }
                let query = parts[1..].join(" ");
                
                match provider.search_ticker(&query).await {
                    Ok(response) => {
                        for quote in response.quotes.iter().take(5) {
                            println!("{:<10} - {}", quote.symbol, quote.long_name);
                        }
                    }
                    Err(e) => println!("Error searching: {}", e),
                }
            }
            "info" => {
                if parts.len() < 2 {
                    println!("Usage: info <SYMBOL>");
                    continue;
                }
                let symbol = parts[1];
                
                match provider.search_ticker(symbol).await {
                    Ok(response) => {
                        if let Some(quote) = response.quotes.first() {
                            println!("Symbol: {}", quote.symbol);
                            println!("Name: {}", quote.long_name);
                            println!("Exchange: {}", quote.exchange);
                            println!("Type: {}", quote.quote_type);
                        } else {
                            println!("No information found");
                        }
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            "exit" | "quit" => {
                println!("Goodbye!");
                break;
            }
            _ => {
                println!("Unknown command: {}. Type 'help' for available commands.", parts[0]);
            }
        }
    }
    
    Ok(())
}

// Helper functions

fn format_quotes_csv(quotes: &[eeyf::Quote]) -> String {
    let mut csv = String::from("timestamp,date,open,high,low,close,volume,adjclose\n");
    
    for quote in quotes {
        let dt = OffsetDateTime::from_unix_timestamp(quote.timestamp).unwrap();
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            quote.timestamp,
            dt.date(),
            quote.open,
            quote.high,
            quote.low,
            quote.close,
            quote.volume,
            quote.adjclose
        ));
    }
    
    csv
}

fn print_quotes_table(quotes: &[eeyf::Quote]) {
    println!("\n{:<12} {:<10} {:<10} {:<10} {:<10} {:<12}", 
        "Date", "Open", "High", "Low", "Close", "Volume");
    println!("{}", "-".repeat(70));
    
    for quote in quotes {
        let dt = OffsetDateTime::from_unix_timestamp(quote.timestamp).unwrap();
        println!("{:<12} {:<10.2} {:<10.2} {:<10.2} {:<10.2} {:<12}",
            dt.date(),
            quote.open,
            quote.high,
            quote.low,
            quote.close,
            quote.volume
        );
    }
    
    println!("\nTotal: {} quotes\n", quotes.len());
}
