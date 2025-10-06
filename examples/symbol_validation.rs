//! Example demonstrating symbol validation and lookup functionality
//!
//! This example shows how to:
//! - Validate stock symbols before making requests
//! - Get suggestions for misspelled symbols
//! - Search for symbols by company name
//! - Batch validate multiple symbols
//! - Use caching for performance
//!
//! Run with:
//! ```bash
//! cargo run --example symbol_validation
//! ```

use eeyf::validation::{SymbolValidator, ValidatorConfig};
use eeyf::YahooConnector;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 EEYF Symbol Validation Demo");
    println!("{}", "=".repeat(60));
    println!();

    // Create connector
    let provider = YahooConnector::new()?;

    // Example 1: Basic symbol validation
    println!("📊 Example 1: Validating symbols");
    println!("{}", "-".repeat(60));

    let validator = SymbolValidator::new(&provider);

    let symbols_to_check = vec!["AAPL", "GOOGL", "INVALID123", "MSFT"];

    for symbol in &symbols_to_check {
        let result = validator.validate(symbol).await?;

        if result.is_valid {
            println!("✓ {:<12} Valid", result.symbol);
            println!(
                "  Exchange:   {}",
                result.exchange.as_ref().unwrap_or(&"N/A".to_string())
            );
            println!(
                "  Type:       {}",
                result.quote_type.as_ref().unwrap_or(&"N/A".to_string())
            );
            println!(
                "  Name:       {}",
                result.short_name.as_ref().unwrap_or(&"N/A".to_string())
            );
        } else {
            println!("✗ {:<12} Invalid - symbol not found", result.symbol);
        }
        println!();
    }

    println!();

    // Example 2: Handling typos with suggestions
    println!("📊 Example 2: Suggesting corrections for typos");
    println!("{}", "-".repeat(60));

    let typos = vec![
        ("APPL", "AAPL"),   // Missing A
        ("GOOGL", "GOOGL"), // Correct
        ("MSFT", "MSFT"),   // Correct
        ("AMZN", "AMZN"),   // Correct
        ("TSLA", "TSLA"),   // Correct
    ];

    for (query, _expected) in &typos {
        println!("Query: '{}'", query);

        let suggestions = validator.suggest(query).await?;

        if suggestions.is_empty() {
            println!("  No suggestions found");
        } else {
            println!("  Suggestions:");
            for (i, suggestion) in suggestions.iter().take(3).enumerate() {
                println!(
                    "    {}. {} - {} ({}, score: {:.2})",
                    i + 1,
                    suggestion.symbol,
                    suggestion.short_name,
                    suggestion.quote_type,
                    suggestion.score
                );
            }
        }
        println!();
    }

    println!();

    // Example 3: Search by company name
    println!("📊 Example 3: Search by company name");
    println!("{}", "-".repeat(60));

    let company_searches = vec!["Apple", "Microsoft", "Tesla", "Amazon"];

    for company in &company_searches {
        println!("Searching for: '{}'", company);

        let results = validator.search_by_name(company).await?;

        if results.is_empty() {
            println!("  No results found");
        } else {
            println!("  Found {} results:", results.len());
            for (i, result) in results.iter().take(3).enumerate() {
                println!(
                    "    {}. {} ({}) - {}",
                    i + 1,
                    result.symbol,
                    result.exchange,
                    result.name
                );
            }
        }
        println!();
    }

    println!();

    // Example 4: Batch validation
    println!("📊 Example 4: Batch validation");
    println!("{}", "-".repeat(60));

    let symbols = vec!["AAPL", "GOOGL", "MSFT", "AMZN", "TSLA", "INVALID", "FAKE123"];

    println!("Validating {} symbols...", symbols.len());

    let results = validator.validate_many(&symbols).await?;

    let mut valid_count = 0;
    let mut invalid_count = 0;

    for (symbol, result) in &results {
        if result.is_valid {
            valid_count += 1;
            println!("  ✓ {}", symbol);
        } else {
            invalid_count += 1;
            println!("  ✗ {} (invalid)", symbol);
        }
    }

    println!();
    println!("Summary:");
    println!("  Valid:   {} symbols", valid_count);
    println!("  Invalid: {} symbols", invalid_count);
    println!("  Total:   {} symbols", results.len());

    println!();
    println!();

    // Example 5: Quick validity check
    println!("📊 Example 5: Quick validity checks");
    println!("{}", "-".repeat(60));

    let symbols_to_verify = vec!["AAPL", "GOOGL", "NOTREAL", "MSFT"];

    for symbol in &symbols_to_verify {
        let is_valid = validator.is_valid(symbol).await?;
        let status = if is_valid { "✓ Valid" } else { "✗ Invalid" };
        println!("{:<12} {}", symbol, status);
    }

    println!();
    println!();

    // Example 6: Cache statistics
    println!("📊 Example 6: Cache performance");
    println!("{}", "-".repeat(60));

    let stats = validator.cache_stats();
    println!("Cache Statistics:");
    println!("  Total entries:   {}", stats.total_entries);
    println!("  Valid entries:   {}", stats.valid_entries);
    println!("  Invalid entries: {}", stats.invalid_entries);
    println!("  Cache usage:     {:.1}%", stats.usage_percent());
    println!("  Valid rate:      {:.1}%", stats.hit_rate());
    println!("  Max size:        {}", stats.max_size);

    println!();
    println!();

    // Example 7: Custom configuration
    println!("📊 Example 7: Custom validator configuration");
    println!("{}", "-".repeat(60));

    let custom_config = ValidatorConfig::default()
        .with_cache_ttl(Duration::from_secs(1800)) // 30 minutes
        .with_max_cache_size(5000)
        .with_max_suggestions(10)
        .with_min_score(0.3); // Higher threshold

    let custom_validator = SymbolValidator::with_config(&provider, custom_config);

    println!("Created validator with custom config:");
    println!("  Cache TTL:       30 minutes");
    println!("  Max cache size:  5000 entries");
    println!("  Max suggestions: 10");
    println!("  Min score:       0.3");

    println!();

    // Use custom validator
    let suggestions = custom_validator.suggest("Apple").await?;
    println!(
        "  Found {} high-quality suggestions for 'Apple'",
        suggestions.len()
    );

    println!();
    println!();

    // Example 8: Metadata retrieval
    println!("📊 Example 8: Get detailed metadata");
    println!("{}", "-".repeat(60));

    let symbols_for_metadata = vec!["AAPL", "GOOGL", "BTC-USD"];

    for symbol in &symbols_for_metadata {
        println!("Symbol: {}", symbol);

        match validator.get_metadata(symbol).await? {
            Some(metadata) => {
                println!("  ✓ Valid symbol");
                println!(
                    "  Full name:  {}",
                    metadata.name.as_ref().unwrap_or(&"N/A".to_string())
                );
                println!(
                    "  Short name: {}",
                    metadata.short_name.as_ref().unwrap_or(&"N/A".to_string())
                );
                println!(
                    "  Exchange:   {}",
                    metadata.exchange.as_ref().unwrap_or(&"N/A".to_string())
                );
                println!(
                    "  Type:       {}",
                    metadata.quote_type.as_ref().unwrap_or(&"N/A".to_string())
                );
            }
            None => {
                println!("  ✗ Invalid or not found");
            }
        }
        println!();
    }

    println!();

    // Example 9: Error handling patterns
    println!("📊 Example 9: Best practices for error handling");
    println!("{}", "-".repeat(60));

    let user_input = "AAPL"; // Simulating user input

    match validator.validate(user_input).await {
        Ok(result) => {
            if result.is_valid {
                println!("✓ Symbol '{}' is valid - proceeding with request", user_input);
                // Make API request here
            } else {
                println!("✗ Symbol '{}' is invalid", user_input);

                // Suggest alternatives
                match validator.suggest(user_input).await {
                    Ok(suggestions) if !suggestions.is_empty() => {
                        println!("  Did you mean:");
                        for suggestion in suggestions.iter().take(3) {
                            println!("    - {} ({})", suggestion.symbol, suggestion.short_name);
                        }
                    }
                    _ => {
                        println!("  No similar symbols found");
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Error validating symbol: {}", e);
            // Handle API errors gracefully
        }
    }

    println!();
    println!();

    // Final statistics
    println!("{}", "=".repeat(60));
    println!("✨ Symbol validation demo complete!");
    println!();

    let final_stats = validator.cache_stats();
    println!("Final cache statistics:");
    println!("  Cached symbols: {}", final_stats.total_entries);
    println!("  Cache usage: {:.1}%", final_stats.usage_percent());
    println!();

    println!("Key takeaways:");
    println!("  • Pre-validate symbols to avoid invalid requests");
    println!("  • Use suggestions to help users correct typos");
    println!("  • Search by company name for user-friendly lookups");
    println!("  • Batch validation is efficient for multiple symbols");
    println!("  • Caching improves performance for repeated lookups");
    println!("  • Custom config allows fine-tuning for your use case");

    Ok(())
}
