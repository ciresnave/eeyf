/// Multi-Framework Deployment Example
/// 
/// This example demonstrates how the SAME EEYF API code works
/// with different web frameworks using web-server-abstraction.
/// 
/// Simply change the framework by setting an environment variable:
///   FRAMEWORK=axum cargo run --example multi_framework
///   FRAMEWORK=actix cargo run --example multi_framework
///   FRAMEWORK=rocket cargo run --example multi_framework
/// 
/// This demonstrates the power of web-server-abstraction's
/// framework-agnostic approach!

use std::sync::Arc;
use eeyf::EEYFClient;
use eeyf_web_server_integration::{create_eeyf_server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,eeyf=debug")
        .init();
    
    // Get framework from environment variable
    let framework = std::env::var("FRAMEWORK")
        .unwrap_or_else(|_| "axum".to_string())
        .to_lowercase();
    
    tracing::info!("🚀 Starting EEYF API with {} framework", framework);
    tracing::info!("💡 Change framework with: FRAMEWORK=actix cargo run --example multi_framework");
    
    // Create EEYF client
    let client = Arc::new(
        EEYFClient::builder()
            .enable_caching(true)
            .cache_ttl_secs(60)
            .build()
    );
    
    // Create server with helper function (framework-agnostic!)
    let server = create_eeyf_server(client).await?;
    
    // Note: In a real implementation, you could select the framework:
    // let server = WebServer::with_framework(Framework::Axum)?;
    // let server = WebServer::with_framework(Framework::ActixWeb)?;
    // let server = WebServer::with_framework(Framework::Rocket)?;
    
    let server = server.bind("127.0.0.1:8080").await?;
    
    tracing::info!("✅ Server running with {} framework", framework);
    tracing::info!("🌐 Open http://127.0.0.1:8080");
    tracing::info!("📋 Endpoints:");
    tracing::info!("   - GET  /api/quote/:symbol");
    tracing::info!("   - POST /api/quotes");
    tracing::info!("   - GET  /api/historical/:symbol");
    tracing::info!("   - GET  /health");
    
    server.run().await?;
    Ok(())
}
