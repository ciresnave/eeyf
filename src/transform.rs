//! Data transformation utilities for quote data.
//!
//! This module provides utilities for transforming and aggregating financial data,
//! including OHLC data aggregation, moving averages, and technical indicators.

use crate::quotes::decimal::{Decimal, ZERO, ONE, from_usize, from_f64};
use crate::quotes::Quote;
use crate::yahoo_error::YahooError;

/// Represents different timeframe intervals for aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Interval {
    /// 1 minute
    Minute1,
    /// 5 minutes
    Minute5,
    /// 15 minutes
    Minute15,
    /// 30 minutes
    Minute30,
    /// 1 hour
    Hour1,
    /// 4 hours
    Hour4,
    /// 1 day
    Day1,
    /// 1 week
    Week1,
    /// 1 month
    Month1,
}

impl Interval {
    /// Returns the interval duration in seconds
    pub fn as_seconds(&self) -> i64 {
        match self {
            Interval::Minute1 => 60,
            Interval::Minute5 => 300,
            Interval::Minute15 => 900,
            Interval::Minute30 => 1800,
            Interval::Hour1 => 3600,
            Interval::Hour4 => 14400,
            Interval::Day1 => 86400,
            Interval::Week1 => 604800,
            Interval::Month1 => 2592000, // Approximate (30 days)
        }
    }
}

/// Aggregates quotes into larger time intervals
///
/// # Example
/// ```
/// use eeyf::transform::{aggregate_quotes, Interval};
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
///         timestamp: 1300,
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let aggregated = aggregate_quotes(&quotes, Interval::Minute5);
/// ```
pub fn aggregate_quotes(quotes: &[Quote], interval: Interval) -> Vec<Quote> {
    if quotes.is_empty() {
        return Vec::new();
    }

    let interval_seconds = interval.as_seconds();
    let mut result = Vec::new();
    let mut current_group: Vec<&Quote> = Vec::new();
    let mut current_bucket = quotes[0].timestamp / interval_seconds;

    for quote in quotes {
        let bucket = quote.timestamp / interval_seconds;

        if bucket != current_bucket {
            // Aggregate the current group
            if !current_group.is_empty() {
                result.push(aggregate_group(&current_group, current_bucket * interval_seconds));
                current_group.clear();
            }
            current_bucket = bucket;
        }

        current_group.push(quote);
    }

    // Aggregate the last group
    if !current_group.is_empty() {
        result.push(aggregate_group(&current_group, current_bucket * interval_seconds));
    }

    result
}

/// Aggregates a group of quotes into a single quote
fn aggregate_group(quotes: &[&Quote], timestamp: i64) -> Quote {
    let open = quotes.first().unwrap().open;
    let close = quotes.last().unwrap().close;
    let high = quotes.iter().map(|q| q.high).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let low = quotes.iter().map(|q| q.low).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
    let volume = quotes.iter().map(|q| q.volume).sum();
    let adjclose = quotes.last().unwrap().adjclose;

    Quote {
        timestamp,
        open,
        high,
        low,
        close,
        volume,
        adjclose,
    }
}

/// Calculates Simple Moving Average (SMA)
///
/// # Example
/// ```
/// use eeyf::transform::calculate_sma;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(102, 0),
///     Decimal::new(104, 0),
///     Decimal::new(103, 0),
///     Decimal::new(105, 0),
/// ];
///
/// let sma = calculate_sma(&prices, 3);
/// assert_eq!(sma.len(), 3); // 5 - 3 + 1 = 3 values
/// ```
pub fn calculate_sma(values: &[Decimal], period: usize) -> Vec<Decimal> {
    if values.len() < period {
        return Vec::new();
    }

    let mut result = Vec::new();
    let mut sum = values[..period].iter().sum::<Decimal>();

    result.push(sum / Decimal::from(period));

    for i in period..values.len() {
        sum = sum - values[i - period] + values[i];
        result.push(sum / Decimal::from(period));
    }

    result
}

