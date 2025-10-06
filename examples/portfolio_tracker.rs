use chrono::{DateTime, Utc};
use eeyf::YahooConnector;
use futures::future::try_join_all;
use rust_decimal::prelude::*;
use std::collections::HashMap;
use std::sync::Arc;
use time::OffsetDateTime;
use tokio::time::{Duration, interval};

/// Portfolio Tracker - Real-world Example
///
/// This example demonstrates building a portfolio tracking application
/// using EEYF with enterprise features like rate limiting, caching,
/// and concurrent request handling.

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub shares: f64,
    pub cost_basis: f64,
}

#[derive(Debug)]
pub struct PortfolioValue {
    pub symbol: String,
    pub shares: f64,
    pub current_price: f64,
    pub market_value: f64,
    pub cost_basis: f64,
    pub total_cost: f64,
    pub gain_loss: f64,
    pub gain_loss_percent: f64,
}

pub struct PortfolioTracker {
    connector: Arc<YahooConnector>,
    positions: HashMap<String, Position>,
}

impl PortfolioTracker {
    /// Create a new portfolio tracker with enterprise-grade reliability
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Use enterprise preset for maximum reliability
        let connector = YahooConnector::from_preset("enterprise")?;

        Ok(Self {
            connector: Arc::new(connector),
            positions: HashMap::new(),
        })
    }

    /// Add a position to the portfolio
    pub fn add_position(&mut self, symbol: String, shares: f64, cost_basis: f64) {
        let position = Position {
            symbol: symbol.clone(),
            shares,
            cost_basis,
        };
        self.positions.insert(symbol, position);
    }

    /// Add a position using real historical price data
    pub async fn add_realistic_position(
        &mut self,
        symbol: &str,
        shares: f64,
        purchase_date: DateTime<Utc>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "   Fetching historical price for {} on {}...",
            symbol,
            purchase_date.format("%Y-%m-%d")
        );

        // Convert chrono DateTime to time OffsetDateTime for EEYF API
        let start_time = OffsetDateTime::from_unix_timestamp(purchase_date.timestamp())
            .map_err(|e| format!("Invalid timestamp: {}", e))?;
        let end_time = OffsetDateTime::from_unix_timestamp(purchase_date.timestamp() + 86400) // +1 day
            .map_err(|e| format!("Invalid timestamp: {}", e))?;

        // Fetch historical data for that date
        let response = self
            .connector
            .get_quote_history(symbol, start_time, end_time)
            .await?;

        let quotes = response.quotes().map_err(|e| {
            format!(
                "No historical quotes available for {} on {}: {}",
                symbol,
                purchase_date.format("%Y-%m-%d"),
                e
            )
        })?;

        if let Some(quote) = quotes.first() {
            let historical_price = quote.close;
            println!(
                "   ✅ {} was ${:.2} on {}",
                symbol,
                historical_price,
                purchase_date.format("%Y-%m-%d")
            );

            self.add_position(
                symbol.to_string(),
                shares,
                historical_price.to_f64().unwrap_or(0.0),
            );
            Ok(())
        } else {
            // Fallback: use current price with a warning
            println!(
                "   ⚠️  No historical data available for {}, using current price as fallback",
                symbol
            );
            let response = self.connector.get_latest_quotes(symbol, "1d").await?;
            let quote = response
                .last_quote()
                .map_err(|e| format!("Could not get current price for {}: {}", symbol, e))?;

            self.add_position(
                symbol.to_string(),
                shares,
                quote.close.to_f64().unwrap_or(0.0),
            );
            Ok(())
        }
    }

    /// Get current portfolio value with concurrent price fetching
    pub async fn get_portfolio_value(
        &self,
    ) -> Result<Vec<PortfolioValue>, Box<dyn std::error::Error>> {
        if self.positions.is_empty() {
            return Ok(Vec::new());
        }

        println!(
            "📊 Fetching current prices for {} positions...",
            self.positions.len()
        );

        // Create futures for all price requests
        let price_futures: Vec<_> = self
            .positions
            .values()
            .map(|position| {
                let connector = Arc::clone(&self.connector);
                let symbol = position.symbol.clone();

                async move {
                    println!("   Fetching price for {}...", symbol);
                    let response = connector.get_latest_quotes(&symbol, "1d").await?;
                    let quote =
                        response
                            .last_quote()
                            .map_err(|e| -> Box<dyn std::error::Error> {
                                format!("No quotes available for {}: {}", symbol, e).into()
                            })?;

                    Ok::<(String, f64), Box<dyn std::error::Error>>((
                        symbol,
                        quote.close.to_f64().unwrap_or(0.0),
                    ))
                }
            })
            .collect();

        // Execute all requests concurrently (rate limiter handles spacing)
        let prices = try_join_all(price_futures).await?;

        // Calculate portfolio values
        let mut portfolio_values = Vec::new();

        for (symbol, current_price) in prices {
            if let Some(position) = self.positions.get(symbol.as_str()) {
                let market_value = position.shares * current_price;
                let total_cost = position.shares * position.cost_basis;
                let gain_loss = market_value - total_cost;
                let gain_loss_percent = (gain_loss / total_cost) * 100.0;

                portfolio_values.push(PortfolioValue {
                    symbol: symbol.clone(),
                    shares: position.shares,
                    current_price,
                    market_value,
                    cost_basis: position.cost_basis,
                    total_cost,
                    gain_loss,
                    gain_loss_percent,
                });
            }
        }

        Ok(portfolio_values)
    }

    /// Display portfolio summary
    pub fn display_portfolio_summary(&self, values: &[PortfolioValue]) {
        if values.is_empty() {
            println!("📋 Portfolio is empty");
            return;
        }

        println!("\n📊 Portfolio Summary");
        println!("═══════════════════════════════════════════════════════════════");
        println!(
            "{:<8} {:>8} {:>12} {:>12} {:>12} {:>12} {:>10}",
            "Symbol", "Shares", "Price", "Market Val", "Cost Basis", "Gain/Loss", "Gain %"
        );
        println!("─────────────────────────────────────────────────────────────");

        let mut total_market_value = 0.0;
        let mut total_cost = 0.0;

        for value in values {
            println!(
                "{:<8} {:>8.2} {:>12.2} {:>12.2} {:>12.2} {:>12.2} {:>9.1}%",
                value.symbol,
                value.shares,
                value.current_price,
                value.market_value,
                value.total_cost,
                value.gain_loss,
                value.gain_loss_percent
            );

            total_market_value += value.market_value;
            total_cost += value.total_cost;
        }

        let total_gain_loss = total_market_value - total_cost;
        let total_gain_loss_percent = (total_gain_loss / total_cost) * 100.0;

        println!("─────────────────────────────────────────────────────────────");
        println!(
            "{:<8} {:>8} {:>12} {:>12.2} {:>12.2} {:>12.2} {:>9.1}%",
            "TOTAL",
            "",
            "",
            total_market_value,
            total_cost,
            total_gain_loss,
            total_gain_loss_percent
        );
        println!("═══════════════════════════════════════════════════════════════");

        // Summary statistics
        let winners = values.iter().filter(|v| v.gain_loss > 0.0).count();
        let losers = values.iter().filter(|v| v.gain_loss < 0.0).count();
        let unchanged = values.len() - winners - losers;

        println!("\n📈 Performance Summary:");
        println!("   Winners: {} positions", winners);
        println!("   Losers: {} positions", losers);
        println!("   Unchanged: {} positions", unchanged);
        println!("   Total Portfolio Value: ${:.2}", total_market_value);
        println!(
            "   Total Gain/Loss: ${:.2} ({:.1}%)",
            total_gain_loss, total_gain_loss_percent
        );
    }

    /// Start real-time monitoring (updates every 30 seconds during market hours)
    pub async fn start_monitoring(
        &self,
        update_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🔄 Starting portfolio monitoring (updates every {} seconds)",
            update_interval_secs
        );
        println!("   Press Ctrl+C to stop monitoring");

        let mut interval = interval(Duration::from_secs(update_interval_secs));

        loop {
            interval.tick().await;

            match self.get_portfolio_value().await {
                Ok(values) => {
                    // Clear screen (simple version)
                    print!("\x1B[2J\x1B[1;1H");

                    println!(
                        "🕒 Last Updated: {}",
                        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
                    );
                    self.display_portfolio_summary(&values);

                    // Show next update time
                    println!("\n⏳ Next update in {} seconds...", update_interval_secs);
                }
                Err(e) => {
                    eprintln!("❌ Error updating portfolio: {}", e);
                    println!("   Will retry in {} seconds...", update_interval_secs);
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Portfolio Tracker Demo");
    println!("═══════════════════════════════════════════════");

    // Create portfolio tracker with enterprise-grade features
    let mut tracker = PortfolioTracker::new()?;

    // Add sample portfolio positions using REAL historical data
    println!("📋 Setting up sample portfolio with real historical prices...");

    // Fetch historical prices from 6 months ago for realistic cost basis
    let six_months_ago = chrono::Utc::now() - chrono::Duration::days(180);
    let one_year_ago = chrono::Utc::now() - chrono::Duration::days(365);

    tracker
        .add_realistic_position("AAPL", 100.0, six_months_ago)
        .await?;
    tracker
        .add_realistic_position("GOOGL", 50.0, one_year_ago)
        .await?;
    tracker
        .add_realistic_position("MSFT", 75.0, six_months_ago)
        .await?;
    tracker
        .add_realistic_position("TSLA", 25.0, one_year_ago)
        .await?;
    tracker
        .add_realistic_position("NVDA", 40.0, six_months_ago)
        .await?;

    println!("✅ Portfolio configured with 5 positions");

    // Get initial portfolio values
    println!("\n📊 Fetching initial portfolio values...");
    let values = tracker.get_portfolio_value().await?;
    tracker.display_portfolio_summary(&values);

    // Demonstrate error handling and recovery
    println!("\n🔧 Testing error handling...");
    tracker.add_position("INVALID".to_string(), 10.0, 100.0);

    match tracker.get_portfolio_value().await {
        Ok(values) => {
            println!("✅ Portfolio updated successfully (invalid symbol filtered out)");
            // Filter out invalid positions for display
            let valid_values: Vec<_> = values
                .into_iter()
                .filter(|v| v.symbol != "INVALID")
                .collect();
            tracker.display_portfolio_summary(&valid_values);
        }
        Err(e) => {
            println!("⚠️  Some positions failed to update: {}", e);
        }
    }

    // Remove invalid position
    tracker.positions.remove("INVALID");

    println!("\n🔄 Starting real-time monitoring...");
    println!("   Note: This demo will update every 30 seconds");
    println!("   In production, you might want updates every 15-60 seconds during market hours");

    // Start monitoring (this will run forever until Ctrl+C)
    tracker.start_monitoring(30).await?;

    Ok(())
}

// Additional helper functions for production use

impl PortfolioTracker {
    /// Load portfolio from CSV file
    pub fn load_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            // Skip header
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if parts.len() >= 3 {
                let symbol = parts[0].trim().to_string();
                let shares: f64 = parts[1].trim().parse()?;
                let cost_basis: f64 = parts[2].trim().parse()?;

                self.add_position(symbol, shares, cost_basis);
            }
        }

        Ok(())
    }

    /// Save portfolio to CSV file
    pub fn save_to_csv(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::Write;

        let mut file = File::create(filepath)?;
        writeln!(file, "Symbol,Shares,CostBasis")?;

        for position in self.positions.values() {
            writeln!(
                file,
                "{},{},{}",
                position.symbol, position.shares, position.cost_basis
            )?;
        }

        Ok(())
    }

    /// Calculate portfolio beta (requires historical data - advanced feature)
    pub async fn calculate_portfolio_beta(&self) -> Result<f64, Box<dyn std::error::Error>> {
        // This would require historical price data and market index data
        // Left as an exercise for production implementation
        println!("📊 Portfolio beta calculation would require historical data analysis");
        Ok(1.0) // Placeholder
    }

    /// Generate performance report
    pub fn generate_report(&self, values: &[PortfolioValue]) -> String {
        let mut report = String::new();

        report.push_str("# Portfolio Performance Report\n\n");
        report.push_str(&format!("Generated: {}\n\n", chrono::Utc::now()));

        let total_value: f64 = values.iter().map(|v| v.market_value).sum();
        let total_cost: f64 = values.iter().map(|v| v.total_cost).sum();
        let total_gain: f64 = total_value - total_cost;

        report.push_str(&format!("## Summary\n"));
        report.push_str(&format!("- Total Positions: {}\n", values.len()));
        report.push_str(&format!("- Total Market Value: ${:.2}\n", total_value));
        report.push_str(&format!("- Total Cost Basis: ${:.2}\n", total_cost));
        report.push_str(&format!(
            "- Total Gain/Loss: ${:.2} ({:.2}%)\n\n",
            total_gain,
            (total_gain / total_cost) * 100.0
        ));

        report.push_str("## Individual Positions\n\n");
        for value in values {
            report.push_str(&format!("### {}\n", value.symbol));
            report.push_str(&format!("- Shares: {:.2}\n", value.shares));
            report.push_str(&format!("- Current Price: ${:.2}\n", value.current_price));
            report.push_str(&format!("- Market Value: ${:.2}\n", value.market_value));
            report.push_str(&format!(
                "- Gain/Loss: ${:.2} ({:.2}%)\n\n",
                value.gain_loss, value.gain_loss_percent
            ));
        }

        report
    }
}

/*
Example CSV format for loading portfolio (save as portfolio.csv):

Symbol,Shares,CostBasis
AAPL,100,150.00
GOOGL,50,2800.00
MSFT,75,300.00
TSLA,25,800.00
NVDA,40,500.00

Usage:
let mut tracker = PortfolioTracker::new()?;
tracker.load_from_csv("portfolio.csv")?;

This example demonstrates:
1. Enterprise-grade reliability with EEYF presets
2. Concurrent request handling for multiple symbols
3. Real-time monitoring with configurable intervals
4. Proper error handling and recovery
5. CSV import/export for portfolio persistence
6. Professional portfolio analysis and reporting
7. Rate limiting awareness (no manual delays needed)
8. Caching for improved performance on repeated requests
*/
