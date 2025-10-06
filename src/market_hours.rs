//! Market hours checking module for Yahoo Finance API
//!
//! This module provides client-side market hours detection and validation.
//! It includes static schedules for major exchanges and supports holiday calendars.
//!
//! # Features
//!
//! - Check if a market is currently open
//! - Get next open/close times for exchanges
//! - Support for major global exchanges (NYSE, NASDAQ, TSX, LSE, etc.)
//! - Configurable holiday calendars
//! - Timezone-aware calculations
//!
//! # Examples
//!
//! ```rust
//! use eeyf::market_hours::{MarketHoursChecker, Exchange};
//!
//! let checker = MarketHoursChecker::new();
//!
//! // Check if NYSE is open
//! if checker.is_market_open(Exchange::NYSE) {
//!     println!("NYSE is open for trading");
//! }
//!
//! // Get next market open time
//! if let Some(next_open) = checker.next_open_time(Exchange::NASDAQ) {
//!     println!("NASDAQ opens at: {}", next_open);
//! }
//! ```

#[allow(unused_imports)] // TimeZone and Timelike traits needed for method calls
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Supported stock exchanges with their trading schedules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Exchange {
    /// New York Stock Exchange (NYSE)
    NYSE,
    /// NASDAQ Stock Market
    NASDAQ,
    /// Toronto Stock Exchange
    TSX,
    /// London Stock Exchange
    LSE,
    /// Euronext (Paris, Amsterdam, Brussels)
    EURONEXT,
    /// Deutsche Börse (Frankfurt)
    XETRA,
    /// Tokyo Stock Exchange
    TSE,
    /// Hong Kong Stock Exchange
    HKEX,
    /// Shanghai Stock Exchange
    SSE,
    /// Australian Securities Exchange
    ASX,
}

impl Exchange {
    /// Get the timezone for this exchange
    pub fn timezone(&self) -> Tz {
        match self {
            Exchange::NYSE | Exchange::NASDAQ => chrono_tz::America::New_York,
            Exchange::TSX => chrono_tz::America::Toronto,
            Exchange::LSE => chrono_tz::Europe::London,
            Exchange::EURONEXT => chrono_tz::Europe::Paris,
            Exchange::XETRA => chrono_tz::Europe::Berlin,
            Exchange::TSE => chrono_tz::Asia::Tokyo,
            Exchange::HKEX => chrono_tz::Asia::Hong_Kong,
            Exchange::SSE => chrono_tz::Asia::Shanghai,
            Exchange::ASX => chrono_tz::Australia::Sydney,
        }
    }

    /// Get the regular trading hours for this exchange
    pub fn trading_hours(&self) -> (NaiveTime, NaiveTime) {
        match self {
            Exchange::NYSE | Exchange::NASDAQ => (
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            ),
            Exchange::TSX => (
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            ),
            Exchange::LSE => (
                NaiveTime::from_hms_opt(8, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 30, 0).unwrap(),
            ),
            Exchange::EURONEXT => (
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 30, 0).unwrap(),
            ),
            Exchange::XETRA => (
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(17, 30, 0).unwrap(),
            ),
            Exchange::TSE => (
                NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
            ),
            Exchange::HKEX => (
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            ),
            Exchange::SSE => (
                NaiveTime::from_hms_opt(9, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(15, 0, 0).unwrap(),
            ),
            Exchange::ASX => (
                NaiveTime::from_hms_opt(10, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(16, 0, 0).unwrap(),
            ),
        }
    }

    /// Get the lunch break hours for exchanges that have them (if any)
    pub fn lunch_break(&self) -> Option<(NaiveTime, NaiveTime)> {
        match self {
            Exchange::TSE => Some((
                NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(12, 30, 0).unwrap(),
            )),
            Exchange::HKEX => Some((
                NaiveTime::from_hms_opt(12, 0, 0).unwrap(),
                NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            )),
            Exchange::SSE => Some((
                NaiveTime::from_hms_opt(11, 30, 0).unwrap(),
                NaiveTime::from_hms_opt(13, 0, 0).unwrap(),
            )),
            _ => None,
        }
    }

    /// Check if this exchange trades on a given weekday
    pub fn is_trading_day(&self, weekday: Weekday) -> bool {
        // Most exchanges trade Monday-Friday
        !matches!(weekday, Weekday::Sat | Weekday::Sun)
    }

    /// Get the name of this exchange
    pub fn name(&self) -> &'static str {
        match self {
            Exchange::NYSE => "New York Stock Exchange",
            Exchange::NASDAQ => "NASDAQ",
            Exchange::TSX => "Toronto Stock Exchange",
            Exchange::LSE => "London Stock Exchange",
            Exchange::EURONEXT => "Euronext",
            Exchange::XETRA => "Deutsche Börse (XETRA)",
            Exchange::TSE => "Tokyo Stock Exchange",
            Exchange::HKEX => "Hong Kong Stock Exchange",
            Exchange::SSE => "Shanghai Stock Exchange",
            Exchange::ASX => "Australian Securities Exchange",
        }
    }
}

