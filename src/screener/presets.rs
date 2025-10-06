//! Predefined screener query presets
//!
//! This module provides pre-built query templates for common screening scenarios.

use super::query::{Field, Query};

/// Get a query for day gainers (stocks up >3% with decent volume)
pub fn day_gainers() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gt(Field::PercentChange, 3.0),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 15_000),
    ])
}

/// Get a query for day losers (stocks down >3% with decent volume)
pub fn day_losers() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::lt(Field::PercentChange, -3.0),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 15_000),
    ])
}

/// Get a query for most active stocks (highest volume)
pub fn most_actives() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 500_000),
    ])
}

/// Get a query for growth technology stocks
pub fn growth_technology_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::eq(Field::Sector, "Technology"),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
        Query::gt(Field::EPSGrowthTTM, 25.0),
    ])
}

/// Get a query for aggressive small caps (<$2B market cap with high growth)
pub fn aggressive_small_caps() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::lte(Field::IntradayMarketCap, 2_000_000_000.0),
        Query::gte(Field::IntradayMarketCap, 300_000_000.0),
        Query::gt(Field::EPSGrowthTTM, 25.0),
        Query::gte(Field::IntradayPrice, 5.0),
    ])
}

/// Get a query for undervalued growth stocks (low P/E, high growth)
pub fn undervalued_growth_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
        Query::lte(Field::PERatioTTM, 20.0),
        Query::gt(Field::EPSGrowthTTM, 20.0),
        Query::lte(Field::PEGRatio5Y, 1.0),
    ])
}

/// Get a query for undervalued large caps (>$10B market cap, low P/E)
pub fn undervalued_large_caps() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
        Query::lte(Field::PERatioTTM, 15.0),
        Query::gte(Field::ReturnOnEquity, 0.15),
    ])
}

/// Get a query for high yield dividend stocks (>3% yield)
pub fn high_yield_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::DividendYield, 0.03),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
        Query::gte(Field::IntradayPrice, 5.0),
    ])
}

/// Get a query for stocks near 52-week high (within 5%)
pub fn near_52_week_high() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::PercentFromFiftyTwoWeekHigh, -5.0),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 100_000),
    ])
}

/// Get a query for stocks near 52-week low (within 5%)
pub fn near_52_week_low() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::lte(Field::PercentFromFiftyTwoWeekLow, 5.0),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 100_000),
    ])
}

/// Get a query for solid large cap growth funds (>$10B, consistent growth)
pub fn solid_large_cap_growth() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
        Query::gte(Field::EPSGrowthTTM, 10.0),
        Query::gte(Field::ReturnOnEquity, 0.15),
        Query::gte(Field::ProfitMargin, 0.10),
    ])
}

/// Get a query for solid mid cap growth (>$2B, <$10B market cap)
pub fn solid_mid_cap_growth() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::between(Field::IntradayMarketCap, 2_000_000_000.0, 10_000_000_000.0),
        Query::gte(Field::EPSGrowthTTM, 15.0),
        Query::gte(Field::ReturnOnEquity, 0.15),
    ])
}

/// Get a query for portfolio anchors (stable, profitable, dividend-paying)
pub fn portfolio_anchors() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayMarketCap, 10_000_000_000.0),
        Query::gte(Field::DividendYield, 0.02),
        Query::gte(Field::ReturnOnEquity, 0.12),
        Query::gte(Field::ProfitMargin, 0.10),
        Query::lte(Field::Beta, 1.2),
    ])
}

/// Get a query for value stocks (low P/E, low P/B)
pub fn value_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gte(Field::IntradayMarketCap, 2_000_000_000.0),
        Query::lte(Field::PERatioTTM, 15.0),
        Query::lte(Field::PriceToBook, 3.0),
        Query::gte(Field::IntradayPrice, 5.0),
    ])
}

/// Get a query for momentum stocks (strong recent performance)
pub fn momentum_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gt(Field::PercentChange, 2.0),
        Query::gte(Field::PercentFromFiftyTwoWeekHigh, -10.0),
        Query::gt(Field::DayVolume, 500_000),
        Query::gte(Field::IntradayPrice, 10.0),
    ])
}

/// Get a query for breakout stocks (volume spike + price gain)
pub fn breakout_stocks() -> Query {
    Query::and(vec![
        Query::eq(Field::Region, "us"),
        Query::gt(Field::PercentChange, 5.0),
        Query::gte(Field::IntradayPrice, 5.0),
        Query::gt(Field::DayVolume, 1_000_000),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_day_gainers_query() {
        let query = day_gainers();
        let json = query.to_json();
        assert_eq!(json["operator"], "and");
    }

    #[test]
    fn test_all_preset_queries_compile() {
        // Just make sure all presets can be created without panicking
        let _queries = vec![
            day_gainers(),
            day_losers(),
            most_actives(),
            growth_technology_stocks(),
            aggressive_small_caps(),
            undervalued_growth_stocks(),
            undervalued_large_caps(),
            high_yield_stocks(),
            near_52_week_high(),
            near_52_week_low(),
            solid_large_cap_growth(),
            solid_mid_cap_growth(),
            portfolio_anchors(),
            value_stocks(),
            momentum_stocks(),
            breakout_stocks(),
        ];
    }
}
