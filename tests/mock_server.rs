//! Mock server tests for EEYF
//! 
//! This module provides mock HTTP responses for testing without hitting the real Yahoo Finance API.
//! Uses wiremock to simulate various API responses and error conditions.

use eeyf::{YahooConnector, YahooError};
use wiremock::{
    matchers::{method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

/// Sample valid quote response from Yahoo Finance
const MOCK_QUOTE_RESPONSE: &str = r#"{
    "chart": {
        "result": [{
            "meta": {
                "currency": "USD",
                "symbol": "AAPL",
                "exchangeName": "NASDAQ",
                "instrumentType": "EQUITY",
                "firstTradeDate": 345459600,
                "regularMarketTime": 1696540800,
                "gmtoffset": -14400,
                "timezone": "EDT",
                "exchangeTimezoneName": "America/New_York",
                "regularMarketPrice": 175.50,
                "chartPreviousClose": 174.00,
                "previousClose": 174.00,
                "scale": 3,
                "priceHint": 2,
                "currentTradingPeriod": {
                    "pre": {
                        "timezone": "EDT",
                        "start": 1696492800,
                        "end": 1696513800,
                        "gmtoffset": -14400
                    },
                    "regular": {
                        "timezone": "EDT",
                        "start": 1696513800,
                        "end": 1696537200,
                        "gmtoffset": -14400
                    },
                    "post": {
                        "timezone": "EDT",
                        "start": 1696537200,
                        "end": 1696551600,
                        "gmtoffset": -14400
                    }
                },
                "tradingPeriods": [[{
                    "timezone": "EDT",
                    "start": 1696513800,
                    "end": 1696537200,
                    "gmtoffset": -14400
                }]],
                "dataGranularity": "1d",
                "range": "1mo",
                "validRanges": ["1d", "5d", "1mo", "3mo", "6mo", "1y", "2y", "5y", "10y", "ytd", "max"]
            },
            "timestamp": [1696540800, 1696454400, 1696368000],
            "indicators": {
                "quote": [{
                    "volume": [50000000, 48000000, 52000000],
                    "high": [176.00, 175.00, 177.00],
                    "close": [175.50, 174.00, 176.50],
                    "low": [174.00, 172.50, 175.00],
                    "open": [174.50, 173.00, 176.00]
                }],
                "adjclose": [{
                    "adjclose": [175.50, 174.00, 176.50]
                }]
            }
        }],
        "error": null
    }
}"#;

/// Mock error response for rate limiting
const MOCK_RATE_LIMIT_RESPONSE: &str = r#"{
    "chart": {
        "result": null,
        "error": {
            "code": "Too Many Requests",
            "description": "Rate limit exceeded. Please try again later."
        }
    }
}"#;

/// Mock error response for invalid symbol
const MOCK_INVALID_SYMBOL_RESPONSE: &str = r#"{
    "chart": {
        "result": null,
        "error": {
            "code": "Not Found",
            "description": "No data found for symbol"
        }
    }
}"#;

#[tokio::test]
async fn test_mock_successful_quote_fetch() {
    // Start mock server
    let mock_server = MockServer::start().await;

    // Configure mock response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MOCK_QUOTE_RESPONSE))
        .mount(&mock_server)
        .await;

    // Note: This test demonstrates mock setup
    // In a real implementation, we'd need to inject the mock server URL into YahooConnector
    println!("Mock server running at: {}", mock_server.uri());
    println!("✅ Mock server test infrastructure ready");
}

#[tokio::test]
async fn test_mock_rate_limit_error() {
    let mock_server = MockServer::start().await;

    // Configure mock rate limit response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(429).set_body_string(MOCK_RATE_LIMIT_RESPONSE))
        .mount(&mock_server)
        .await;

    println!("Mock server configured for rate limit testing at: {}", mock_server.uri());
    println!("✅ Rate limit mock ready");
}

#[tokio::test]
async fn test_mock_invalid_symbol() {
    let mock_server = MockServer::start().await;

    // Configure mock invalid symbol response
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/INVALID"))
        .respond_with(ResponseTemplate::new(404).set_body_string(MOCK_INVALID_SYMBOL_RESPONSE))
        .mount(&mock_server)
        .await;

    println!("Mock server configured for invalid symbol testing at: {}", mock_server.uri());
    println!("✅ Invalid symbol mock ready");
}