/// A specific date marked as a market holiday
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Holiday {
    /// Year of the holiday
    pub year: i32,
    /// Month of the holiday (1-12)
    pub month: u32,
    /// Day of the month (1-31)
    pub day: u32,
    /// Name of the holiday
    pub name: String,
}

impl Holiday {
    /// Create a new holiday
    pub fn new(year: i32, month: u32, day: u32, name: impl Into<String>) -> Self {
        Self {
            year,
            month,
            day,
            name: name.into(),
        }
    }

    /// Check if this holiday matches a given date
    pub fn matches(&self, date: &DateTime<Tz>) -> bool {
        date.year() == self.year && date.month() == self.month && date.day() == self.day
    }
}

/// Configuration for market hours checking
#[derive(Debug, Clone)]
pub struct MarketHoursConfig {
    /// Custom holidays per exchange
    holidays: std::collections::HashMap<Exchange, HashSet<Holiday>>,
    /// Whether to log warnings for closed-hours requests
    warn_on_closed: bool,
}

impl Default for MarketHoursConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketHoursConfig {
    /// Create a new configuration with default US market holidays
    pub fn new() -> Self {
        let mut config = Self {
            holidays: std::collections::HashMap::new(),
            warn_on_closed: true,
        };
        config.add_default_us_holidays(2024);
        config.add_default_us_holidays(2025);
        config
    }

    /// Enable or disable warnings for closed-hours requests
    pub fn with_warn_on_closed(mut self, warn: bool) -> Self {
        self.warn_on_closed = warn;
        self
    }

    /// Add a holiday for a specific exchange
    pub fn add_holiday(&mut self, exchange: Exchange, holiday: Holiday) {
        self.holidays.entry(exchange).or_default().insert(holiday);
    }

    /// Add default US market holidays for a given year
    pub fn add_default_us_holidays(&mut self, year: i32) {
        let holidays = vec![
            Holiday::new(year, 1, 1, "New Year's Day"),
            Holiday::new(year, 1, 15, "Martin Luther King Jr. Day"), // 3rd Monday
            Holiday::new(year, 2, 19, "Presidents Day"),              // 3rd Monday
            Holiday::new(year, 4, 7, "Good Friday"),
            Holiday::new(year, 5, 27, "Memorial Day"), // Last Monday
            Holiday::new(year, 6, 19, "Juneteenth"),
            Holiday::new(year, 7, 4, "Independence Day"),
            Holiday::new(year, 9, 2, "Labor Day"), // 1st Monday
            Holiday::new(year, 11, 28, "Thanksgiving"), // 4th Thursday
            Holiday::new(year, 12, 25, "Christmas Day"),
        ];

        for holiday in holidays {
            self.add_holiday(Exchange::NYSE, holiday.clone());
            self.add_holiday(Exchange::NASDAQ, holiday);
        }
    }

    /// Check if a date is a holiday for the given exchange
    pub fn is_holiday(&self, exchange: Exchange, date: &DateTime<Tz>) -> bool {
        self.holidays
            .get(&exchange)
            .map(|holidays| holidays.iter().any(|h| h.matches(date)))
            .unwrap_or(false)
    }

