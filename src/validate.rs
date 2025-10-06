//! Data validation utilities for financial data.
//!
//! This module provides utilities for validating quote data integrity,
//! detecting anomalies, and filling missing data.

use crate::quotes::decimal::{Decimal, ZERO, ONE, from_usize};
use crate::quotes::Quote;

/// Validation errors that can occur during data validation
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// Missing required data
    MissingData { field: String, index: usize },
    /// Invalid price (negative or zero)
    InvalidPrice { index: usize, price: Decimal },
    /// OHLC data inconsistency (e.g., high < low)
    OhlcInconsistency { index: usize, reason: String },
    /// Timestamp out of order
    TimestampOutOfOrder { index: usize },
    /// Duplicate timestamp
    DuplicateTimestamp { index: usize, timestamp: i64 },
    /// Excessive gap in data
    DataGap { start_index: usize, gap_seconds: i64 },
    /// Anomalous value detected
    Anomaly { index: usize, reason: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::MissingData { field, index } => {
                write!(f, "Missing data for field '{}' at index {}", field, index)
            }
            ValidationError::InvalidPrice { index, price } => {
                write!(f, "Invalid price {} at index {}", price, index)
            }
            ValidationError::OhlcInconsistency { index, reason } => {
                write!(f, "OHLC inconsistency at index {}: {}", index, reason)
            }
            ValidationError::TimestampOutOfOrder { index } => {
                write!(f, "Timestamp out of order at index {}", index)
            }
            ValidationError::DuplicateTimestamp { index, timestamp } => {
                write!(f, "Duplicate timestamp {} at index {}", timestamp, index)
            }
            ValidationError::DataGap {
                start_index,
                gap_seconds,
            } => {
                write!(
                    f,
                    "Data gap of {} seconds starting at index {}",
                    gap_seconds, start_index
                )
            }
            ValidationError::Anomaly { index, reason } => {
                write!(f, "Anomaly detected at index {}: {}", index, reason)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Result of validation check
#[derive(Debug)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// List of validation errors found
    pub errors: Vec<ValidationError>,
    /// List of warnings (non-critical issues)
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Creates a successful validation result
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Creates a failed validation result
    pub fn failure(errors: Vec<ValidationError>) -> Self {
        Self {
            valid: false,
            errors,
            warnings: Vec::new(),
        }
    }

    /// Adds a warning to the result
    pub fn with_warning(mut self, warning: String) -> Self {
        self.warnings.push(warning);
        self
    }
}

/// Validates quote data integrity
///
/// Checks for:
/// - Valid OHLC relationships (high >= low, open/close within high/low)
/// - Non-negative prices
/// - Timestamps in order
/// - No duplicate timestamps
///
/// # Example
/// ```
/// use eeyf::validate::validate_quotes;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1000,
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
/// ];
///
/// let result = validate_quotes(&quotes);
/// assert!(result.valid);
/// ```
pub fn validate_quotes(quotes: &[Quote]) -> ValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if quotes.is_empty() {
        warnings.push("Empty quote list".to_string());
        return ValidationResult {
            valid: true,
            errors,
            warnings,
        };
    }

    for (i, quote) in quotes.iter().enumerate() {
        // Check for valid OHLC relationships
        if quote.high < quote.low {
            errors.push(ValidationError::OhlcInconsistency {
                index: i,
                reason: format!("High ({}) < Low ({})", quote.high, quote.low),
            });
        }

        if quote.high < quote.open {
            errors.push(ValidationError::OhlcInconsistency {
                index: i,
                reason: format!("High ({}) < Open ({})", quote.high, quote.open),
            });
        }

        if quote.high < quote.close {
            errors.push(ValidationError::OhlcInconsistency {
                index: i,
                reason: format!("High ({}) < Close ({})", quote.high, quote.close),
            });
        }

        if quote.low > quote.open {
            errors.push(ValidationError::OhlcInconsistency {
                index: i,
                reason: format!("Low ({}) > Open ({})", quote.low, quote.open),
            });
        }

        if quote.low > quote.close {
            errors.push(ValidationError::OhlcInconsistency {
                index: i,
                reason: format!("Low ({}) > Close ({})", quote.low, quote.close),
            });
        }

        // Check for non-negative prices
        if quote.open <= Decimal::ZERO {
            errors.push(ValidationError::InvalidPrice {
                index: i,
                price: quote.open,
            });
        }

        if quote.high <= Decimal::ZERO {
            errors.push(ValidationError::InvalidPrice {
                index: i,
                price: quote.high,
            });
        }

        if quote.low <= Decimal::ZERO {
            errors.push(ValidationError::InvalidPrice {
                index: i,
                price: quote.low,
            });
        }

        if quote.close <= Decimal::ZERO {
            errors.push(ValidationError::InvalidPrice {
                index: i,
                price: quote.close,
            });
        }

        // Check timestamp order
        if i > 0 {
            let prev_timestamp = quotes[i - 1].timestamp;
            if quote.timestamp < prev_timestamp {
                errors.push(ValidationError::TimestampOutOfOrder { index: i });
            } else if quote.timestamp == prev_timestamp {
                errors.push(ValidationError::DuplicateTimestamp {
                    index: i,
                    timestamp: quote.timestamp,
                });
            }
        }
    }

    let valid = errors.is_empty();
    ValidationResult {
        valid,
        errors,
        warnings,
    }
}

