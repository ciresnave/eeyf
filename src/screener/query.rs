//! Query DSL for building custom screener queries
//!
//! This module provides a type-safe DSL for building complex screener queries
//! that are translated to Yahoo Finance's JSON query format.
//!
//! # Examples
//!
//! ```
//! use eeyf::screener::query::{Query, Field};
//!
//! // Find large tech stocks with strong gains
//! let query = Query::and(vec![
//!     Query::eq(Field::Region, "us"),
//!     Query::eq(Field::Sector, "Technology"),
//!     Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
//!     Query::gt(Field::PercentChange, 3.0),
//! ]);
//! ```

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

/// Fields that can be used in screener queries
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Field {
    // Price fields
    /// Current intraday price
    IntradayPrice,
    /// Percentage change
    PercentChange,
    /// Price change amount
    PriceChange,
    
    // Volume fields
    /// Current day volume
    DayVolume,
    /// Average daily volume (3 months)
    AvgDailyVolume3Month,
    /// Average daily volume (10 days)
    AvgDailyVolume10Day,
    
    // Market cap
    /// Current intraday market cap
    IntradayMarketCap,
    
    // Valuation ratios
    /// Trailing P/E ratio (last 12 months)
    PERatioTTM,
    /// Forward P/E ratio
    PERatioForward,
    /// PEG ratio (5 year expected)
    PEGRatio5Y,
    /// Price to book ratio
    PriceToBook,
    /// Price to sales ratio
    PriceToSales,
    
    // Growth metrics
    /// EPS growth (last 12 months)
    EPSGrowthTTM,
    /// EPS growth (quarterly YoY)
    EPSGrowthQuarterlyYoY,
    /// Revenue growth (last 12 months)
    RevenueGrowthTTM,
    
    // Profitability
    /// Profit margin
    ProfitMargin,
    /// Operating margin
    OperatingMargin,
    /// Return on equity
    ReturnOnEquity,
    /// Return on assets
    ReturnOnAssets,
    
    // Dividends
    /// Dividend yield
    DividendYield,
    /// Trailing annual dividend rate
    TrailingAnnualDividendRate,
    /// Trailing annual dividend yield
    TrailingAnnualDividendYield,
    
    // 52-week metrics
    /// 52-week high price
    FiftyTwoWeekHigh,
    /// 52-week low price
    FiftyTwoWeekLow,
    /// Percentage from 52-week high
    PercentFromFiftyTwoWeekHigh,
    /// Percentage from 52-week low
    PercentFromFiftyTwoWeekLow,
    
    // Beta
    /// Beta (volatility measure)
    Beta,
    
    // Categorical fields
    /// Geographic region
    Region,
    /// Market sector
    Sector,
    /// Industry
    Industry,
    /// Exchange
    Exchange,
    /// Quote type (EQUITY, ETF, etc.)
    QuoteType,
}

impl Field {
    /// Get the Yahoo Finance field name for this field
    pub fn yahoo_name(&self) -> &'static str {
        match self {
            // Price
            Field::IntradayPrice => "intradayprice",
            Field::PercentChange => "percentchange",
            Field::PriceChange => "pricechange",
            
            // Volume
            Field::DayVolume => "dayvolume",
            Field::AvgDailyVolume3Month => "avgdailyvol3m",
            Field::AvgDailyVolume10Day => "avgdailyvol10d",
            
            // Market cap
            Field::IntradayMarketCap => "intradaymarketcap",
            
            // Valuation
            Field::PERatioTTM => "peratio.lasttwelvemonths",
            Field::PERatioForward => "peratio.forward",
            Field::PEGRatio5Y => "pegratio_5y",
            Field::PriceToBook => "pricetobook",
            Field::PriceToSales => "pricetosales",
            
            // Growth
            Field::EPSGrowthTTM => "epsgrowth.lasttwelvemonths",
            Field::EPSGrowthQuarterlyYoY => "epsgrowth.quarterly.yoy",
            Field::RevenueGrowthTTM => "revenuegrowth.lasttwelvemonths",
            
            // Profitability
            Field::ProfitMargin => "profitmargin",
            Field::OperatingMargin => "operatingmargin",
            Field::ReturnOnEquity => "returnonequity",
            Field::ReturnOnAssets => "returnonassets",
            
            // Dividends
            Field::DividendYield => "dividendyield",
            Field::TrailingAnnualDividendRate => "trailingannualdividendrate",
            Field::TrailingAnnualDividendYield => "trailingannualdividendyield",
            
            // 52-week
            Field::FiftyTwoWeekHigh => "fiftytwoweek.high",
            Field::FiftyTwoWeekLow => "fiftytwoweek.low",
            Field::PercentFromFiftyTwoWeekHigh => "percentfromfiftytwoweek.high",
            Field::PercentFromFiftyTwoWeekLow => "percentfromfiftytwoweek.low",
            
            // Beta
            Field::Beta => "beta",
            
            // Categorical
            Field::Region => "region",
            Field::Sector => "sector",
            Field::Industry => "industry",
            Field::Exchange => "exchange",
            Field::QuoteType => "quotetype",
        }
    }

    /// Check if this field is numeric (vs categorical)
    pub fn is_numeric(&self) -> bool {
        !matches!(
            self,
            Field::Region | Field::Sector | Field::Industry | Field::Exchange | Field::QuoteType
        )
    }
}

