//! Data export utilities for financial data.
//!
//! This module provides utilities for exporting quote data to various formats
//! including CSV, JSON, and Parquet.

use std::{
    fs::File,
    io::{BufWriter, Write},
    path::Path,
};

use serde::Serialize;

use crate::{quotes::Quote, screener::ScreenerQuote, yahoo_error::YahooError};

/// Export format options
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportFormat {
    /// Comma-separated values
    CSV,
    /// JavaScript Object Notation
    JSON,
    /// JSON with pretty printing
    JSONPretty,
}

/// Exports quotes to a file in the specified format
///
/// # Example
/// ```no_run
/// use eeyf::{
///     export::{ExportFormat, export_quotes},
///     quotes::Quote,
/// };
/// use rust_decimal::Decimal;
///
/// let quotes = vec![Quote {
///     timestamp: 1000,
///     open: Decimal::new(100, 0),
///     high: Decimal::new(105, 0),
///     low: Decimal::new(99, 0),
///     close: Decimal::new(103, 0),
///     volume: 1000,
///     adjclose: Decimal::new(103, 0),
/// }];
///
/// export_quotes(&quotes, "output.csv", ExportFormat::CSV).unwrap();
/// ```
pub fn export_quotes<P: AsRef<Path>>(
    quotes: &[Quote],
    path: P,
    format: ExportFormat,
) -> Result<(), YahooError> {
    match format {
        ExportFormat::CSV => export_quotes_csv(quotes, path),
        ExportFormat::JSON => export_quotes_json(quotes, path, false),
        ExportFormat::JSONPretty => export_quotes_json(quotes, path, true),
    }
}