/// Calculates Exponential Moving Average (EMA)
///
/// # Example
/// ```
/// use eeyf::transform::calculate_ema;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(102, 0),
///     Decimal::new(104, 0),
///     Decimal::new(103, 0),
///     Decimal::new(105, 0),
/// ];
///
/// let ema = calculate_ema(&prices, 3);
/// assert_eq!(ema.len(), 5);
/// ```
pub fn calculate_ema(values: &[Decimal], period: usize) -> Vec<Decimal> {
    if values.is_empty() || period == 0 {
        return Vec::new();
    }

    let mut result = Vec::new();
    let multiplier = Decimal::from(2) / Decimal::from(period + 1);

    // First EMA is the first value
    result.push(values[0]);

    for i in 1..values.len() {
        let ema = (values[i] - result[i - 1]) * multiplier + result[i - 1];
        result.push(ema);
    }

    result
}

/// Calculates Relative Strength Index (RSI)
///
/// # Example
/// ```
/// use eeyf::transform::calculate_rsi;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(102, 0),
///     Decimal::new(104, 0),
///     Decimal::new(103, 0),
///     Decimal::new(105, 0),
///     Decimal::new(107, 0),
///     Decimal::new(106, 0),
///     Decimal::new(108, 0),
/// ];
///
/// let rsi = calculate_rsi(&prices, 7);
/// ```
pub fn calculate_rsi(values: &[Decimal], period: usize) -> Result<Vec<Decimal>, YahooError> {
    if values.len() < period + 1 {
        return Err(YahooError::DataInconsistency);
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    // Calculate price changes
    for i in 1..values.len() {
        let change = values[i] - values[i - 1];
        if change > Decimal::ZERO {
            gains.push(change);
            losses.push(Decimal::ZERO);
        } else {
            gains.push(Decimal::ZERO);
            losses.push(change.abs());
        }
    }

    let mut result = Vec::new();

    // Calculate initial average gain and loss
    let mut avg_gain: Decimal = gains[..period].iter().sum::<Decimal>() / Decimal::from(period);
    let mut avg_loss: Decimal = losses[..period].iter().sum::<Decimal>() / Decimal::from(period);

    // Calculate first RSI
    let rs = if avg_loss == Decimal::ZERO {
        Decimal::from(100)
    } else {
        avg_gain / avg_loss
    };
    result.push(Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs)));

    // Calculate subsequent RSI values
    for i in period..gains.len() {
        avg_gain = (avg_gain * Decimal::from(period - 1) + gains[i]) / Decimal::from(period);
        avg_loss = (avg_loss * Decimal::from(period - 1) + losses[i]) / Decimal::from(period);

        let rs = if avg_loss == Decimal::ZERO {
            Decimal::from(100)
        } else {
            avg_gain / avg_loss
        };
        result.push(Decimal::from(100) - (Decimal::from(100) / (Decimal::ONE + rs)));
    }

    Ok(result)
}

/// Calculates Bollinger Bands
///
/// Returns a tuple of (middle_band, upper_band, lower_band)
///
/// # Example
/// ```
/// use eeyf::transform::calculate_bollinger_bands;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(102, 0),
///     Decimal::new(104, 0),
///     Decimal::new(103, 0),
///     Decimal::new(105, 0),
/// ];
///
/// let (middle, upper, lower) = calculate_bollinger_bands(&prices, 3, Decimal::new(2, 0));
/// ```
pub fn calculate_bollinger_bands(
    values: &[Decimal],
    period: usize,
    std_dev_multiplier: Decimal,
) -> (Vec<Decimal>, Vec<Decimal>, Vec<Decimal>) {
    let sma = calculate_sma(values, period);
    let mut upper = Vec::new();
    let mut lower = Vec::new();

    for (i, &middle_value) in sma.iter().enumerate() {
        let start_idx = i;
        let end_idx = i + period;
        let window = &values[start_idx..end_idx];

        // Calculate standard deviation
        let mean = middle_value;
        let variance: Decimal = window
            .iter()
            .map(|&x| {
                let diff = x - mean;
                diff * diff
            })
            .sum::<Decimal>()
            / Decimal::from(period);

        let std_dev = Decimal::from_f64(variance.to_f64().unwrap_or(0.0).sqrt()).unwrap_or(Decimal::ZERO);
        let band_width = std_dev * std_dev_multiplier;

        upper.push(middle_value + band_width);
        lower.push(middle_value - band_width);
    }

    (sma, upper, lower)
}