#[tokio::test]
async fn test_mock_network_timeout() {
    let mock_server = MockServer::start().await;

    // Configure mock to simulate timeout with delay
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(MOCK_QUOTE_RESPONSE)
                .set_delay(std::time::Duration::from_secs(10))
        )
        .mount(&mock_server)
        .await;

    println!("Mock server configured for timeout testing at: {}", mock_server.uri());
    println!("✅ Timeout mock ready (10s delay)");
}

#[tokio::test]
async fn test_mock_server_error() {
    let mock_server = MockServer::start().await;

    // Configure mock 500 error
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    println!("Mock server configured for 500 error testing at: {}", mock_server.uri());
    println!("✅ Server error mock ready");
}

#[tokio::test]
async fn test_mock_malformed_json() {
    let mock_server = MockServer::start().await;

    // Configure mock with malformed JSON
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{invalid json"))
        .mount(&mock_server)
        .await;

    println!("Mock server configured for malformed JSON testing at: {}", mock_server.uri());
    println!("✅ Malformed JSON mock ready");
}

#[tokio::test]
async fn test_mock_query_parameters() {
    let mock_server = MockServer::start().await;

    // Configure mock with query parameter matching
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .and(query_param("interval", "1d"))
        .and(query_param("range", "1mo"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MOCK_QUOTE_RESPONSE))
        .mount(&mock_server)
        .await;

    println!("Mock server configured for query parameter testing at: {}", mock_server.uri());
    println!("✅ Query parameter mock ready");
}

#[tokio::test]
async fn test_mock_concurrent_requests() {
    let mock_server = MockServer::start().await;

    // Configure mock for concurrent testing
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MOCK_QUOTE_RESPONSE))
        .expect(5..) // Expect at least 5 requests
        .mount(&mock_server)
        .await;

    println!("Mock server configured for concurrent request testing at: {}", mock_server.uri());
    println!("✅ Concurrent request mock ready (expects 5+ requests)");
}

/// Helper function to create a mock server with common setup
pub async fn create_mock_quote_server() -> MockServer {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .and(path("/v8/finance/chart/AAPL"))
        .respond_with(ResponseTemplate::new(200).set_body_string(MOCK_QUOTE_RESPONSE))
        .mount(&mock_server)
        .await;
    
    mock_server
}

/// Helper function to create a mock server that simulates rate limiting
pub async fn create_mock_rate_limit_server() -> MockServer {
    let mock_server = MockServer::start().await;
    
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(429).set_body_string(MOCK_RATE_LIMIT_RESPONSE))
        .mount(&mock_server)
        .await;
    
    mock_server
}

#[cfg(test)]
mod integration {
    use super::*;

    #[tokio::test]
    async fn test_mock_server_lifecycle() {
        // Test that mock server can be created and destroyed multiple times
        for i in 0..3 {
            let mock_server = create_mock_quote_server().await;
            println!("  Iteration {}: Mock server created at {}", i + 1, mock_server.uri());
            // Mock server automatically cleaned up when dropped
        }
        println!("✅ Mock server lifecycle test complete");
    }

    #[tokio::test]
    async fn test_different_status_codes() {
        let mock_server = MockServer::start().await;
        
        // Test various HTTP status codes
        let test_cases = vec![
            (200, "OK"),
            (400, "Bad Request"),
            (401, "Unauthorized"),
            (403, "Forbidden"),
            (404, "Not Found"),
            (429, "Too Many Requests"),
            (500, "Internal Server Error"),
            (502, "Bad Gateway"),
            (503, "Service Unavailable"),
        ];
        
        for (status, description) in test_cases {
            Mock::given(method("GET"))
                .and(path(format!("/status/{}", status)))
                .respond_with(ResponseTemplate::new(status))
                .up_to_n_times(1)
                .mount(&mock_server)
                .await;
            
            println!("  ✅ Configured mock for {} {}", status, description);
        }
        
        println!("✅ All status code mocks configured");
    }
}
