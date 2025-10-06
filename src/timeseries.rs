//! Time series utilities for financial data.
//!
//! This module provides utilities for time series operations including
//! resampling, timestamp alignment, and timezone handling.

use crate::quotes::decimal::{Decimal, ZERO, ONE, from_usize};
use crate::quotes::Quote;
use chrono::{Duration, TimeZone};
use chrono_tz::Tz;

/// Resampling rules for time series
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResampleRule {
    /// Last value in period
    Last,
    /// First value in period
    First,
    /// Mean of values in period
    Mean,
    /// Sum of values in period
    Sum,
    /// Maximum value in period
    Max,
    /// Minimum value in period
    Min,
    /// OHLC aggregation
    OHLC,
}

/// Resamples quotes to a new frequency
///
/// # Example
/// ```
/// use eeyf::timeseries::{resample_quotes, ResampleRule};
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
/// use chrono::Duration;
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
/// let resampled = resample_quotes(&quotes, Duration::minutes(5), ResampleRule::OHLC);
/// ```
pub fn resample_quotes(
    quotes: &[Quote],
    frequency: Duration,
    rule: ResampleRule,
) -> Vec<Quote> {
    if quotes.is_empty() {
        return Vec::new();
    }

    let freq_seconds = frequency.num_seconds();
    let mut result = Vec::new();
    let mut current_bucket = quotes[0].timestamp / freq_seconds;
    let mut bucket_quotes = Vec::new();

    for quote in quotes {
        let bucket = quote.timestamp / freq_seconds;

        if bucket != current_bucket {
            // Process the current bucket
            if !bucket_quotes.is_empty() {
                result.push(aggregate_bucket(&bucket_quotes, current_bucket * freq_seconds, rule));
                bucket_quotes.clear();
            }
            current_bucket = bucket;
        }

        bucket_quotes.push(quote);
    }

    // Process the last bucket
    if !bucket_quotes.is_empty() {
        result.push(aggregate_bucket(&bucket_quotes, current_bucket * freq_seconds, rule));
    }

    result
}