/// Calculates Moving Average Convergence Divergence (MACD)
///
/// Returns a tuple of (macd_line, signal_line, histogram)
///
/// # Example
/// ```
/// use eeyf::transform::calculate_macd;
/// use rust_decimal::Decimal;
///
/// let prices: Vec<Decimal> = (0..50).map(|i| Decimal::from(100 + i)).collect();
///
/// let (macd, signal, histogram) = calculate_macd(&prices, 12, 26, 9);
/// ```
pub fn calculate_macd(
    values: &[Decimal],
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
) -> (Vec<Decimal>, Vec<Decimal>, Vec<Decimal>) {
    let fast_ema = calculate_ema(values, fast_period);
    let slow_ema = calculate_ema(values, slow_period);

    // Calculate MACD line
    let macd_line: Vec<Decimal> = fast_ema
        .iter()
        .zip(slow_ema.iter())
        .map(|(&fast, &slow)| fast - slow)
        .collect();

    // Calculate signal line (EMA of MACD)
    let signal_line = calculate_ema(&macd_line, signal_period);

    // Calculate histogram
    let histogram: Vec<Decimal> = macd_line[macd_line.len() - signal_line.len()..]
        .iter()
        .zip(signal_line.iter())
        .map(|(&macd, &signal)| macd - signal)
        .collect();

    // Trim MACD and signal to match histogram length
    let start_idx = macd_line.len() - histogram.len();
    let macd_trimmed = macd_line[start_idx..].to_vec();
    let signal_trimmed = signal_line[signal_line.len() - histogram.len()..].to_vec();

    (macd_trimmed, signal_trimmed, histogram)
}

/// Calculates percentage returns
///
/// # Example
/// ```
/// use eeyf::transform::calculate_returns;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(105, 0),
///     Decimal::new(103, 0),
/// ];
///
/// let returns = calculate_returns(&prices);
/// // returns[0] = (105 - 100) / 100 = 0.05 = 5%
/// ```
pub fn calculate_returns(values: &[Decimal]) -> Vec<Decimal> {
    if values.len() < 2 {
        return Vec::new();
    }

    values
        .windows(2)
        .map(|w| (w[1] - w[0]) / w[0])
        .collect()
}

/// Calculates log returns
///
/// # Example
/// ```
/// use eeyf::transform::calculate_log_returns;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(105, 0),
///     Decimal::new(103, 0),
/// ];
///
/// let log_returns = calculate_log_returns(&prices);
/// ```
pub fn calculate_log_returns(values: &[Decimal]) -> Vec<f64> {
    if values.len() < 2 {
        return Vec::new();
    }

    values
        .windows(2)
        .map(|w| {
            let ratio = w[1].to_f64().unwrap_or(1.0) / w[0].to_f64().unwrap_or(1.0);
            ratio.ln()
        })
        .collect()
}

