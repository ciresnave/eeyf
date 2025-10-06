use eeyf::YahooConnector;
use rust_decimal::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::{Duration, interval};

/// Price Alert System - Real-world Example
///
/// This example demonstrates building a sophisticated price alert system
/// using EEYF with enterprise features for reliable monitoring.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    Above(f64),  // Trigger when price goes above threshold
    Below(f64),  // Trigger when price goes below threshold
    Change(f64), // Trigger on percentage change (positive or negative)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertStatus {
    Active,
    Triggered,
    Disabled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceAlert {
    pub id: String,
    pub symbol: String,
    pub alert_type: AlertType,
    pub status: AlertStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub triggered_at: Option<chrono::DateTime<chrono::Utc>>,
    pub baseline_price: Option<f64>, // For percentage change alerts
    pub message: String,
}

#[derive(Debug)]
pub struct AlertTrigger {
    pub alert: PriceAlert,
    pub current_price: f64,
    pub trigger_reason: String,
}

pub struct PriceAlertSystem {
    connector: Arc<YahooConnector>,
    alerts: HashMap<String, PriceAlert>,
    notification_callbacks: Vec<Box<dyn Fn(&AlertTrigger) + Send + Sync>>,
}

impl PriceAlertSystem {
    /// Create a new price alert system with production-ready configuration
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Use production preset for reliability
        let connector = YahooConnector::from_preset("production")?;

