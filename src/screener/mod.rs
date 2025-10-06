//! Stock Screener Module
//!
//! This module provides functionality to screen stocks using Yahoo Finance's screener API.
//! You can use predefined screeners (like "day_gainers", "most_actives") or build custom
//! queries with a powerful DSL.
//!
//! # Examples
//!
//! ## Using Predefined Screeners
//!
//! ```no_run
//! use eeyf::screener::{Screener, PredefinedScreener};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let screener = Screener::new();
//!
//! // Get today's top gainers
//! let gainers = screener.predefined(PredefinedScreener::DayGainers)
//!     .limit(25)
//!     .execute()
//!     .await?;
//!
//! for result in gainers.quotes {
//!     println!("{}: +{:.2}%", result.symbol, result.percent_change);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Building Custom Queries
//!
//! ```no_run
//! use eeyf::screener::{Screener, Query, Operator, Field};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let screener = Screener::new();
//!
//! // Find US tech stocks with:
//! // - Market cap > $10B
//! // - Price change > 3%
//! // - Volume > 1M
//! let query = Query::and(vec![
//!     Query::eq(Field::Region, "us"),
//!     Query::eq(Field::Sector, "Technology"),
//!     Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
//!     Query::gt(Field::PercentChange, 3.0),
//!     Query::gt(Field::DayVolume, 1_000_000.0),
//! ]);
//!
//! let results = screener.query(query)
//!     .limit(50)
//!     .sort_by(Field::PercentChange, false) // descending
//!     .execute()
//!     .await?;
//! # Ok(())
//! # }
//! ```

pub mod presets;
pub mod query;

use crate::YahooError;
use query::{Field, Query};
use serde::{Deserialize, Serialize};

/// Predefined screener types provided by Yahoo Finance
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PredefinedScreener {
    /// Stocks with the highest percentage gain today
    DayGainers,
    /// Stocks with the highest percentage loss today
    DayLosers,
    /// Stocks with the highest trading volume today
    MostActives,
    /// Technology stocks with strong growth
    GrowthTechnologyStocks,
    /// Small cap stocks (< $2B market cap) with aggressive growth
    AggressiveSmallCaps,
    /// Stocks with the highest short interest
    MostShortedStocks,
    /// Growth stocks that appear undervalued
    UndervaluedGrowthStocks,
    /// Large cap stocks (> $10B market cap)
    UndervaluedLargeCaps,
    /// Stocks with strong recent momentum
    ConservativeForeignFunds,
    /// High dividend yield stocks
    HighYieldStocks,
    /// Stocks trading near 52-week high
    TradingNear52WeekHigh,
    /// Stocks trading near 52-week low
    TradingNear52WeekLow,
    /// Top mutual funds by performance
    TopMutualFunds,
    /// Stocks with strong buy ratings
    PortfolioAnchors,
    /// Solid large cap growth stocks
    SolidLargeCapGrowthFunds,
    /// Solid mid cap growth stocks
    SolidMidCapGrowthFunds,
}

impl PredefinedScreener {
    /// Get the Yahoo Finance screener ID for this predefined screener
    pub fn id(&self) -> &'static str {
        match self {
            PredefinedScreener::DayGainers => "day_gainers",
            PredefinedScreener::DayLosers => "day_losers",
            PredefinedScreener::MostActives => "most_actives",
            PredefinedScreener::GrowthTechnologyStocks => "growth_technology_stocks",
            PredefinedScreener::AggressiveSmallCaps => "aggressive_small_caps",
            PredefinedScreener::MostShortedStocks => "most_shorted_stocks",
            PredefinedScreener::UndervaluedGrowthStocks => "undervalued_growth_stocks",
            PredefinedScreener::UndervaluedLargeCaps => "undervalued_large_caps",
            PredefinedScreener::ConservativeForeignFunds => "conservative_foreign_funds",
            PredefinedScreener::HighYieldStocks => "high_yield_stock",
            PredefinedScreener::TradingNear52WeekHigh => "near_52_week_high",
            PredefinedScreener::TradingNear52WeekLow => "near_52_week_low",
            PredefinedScreener::TopMutualFunds => "top_mutual_funds",
            PredefinedScreener::PortfolioAnchors => "portfolio_anchors",
            PredefinedScreener::SolidLargeCapGrowthFunds => "solid_large_growth_funds",
            PredefinedScreener::SolidMidCapGrowthFunds => "solid_midcap_growth_funds",
        }
    }

    /// Get a human-readable description of this screener
    pub fn description(&self) -> &'static str {
        match self {
            PredefinedScreener::DayGainers => "Stocks with the highest percentage gain today",
            PredefinedScreener::DayLosers => "Stocks with the highest percentage loss today",
            PredefinedScreener::MostActives => "Stocks with the highest trading volume",
            PredefinedScreener::GrowthTechnologyStocks => "Technology stocks with strong growth",
            PredefinedScreener::AggressiveSmallCaps => "Small cap stocks with aggressive growth",
            PredefinedScreener::MostShortedStocks => "Stocks with highest short interest",
            PredefinedScreener::UndervaluedGrowthStocks => "Growth stocks that appear undervalued",
            PredefinedScreener::UndervaluedLargeCaps => "Undervalued large cap stocks",
            PredefinedScreener::ConservativeForeignFunds => "Conservative foreign funds",
            PredefinedScreener::HighYieldStocks => "Stocks with high dividend yield",
            PredefinedScreener::TradingNear52WeekHigh => "Stocks near 52-week high",
            PredefinedScreener::TradingNear52WeekLow => "Stocks near 52-week low",
            PredefinedScreener::TopMutualFunds => "Top performing mutual funds",
            PredefinedScreener::PortfolioAnchors => "Solid stocks for portfolio foundation",
            PredefinedScreener::SolidLargeCapGrowthFunds => "Solid large cap growth funds",
            PredefinedScreener::SolidMidCapGrowthFunds => "Solid mid cap growth funds",
        }
    }

    /// Get all available predefined screeners
    pub fn all() -> Vec<PredefinedScreener> {
        vec![
            PredefinedScreener::DayGainers,
            PredefinedScreener::DayLosers,
            PredefinedScreener::MostActives,
            PredefinedScreener::GrowthTechnologyStocks,
            PredefinedScreener::AggressiveSmallCaps,
            PredefinedScreener::MostShortedStocks,
            PredefinedScreener::UndervaluedGrowthStocks,
            PredefinedScreener::UndervaluedLargeCaps,
            PredefinedScreener::ConservativeForeignFunds,
            PredefinedScreener::HighYieldStocks,
            PredefinedScreener::TradingNear52WeekHigh,
            PredefinedScreener::TradingNear52WeekLow,
            PredefinedScreener::TopMutualFunds,
            PredefinedScreener::PortfolioAnchors,
            PredefinedScreener::SolidLargeCapGrowthFunds,
            PredefinedScreener::SolidMidCapGrowthFunds,
        ]
    }
}