/// Calculates cumulative returns
///
/// # Example
/// ```
/// use eeyf::transform::calculate_cumulative_returns;
/// use rust_decimal::Decimal;
///
/// let prices = vec![
///     Decimal::new(100, 0),
///     Decimal::new(105, 0),
///     Decimal::new(110, 0),
/// ];
///
/// let cum_returns = calculate_cumulative_returns(&prices);
/// ```
pub fn calculate_cumulative_returns(values: &[Decimal]) -> Vec<Decimal> {
    if values.is_empty() {
        return Vec::new();
    }

    let base = values[0];
    values.iter().map(|&v| (v - base) / base).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interval_seconds() {
        assert_eq!(Interval::Minute1.as_seconds(), 60);
        assert_eq!(Interval::Minute5.as_seconds(), 300);
        assert_eq!(Interval::Day1.as_seconds(), 86400);
    }

    #[test]
    fn test_aggregate_quotes() {
        let quotes = vec![
            Quote {
                timestamp: 100,
                open: Decimal::new(100, 0),
                high: Decimal::new(105, 0),
                low: Decimal::new(99, 0),
                close: Decimal::new(103, 0),
                volume: 1000,
                adjclose: Decimal::new(103, 0),
            },
            Quote {
                timestamp: 150,
                open: Decimal::new(103, 0),
                high: Decimal::new(107, 0),
                low: Decimal::new(102, 0),
                close: Decimal::new(106, 0),
                volume: 1500,
                adjclose: Decimal::new(106, 0),
            },
        ];

        let aggregated = aggregate_quotes(&quotes, Interval::Minute5);
        assert_eq!(aggregated.len(), 1);
        assert_eq!(aggregated[0].open, Decimal::new(100, 0));
        assert_eq!(aggregated[0].close, Decimal::new(106, 0));
        assert_eq!(aggregated[0].high, Decimal::new(107, 0));
        assert_eq!(aggregated[0].low, Decimal::new(99, 0));
        assert_eq!(aggregated[0].volume, 2500);
    }

    #[test]
    fn test_calculate_sma() {
        let values = vec![
            Decimal::new(10, 0),
            Decimal::new(20, 0),
            Decimal::new(30, 0),
            Decimal::new(40, 0),
            Decimal::new(50, 0),
        ];

        let sma = calculate_sma(&values, 3);
        assert_eq!(sma.len(), 3);
        assert_eq!(sma[0], Decimal::new(20, 0)); // (10+20+30)/3
        assert_eq!(sma[1], Decimal::new(30, 0)); // (20+30+40)/3
        assert_eq!(sma[2], Decimal::new(40, 0)); // (30+40+50)/3
    }

    #[test]
    fn test_calculate_ema() {
        let values = vec![
            Decimal::new(100, 0),
            Decimal::new(102, 0),
            Decimal::new(104, 0),
        ];

        let ema = calculate_ema(&values, 2);
        assert_eq!(ema.len(), 3);
        assert_eq!(ema[0], Decimal::new(100, 0)); // First value
    }

    #[test]
    fn test_calculate_returns() {
        let values = vec![
            Decimal::new(100, 0),
            Decimal::new(105, 0),
            Decimal::new(103, 0),
        ];

        let returns = calculate_returns(&values);
        assert_eq!(returns.len(), 2);
        assert_eq!(returns[0], Decimal::new(5, 2)); // 5%
    }

    #[test]
    fn test_calculate_cumulative_returns() {
        let values = vec![
            Decimal::new(100, 0),
            Decimal::new(110, 0),
            Decimal::new(120, 0),
        ];

        let cum_returns = calculate_cumulative_returns(&values);
        assert_eq!(cum_returns.len(), 3);
        assert_eq!(cum_returns[0], Decimal::ZERO); // Base
        assert_eq!(cum_returns[1], Decimal::new(10, 2)); // 10%
        assert_eq!(cum_returns[2], Decimal::new(20, 2)); // 20%
    }

    #[test]
    fn test_calculate_rsi() {
        let values: Vec<Decimal> = vec![
            Decimal::new(44, 0),
            Decimal::new(45, 0),
            Decimal::new(46, 0),
            Decimal::new(47, 0),
            Decimal::new(46, 0),
            Decimal::new(47, 0),
            Decimal::new(48, 0),
            Decimal::new(49, 0),
        ];

        let rsi = calculate_rsi(&values, 7).unwrap();
        assert!(rsi.len() > 0);
        // RSI should be between 0 and 100
        for &value in &rsi {
            assert!(value >= Decimal::ZERO && value <= Decimal::from(100));
        }
    }

    #[test]
    fn test_calculate_bollinger_bands() {
        let values = vec![
            Decimal::new(100, 0),
            Decimal::new(102, 0),
            Decimal::new(104, 0),
            Decimal::new(103, 0),
            Decimal::new(105, 0),
        ];

        let (middle, upper, lower) = calculate_bollinger_bands(&values, 3, Decimal::new(2, 0));
        assert_eq!(middle.len(), upper.len());
        assert_eq!(middle.len(), lower.len());
        assert_eq!(middle.len(), 3);

        // Upper band should be above middle, lower should be below
        for i in 0..middle.len() {
            assert!(upper[i] >= middle[i]);
            assert!(lower[i] <= middle[i]);
        }
    }

    #[test]
    fn test_calculate_macd() {
        let values: Vec<Decimal> = (0..50).map(|i| Decimal::from(100 + i)).collect();

        let (macd, signal, histogram) = calculate_macd(&values, 12, 26, 9);
        assert_eq!(macd.len(), signal.len());
        assert_eq!(macd.len(), histogram.len());
    }
}