/// Detects anomalies in quote data using statistical methods
///
/// Uses the Interquartile Range (IQR) method to detect outliers
///
/// # Example
/// ```
/// use eeyf::validate::detect_anomalies;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1000,
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
///     // More quotes...
/// ];
///
/// let anomalies = detect_anomalies(&quotes, 3.0);
/// ```
pub fn detect_anomalies(quotes: &[Quote], threshold: f64) -> Vec<ValidationError> {
    if quotes.len() < 4 {
        return Vec::new(); // Need at least 4 points for IQR
    }

    let mut errors = Vec::new();
    let mut prices: Vec<f64> = quotes.iter().map(|q| q.close).collect();
    prices.sort_by(|a: &f64, b: &f64| a.partial_cmp(b).unwrap());

    // Calculate IQR
    let q1_idx = prices.len() / 4;
    let q3_idx = (prices.len() * 3) / 4;
    let q1 = prices[q1_idx];
    let q3 = prices[q3_idx];
    let iqr = q3 - q1;

    let lower_bound = q1 - threshold * iqr;
    let upper_bound = q3 + threshold * iqr;

    // Check each quote for anomalies
    for (i, quote) in quotes.iter().enumerate() {
        let price = quote.close;
        if price < lower_bound {
            errors.push(ValidationError::Anomaly {
                index: i,
                reason: format!(
                    "Price {} below lower bound {:.2}",
                    quote.close, lower_bound
                ),
            });
        } else if price > upper_bound {
            errors.push(ValidationError::Anomaly {
                index: i,
                reason: format!(
                    "Price {} above upper bound {:.2}",
                    quote.close, upper_bound
                ),
            });
        }

        // Check for excessive volume anomalies
        if i > 0 {
            let avg_volume = quotes.iter().map(|q| q.volume as f64).sum::<f64>()
                / quotes.len() as f64;
            if quote.volume as f64 > avg_volume * 10.0 {
                errors.push(ValidationError::Anomaly {
                    index: i,
                    reason: format!("Volume {} is 10x average volume", quote.volume),
                });
            }
        }
    }

    errors
}

/// Detects gaps in time series data
///
/// # Example
/// ```
/// use eeyf::validate::detect_gaps;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1000,
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
///     Quote {
///         timestamp: 5000,  // Gap of 4000 seconds
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let gaps = detect_gaps(&quotes, 3600); // 1 hour threshold
/// assert_eq!(gaps.len(), 1);
/// ```
pub fn detect_gaps(quotes: &[Quote], max_gap_seconds: i64) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    for i in 1..quotes.len() {
        let gap = quotes[i].timestamp - quotes[i - 1].timestamp;
        if gap > max_gap_seconds {
            errors.push(ValidationError::DataGap {
                start_index: i - 1,
                gap_seconds: gap,
            });
        }
    }

    errors
}

/// Fill methods for missing data
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FillMethod {
    /// Forward fill - use previous value
    Forward,
    /// Backward fill - use next value
    Backward,
    /// Linear interpolation
    Linear,
    /// Use zero
    Zero,
}

