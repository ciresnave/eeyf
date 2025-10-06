use eeyf::{RateLimitConfig, YahooConnector};
use std::time::Duration;
use tokio_test;

fn main() {
    // Example 1: Using default rate limiting (recommended)
    let provider = YahooConnector::with_rate_limiting().unwrap();

    println!("Rate limiting enabled: {}", provider.is_rate_limited());

    if let Some(status) = provider.rate_limit_status() {
        println!(
            "Rate limit status: {}/{} requests used this hour",
            status.hourly_used, status.hourly_limit
        );
    }

    // Example 2: Custom rate limiting configuration
    let custom_config = RateLimitConfig {
        requests_per_hour: 1000,                  // More aggressive than default
        burst_limit: 5,                           // Allow 5 rapid requests
        min_interval: Duration::from_millis(200), // 200ms between requests
    };

    let _provider_custom = YahooConnector::with_custom_rate_limiting(custom_config).unwrap();

    // Example 3: Making rate-limited requests
    let response = tokio_test::block_on(async {
        // The rate limiter will automatically delay requests to stay within limits
        provider.get_latest_quotes("AAPL", "1d").await
    })
    .unwrap();

    let quote = response.last_quote().unwrap();
    println!("Apple's latest price: {}", quote.close);

    // Check rate limit status after request
    if let Some(status) = provider.rate_limit_status() {
        println!(
            "After request - Rate limit status: {}/{} requests used",
            status.hourly_used, status.hourly_limit
        );
        println!(
            "Remaining requests this hour: {}",
            status.hourly_remaining()
        );
        println!("Near limit warning: {}", status.is_near_limit());
    }

    // Example 4: Bulk requests with automatic rate limiting
    let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"];

    tokio_test::block_on(async {
        for symbol in symbols {
            match provider.get_latest_quotes(symbol, "1d").await {
                Ok(response) => {
                    if let Ok(quote) = response.last_quote() {
                        println!("{}: ${}", symbol, quote.close);
                    }
                }
                Err(e) => {
                    eprintln!("Error fetching {}: {}", symbol, e);
                }
            }

            // Status check
            if let Some(status) = provider.rate_limit_status() {
                if status.is_near_limit() {
                    println!(
                        "Warning: Approaching rate limit ({:.1}% used)",
                        status.hourly_percent_used()
                    );
                }
            }
        }
    });
}