/// Operators for building queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operator {
    /// Logical AND
    And,
    /// Logical OR
    Or,
    /// Greater than
    GreaterThan,
    /// Less than
    LessThan,
    /// Greater than or equal
    GreaterThanOrEqual,
    /// Less than or equal
    LessThanOrEqual,
    /// Equal to
    Equal,
    /// Between two values
    Between,
    /// In a list of values
    In,
}

impl Operator {
    /// Get the Yahoo Finance operator name
    pub fn yahoo_name(&self) -> &'static str {
        match self {
            Operator::And => "and",
            Operator::Or => "or",
            Operator::GreaterThan => "gt",
            Operator::LessThan => "lt",
            Operator::GreaterThanOrEqual => "gte",
            Operator::LessThanOrEqual => "lte",
            Operator::Equal => "eq",
            Operator::Between => "btwn",
            Operator::In => "in",
        }
    }
}

/// Value type for query operands
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum QueryValue {
    /// String value
    String(String),
    /// Numeric value
    Number(f64),
    /// Integer value
    Integer(i64),
    /// Array of values
    Array(Vec<QueryValue>),
}

impl From<&str> for QueryValue {
    fn from(s: &str) -> Self {
        QueryValue::String(s.to_string())
    }
}

impl From<String> for QueryValue {
    fn from(s: String) -> Self {
        QueryValue::String(s)
    }
}

impl From<f64> for QueryValue {
    fn from(n: f64) -> Self {
        QueryValue::Number(n)
    }
}

impl From<i64> for QueryValue {
    fn from(n: i64) -> Self {
        QueryValue::Integer(n)
    }
}

impl From<Vec<QueryValue>> for QueryValue {
    fn from(arr: Vec<QueryValue>) -> Self {
        QueryValue::Array(arr)
    }
}

/// A screener query that can be built using the DSL
#[derive(Debug, Clone)]
pub struct Query {
    operator: Operator,
    operands: Vec<QueryOperand>,
}

/// Operand for a query (can be a field or nested query)
#[derive(Debug, Clone)]
enum QueryOperand {
    Field(Field),
    Value(QueryValue),
    NestedQuery(Box<Query>),
}

impl Query {
    /// Create a logical AND query combining multiple queries
    ///
    /// # Example
    ///
    /// ```
    /// use eeyf::screener::query::{Query, Field};
    ///
    /// let query = Query::and(vec![
    ///     Query::eq(Field::Region, "us"),
    ///     Query::gt(Field::PercentChange, 5.0),
    /// ]);
    /// ```
    pub fn and(queries: Vec<Query>) -> Self {
        Self {
            operator: Operator::And,
            operands: queries.into_iter().map(|q| QueryOperand::NestedQuery(Box::new(q))).collect(),
        }
    }

    /// Create a logical OR query combining multiple queries
    pub fn or(queries: Vec<Query>) -> Self {
        Self {
            operator: Operator::Or,
            operands: queries.into_iter().map(|q| QueryOperand::NestedQuery(Box::new(q))).collect(),
        }
    }

    /// Create an equality query (field == value)
    ///
    /// # Example
    ///
    /// ```
    /// use eeyf::screener::query::{Query, Field};
    ///
    /// let query = Query::eq(Field::Region, "us");
    /// ```
    pub fn eq<V: Into<QueryValue>>(field: Field, value: V) -> Self {
        Self {
            operator: Operator::Equal,
            operands: vec![QueryOperand::Field(field), QueryOperand::Value(value.into())],
        }
    }

    /// Create a greater-than query (field > value)
    pub fn gt<V: Into<QueryValue>>(field: Field, value: V) -> Self {
        Self {
            operator: Operator::GreaterThan,
            operands: vec![QueryOperand::Field(field), QueryOperand::Value(value.into())],
        }
    }

    /// Create a less-than query (field < value)
    pub fn lt<V: Into<QueryValue>>(field: Field, value: V) -> Self {
        Self {
            operator: Operator::LessThan,
            operands: vec![QueryOperand::Field(field), QueryOperand::Value(value.into())],
        }
    }

    /// Create a greater-than-or-equal query (field >= value)
    pub fn gte<V: Into<QueryValue>>(field: Field, value: V) -> Self {
        Self {
            operator: Operator::GreaterThanOrEqual,
            operands: vec![QueryOperand::Field(field), QueryOperand::Value(value.into())],
        }
    }

    /// Create a less-than-or-equal query (field <= value)
    pub fn lte<V: Into<QueryValue>>(field: Field, value: V) -> Self {
        Self {
            operator: Operator::LessThanOrEqual,
            operands: vec![QueryOperand::Field(field), QueryOperand::Value(value.into())],
        }
    }

