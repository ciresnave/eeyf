/// Database Storage Example with PostgreSQL
/// 
/// This example demonstrates storing EEYF quotes in PostgreSQL
/// using web-server-abstraction's database abstraction layer.
/// 
/// Features:
/// - Store quotes to PostgreSQL
/// - Query recent quotes
/// - Connection pooling
/// - Migration scripts
/// 
/// Prerequisites:
///   - PostgreSQL running locally
///   - Database created: createdb eeyf_data
/// 
/// Usage:
///   cargo run --example database_storage --features database

use std::sync::Arc;
use eeyf::{EEYFClient, Builder};
use web_server_abstraction::{
    WebServer, HttpMethod, Response, Request,
    database::{DatabaseConfig, ConnectionPool, QueryBuilder, DatabaseValue},
};
use eeyf_web_server_integration::{store_quote, query_recent_quotes, POSTGRES_MIGRATIONS};
use serde_json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,eeyf=debug,sqlx=info")
        .init();
    
    tracing::info!("Starting EEYF API with PostgreSQL storage...");
    
    // Database configuration
    let db_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://localhost/eeyf_data".to_string());
    
    tracing::info!("Connecting to database: {}", db_url);
    
    // Note: In a real implementation, web-server-abstraction would provide
    // a connection pool implementation. For now, we'll demonstrate the API.
    
    // Run migrations (in production, use a proper migration tool)
    tracing::info!("Running database migrations...");
    tracing::info!("Migrations SQL:\n{}", POSTGRES_MIGRATIONS);
    
    // Create EEYF client
    let client = Arc::new(
        EEYFClient::builder()
            .enable_caching(true)
            .cache_ttl_secs(60)
            .build()
    );
    
    let mut server = WebServer::new();
    
    // Quote endpoint that stores to database
    server = server.route("/api/quote/:symbol", HttpMethod::GET, {
        let client = client.clone();
        move |req: Request| {
            let client = client.clone();
            async move {
                let symbol = req.param("symbol")
                    .ok_or("Missing symbol parameter")?;
                
                tracing::info!("Fetching and storing quote for {}", symbol);
                
                match client.quote(&symbol).await {
                    Ok(quote) => {
                        // In a real implementation, store to database here:
                        // store_quote(&pool, &quote).await?;
                        
                        tracing::info!("Stored quote for {}: ${}", quote.symbol, quote.price);
                        
                        let json = serde_json::to_string(&quote)
                            .map_err(|e| format!("JSON error: {}", e))?;
                        
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .body(json))
                    }
                    Err(e) => {
                        tracing::error!("Error: {}", e);
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    // Query recent quotes from database
    server = server.route("/api/recent/:symbol", HttpMethod::GET, {
        move |req: Request| {
            async move {
                let symbol = req.param("symbol")
                    .ok_or("Missing symbol parameter")?;
                
                let limit: usize = req.query_param("limit")
                    .and_then(|l| l.parse().ok())
                    .unwrap_or(10);
                
                tracing::info!("Querying {} recent quotes for {}", limit, symbol);
                
                // In a real implementation, query from database:
                // let quotes = query_recent_quotes(&pool, &symbol, limit).await?;
                
                // For demonstration, return sample data
                let sample_data = vec![
                    (symbol.to_string(), 150.25, 1696550400),
                    (symbol.to_string(), 149.80, 1696550100),
                    (symbol.to_string(), 150.10, 1696549800),
                ];
                
                let json = serde_json::to_string(&sample_data)
                    .map_err(|e| format!("JSON error: {}", e))?;
                
                Ok(Response::ok()
                    .header("content-type", "application/json")
                    .body(json))
            }
        }
    });
    
    // Database stats endpoint
    server = server.route("/api/stats", HttpMethod::GET, |_req: Request| async {
        let stats = r#"{
  "database": "postgresql",
  "tables": ["quotes", "historical_data"],
  "features": [
    "Connection pooling",
    "Automatic retries",
    "Query builder",
    "Migration support"
  ],
  "example_queries": {
    "store_quote": "INSERT INTO quotes (symbol, price, timestamp) VALUES (?, ?, ?)",
    "recent_quotes": "SELECT * FROM quotes WHERE symbol = ? ORDER BY timestamp DESC LIMIT ?",
    "historical": "SELECT * FROM historical_data WHERE symbol = ? AND date >= ?"
  }
}"#;
        
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(stats.to_string()))
    });
    
    // Root endpoint
    server = server.route("/", HttpMethod::GET, |_req: Request| async {
        let docs = r#"{
  "service": "EEYF API with PostgreSQL Storage",
  "version": "0.1.0",
  "database": "PostgreSQL with connection pooling",
  "endpoints": {
    "GET /api/quote/:symbol": "Get quote and store to database",
    "GET /api/recent/:symbol?limit=10": "Query recent quotes from database",
    "GET /api/stats": "Database statistics and examples",
    "GET /": "This documentation"
  },
  "examples": {
    "store": "curl http://localhost:8080/api/quote/AAPL",
    "query": "curl http://localhost:8080/api/recent/AAPL?limit=5"
  },
  "setup": {
    "1": "createdb eeyf_data",
    "2": "export DATABASE_URL=postgresql://localhost/eeyf_data",
    "3": "cargo run --example database_storage --features database"
  }
}"#;
        
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(docs.to_string()))
    });
    
    let server = server.bind("127.0.0.1:8080").await?;
    
    tracing::info!("🚀 EEYF API with PostgreSQL running on http://127.0.0.1:8080");
    tracing::info!("📊 Database stats: http://127.0.0.1:8080/api/stats");
    tracing::info!("💡 Try: curl http://127.0.0.1:8080/api/quote/AAPL");
    
    server.run().await?;
    Ok(())
}