/// A single stock result from a screener query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenerQuote {
    /// Stock symbol
    pub symbol: String,
    /// Company name
    #[serde(rename = "shortName")]
    pub short_name: Option<String>,
    /// Long company name
    #[serde(rename = "longName")]
    pub long_name: Option<String>,
    /// Current price
    #[serde(rename = "regularMarketPrice")]
    pub regular_market_price: Option<f64>,
    /// Price change
    #[serde(rename = "regularMarketChange")]
    pub regular_market_change: Option<f64>,
    /// Percent change
    #[serde(rename = "regularMarketChangePercent")]
    pub regular_market_change_percent: Option<f64>,
    /// Trading volume
    #[serde(rename = "regularMarketVolume")]
    pub regular_market_volume: Option<i64>,
    /// Average volume (3 months)
    #[serde(rename = "averageDailyVolume3Month")]
    pub average_daily_volume_3_month: Option<i64>,
    /// Market cap
    #[serde(rename = "marketCap")]
    pub market_cap: Option<i64>,
    /// P/E ratio
    #[serde(rename = "trailingPE")]
    pub trailing_pe: Option<f64>,
    /// Forward P/E ratio
    #[serde(rename = "forwardPE")]
    pub forward_pe: Option<f64>,
    /// Dividend yield
    #[serde(rename = "dividendYield")]
    pub dividend_yield: Option<f64>,
    /// EPS
    #[serde(rename = "epsTrailingTwelveMonths")]
    pub eps_trailing_twelve_months: Option<f64>,
    /// 52-week high
    #[serde(rename = "fiftyTwoWeekHigh")]
    pub fifty_two_week_high: Option<f64>,
    /// 52-week low
    #[serde(rename = "fiftyTwoWeekLow")]
    pub fifty_two_week_low: Option<f64>,
    /// Exchange
    pub exchange: Option<String>,
    /// Quote type (EQUITY, ETF, etc.)
    #[serde(rename = "quoteType")]
    pub quote_type: Option<String>,
}

/// Results from a screener query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenerResults {
    /// Total number of results matching the query
    pub total: usize,
    /// Number of results in this response
    pub count: usize,
    /// List of matching quotes
    pub quotes: Vec<ScreenerQuote>,
}

/// Screener request builder
pub struct ScreenerRequest {
    screener_id: Option<String>,
    query: Option<Query>,
    limit: usize,
    offset: usize,
    sort_field: Option<Field>,
    sort_ascending: bool,
}

impl ScreenerRequest {
    /// Create a new screener request with a predefined screener
    pub fn predefined(screener: PredefinedScreener) -> Self {
        Self {
            screener_id: Some(screener.id().to_string()),
            query: None,
            limit: 25,
            offset: 0,
            sort_field: None,
            sort_ascending: false,
        }
    }

    /// Create a new screener request with a custom query
    pub fn custom(query: Query) -> Self {
        Self {
            screener_id: None,
            query: Some(query),
            limit: 25,
            offset: 0,
            sort_field: None,
            sort_ascending: false,
        }
    }