    /// Create a between query (min <= field <= max)
    ///
    /// # Example
    ///
    /// ```
    /// use eeyf::screener::query::{Query, Field};
    ///
    /// // Find stocks priced between $10 and $50
    /// let query = Query::between(Field::IntradayPrice, 10.0, 50.0);
    /// ```
    pub fn between<V1: Into<QueryValue>, V2: Into<QueryValue>>(
        field: Field,
        min: V1,
        max: V2,
    ) -> Self {
        Self {
            operator: Operator::Between,
            operands: vec![
                QueryOperand::Field(field),
                QueryOperand::Value(QueryValue::Array(vec![min.into(), max.into()])),
            ],
        }
    }

    /// Create an IN query (field IN [values...])
    ///
    /// # Example
    ///
    /// ```
    /// use eeyf::screener::query::{Query, Field, QueryValue};
    ///
    /// // Find stocks in specific sectors
    /// let query = Query::in_list(
    ///     Field::Sector,
    ///     vec![
    ///         QueryValue::from("Technology"),
    ///         QueryValue::from("Healthcare"),
    ///     ],
    /// );
    /// ```
    pub fn in_list(field: Field, values: Vec<QueryValue>) -> Self {
        Self {
            operator: Operator::In,
            operands: vec![
                QueryOperand::Field(field),
                QueryOperand::Value(QueryValue::Array(values)),
            ],
        }
    }

    /// Convert this query to Yahoo Finance's JSON format
    pub(crate) fn to_json(&self) -> JsonValue {
        let operator_name = self.operator.yahoo_name();
        
        let operands: Vec<JsonValue> = self
            .operands
            .iter()
            .map(|operand| match operand {
                QueryOperand::Field(field) => JsonValue::String(field.yahoo_name().to_string()),
                QueryOperand::Value(value) => serde_json::to_value(value).unwrap(),
                QueryOperand::NestedQuery(query) => query.to_json(),
            })
            .collect();

        serde_json::json!({
            "operator": operator_name,
            "operands": operands,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_yahoo_names() {
        assert_eq!(Field::IntradayPrice.yahoo_name(), "intradayprice");
        assert_eq!(Field::PercentChange.yahoo_name(), "percentchange");
        assert_eq!(Field::IntradayMarketCap.yahoo_name(), "intradaymarketcap");
    }

    #[test]
    fn test_field_is_numeric() {
        assert!(Field::IntradayPrice.is_numeric());
        assert!(Field::PercentChange.is_numeric());
        assert!(!Field::Region.is_numeric());
        assert!(!Field::Sector.is_numeric());
    }

    #[test]
    fn test_operator_yahoo_names() {
        assert_eq!(Operator::And.yahoo_name(), "and");
        assert_eq!(Operator::GreaterThan.yahoo_name(), "gt");
        assert_eq!(Operator::Equal.yahoo_name(), "eq");
    }

    #[test]
    fn test_query_value_conversions() {
        let v1: QueryValue = "test".into();
        assert!(matches!(v1, QueryValue::String(_)));

        let v2: QueryValue = 42.5.into();
        assert!(matches!(v2, QueryValue::Number(_)));

        let v3: QueryValue = 100i64.into();
        assert!(matches!(v3, QueryValue::Integer(_)));
    }

    #[test]
    fn test_simple_query_to_json() {
        let query = Query::eq(Field::Region, "us");
        let json = query.to_json();

        assert_eq!(json["operator"], "eq");
        assert_eq!(json["operands"][0], "region");
        assert_eq!(json["operands"][1], "us");
    }

    #[test]
    fn test_comparison_query_to_json() {
        let query = Query::gt(Field::PercentChange, 5.0);
        let json = query.to_json();

        assert_eq!(json["operator"], "gt");
        assert_eq!(json["operands"][0], "percentchange");
        assert_eq!(json["operands"][1], 5.0);
    }

    #[test]
    fn test_between_query_to_json() {
        let query = Query::between(Field::IntradayPrice, 10.0, 50.0);
        let json = query.to_json();

        assert_eq!(json["operator"], "btwn");
        assert_eq!(json["operands"][0], "intradayprice");
        assert!(json["operands"][1].is_array());
    }

    #[test]
    fn test_and_query_to_json() {
        let query = Query::and(vec![
            Query::eq(Field::Region, "us"),
            Query::gt(Field::PercentChange, 3.0),
        ]);

        let json = query.to_json();
        assert_eq!(json["operator"], "and");
        assert!(json["operands"].is_array());
        assert_eq!(json["operands"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn test_complex_nested_query() {
        let query = Query::and(vec![
            Query::eq(Field::Region, "us"),
            Query::or(vec![
                Query::eq(Field::Sector, "Technology"),
                Query::eq(Field::Sector, "Healthcare"),
            ]),
            Query::gt(Field::IntradayMarketCap, 1_000_000_000.0),
        ]);

        let json = query.to_json();
        assert_eq!(json["operator"], "and");
        assert_eq!(json["operands"].as_array().unwrap().len(), 3);
        
        // Check nested OR
        let nested_or = &json["operands"][1];
        assert_eq!(nested_or["operator"], "or");
    }
}