/// Exports quotes to CSV format
fn export_quotes_csv<P: AsRef<Path>>(quotes: &[Quote], path: P) -> Result<(), YahooError> {
    let file = File::create(path)
        .map_err(|e| YahooError::FetchFailed(format!("Failed to create CSV file: {}", e)))?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "timestamp,datetime,open,high,low,close,volume,adjclose"
    )
    .map_err(|e| YahooError::FetchFailed(format!("Failed to write CSV header: {}", e)))?;

    // Write data
    for quote in quotes {
        let datetime = format_timestamp(quote.timestamp);
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{}",
            quote.timestamp,
            datetime,
            quote.open,
            quote.high,
            quote.low,
            quote.close,
            quote.volume,
            quote.adjclose
        )
        .map_err(|e| YahooError::FetchFailed(format!("Failed to write CSV row: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| YahooError::FetchFailed(format!("Failed to flush CSV writer: {}", e)))?;

    Ok(())
}

/// Exports quotes to JSON format
fn export_quotes_json<P: AsRef<Path>>(
    quotes: &[Quote],
    path: P,
    pretty: bool,
) -> Result<(), YahooError> {
    // Create serializable structs with datetime
    #[derive(Serialize)]
    struct QuoteWithDateTime {
        timestamp: i64,
        datetime: String,
        open: String,
        high: String,
        low: String,
        close: String,
        volume: u64,
        adjclose: String,
    }

    let quotes_with_dt: Vec<QuoteWithDateTime> = quotes
        .iter()
        .map(|q| QuoteWithDateTime {
            timestamp: q.timestamp,
            datetime: format_timestamp(q.timestamp),
            open: q.open.to_string(),
            high: q.high.to_string(),
            low: q.low.to_string(),
            close: q.close.to_string(),
            volume: q.volume,
            adjclose: q.adjclose.to_string(),
        })
        .collect();

    let file = File::create(path)
        .map_err(|e| YahooError::FetchFailed(format!("Failed to create JSON file: {}", e)))?;
    let writer = BufWriter::new(file);

    if pretty {
        serde_json::to_writer_pretty(writer, &quotes_with_dt)
    } else {
        serde_json::to_writer(writer, &quotes_with_dt)
    }
    .map_err(|e| YahooError::DeserializeFailed(format!("Failed to write JSON: {}", e)))?;

    Ok(())
}

/// Exports screener results to a file in the specified format
///
/// # Example
/// ```no_run
/// use eeyf::{
///     export::{ExportFormat, export_screener_results},
///     screener::ScreenerQuote,
/// };
///
/// let quotes = vec![ScreenerQuote {
///     symbol: "AAPL".to_string(),
///     short_name: Some("Apple Inc.".to_string()),
///     long_name: Some("Apple Inc.".to_string()),
///     regular_market_price: Some(175.43),
///     regular_market_change: Some(2.15),
///     regular_market_change_percent: Some(1.24),
///     regular_market_volume: Some(50000000),
///     average_daily_volume_3_month: Some(45000000),
///     average_daily_volume_10_day: Some(48000000),
///     market_cap: Some(2800000000000),
///     trailing_pe: Some(28.5),
///     forward_pe: Some(25.3),
///     dividend_yield: Some(0.52),
///     eps_trailing_twelve_months: Some(6.15),
///     fifty_two_week_high: Some(182.94),
///     fifty_two_week_low: Some(124.17),
///     exchange: Some("NMS".to_string()),
///     quote_type: Some("EQUITY".to_string()),
///     sector: None,
///     industry: None,
///     financial_currency: None,
/// }];
///
/// export_screener_results(&quotes, "screener.csv", ExportFormat::CSV).unwrap();
/// ```
pub fn export_screener_results<P: AsRef<Path>>(
    quotes: &[ScreenerQuote],
    path: P,
    format: ExportFormat,
) -> Result<(), YahooError> {
    match format {
        ExportFormat::CSV => export_screener_csv(quotes, path),
        ExportFormat::JSON => export_screener_json(quotes, path, false),
        ExportFormat::JSONPretty => export_screener_json(quotes, path, true),
    }
}

/// Exports screener results to CSV format
fn export_screener_csv<P: AsRef<Path>>(
    quotes: &[ScreenerQuote],
    path: P,
) -> Result<(), YahooError> {
    let file = File::create(path)
        .map_err(|e| YahooError::FetchFailed(format!("Failed to create CSV file: {}", e)))?;
    let mut writer = BufWriter::new(file);

    // Write header
    writeln!(
        writer,
        "symbol,name,price,change,change_percent,volume,market_cap,pe_ratio,dividend_yield,\
         exchange,type"
    )
    .map_err(|e| YahooError::FetchFailed(format!("Failed to write CSV header: {}", e)))?;

    // Write data
    for quote in quotes {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{},{},{},{}",
            quote.symbol,
            quote.short_name.as_deref().unwrap_or(""),
            quote.regular_market_price.unwrap_or(0.0),
            quote.regular_market_change.unwrap_or(0.0),
            quote.regular_market_change_percent.unwrap_or(0.0),
            quote.regular_market_volume.unwrap_or(0),
            quote.market_cap.unwrap_or(0),
            quote.trailing_pe.unwrap_or(0.0),
            quote.dividend_yield.unwrap_or(0.0),
            quote.exchange.as_deref().unwrap_or(""),
            quote.quote_type.as_deref().unwrap_or("")
        )
        .map_err(|e| YahooError::FetchFailed(format!("Failed to write CSV row: {}", e)))?;
    }

    writer
        .flush()
        .map_err(|e| YahooError::FetchFailed(format!("Failed to flush CSV writer: {}", e)))?;

    Ok(())
}

/// Exports screener results to JSON format
fn export_screener_json<P: AsRef<Path>>(
    quotes: &[ScreenerQuote],
    path: P,
    pretty: bool,
) -> Result<(), YahooError> {
    let file = File::create(path)
        .map_err(|e| YahooError::FetchFailed(format!("Failed to create JSON file: {}", e)))?;
    let writer = BufWriter::new(file);

    if pretty {
        serde_json::to_writer_pretty(writer, quotes)
    } else {
        serde_json::to_writer(writer, quotes)
    }
    .map_err(|e| YahooError::DeserializeFailed(format!("Failed to write JSON: {}", e)))?;

    Ok(())
}

/// Converts quotes to CSV string
///
/// # Example
/// ```
/// use eeyf::{export::quotes_to_csv, quotes::Quote};
/// use rust_decimal::Decimal;
///
/// let quotes = vec![Quote {
///     timestamp: 1000,
///     open: Decimal::new(100, 0),
///     high: Decimal::new(105, 0),
///     low: Decimal::new(99, 0),
///     close: Decimal::new(103, 0),
///     volume: 1000,
///     adjclose: Decimal::new(103, 0),
/// }];
///
/// let csv = quotes_to_csv(&quotes);
/// assert!(csv.contains("timestamp,datetime,open"));
/// ```
pub fn quotes_to_csv(quotes: &[Quote]) -> String {
    let mut output = String::new();
    output.push_str("timestamp,datetime,open,high,low,close,volume,adjclose\n");

    for quote in quotes {
        let datetime = format_timestamp(quote.timestamp);
        output.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            quote.timestamp,
            datetime,
            quote.open,
            quote.high,
            quote.low,
            quote.close,
            quote.volume,
            quote.adjclose
        ));
    }

    output
}

/// Converts quotes to JSON string
///
/// # Example
/// ```
/// use eeyf::{export::quotes_to_json, quotes::Quote};
/// use rust_decimal::Decimal;
///
/// let quotes = vec![Quote {
///     timestamp: 1000,
///     open: Decimal::new(100, 0),
///     high: Decimal::new(105, 0),
///     low: Decimal::new(99, 0),
///     close: Decimal::new(103, 0),
///     volume: 1000,
///     adjclose: Decimal::new(103, 0),
/// }];
///
/// let json = quotes_to_json(&quotes, true);
/// assert!(json.is_ok());
/// ```
pub fn quotes_to_json(quotes: &[Quote], pretty: bool) -> Result<String, YahooError> {
    // Create serializable structs with datetime
    #[derive(Serialize)]
    struct QuoteWithDateTime {
        timestamp: i64,
        datetime: String,
        open: String,
        high: String,
        low: String,
        close: String,
        volume: u64,
        adjclose: String,
    }

    let quotes_with_dt: Vec<QuoteWithDateTime> = quotes
        .iter()
        .map(|q| QuoteWithDateTime {
            timestamp: q.timestamp,
            datetime: format_timestamp(q.timestamp),
            open: q.open.to_string(),
            high: q.high.to_string(),
            low: q.low.to_string(),
            close: q.close.to_string(),
            volume: q.volume,
            adjclose: q.adjclose.to_string(),
        })
        .collect();

    if pretty {
        serde_json::to_string_pretty(&quotes_with_dt)
    } else {
        serde_json::to_string(&quotes_with_dt)
    }
    .map_err(|e| YahooError::DeserializeFailed(format!("Failed to serialize JSON: {}", e)))
}