/// Fills missing data gaps by interpolating
///
/// # Example
/// ```
/// use eeyf::validate::{fill_gaps, FillMethod};
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1000,
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
///     Quote {
///         timestamp: 3000,  // Gap
///         open: Decimal::new(106, 0),
///         high: Decimal::new(110, 0),
///         low: Decimal::new(105, 0),
///         close: Decimal::new(109, 0),
///         volume: 1500,
///         adjclose: Decimal::new(109, 0),
///     },
/// ];
///
/// let filled = fill_gaps(&quotes, 600, FillMethod::Linear);
/// ```
pub fn fill_gaps(quotes: &[Quote], interval_seconds: i64, method: FillMethod) -> Vec<Quote> {
    if quotes.is_empty() {
        return Vec::new();
    }

    let mut result = Vec::new();
    result.push(quotes[0].clone());

    for i in 1..quotes.len() {
        let prev = &quotes[i - 1];
        let curr = &quotes[i];
        let gap = curr.timestamp - prev.timestamp;

        if gap > interval_seconds {
            // Fill the gap
            let num_missing = ((gap / interval_seconds) - 1) as usize;
            for j in 1..=num_missing {
                let ts = prev.timestamp + (j as i64 * interval_seconds);
                let filled_quote = match method {
                    FillMethod::Forward => Quote {
                        timestamp: ts,
                        open: prev.close,
                        high: prev.close,
                        low: prev.close,
                        close: prev.close,
                        volume: 0,
                        adjclose: prev.adjclose,
                    },
                    FillMethod::Backward => Quote {
                        timestamp: ts,
                        open: curr.open,
                        high: curr.open,
                        low: curr.open,
                        close: curr.open,
                        volume: 0,
                        adjclose: curr.adjclose,
                    },
                    FillMethod::Linear => {
                        let ratio = Decimal::from(j) / Decimal::from(num_missing + 1);
                        let interpolated = prev.close + (curr.close - prev.close) * ratio;
                        Quote {
                            timestamp: ts,
                            open: interpolated,
                            high: interpolated,
                            low: interpolated,
                            close: interpolated,
                            volume: 0,
                            adjclose: interpolated,
                        }
                    }
                    FillMethod::Zero => Quote {
                        timestamp: ts,
                        open: Decimal::ZERO,
                        high: Decimal::ZERO,
                        low: Decimal::ZERO,
                        close: Decimal::ZERO,
                        volume: 0,
                        adjclose: Decimal::ZERO,
                    },
                };
                result.push(filled_quote);
            }
        }

        result.push(curr.clone());
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_quote(timestamp: i64, close: i64) -> Quote {
        Quote {
            timestamp,
            open: Decimal::new(close, 0),
            high: Decimal::new(close + 5, 0),
            low: Decimal::new(close - 5, 0),
            close: Decimal::new(close, 0),
            volume: 1000,
            adjclose: Decimal::new(close, 0),
        }
    }

    #[test]
    fn test_validate_quotes_success() {
        let quotes = vec![create_valid_quote(1000, 100), create_valid_quote(2000, 105)];

        let result = validate_quotes(&quotes);
        assert!(result.valid);
        assert_eq!(result.errors.len(), 0);
    }

    #[test]
    fn test_validate_quotes_ohlc_inconsistency() {
        let mut quote = create_valid_quote(1000, 100);
        quote.high = Decimal::new(90, 0); // High < Low

        let result = validate_quotes(&[quote]);
        assert!(!result.valid);
        assert!(result.errors.len() > 0);
    }

    #[test]
    fn test_validate_quotes_negative_price() {
        let mut quote = create_valid_quote(1000, 100);
        quote.close = Decimal::new(-10, 0);

        let result = validate_quotes(&[quote]);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::InvalidPrice { .. }
        )));
    }

    #[test]
    fn test_validate_quotes_out_of_order() {
        let quotes = vec![create_valid_quote(2000, 100), create_valid_quote(1000, 105)];

        let result = validate_quotes(&quotes);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::TimestampOutOfOrder { .. }
        )));
    }

    #[test]
    fn test_validate_quotes_duplicate_timestamp() {
        let quotes = vec![create_valid_quote(1000, 100), create_valid_quote(1000, 105)];

        let result = validate_quotes(&quotes);
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| matches!(
            e,
            ValidationError::DuplicateTimestamp { .. }
        )));
    }

    #[test]
    fn test_detect_gaps() {
        let quotes = vec![create_valid_quote(1000, 100), create_valid_quote(5000, 105)];

        let gaps = detect_gaps(&quotes, 1000);
        assert_eq!(gaps.len(), 1);
    }

    #[test]
    fn test_fill_gaps_forward() {
        let quotes = vec![create_valid_quote(1000, 100), create_valid_quote(3000, 110)];

        let filled = fill_gaps(&quotes, 1000, FillMethod::Forward);
        assert_eq!(filled.len(), 3); // Original 2 + 1 filled
        assert_eq!(filled[1].timestamp, 2000);
        assert_eq!(filled[1].close, Decimal::new(100, 0)); // Forward filled
    }

    #[test]
    fn test_fill_gaps_linear() {
        let quotes = vec![create_valid_quote(1000, 100), create_valid_quote(3000, 110)];

        let filled = fill_gaps(&quotes, 1000, FillMethod::Linear);
        assert_eq!(filled.len(), 3);
        assert_eq!(filled[1].timestamp, 2000);
        assert_eq!(filled[1].close, Decimal::new(105, 0)); // Linear interpolation
    }

    #[test]
    fn test_detect_anomalies() {
        let quotes = vec![
            create_valid_quote(1000, 100),
            create_valid_quote(2000, 102),
            create_valid_quote(3000, 101),
            create_valid_quote(4000, 103),
            create_valid_quote(5000, 500), // Anomaly
        ];

        let anomalies = detect_anomalies(&quotes, 1.5);
        assert!(anomalies.len() > 0);
    }
}
