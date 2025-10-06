//! Helper functions for common EEYF + web-server-abstraction patterns
//! 
//! These helpers add standard EEYF routes to any WebServer instance.

use std::sync::Arc;
use eeyf::{EEYFClient, Quote};
use web_server_abstraction::{WebServer, HttpMethod, Request, Response};
use serde_json;

/// Add standard EEYF quote routes to a WebServer
/// 
/// This adds the following routes:
/// - GET /api/quote/:symbol - Get single quote
/// - POST /api/quotes - Get batch quotes (JSON array of symbols)
/// - GET /api/historical/:symbol - Get 30 days historical data
/// 
/// # Example
/// 
/// ```no_run
/// use eeyf::EEYFClient;
/// use web_server_abstraction::WebServer;
/// use eeyf_web_server_integration::add_quote_routes;
/// use std::sync::Arc;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Arc::new(EEYFClient::builder().build());
/// let mut server = WebServer::new();
/// add_quote_routes(&mut server, client)?;
/// # Ok(())
/// # }
/// ```
pub fn add_quote_routes(
    server: &mut WebServer,
    client: Arc<EEYFClient>,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Single quote endpoint
    *server = server.clone().route("/api/quote/:symbol", HttpMethod::GET, {
        let client = client.clone();
        move |req: Request| {
            let client = client.clone();
            async move {
                let symbol = req.param("symbol")
                    .ok_or("Missing symbol parameter")?;
                
                match client.quote(&symbol).await {
                    Ok(quote) => {
                        let json = serde_json::to_string(&quote)
                            .map_err(|e| format!("JSON error: {}", e))?;
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .header("cache-control", "public, max-age=60")
                            .body(json))
                    }
                    Err(e) => {
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    // Batch quotes endpoint
    *server = server.clone().route("/api/quotes", HttpMethod::POST, {
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
                
                match client.batch_quotes(&symbols).await {
                    Ok(quotes) => {
                        let json = serde_json::to_string(&quotes)
                            .map_err(|e| format!("JSON error: {}", e))?;
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .header("cache-control", "public, max-age=60")
                            .body(json))
                    }
                    Err(e) => {
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    // Historical data endpoint (30 days)
    *server = server.clone().route("/api/historical/:symbol", HttpMethod::GET, {
        let client = client.clone();
        move |req: Request| {
            let client = client.clone();
            async move {
                let symbol = req.param("symbol")
                    .ok_or("Missing symbol parameter")?;
                
                // Get days parameter (default 30)
                let days: u32 = req.query_param("days")
                    .and_then(|d| d.parse().ok())
                    .unwrap_or(30)
                    .min(365); // Max 1 year
                
                match client.historical(&symbol, days).await {
                    Ok(data) => {
                        let json = serde_json::to_string(&data)
                            .map_err(|e| format!("JSON error: {}", e))?;
                        Ok(Response::ok()
                            .header("content-type", "application/json")
                            .header("cache-control", "public, max-age=3600")
                            .body(json))
                    }
                    Err(e) => {
                        Ok(Response::error(500)
                            .header("content-type", "application/json")
                            .body(format!(r#"{{"error": "{}"}}"#, e)))
                    }
                }
            }
        }
    });
    
    Ok(())
}

/// Add monitoring and health check routes
/// 
/// This adds the following routes:
/// - GET /health - Basic health check
/// - GET /ready - Readiness probe
/// - GET /live - Liveness probe
/// 
/// # Example
/// 
/// ```no_run
/// use web_server_abstraction::WebServer;
/// use eeyf_web_server_integration::add_monitoring_routes;
/// 
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let mut server = WebServer::new();
/// add_monitoring_routes(&mut server)?;
/// # Ok(())
/// # }
/// ```
pub fn add_monitoring_routes(
    server: &mut WebServer,
) -> Result<(), Box<dyn std::error::Error>> {
    
    // Health check
    *server = server.clone().route("/health", HttpMethod::GET, |_req: Request| async {
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(r#"{"status":"healthy","service":"eeyf-api"}"#.to_string()))
    });
    
    // Readiness probe
    *server = server.clone().route("/ready", HttpMethod::GET, |_req: Request| async {
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(r#"{"status":"ready"}"#.to_string()))
    });
    
    // Liveness probe
    *server = server.clone().route("/live", HttpMethod::GET, |_req: Request| async {
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(r#"{"status":"alive"}"#.to_string()))
    });
    
    Ok(())
}

/// Create a complete EEYF API server with all standard routes
/// 
/// This is a convenience function that creates a WebServer with:
/// - Quote routes (single, batch, historical)
/// - Monitoring routes (health, ready, live)
/// - Root documentation endpoint
/// 
/// # Example
/// 
/// ```no_run
/// use eeyf::EEYFClient;
/// use eeyf_web_server_integration::create_eeyf_server;
/// use std::sync::Arc;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let client = Arc::new(EEYFClient::builder().build());
/// let server = create_eeyf_server(client).await?;
/// let server = server.bind("127.0.0.1:8080").await?;
/// server.run().await?;
/// # Ok(())
/// # }
/// ```
pub async fn create_eeyf_server(
    client: Arc<EEYFClient>,
) -> Result<WebServer, Box<dyn std::error::Error>> {
    let mut server = WebServer::new();
    
    // Add all routes
    add_quote_routes(&mut server, client)?;
    add_monitoring_routes(&mut server)?;
    
    // Add root documentation
    server = server.route("/", HttpMethod::GET, |_req: Request| async {
        let docs = r#"{
  "service": "EEYF REST API",
  "version": "0.1.0",
  "endpoints": {
    "GET /api/quote/:symbol": "Get quote for a single symbol",
    "POST /api/quotes": "Get quotes for multiple symbols (JSON array)",
    "GET /api/historical/:symbol?days=30": "Get historical data (default 30 days, max 365)",
    "GET /health": "Health check",
    "GET /ready": "Readiness probe",
    "GET /live": "Liveness probe",
    "GET /": "This documentation"
  },
  "examples": {
    "quote": "curl http://localhost:8080/api/quote/AAPL",
    "batch": "curl -X POST http://localhost:8080/api/quotes -H 'Content-Type: application/json' -d '[\"AAPL\",\"GOOGL\"]'",
    "historical": "curl http://localhost:8080/api/historical/AAPL?days=90"
  }
}"#;
        Ok(Response::ok()
            .header("content-type", "application/json")
            .body(docs.to_string()))
    });
    
    Ok(server)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_add_quote_routes_compiles() {
        // This test just ensures the API compiles
        // Real testing would use MockAdapter from web-server-abstraction
    }
}