    /// Get holidays for a specific exchange
    pub fn get_holidays(&self, exchange: Exchange) -> Vec<Holiday> {
        self.holidays
            .get(&exchange)
            .map(|holidays| holidays.iter().cloned().collect())
            .unwrap_or_default()
    }
}

/// Result of a market hours check
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarketStatus {
    /// Market is currently open for trading
    Open,
    /// Market is closed (outside trading hours)
    Closed,
    /// Market is closed for a holiday
    Holiday(String),
    /// Market is closed for the weekend
    Weekend,
    /// Market is closed for lunch break
    LunchBreak,
}

impl MarketStatus {
    /// Check if the market is open
    pub fn is_open(&self) -> bool {
        matches!(self, MarketStatus::Open)
    }

    /// Check if the market is closed
    pub fn is_closed(&self) -> bool {
        !self.is_open()
    }
}

/// Market hours checker with timezone support
pub struct MarketHoursChecker {
    config: MarketHoursConfig,
}

impl Default for MarketHoursChecker {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketHoursChecker {
    /// Create a new market hours checker with default configuration
    pub fn new() -> Self {
        Self {
            config: MarketHoursConfig::new(),
        }
    }

    /// Create a new market hours checker with custom configuration
    pub fn with_config(config: MarketHoursConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MarketHoursConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut MarketHoursConfig {
        &mut self.config
    }

    /// Check the current market status for an exchange
    pub fn market_status(&self, exchange: Exchange) -> MarketStatus {
        let now = Utc::now().with_timezone(&exchange.timezone());
        self.market_status_at(exchange, &now)
    }

    /// Check the market status at a specific time
    pub fn market_status_at(&self, exchange: Exchange, time: &DateTime<Tz>) -> MarketStatus {
        let weekday = time.weekday();

        // Check if it's a weekend
        if !exchange.is_trading_day(weekday) {
            return MarketStatus::Weekend;
        }

        // Check if it's a holiday
        if self.config.is_holiday(exchange, time) {
            let holidays = self.config.get_holidays(exchange);
            let holiday_name = holidays
                .iter()
                .find(|h| h.matches(time))
                .map(|h| h.name.clone())
                .unwrap_or_else(|| "Holiday".to_string());
            return MarketStatus::Holiday(holiday_name);
        }

        let current_time = time.time();
        let (open, close) = exchange.trading_hours();

        // Check if we're in a lunch break
        if let Some((lunch_start, lunch_end)) = exchange.lunch_break() {
            if current_time >= lunch_start && current_time < lunch_end {
                return MarketStatus::LunchBreak;
            }
        }

        // Check regular trading hours
        if current_time >= open && current_time < close {
            // Need to check if we're NOT in lunch break
            if let Some((lunch_start, lunch_end)) = exchange.lunch_break() {
                if current_time >= lunch_start && current_time < lunch_end {
                    return MarketStatus::LunchBreak;
                }
            }
            MarketStatus::Open
        } else {
            MarketStatus::Closed
        }
    }

    /// Check if a market is currently open
    pub fn is_market_open(&self, exchange: Exchange) -> bool {
        self.market_status(exchange).is_open()
    }

    /// Get the next time the market will open
    pub fn next_open_time(&self, exchange: Exchange) -> Option<DateTime<Tz>> {
        let now = Utc::now().with_timezone(&exchange.timezone());
        let mut check_time = now;

        // Look ahead up to 14 days (2 weeks)
        for _ in 0..14 {
            let status = self.market_status_at(exchange, &check_time);

            if status.is_open() {
                return Some(check_time);
            }

            // Move to next potential open time
            match status {
                MarketStatus::Weekend | MarketStatus::Holiday(_) => {
                    // Skip to next day at market open time
                    check_time = (check_time + Duration::days(1))
                        .with_time(exchange.trading_hours().0)
                        .unwrap();
                }
                MarketStatus::LunchBreak => {
                    // Skip to end of lunch break
                    if let Some((_, lunch_end)) = exchange.lunch_break() {
                        check_time = check_time.with_time(lunch_end).unwrap();
                    }
                }
                MarketStatus::Closed => {
                    // Check if we're before market open today
                    let (open_time, _) = exchange.trading_hours();
                    if check_time.time() < open_time {
                        check_time = check_time.with_time(open_time).unwrap();
                    } else {
                        // Move to next day at open time
                        check_time = (check_time + Duration::days(1))
                            .with_time(open_time)
                            .unwrap();
                    }
                }
                MarketStatus::Open => return Some(check_time),
            }
        }

        None
    }

    /// Get the next time the market will close
    pub fn next_close_time(&self, exchange: Exchange) -> Option<DateTime<Tz>> {
        let now = Utc::now().with_timezone(&exchange.timezone());

        // If market is currently open, return today's close time
        if self.is_market_open(exchange) {
            let (_, close_time) = exchange.trading_hours();
            return Some(now.with_time(close_time).unwrap());
        }

        // Otherwise, find the next open time and return that day's close time
        self.next_open_time(exchange).map(|open_time: DateTime<Tz>| {
            let (_, close_time) = exchange.trading_hours();
            open_time.with_time(close_time).unwrap()
        })
    }

    /// Get time until market opens (if closed) or closes (if open)
    pub fn time_until_change(&self, exchange: Exchange) -> Option<Duration> {
        let now = Utc::now().with_timezone(&exchange.timezone());

        if self.is_market_open(exchange) {
            self.next_close_time(exchange)
                .map(|close_time| close_time - now)
        } else {
            self.next_open_time(exchange)
                .map(|open_time| open_time - now)
        }
    }

    /// Check multiple exchanges at once
    pub fn check_markets(&self, exchanges: &[Exchange]) -> Vec<(Exchange, MarketStatus)> {
        exchanges
            .iter()
            .map(|&exchange| (exchange, self.market_status(exchange)))
            .collect()
    }

    /// Log a warning if requesting data during closed hours
    pub fn warn_if_closed(&self, exchange: Exchange) {
        if self.config.warn_on_closed && !self.is_market_open(exchange) {
            let status = self.market_status(exchange);
            match status {
                MarketStatus::Closed => {
                    #[cfg(feature = "observability")]
                    tracing::warn!(
                        exchange = ?exchange,
                        "Requesting data while market is closed (outside trading hours)"
                    );
                    #[cfg(not(feature = "observability"))]
                    log::warn!(
                        "Requesting data for {:?} while market is closed (outside trading hours)",
                        exchange
                    );
                }
                MarketStatus::Weekend => {
                    #[cfg(feature = "observability")]
                    tracing::warn!(
                        exchange = ?exchange,
                        "Requesting data on weekend - market is closed"
                    );
                    #[cfg(not(feature = "observability"))]
                    log::warn!("Requesting data for {:?} on weekend - market is closed", exchange);
                }
                MarketStatus::Holiday(ref name) => {
                    #[cfg(feature = "observability")]
                    tracing::warn!(
                        exchange = ?exchange,
                        holiday = %name,
                        "Requesting data on holiday - market is closed"
                    );
                    #[cfg(not(feature = "observability"))]
                    log::warn!(
                        "Requesting data for {:?} on holiday ({}) - market is closed",
                        exchange,
                        name
                    );
                }
                MarketStatus::LunchBreak => {
                    #[cfg(feature = "observability")]
                    tracing::warn!(
                        exchange = ?exchange,
                        "Requesting data during lunch break - market is closed"
                    );
                    #[cfg(not(feature = "observability"))]
                    log::warn!(
                        "Requesting data for {:?} during lunch break - market is closed",
                        exchange
                    );
                }
                MarketStatus::Open => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exchange_timezones() {
        assert_eq!(Exchange::NYSE.timezone(), chrono_tz::America::New_York);
        assert_eq!(Exchange::LSE.timezone(), chrono_tz::Europe::London);
        assert_eq!(Exchange::TSE.timezone(), chrono_tz::Asia::Tokyo);
    }

    #[test]
    fn test_trading_hours() {
        let (open, close) = Exchange::NYSE.trading_hours();
        assert_eq!(open.hour(), 9);
        assert_eq!(open.minute(), 30);
        assert_eq!(close.hour(), 16);
        assert_eq!(close.minute(), 0);
    }

    #[test]
    fn test_lunch_breaks() {
        assert!(Exchange::NYSE.lunch_break().is_none());
        assert!(Exchange::TSE.lunch_break().is_some());
        assert!(Exchange::HKEX.lunch_break().is_some());
    }

    #[test]
    fn test_is_trading_day() {
        assert!(Exchange::NYSE.is_trading_day(Weekday::Mon));
        assert!(Exchange::NYSE.is_trading_day(Weekday::Fri));
        assert!(!Exchange::NYSE.is_trading_day(Weekday::Sat));
        assert!(!Exchange::NYSE.is_trading_day(Weekday::Sun));
    }

    #[test]
    fn test_holiday_creation() {
        let holiday = Holiday::new(2024, 12, 25, "Christmas");
        assert_eq!(holiday.year, 2024);
        assert_eq!(holiday.month, 12);
        assert_eq!(holiday.day, 25);
        assert_eq!(holiday.name, "Christmas");
    }

    #[test]
    fn test_holiday_matching() {
        let holiday = Holiday::new(2024, 12, 25, "Christmas");
        let tz = chrono_tz::America::New_York;
        let date = tz.with_ymd_and_hms(2024, 12, 25, 10, 0, 0).unwrap();
        assert!(holiday.matches(&date));

        let other_date = tz.with_ymd_and_hms(2024, 12, 26, 10, 0, 0).unwrap();
        assert!(!holiday.matches(&other_date));
    }

    #[test]
    fn test_config_add_holiday() {
        let mut config = MarketHoursConfig::new();
        let holiday = Holiday::new(2024, 12, 25, "Christmas");
        config.add_holiday(Exchange::NYSE, holiday.clone());

        let tz = chrono_tz::America::New_York;
        let date = tz.with_ymd_and_hms(2024, 12, 25, 10, 0, 0).unwrap();
        assert!(config.is_holiday(Exchange::NYSE, &date));
    }

    #[test]
    fn test_market_status_weekend() {
        let checker = MarketHoursChecker::new();
        let tz = chrono_tz::America::New_York;
        let saturday = tz.with_ymd_and_hms(2024, 12, 7, 10, 0, 0).unwrap(); // Saturday
        assert_eq!(
            checker.market_status_at(Exchange::NYSE, &saturday),
            MarketStatus::Weekend
        );
    }

    #[test]
    fn test_market_status_during_hours() {
        let checker = MarketHoursChecker::new();
        let tz = chrono_tz::America::New_York;
        let weekday = tz.with_ymd_and_hms(2024, 12, 9, 10, 0, 0).unwrap(); // Monday 10:00 AM
        assert_eq!(
            checker.market_status_at(Exchange::NYSE, &weekday),
            MarketStatus::Open
        );
    }

    #[test]
    fn test_market_status_outside_hours() {
        let checker = MarketHoursChecker::new();
        let tz = chrono_tz::America::New_York;
        let weekday = tz.with_ymd_and_hms(2024, 12, 9, 8, 0, 0).unwrap(); // Monday 8:00 AM (before open)
        assert_eq!(
            checker.market_status_at(Exchange::NYSE, &weekday),
            MarketStatus::Closed
        );
    }

    #[test]
    fn test_market_status_is_open() {
        let status = MarketStatus::Open;
        assert!(status.is_open());
        assert!(!status.is_closed());

        let status = MarketStatus::Closed;
        assert!(!status.is_open());
        assert!(status.is_closed());
    }

    #[test]
    fn test_check_multiple_markets() {
        let checker = MarketHoursChecker::new();
        let exchanges = vec![Exchange::NYSE, Exchange::NASDAQ, Exchange::LSE];
        let results = checker.check_markets(&exchanges);
        assert_eq!(results.len(), 3);
    }
}