    /// Set the maximum number of results to return (1-250)
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit.clamp(1, 250);
        self
    }

    /// Set the offset for pagination
    pub fn offset(mut self, offset: usize) -> Self {
        self.offset = offset;
        self
    }

    /// Sort results by a specific field
    pub fn sort_by(mut self, field: Field, ascending: bool) -> Self {
        self.sort_field = Some(field);
        self.sort_ascending = ascending;
        self
    }

    /// Build the request payload for Yahoo's API
    pub(crate) fn build_payload(&self) -> Result<serde_json::Value, YahooError> {
        let mut payload = serde_json::json!({
            "size": self.limit,
            "offset": self.offset,
        });

        // Add screener ID or custom query
        if let Some(ref screener_id) = self.screener_id {
            payload["scrIds"] = serde_json::json!(screener_id);
        } else if let Some(ref query) = self.query {
            payload["query"] = query.to_json();
        } else {
            return Err(YahooError::DataInconsistency);
        }

        // Add sorting if specified
        if let Some(ref field) = self.sort_field {
            payload["sortField"] = serde_json::json!(field.yahoo_name());
            payload["sortType"] = if self.sort_ascending {
                serde_json::json!("ASC")
            } else {
                serde_json::json!("DESC")
            };
        }

        Ok(payload)
    }
}

/// Main screener client
pub struct Screener {
    client: reqwest::Client,
    base_url: String,
}

impl Screener {
    /// Create a new screener client
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: "https://query1.finance.yahoo.com/v1/finance/screener".to_string(),
        }
    }

    /// Create a screener request with a predefined screener
    pub fn predefined(&self, screener: PredefinedScreener) -> ScreenerRequest {
        ScreenerRequest::predefined(screener)
    }

    /// Create a screener request with a custom query
    pub fn query(&self, query: Query) -> ScreenerRequest {
        ScreenerRequest::custom(query)
    }

    /// Execute a screener request
    pub async fn execute(&self, request: ScreenerRequest) -> Result<ScreenerResults, YahooError> {
        let payload = request.build_payload()?;

        let response = self
            .client
            .post(&self.base_url)
            .header("User-Agent", "Mozilla/5.0")
            .json(&payload)
            .send()
            .await
            .map_err(|e| YahooError::ConnectionFailed(format!("Failed to send screener request: {}", e)))?;

        if !response.status().is_success() {
            return Err(YahooError::FetchFailed(format!("Screener API returned status: {}", response.status())));
        }

        let body: serde_json::Value = response.json().await.map_err(|e| {
            YahooError::DeserializeFailed(format!("Failed to parse screener response: {}", e))
        })?;

        // Parse the response
        let finance = body
            .get("finance")
            .ok_or_else(|| YahooError::DataInconsistency)?;

        let result = finance
            .get("result")
            .and_then(|r| r.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| YahooError::DataInconsistency)?;

        let total = result
            .get("total")
            .and_then(|t| t.as_u64())
            .unwrap_or(0) as usize;

        let quotes = result
            .get("quotes")
            .and_then(|q| q.as_array())
            .ok_or_else(|| YahooError::DataInconsistency)?;

        let parsed_quotes: Result<Vec<ScreenerQuote>, _> = quotes
            .iter()
            .map(|q| serde_json::from_value(q.clone()))
            .collect();

        let parsed_quotes = parsed_quotes.map_err(|e| YahooError::DeserializeFailed(format!("Failed to parse quote: {}", e)))?;

        Ok(ScreenerResults {
            total,
            count: parsed_quotes.len(),
            quotes: parsed_quotes,
        })
    }
}

impl Default for Screener {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predefined_screener_ids() {
        assert_eq!(PredefinedScreener::DayGainers.id(), "day_gainers");
        assert_eq!(PredefinedScreener::MostActives.id(), "most_actives");
        assert_eq!(PredefinedScreener::DayLosers.id(), "day_losers");
    }

    #[test]
    fn test_predefined_screener_all() {
        let all = PredefinedScreener::all();
        assert_eq!(all.len(), 16);
        assert!(all.contains(&PredefinedScreener::DayGainers));
        assert!(all.contains(&PredefinedScreener::MostActives));
    }

    #[test]
    fn test_screener_request_builder() {
        let request = ScreenerRequest::predefined(PredefinedScreener::DayGainers)
            .limit(50)
            .offset(10);

        assert_eq!(request.limit, 50);
        assert_eq!(request.offset, 10);
        assert!(request.screener_id.is_some());
        assert!(request.query.is_none());
    }

    #[test]
    fn test_screener_request_limit_clamping() {
        let request = ScreenerRequest::predefined(PredefinedScreener::DayGainers).limit(1000);
        assert_eq!(request.limit, 250); // Should be clamped to max

        let request = ScreenerRequest::predefined(PredefinedScreener::DayGainers).limit(0);
        assert_eq!(request.limit, 1); // Should be clamped to min
    }

    #[test]
    fn test_screener_request_build_payload_predefined() {
        let request = ScreenerRequest::predefined(PredefinedScreener::DayGainers)
            .limit(25)
            .offset(0);

        let payload = request.build_payload().unwrap();
        assert_eq!(payload["size"], 25);
        assert_eq!(payload["offset"], 0);
        assert_eq!(payload["scrIds"], "day_gainers");
    }
}
