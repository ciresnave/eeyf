/// Basic EEYF REST API using web-server-abstraction
/// 
/// This example demonstrates how to create a simple REST API serving
/// stock quotes using EEYF with web-server-abstraction.
/// 
/// The same code works with any supported framework (Axum, Actix-Web, Warp, Rocket, Salvo, Poem)!
/// 
/// Usage:
///   cargo run --example basic_api
/// 
/// Test:
///   curl http://localhost:8080/api/quote/AAPL
///   curl -X POST http://localhost:8080/api/quotes -H "Content-Type: application/json" -d '["AAPL","GOOGL"]'
///   curl http://localhost:8080/health

use std::sync::Arc;
use eeyf::{EEYFClient, Builder};
use web_server_abstraction::{WebServer, HttpMethod, Response, Request};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,eeyf=debug")
        .init();
    
    tracing::info!("Starting EEYF REST API server...");
    
    // Create EEYF client with production settings
    let client = EEYFClient::builder()
        .enable_caching(true)
        .cache_ttl_secs(60)
        .max_retries(3)
        .timeout_secs(30)
        .build();
    
    // Wrap in Arc for sharing across handlers
    let client = Arc::new(client);
    
    tracing::info!("EEYF client initialized with caching enabled");
    
    // Create web server (defaults to Axum, but works with any framework)
    let mut server = WebServer::new();
    
    // Single quote endpoint: GET /api/quote/:symbol
    server = server.route("/api/quote/:symbol", HttpMethod::GET, {
        let client = client.clone();
        move |req: Request| {
            let client = client.clone();
            async move {
                let symbol = req.param("symbol")
                    .ok_or("Missing symbol parameter")?;
                
                tracing::info!("Fetching quote for {}", symbol);
                
                match client.quote(&symbol).await {
                    Ok(quote) => {
                        tracing::info!("Successfully fetched quote for {}: ${}", symbol, quote.price);
                        let json = serde_json::to_string(&quote)
                            .map_err(|e| format!("JSON serialization error: {}", e))?;
                        
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .header("cache-control", "public, max-age=60")
                            .body(json))
                    }
                    Err(e) => {
                        tracing::error!("Error fetching quote for {}: {}", symbol, e);
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    // Batch quotes endpoint: POST /api/quotes
    server = server.route("/api/quotes", HttpMethod::POST, {
        let client = client.clone();
        move |req: Request| {
            let client = client.clone();
            async move {
                let body = req.body_string()
                    .ok_or("Missing request body")?;
                
                let symbols: Vec<String> = serde_json::from_str(&body)
                    .map_err(|e| format!("Invalid JSON: {}", e))?;
                
                if symbols.is_empty() {
                    return Ok(Response::error(400)
                        .header("content-type", "application/json")
                        .body(r#"{"error": "No symbols provided"}"#.to_string()));
                }
                
                if symbols.len() > 50 {
                    return Ok(Response::error(400)
                        .header("content-type", "application/json")
                        .body(r#"{"error": "Too many symbols (max 50)"}"#.to_string()));
                }
                
                tracing::info!("Fetching batch quotes for {} symbols", symbols.len());
                
                match client.batch_quotes(&symbols).await {
                    Ok(quotes) => {
                        tracing::info!("Successfully fetched {} quotes", quotes.len());
                        let json = serde_json::to_string(&quotes)
                            .map_err(|e| format!("JSON serialization error: {}", e))?;
                        
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .header("cache-control", "public, max-age=60")
                            .body(json))
                    }
                    Err(e) => {
                        tracing::error!("Error fetching batch quotes: {}", e);
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    // Health check endpoint: GET /health
    server = server.route("/health", HttpMethod::GET, |_req: Request| async {
        tracing::debug!("Health check requested");
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(r#"{"status": "healthy", "service": "eeyf-api", "version": "0.1.0"}"#.to_string()))
    });
    
    // Root endpoint with API documentation
    server = server.route("/", HttpMethod::GET, |_req: Request| async {
        let docs = r#"{
  "service": "EEYF REST API",
  "version": "0.1.0",
  "endpoints": {
    "GET /api/quote/:symbol": "Get quote for a single symbol",
    "POST /api/quotes": "Get quotes for multiple symbols (JSON array)",
    "GET /health": "Health check",
    "GET /": "This documentation"
  },
  "examples": {
    "single": "curl http://localhost:8080/api/quote/AAPL",
    "batch": "curl -X POST http://localhost:8080/api/quotes -H 'Content-Type: application/json' -d '[\"AAPL\",\"GOOGL\",\"MSFT\"]'",
    "health": "curl http://localhost:8080/health"
  }
}"#;
        
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(docs.to_string()))
    });
    
    // Bind to address
    let server = server.bind("127.0.0.1:8080").await?;
    
    tracing::info!("🚀 EEYF REST API server running on http://127.0.0.1:8080");
    tracing::info!("📖 API Documentation: http://127.0.0.1:8080/");
    tracing::info!("💡 Try: curl http://127.0.0.1:8080/api/quote/AAPL");
    
    // Run server (blocks until shutdown)
    server.run().await?;
    
    Ok(())
}