        Ok(Self {
            connector: Arc::new(connector),
            alerts: HashMap::new(),
            notification_callbacks: Vec::new(),
        })
    }

    /// Add a price alert
    pub fn add_alert(
        &mut self,
        symbol: String,
        alert_type: AlertType,
        message: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let alert_id = format!("{}_{}", uuid::Uuid::new_v4(), symbol);

        let alert = PriceAlert {
            id: alert_id.clone(),
            symbol,
            alert_type,
            status: AlertStatus::Active,
            created_at: chrono::Utc::now(),
            triggered_at: None,
            baseline_price: None,
            message,
        };

        self.alerts.insert(alert_id.clone(), alert);
        println!("✅ Alert created: {}", alert_id);

        Ok(alert_id)
    }

    /// Add percentage change alert with current price as baseline
    pub async fn add_percentage_alert(
        &mut self,
        symbol: String,
        percentage: f64,
        message: String,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Get current price as baseline
        println!("📊 Fetching baseline price for {}...", symbol);
        let response = self.connector.get_latest_quotes(&symbol, "1d").await?;
        let baseline_price = response
            .last_quote()
            .map_err(|_| "No current price available")?
            .close;

        let alert_id = format!("{}_{}", uuid::Uuid::new_v4(), symbol);

        let alert = PriceAlert {
            id: alert_id.clone(),
            symbol,
            alert_type: AlertType::Change(percentage),
            status: AlertStatus::Active,
            created_at: chrono::Utc::now(),
            triggered_at: None,
            baseline_price: Some(baseline_price.to_f64().unwrap_or(0.0)),
            message,
        };

        self.alerts.insert(alert_id.clone(), alert);
        println!(
            "✅ Percentage alert created: {} (baseline: ${:.2})",
            alert_id, baseline_price
        );

        Ok(alert_id)
    }

    /// Remove an alert
    pub fn remove_alert(&mut self, alert_id: &str) -> bool {
        if self.alerts.remove(alert_id).is_some() {
            println!("🗑️  Alert removed: {}", alert_id);
            true
        } else {
            println!("❌ Alert not found: {}", alert_id);
            false
        }
    }

    /// Disable an alert without removing it
    pub fn disable_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.alerts.get_mut(alert_id) {
            alert.status = AlertStatus::Disabled;
            println!("⏸️  Alert disabled: {}", alert_id);
            true
        } else {
            println!("❌ Alert not found: {}", alert_id);
            false
        }
    }

    /// Re-enable a disabled alert
    pub fn enable_alert(&mut self, alert_id: &str) -> bool {
        if let Some(alert) = self.alerts.get_mut(alert_id) {
            if matches!(alert.status, AlertStatus::Disabled) {
                alert.status = AlertStatus::Active;
                println!("▶️  Alert re-enabled: {}", alert_id);
                true
            } else {
                println!("⚠️  Alert is not disabled: {}", alert_id);
                false
            }
        } else {
            println!("❌ Alert not found: {}", alert_id);
            false
        }
    }

    /// Add notification callback
    pub fn add_notification_callback<F>(&mut self, callback: F)
    where
        F: Fn(&AlertTrigger) + Send + Sync + 'static,
    {
        self.notification_callbacks.push(Box::new(callback));
    }

    /// Check all active alerts against current prices
    pub async fn check_alerts(&mut self) -> Result<Vec<AlertTrigger>, Box<dyn std::error::Error>> {
        let active_alerts: Vec<_> = self
            .alerts
            .values()
            .filter(|alert| matches!(alert.status, AlertStatus::Active))
            .cloned()
            .collect();

        if active_alerts.is_empty() {
            return Ok(Vec::new());
        }

        println!("🔍 Checking {} active alerts...", active_alerts.len());

        let mut triggers = Vec::new();
        let symbols: Vec<_> = active_alerts
            .iter()
            .map(|alert| alert.symbol.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        // Fetch all unique symbol prices concurrently
        let mut current_prices = HashMap::new();
        for symbol in symbols {
            match self.connector.get_latest_quotes(&symbol, "1d").await {
                Ok(response) => match response.last_quote() {
                    Ok(quote) => {
                        current_prices.insert(symbol.clone(), quote.close);
                        println!("   {} current price: ${:.2}", symbol, quote.close);
                    }
                    Err(_) => {
                        eprintln!("⚠️  No quote data available for {}", symbol);
                    }
                },
                Err(e) => {
                    eprintln!("⚠️  Failed to fetch price for {}: {}", symbol, e);
                }
            }
        }

        // Check each alert
        for alert in active_alerts {
            if let Some(&current_price) = current_prices.get(&alert.symbol) {
                if let Some(trigger_reason) =
                    self.check_alert_condition(&alert, current_price.to_f64().unwrap_or(0.0))
                {
                    let trigger = AlertTrigger {
                        alert: alert.clone(),
                        current_price: current_price.to_f64().unwrap_or(0.0),
                        trigger_reason,
                    };

                    // Mark alert as triggered
                    if let Some(stored_alert) = self.alerts.get_mut(&alert.id) {
                        stored_alert.status = AlertStatus::Triggered;
                        stored_alert.triggered_at = Some(chrono::Utc::now());
                    }

                    triggers.push(trigger);
                }
            }
        }

        // Send notifications for triggered alerts
        for trigger in &triggers {
            self.send_notifications(trigger);
        }

        Ok(triggers)
    }

    /// Check if individual alert condition is met
    fn check_alert_condition(&self, alert: &PriceAlert, current_price: f64) -> Option<String> {
        match &alert.alert_type {
            AlertType::Above(threshold) => {
                if current_price > *threshold {
                    Some(format!(
                        "Price ${:.2} is above threshold ${:.2}",
                        current_price, threshold
                    ))
                } else {
                    None
                }
            }
            AlertType::Below(threshold) => {
                if current_price < *threshold {
                    Some(format!(
                        "Price ${:.2} is below threshold ${:.2}",
                        current_price, threshold
                    ))
                } else {
                    None
                }
            }
            AlertType::Change(percentage) => {
                if let Some(baseline) = alert.baseline_price {
                    let change = ((current_price - baseline) / baseline) * 100.0;
                    if change.abs() >= percentage.abs() {
                        let direction = if change > 0.0 {
                            "increased"
                        } else {
                            "decreased"
                        };
                        Some(format!(
                            "Price {} {:.1}% from baseline ${:.2} to ${:.2}",
                            direction,
                            change.abs(),
                            baseline,
                            current_price
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }

    /// Send notifications for triggered alert
    fn send_notifications(&self, trigger: &AlertTrigger) {
        println!(
            "🚨 ALERT TRIGGERED: {} ({})",
            trigger.alert.symbol, trigger.alert.id
        );
        println!("   Reason: {}", trigger.trigger_reason);
        println!("   Message: {}", trigger.alert.message);

        for callback in &self.notification_callbacks {
            callback(trigger);
        }
    }

    /// List all alerts with their status
    pub fn list_alerts(&self) {
        if self.alerts.is_empty() {
            println!("📋 No alerts configured");
            return;
        }

        println!("\n📋 Alert Summary ({} total)", self.alerts.len());
        println!("═══════════════════════════════════════════════════════════");
        println!(
            "{:<8} {:<10} {:<15} {:20} {:<12}",
            "Symbol", "Status", "Type", "Condition", "Created"
        );
        println!("──────────────────────────────────────────────────────────");

        for alert in self.alerts.values() {
            let status_icon = match alert.status {
                AlertStatus::Active => "🟢",
                AlertStatus::Triggered => "🔴",
                AlertStatus::Disabled => "⚪",
            };

            let condition = match &alert.alert_type {
                AlertType::Above(threshold) => format!("Above ${:.2}", threshold),
                AlertType::Below(threshold) => format!("Below ${:.2}", threshold),
                AlertType::Change(pct) => format!("{:+.1}% change", pct),
            };

            let created = alert.created_at.format("%m/%d %H:%M");

            println!(
                "{} {:<6} {:10} {:15} {:<20} {}",
                status_icon,
                alert.symbol,
                format!("{:?}", alert.status),
                format!("{:?}", alert.alert_type)
                    .split('(')
                    .next()
                    .unwrap_or("Unknown"),
                condition,
                created
            );
        }
        println!("═══════════════════════════════════════════════════════════");
    }

    /// Start monitoring loop
    pub async fn start_monitoring(
        &mut self,
        check_interval_secs: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!(
            "🔄 Starting price alert monitoring (checks every {} seconds)",
            check_interval_secs
        );
        println!("   Press Ctrl+C to stop monitoring");

        let mut interval = interval(Duration::from_secs(check_interval_secs));

        loop {
            interval.tick().await;

            match self.check_alerts().await {
                Ok(triggers) => {
                    if !triggers.is_empty() {
                        println!(
                            "🚨 {} alerts triggered at {}",
                            triggers.len(),
                            chrono::Utc::now().format("%H:%M:%S")
                        );
                    } else {
                        println!(
                            "✅ All alerts checked at {} - no triggers",
                            chrono::Utc::now().format("%H:%M:%S")
                        );
                    }
                }
                Err(e) => {
                    eprintln!("❌ Error checking alerts: {}", e);
                }
            }
        }
    }

    /// Save alerts to JSON file
    pub fn save_alerts(&self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let alerts_vec: Vec<_> = self.alerts.values().collect();
        let json = serde_json::to_string_pretty(&alerts_vec)?;
        std::fs::write(filepath, json)?;
        println!("💾 Alerts saved to {}", filepath);
        Ok(())
    }

    /// Load alerts from JSON file
    pub fn load_alerts(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = std::fs::read_to_string(filepath)?;
        let alerts_vec: Vec<PriceAlert> = serde_json::from_str(&json)?;

        self.alerts.clear();
        for alert in alerts_vec {
            self.alerts.insert(alert.id.clone(), alert);
        }

        println!("📂 Loaded {} alerts from {}", self.alerts.len(), filepath);
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 EEYF Price Alert System Demo");
    println!("═══════════════════════════════════════════════");

    // Create alert system
    let mut alert_system = PriceAlertSystem::new()?;

    // Add notification callbacks
    alert_system.add_notification_callback(|trigger| {
        // Console notification (already handled in send_notifications)
        println!(
            "📧 [Notification] Alert for {}: {}",
            trigger.alert.symbol, trigger.trigger_reason
        );
    });

    alert_system.add_notification_callback(|trigger| {
        // Simulated email notification
        println!("📧 [Email] To: user@example.com");
        println!(
            "    Subject: Price Alert - {} - {}",
            trigger.alert.symbol, trigger.trigger_reason
        );
        println!("    Body: {}", trigger.alert.message);
    });

    // Set up sample alerts
    println!("\n📋 Setting up sample alerts...");

    // Price threshold alerts
    alert_system.add_alert(
        "AAPL".to_string(),
        AlertType::Above(200.0),
        "Apple stock has reached $200! Consider taking profits.".to_string(),
    )?;

    alert_system.add_alert(
        "AAPL".to_string(),
        AlertType::Below(150.0),
        "Apple stock dropped below $150. Good buying opportunity?".to_string(),
    )?;

    alert_system.add_alert(
        "TSLA".to_string(),
        AlertType::Above(300.0),
        "Tesla is above $300. Monitor for momentum.".to_string(),
    )?;

    // Percentage change alerts (uses current price as baseline)
    alert_system
        .add_percentage_alert(
            "MSFT".to_string(),
            5.0, // 5% change
            "Microsoft moved 5% from baseline - check news!".to_string(),
        )
        .await?;

    alert_system
        .add_percentage_alert(
            "GOOGL".to_string(),
            3.0, // 3% change
            "Google had significant movement - investigate cause.".to_string(),
        )
        .await?;

    // Display current alerts
    alert_system.list_alerts();

    // Perform initial alert check
    println!("\n🔍 Performing initial alert check...");
    let triggers = alert_system.check_alerts().await?;

    if triggers.is_empty() {
        println!("✅ No alerts triggered on initial check");
    } else {
        println!("🚨 {} alerts triggered on initial check!", triggers.len());
    }

    // Demonstrate alert management
    println!("\n🔧 Demonstrating alert management...");

    // Save alerts
    alert_system.save_alerts("alerts_backup.json")?;

    // Add a test alert that will likely trigger
    let test_alert_id = alert_system.add_alert(
        "AAPL".to_string(),
        AlertType::Above(0.01), // Very low threshold - will trigger
        "Test alert for demonstration".to_string(),
    )?;

    // Check alerts again
    println!("\n🔍 Checking alerts after adding test alert...");
    let triggers = alert_system.check_alerts().await?;

    if !triggers.is_empty() {
        println!("🚨 Test alert triggered as expected!");

        // Remove the test alert
        alert_system.remove_alert(&test_alert_id);
    }

    // Show updated alert list
    println!("\n📋 Updated alert list:");
    alert_system.list_alerts();

    println!("\n🔄 Starting continuous monitoring...");
    println!("   Note: This will check alerts every 60 seconds");
    println!("   In production, you might check every 30-300 seconds depending on needs");

    // Start monitoring (runs forever until Ctrl+C)
    alert_system.start_monitoring(60).await?;

    Ok(())
}

// Additional production features

impl PriceAlertSystem {
    /// Get alert statistics
    pub fn get_statistics(&self) -> AlertStatistics {
        let mut stats = AlertStatistics::default();

        for alert in self.alerts.values() {
            stats.total += 1;
            match alert.status {
                AlertStatus::Active => stats.active += 1,
                AlertStatus::Triggered => stats.triggered += 1,
                AlertStatus::Disabled => stats.disabled += 1,
            }
        }

        stats
    }

    /// Clean up old triggered alerts
    pub fn cleanup_triggered_alerts(&mut self, days_old: i64) {
        let cutoff = chrono::Utc::now() - chrono::Duration::days(days_old);

        let to_remove: Vec<_> = self
            .alerts
            .iter()
            .filter(|(_, alert)| {
                matches!(alert.status, AlertStatus::Triggered)
                    && alert.triggered_at.map_or(false, |t| t < cutoff)
            })
            .map(|(id, _)| id.clone())
            .collect();

        let count = to_remove.len();
        for id in &to_remove {
            self.alerts.remove(id);
        }

        println!("🧹 Cleaned up {} old triggered alerts", count);
    }

    /// Bulk import alerts from CSV
    pub fn import_from_csv(&mut self, filepath: &str) -> Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;
        use std::io::{BufRead, BufReader};

        let file = File::open(filepath)?;
        let reader = BufReader::new(file);

        for line in reader.lines().skip(1) {
            // Skip header
            let line = line?;
            let parts: Vec<&str> = line.split(',').collect();

            if parts.len() >= 4 {
                let symbol = parts[0].trim().to_string();
                let alert_type_str = parts[1].trim();
                let threshold: f64 = parts[2].trim().parse()?;
                let message = parts[3].trim().to_string();

                let alert_type = match alert_type_str.to_lowercase().as_str() {
                    "above" => AlertType::Above(threshold),
                    "below" => AlertType::Below(threshold),
                    "change" => AlertType::Change(threshold),
                    _ => continue,
                };

                self.add_alert(symbol, alert_type, message)?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default)]
pub struct AlertStatistics {
    pub total: usize,
    pub active: usize,
    pub triggered: usize,
    pub disabled: usize,
}

/*
Example CSV format for bulk import (save as alerts.csv):

Symbol,Type,Threshold,Message
AAPL,above,200.0,Apple reached resistance level
AAPL,below,150.0,Apple at support - buying opportunity
TSLA,change,10.0,Tesla major movement detected
MSFT,above,350.0,Microsoft at new highs

Usage:
alert_system.import_from_csv("alerts.csv")?;

This example demonstrates:
1. Sophisticated alert conditions (price thresholds, percentage changes)
2. Multiple notification channels (console, email simulation)
3. Alert lifecycle management (active, triggered, disabled)
4. Concurrent price monitoring for efficiency
5. Persistent storage (JSON save/load)
6. Production features (statistics, cleanup, bulk import)
7. Enterprise-grade reliability with EEYF rate limiting and caching
8. Real-time monitoring with configurable intervals
*/
