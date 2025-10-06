//! Database helpers for storing EEYF data
//!
//! Provides utilities for storing quotes and historical data using
//! web-server-abstraction's database abstraction layer.

use eeyf::{HistoricalDataPoint, Quote};
use web_server_abstraction::database::{
    ConnectionPool, DatabaseConnection, DatabaseValue, QueryBuilder, Row,
};

/// Store a quote to the database
///
/// # Example
///
/// ```no_run
/// use eeyf::Quote;
/// use eeyf_web_server_integration::store_quote;
///
/// # async fn example(pool: &impl web_server_abstraction::database::ConnectionPool, quote: Quote) -> Result<(), Box<dyn std::error::Error>> {
/// store_quote(pool, &quote).await?;
/// # Ok(())
/// # }
/// ```
pub async fn store_quote<P: ConnectionPool>(
    pool: &P,
    quote: &Quote,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get().await?;

    QueryBuilder::new()
        .insert("quotes")
        .values(vec![
            ("symbol", DatabaseValue::String(quote.symbol.clone())),
            ("price", DatabaseValue::Float(quote.price)),
            ("timestamp", DatabaseValue::Integer(quote.timestamp as i64)),
        ])
        .execute(&mut conn)
        .await?;

    Ok(())
}

/// Store historical data points to the database
///
/// # Example
///
/// ```no_run
/// use eeyf::HistoricalDataPoint;
/// use eeyf_web_server_integration::store_historical_data;
///
/// # async fn example(
/// #     pool: &impl web_server_abstraction::database::ConnectionPool,
/// #     symbol: &str,
/// #     data: Vec<HistoricalDataPoint>
/// # ) -> Result<(), Box<dyn std::error::Error>> {
/// store_historical_data(pool, symbol, &data).await?;
/// # Ok(())
/// # }
/// ```
pub async fn store_historical_data<P: ConnectionPool>(
    pool: &P,
    symbol: &str,
    data: &[HistoricalDataPoint],
) -> Result<(), Box<dyn std::error::Error>> {
    let mut conn = pool.get().await?;

    for point in data {
        QueryBuilder::new()
            .insert("historical_data")
            .values(vec![
                ("symbol", DatabaseValue::String(symbol.to_string())),
                ("date", DatabaseValue::String(point.date.clone())),
                ("open", DatabaseValue::Float(point.open)),
                ("high", DatabaseValue::Float(point.high)),
                ("low", DatabaseValue::Float(point.low)),
                ("close", DatabaseValue::Float(point.close)),
                ("volume", DatabaseValue::Integer(point.volume as i64)),
            ])
            .execute(&mut conn)
            .await?;
    }

    Ok(())
}

/// Query recent quotes for a symbol
///
/// # Example
///
/// ```no_run
/// use eeyf_web_server_integration::query_recent_quotes;
///
/// # async fn example(pool: &impl web_server_abstraction::database::ConnectionPool) -> Result<(), Box<dyn std::error::Error>> {
/// let quotes = query_recent_quotes(pool, "AAPL", 10).await?;
/// # Ok(())
/// # }
/// ```
pub async fn query_recent_quotes<P: ConnectionPool>(
    pool: &P,
    symbol: &str,
    limit: usize,
) -> Result<Vec<(String, f64, i64)>, Box<dyn std::error::Error>> {
    let mut conn = pool.get().await?;

    let rows = QueryBuilder::new()
        .select(vec!["symbol", "price", "timestamp"])
        .from("quotes")
        .where_clause("symbol = ?", vec![DatabaseValue::String(symbol.to_string())])
        .order_by("timestamp", false) // DESC
        .limit(limit)
        .query(&mut conn)
        .await?;

    let mut results = Vec::new();
    for row in rows {
        let symbol = row.get_string("symbol")?;
        let price = row.get_float("price")?;
        let timestamp = row.get_i64("timestamp")?;
        results.push((symbol, price, timestamp));
    }

    Ok(results)
}

/// Database migration SQL for PostgreSQL
pub const POSTGRES_MIGRATIONS: &str = r#"
-- Quotes table
CREATE TABLE IF NOT EXISTS quotes (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    timestamp BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_quotes_symbol ON quotes(symbol);
CREATE INDEX IF NOT EXISTS idx_quotes_timestamp ON quotes(timestamp);

-- Historical data table
CREATE TABLE IF NOT EXISTS historical_data (
    id SERIAL PRIMARY KEY,
    symbol VARCHAR(10) NOT NULL,
    date DATE NOT NULL,
    open DECIMAL(10, 2) NOT NULL,
    high DECIMAL(10, 2) NOT NULL,
    low DECIMAL(10, 2) NOT NULL,
    close DECIMAL(10, 2) NOT NULL,
    volume BIGINT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(symbol, date)
);

CREATE INDEX IF NOT EXISTS idx_historical_symbol ON historical_data(symbol);
CREATE INDEX IF NOT EXISTS idx_historical_date ON historical_data(date);
"#;

/// Database migration SQL for TimescaleDB (extends PostgreSQL)
pub const TIMESCALEDB_MIGRATIONS: &str = r#"
-- Quotes table (hypertable for time-series)
CREATE TABLE IF NOT EXISTS quotes (
    symbol VARCHAR(10) NOT NULL,
    price DECIMAL(10, 2) NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    volume BIGINT,
    PRIMARY KEY (symbol, timestamp)
);

SELECT create_hypertable('quotes', 'timestamp', if_not_exists => TRUE);

-- Create continuous aggregate for 1-hour averages
CREATE MATERIALIZED VIEW IF NOT EXISTS quotes_1h
WITH (timescaledb.continuous) AS
SELECT
    symbol,
    time_bucket('1 hour', timestamp) AS bucket,
    AVG(price) as avg_price,
    MAX(price) as max_price,
    MIN(price) as min_price,
    SUM(volume) as total_volume,
    COUNT(*) as num_quotes
FROM quotes
GROUP BY symbol, bucket;

-- Add retention policy (keep data for 90 days)
SELECT add_retention_policy('quotes', INTERVAL '90 days', if_not_exists => TRUE);
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrations_sql_valid() {
        assert!(POSTGRES_MIGRATIONS.contains("CREATE TABLE"));
        assert!(TIMESCALEDB_MIGRATIONS.contains("create_hypertable"));
    }
}