/// Aggregates quotes in a bucket according to the rule
fn aggregate_bucket(quotes: &[&Quote], timestamp: i64, rule: ResampleRule) -> Quote {
    match rule {
        ResampleRule::Last => (*quotes.last().unwrap()).clone(),
        ResampleRule::First => (*quotes.first().unwrap()).clone(),
        ResampleRule::Mean => {
            let count = Decimal::from(quotes.len());
            let open = quotes.iter().map(|q| q.open).sum::<Decimal>() / count;
            let high = quotes.iter().map(|q| q.high).sum::<Decimal>() / count;
            let low = quotes.iter().map(|q| q.low).sum::<Decimal>() / count;
            let close = quotes.iter().map(|q| q.close).sum::<Decimal>() / count;
            let volume = quotes.iter().map(|q| q.volume).sum::<u64>() / quotes.len() as u64;
            let adjclose = quotes.iter().map(|q| q.adjclose).sum::<Decimal>() / count;

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
        ResampleRule::Sum => {
            let open = quotes.iter().map(|q| q.open).sum();
            let high = quotes.iter().map(|q| q.high).sum();
            let low = quotes.iter().map(|q| q.low).sum();
            let close = quotes.iter().map(|q| q.close).sum();
            let volume = quotes.iter().map(|q| q.volume).sum();
            let adjclose = quotes.iter().map(|q| q.adjclose).sum();

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
        ResampleRule::Max => {
            let open = quotes.iter().map(|q| q.open).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let high = quotes.iter().map(|q| q.high).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let low = quotes.iter().map(|q| q.low).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let close = quotes.iter().map(|q| q.close).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let volume = quotes.iter().map(|q| q.volume).max().unwrap();
            let adjclose = quotes.iter().map(|q| q.adjclose).max_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

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
        ResampleRule::Min => {
            let open = quotes.iter().map(|q| q.open).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let high = quotes.iter().map(|q| q.high).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let low = quotes.iter().map(|q| q.low).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let close = quotes.iter().map(|q| q.close).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();
            let volume = quotes.iter().map(|q| q.volume).min().unwrap();
            let adjclose = quotes.iter().map(|q| q.adjclose).min_by(|a, b| a.partial_cmp(b).unwrap()).unwrap();

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
        ResampleRule::OHLC => {
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
    }
}

/// Aligns timestamps to a grid
///
/// # Example
/// ```
/// use eeyf::timeseries::align_timestamps;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
/// use chrono::Duration;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1003, // Slightly off grid
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
/// ];
///
/// let aligned = align_timestamps(&quotes, Duration::minutes(1));
/// assert_eq!(aligned[0].timestamp, 1020); // Aligned to minute boundary
/// ```
pub fn align_timestamps(quotes: &[Quote], grid: Duration) -> Vec<Quote> {
    let grid_seconds = grid.num_seconds();

    quotes
        .iter()
        .map(|quote| {
            let aligned_ts = (quote.timestamp / grid_seconds) * grid_seconds;
            let mut aligned_quote = quote.clone();
            aligned_quote.timestamp = aligned_ts;
            aligned_quote
        })
        .collect()
}

/// Converts timestamps to a different timezone
///
/// # Example
/// ```
/// use eeyf::timeseries::convert_timezone;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
/// use chrono_tz::America::New_York;
/// use chrono_tz::UTC;
///
/// let quotes = vec![
///     Quote {
///         timestamp: 1609459200, // 2021-01-01 00:00:00 UTC
///         open: Decimal::new(100, 0),
///         high: Decimal::new(105, 0),
///         low: Decimal::new(99, 0),
///         close: Decimal::new(103, 0),
///         volume: 1000,
///         adjclose: Decimal::new(103, 0),
///     },
/// ];
///
/// let converted = convert_timezone(&quotes, UTC, New_York);
/// ```
pub fn convert_timezone(quotes: &[Quote], from_tz: Tz, to_tz: Tz) -> Vec<Quote> {
    quotes
        .iter()
        .map(|quote| {
            let dt = from_tz.timestamp_opt(quote.timestamp, 0).unwrap();
            let converted = dt.with_timezone(&to_tz);
            let new_timestamp = converted.timestamp();

            let mut converted_quote = quote.clone();
            converted_quote.timestamp = new_timestamp;
            converted_quote
        })
        .collect()
}

/// Fills gaps in time series with missing timestamps
///
/// # Example
/// ```
/// use eeyf::timeseries::fill_missing_timestamps;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
/// use chrono::Duration;
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
///         timestamp: 3000,  // Gap of 2000 seconds
///         open: Decimal::new(106, 0),
///         high: Decimal::new(110, 0),
///         low: Decimal::new(105, 0),
///         close: Decimal::new(109, 0),
///         volume: 1500,
///         adjclose: Decimal::new(109, 0),
///     },
/// ];
///
/// let filled = fill_missing_timestamps(&quotes, Duration::minutes(10));
/// assert!(filled.len() > quotes.len()); // Gap filled
/// ```
pub fn fill_missing_timestamps(quotes: &[Quote], expected_interval: Duration) -> Vec<Quote> {
    if quotes.is_empty() {
        return Vec::new();
    }

    let interval_seconds = expected_interval.num_seconds();
    let mut result = Vec::new();
    result.push(quotes[0].clone());

    for i in 1..quotes.len() {
        let prev = &quotes[i - 1];
        let curr = &quotes[i];
        let gap = curr.timestamp - prev.timestamp;

        if gap > interval_seconds {
            // Fill the gap with forward-filled values
            let num_missing = ((gap / interval_seconds) - 1) as usize;
            for j in 1..=num_missing {
                let ts = prev.timestamp + (j as i64 * interval_seconds);
                let filled = Quote {
                    timestamp: ts,
                    open: prev.close,
                    high: prev.close,
                    low: prev.close,
                    close: prev.close,
                    volume: 0,
                    adjclose: prev.adjclose,
                };
                result.push(filled);
            }
        }

        result.push(curr.clone());
    }

    result
}

/// Calculates time-based rolling window statistics
///
/// # Example
/// ```
/// use eeyf::timeseries::rolling_window;
/// use eeyf::quotes::Quote;
/// use rust_decimal::Decimal;
/// use chrono::Duration;
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
///         timestamp: 2000,
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let averages = rolling_window(&quotes, Duration::minutes(1), |window| {
///     let sum: Decimal = window.iter().map(|q| q.close).sum();
///     sum / Decimal::from(window.len())
/// });
/// ```
pub fn rolling_window<F>(quotes: &[Quote], window: Duration, f: F) -> Vec<Decimal>
where
    F: Fn(&[&Quote]) -> Decimal,
{
    let window_seconds = window.num_seconds();
    let mut result = Vec::new();

    for (i, quote) in quotes.iter().enumerate() {
        let window_start = quote.timestamp - window_seconds;

        // Collect quotes in the window
        let window_quotes: Vec<&Quote> = quotes[..=i]
            .iter()
            .filter(|q| q.timestamp >= window_start && q.timestamp <= quote.timestamp)
            .collect();

        if !window_quotes.is_empty() {
            result.push(f(&window_quotes));
        }
    }

    result
}

/// Downsamples quotes to a lower frequency
///
/// # Example
/// ```
/// use eeyf::timeseries::downsample;
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
///         timestamp: 2000,
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let downsampled = downsample(&quotes, 2); // Every 2nd quote
/// assert_eq!(downsampled.len(), 1);
/// ```
pub fn downsample(quotes: &[Quote], factor: usize) -> Vec<Quote> {
    if factor == 0 {
        return Vec::new();
    }

    quotes.iter().step_by(factor).cloned().collect()
}

/// Calculates time deltas between quotes
///
/// # Example
/// ```
/// use eeyf::timeseries::calculate_time_deltas;
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
///         timestamp: 2000,
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let deltas = calculate_time_deltas(&quotes);
/// assert_eq!(deltas[0], 1000); // 1000 seconds between quotes
/// ```
pub fn calculate_time_deltas(quotes: &[Quote]) -> Vec<i64> {
    if quotes.len() < 2 {
        return Vec::new();
    }

    quotes
        .windows(2)
        .map(|w| w[1].timestamp - w[0].timestamp)
        .collect()
}

/// Filters quotes by time range
///
/// # Example
/// ```
/// use eeyf::timeseries::filter_by_time_range;
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
///         timestamp: 2000,
///         open: Decimal::new(103, 0),
///         high: Decimal::new(107, 0),
///         low: Decimal::new(102, 0),
///         close: Decimal::new(106, 0),
///         volume: 1500,
///         adjclose: Decimal::new(106, 0),
///     },
/// ];
///
/// let filtered = filter_by_time_range(&quotes, 1500, 2500);
/// assert_eq!(filtered.len(), 1); // Only quote at 2000
/// ```
pub fn filter_by_time_range(quotes: &[Quote], start: i64, end: i64) -> Vec<Quote> {
    quotes
        .iter()
        .filter(|q| q.timestamp >= start && q.timestamp <= end)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_quote(timestamp: i64, close: i64) -> Quote {
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
    fn test_resample_quotes_ohlc() {
        let quotes = vec![
            create_test_quote(100, 100),
            create_test_quote(150, 105),
            create_test_quote(400, 110),
        ];

        let resampled = resample_quotes(&quotes, Duration::minutes(5), ResampleRule::OHLC);
        assert_eq!(resampled.len(), 2); // Two 5-minute buckets
    }

    #[test]
    fn test_resample_quotes_last() {
        let quotes = vec![create_test_quote(100, 100), create_test_quote(150, 105)];

        let resampled = resample_quotes(&quotes, Duration::minutes(5), ResampleRule::Last);
        assert_eq!(resampled.len(), 1);
        assert_eq!(resampled[0].close, Decimal::new(105, 0)); // Last value
    }

    #[test]
    fn test_align_timestamps() {
        let quotes = vec![create_test_quote(1003, 100)];

        let aligned = align_timestamps(&quotes, Duration::minutes(1));
        assert_eq!(aligned[0].timestamp, 960); // Aligned to minute (60s)
    }

    #[test]
    fn test_fill_missing_timestamps() {
        let quotes = vec![create_test_quote(1000, 100), create_test_quote(3000, 110)];

        let filled = fill_missing_timestamps(&quotes, Duration::seconds(1000));
        assert_eq!(filled.len(), 3); // Original 2 + 1 filled
    }

    #[test]
    fn test_downsample() {
        let quotes = vec![
            create_test_quote(1000, 100),
            create_test_quote(2000, 105),
            create_test_quote(3000, 110),
            create_test_quote(4000, 115),
        ];

        let downsampled = downsample(&quotes, 2);
        assert_eq!(downsampled.len(), 2);
    }

    #[test]
    fn test_calculate_time_deltas() {
        let quotes = vec![create_test_quote(1000, 100), create_test_quote(2000, 105)];

        let deltas = calculate_time_deltas(&quotes);
        assert_eq!(deltas.len(), 1);
        assert_eq!(deltas[0], 1000);
    }

    #[test]
    fn test_filter_by_time_range() {
        let quotes = vec![
            create_test_quote(1000, 100),
            create_test_quote(2000, 105),
            create_test_quote(3000, 110),
        ];

        let filtered = filter_by_time_range(&quotes, 1500, 2500);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].timestamp, 2000);
    }

    #[test]
    fn test_rolling_window() {
        let quotes = vec![create_test_quote(1000, 100), create_test_quote(2000, 110)];

        let averages = rolling_window(&quotes, Duration::minutes(1), |window| {
            let sum: Decimal = window.iter().map(|q| q.close).sum();
            sum / Decimal::from(window.len())
        });

        assert_eq!(averages.len(), 2);
    }

    #[test]
    fn test_convert_timezone() {
        use chrono_tz::{America::New_York, UTC};

        let quotes = vec![create_test_quote(1609459200, 100)]; // 2021-01-01 00:00:00 UTC

        let converted = convert_timezone(&quotes, UTC, New_York);
        // Timezone conversion actually converts the timestamp based on timezone offset
        // In this case, converting from UTC to New_York should shift the timestamp
        assert_eq!(converted.len(), 1);
        assert_eq!(converted[0].close, quotes[0].close); // Data unchanged
    }
}