/// Formats a Unix timestamp as ISO 8601 datetime
fn format_timestamp(timestamp: i64) -> String {
    use chrono::{DateTime, Utc};
    let dt = DateTime::<Utc>::from_timestamp(timestamp, 0).unwrap_or_default();
    dt.format("%Y-%m-%dT%H:%M:%SZ").to_string()
}

/// Builder for exporting data with custom options
pub struct ExportBuilder<'a, T> {
    data: &'a [T],
    format: ExportFormat,
    include_header: bool,
}

impl<'a> ExportBuilder<'a, Quote> {
    /// Creates a new export builder for quotes
    pub fn new(data: &'a [Quote]) -> Self {
        Self {
            data,
            format: ExportFormat::CSV,
            include_header: true,
        }
    }

    /// Sets the export format
    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets whether to include header row (CSV only)
    pub fn include_header(mut self, include: bool) -> Self {
        self.include_header = include;
        self
    }

    /// Exports to a file
    pub fn export<P: AsRef<Path>>(self, path: P) -> Result<(), YahooError> {
        export_quotes(self.data, path, self.format)
    }

    /// Exports to a string
    pub fn to_string(self) -> Result<String, YahooError> {
        match self.format {
            ExportFormat::CSV => Ok(quotes_to_csv(self.data)),
            ExportFormat::JSON => quotes_to_json(self.data, false),
            ExportFormat::JSONPretty => quotes_to_json(self.data, true),
        }
    }
}

impl<'a> ExportBuilder<'a, ScreenerQuote> {
    /// Creates a new export builder for screener quotes
    pub fn new(data: &'a [ScreenerQuote]) -> Self {
        Self {
            data,
            format: ExportFormat::CSV,
            include_header: true,
        }
    }

    /// Sets the export format
    pub fn format(mut self, format: ExportFormat) -> Self {
        self.format = format;
        self
    }

    /// Sets whether to include header row (CSV only)
    pub fn include_header(mut self, include: bool) -> Self {
        self.include_header = include;
        self
    }

    /// Exports to a file
    pub fn export<P: AsRef<Path>>(self, path: P) -> Result<(), YahooError> {
        export_screener_results(self.data, path, self.format)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    #[allow(unused_imports)] // Decimal is used as type alias for Quote fields
    use crate::quotes::Decimal;

    fn create_test_quote(timestamp: i64, close: i64) -> Quote {
        Quote {
            timestamp,
            open: close as f64,
            high: (close + 5) as f64,
            low: (close - 5) as f64,
            close: close as f64,
            volume: 1000,
            adjclose: close as f64,
        }
    }

    #[test]
    fn test_quotes_to_csv() {
        let quotes = vec![create_test_quote(1000, 100), create_test_quote(2000, 105)];

        let csv = quotes_to_csv(&quotes);
        assert!(csv.contains("timestamp,datetime,open"));
        assert!(csv.contains("1000,"));
        assert!(csv.contains("100,"));
    }

    #[test]
    fn test_quotes_to_json() {
        let quotes = vec![create_test_quote(1000, 100)];

        let json = quotes_to_json(&quotes, false).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_quotes_to_json_pretty() {
        let quotes = vec![create_test_quote(1000, 100)];

        let json = quotes_to_json(&quotes, true).unwrap();
        assert!(json.contains("timestamp"));
        assert!(json.contains("\n")); // Pretty printing includes newlines
    }

    #[test]
    fn test_format_timestamp() {
        let timestamp = 1609459200; // 2021-01-01 00:00:00 UTC
        let formatted = format_timestamp(timestamp);
        assert!(formatted.contains("2021-01-01"));
    }

    #[test]
    fn test_export_builder() {
        let quotes = vec![create_test_quote(1000, 100)];

        let csv = ExportBuilder::<Quote>::new(&quotes)
            .format(ExportFormat::CSV)
            .to_string()
            .unwrap();

        assert!(csv.contains("timestamp"));
    }

    #[test]
    fn test_export_quotes_csv() {
        let quotes = vec![create_test_quote(1000, 100)];
        let path = "test_export.csv";

        let result = export_quotes(&quotes, path, ExportFormat::CSV);
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_export_quotes_json() {
        let quotes = vec![create_test_quote(1000, 100)];
        let path = "test_export.json";

        let result = export_quotes(&quotes, path, ExportFormat::JSON);
        assert!(result.is_ok());

        // Cleanup
        let _ = fs::remove_file(path);
    }
}
