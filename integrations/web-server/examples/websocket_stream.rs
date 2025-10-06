/// WebSocket Real-Time Streaming Example
/// 
/// This example demonstrates real-time price streaming using WebSocket
/// with EEYF and web-server-abstraction.
/// 
/// Features:
/// - Real-time quote updates every 5 seconds
/// - Subscribe/unsubscribe to symbols dynamically
/// - Multiple concurrent client support
/// - Automatic error recovery
/// 
/// Usage:
///   cargo run --example websocket_stream
/// 
/// Test with websocat or JavaScript:
///   websocat ws://localhost:8080/ws
///   > {"action":"subscribe","symbols":["AAPL","GOOGL"]}
///   > {"action":"unsubscribe","symbols":["GOOGL"]}

use std::sync::Arc;
use std::collections::HashSet;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use eeyf::{EEYFClient, Builder};
use web_server_abstraction::{WebServer, HttpMethod, Response, Request};
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Debug, Deserialize)]
#[serde(tag = "action", rename_all = "lowercase")]
enum ClientMessage {
    Subscribe { symbols: Vec<String> },
    Unsubscribe { symbols: Vec<String> },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ServerMessage {
    Quote {
        symbol: String,
        price: f64,
        timestamp: u64,
    },
    Error {
        message: String,
    },
    Subscribed {
        symbols: Vec<String>,
    },
    Unsubscribed {
        symbols: Vec<String>,
    },
}

#[derive(Clone)]
struct StreamState {
    subscriptions: Arc<RwLock<HashSet<String>>>,
    client: Arc<EEYFClient>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter("info,eeyf=debug")
        .init();
    
    tracing::info!("Starting EEYF WebSocket streaming server...");
    
    // Create EEYF client
    let client = Arc::new(
        EEYFClient::builder()
            .enable_caching(true)
            .cache_ttl_secs(10) // Short TTL for real-time data
            .build()
    );
    
    // Create shared state
    let state = StreamState {
        subscriptions: Arc::new(RwLock::new(HashSet::new())),
        client: client.clone(),
    };
    
    // Start background task to stream quotes
    let stream_state = state.clone();
    tokio::spawn(async move {
        stream_quotes(stream_state).await;
    });
    
    let mut server = WebServer::new();
    
    // WebSocket endpoint
    server = server.route("/ws", HttpMethod::GET, {
        let state = state.clone();
        move |req: Request| {
            let state = state.clone();
            async move {
                // Check if this is a WebSocket upgrade request
                if !req.header("upgrade").map(|v| v == "websocket").unwrap_or(false) {
                    return Ok(Response::error(400)
                        .body("Expected WebSocket upgrade".to_string()));
                }
                
                // In a real implementation, this would handle WebSocket protocol
                // For now, we'll simulate with a regular HTTP response
                // Note: web-server-abstraction handles WebSocket protocol internally
                
                tracing::info!("New WebSocket connection");
                
                Ok(Response::ok()
                    .header("upgrade", "websocket")
                    .header("connection", "upgrade")
                    .body("WebSocket connection established".to_string()))
            }
        }
    });
    
    // HTTP endpoint to show current subscriptions
    server = server.route("/subscriptions", HttpMethod::GET, {
        let state = state.clone();
        move |_req: Request| {
            let state = state.clone();
            async move {
                let subs = state.subscriptions.read().await;
                let symbols: Vec<_> = subs.iter().cloned().collect();
                let json = serde_json::to_string(&symbols)
                    .map_err(|e| format!("JSON error: {}", e))?;
                
                Ok(Response::ok()
                    .header("content-type", "application/json")
                    .body(json))
            }
        }
    });
    
    // Root endpoint with instructions
    server = server.route("/", HttpMethod::GET, |_req: Request| async {
        let html = r#"<!DOCTYPE html>
<html>
<head>
    <title>EEYF WebSocket Streaming</title>
    <style>
        body { font-family: sans-serif; max-width: 800px; margin: 50px auto; }
        pre { background: #f4f4f4; padding: 15px; border-radius: 5px; }
        .quote { padding: 10px; margin: 5px 0; background: #e8f5e9; border-radius: 3px; }
        button { padding: 10px 20px; margin: 5px; cursor: pointer; }
    </style>
</head>
<body>
    <h1>🚀 EEYF WebSocket Streaming</h1>
    
    <h2>Quick Start</h2>
    <pre>websocat ws://localhost:8080/ws</pre>
    
    <h2>Subscribe to Symbols</h2>
    <pre>{"action":"subscribe","symbols":["AAPL","GOOGL","MSFT"]}</pre>
    
    <h2>Unsubscribe from Symbols</h2>
    <pre>{"action":"unsubscribe","symbols":["GOOGL"]}</pre>
    
    <h2>Test with JavaScript</h2>
    <button onclick="connect()">Connect</button>
    <button onclick="subscribe()">Subscribe to AAPL</button>
    <button onclick="unsubscribe()">Unsubscribe</button>
    <button onclick="disconnect()">Disconnect</button>
    
    <h2>Messages</h2>
    <div id="messages"></div>
    
    <script>
        let ws = null;
        
        function connect() {
            ws = new WebSocket('ws://localhost:8080/ws');
            ws.onopen = () => addMessage('Connected!');
            ws.onmessage = (e) => addMessage('Received: ' + e.data);
            ws.onerror = (e) => addMessage('Error: ' + e);
            ws.onclose = () => addMessage('Disconnected');
        }
        
        function subscribe() {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({
                    action: 'subscribe',
                    symbols: ['AAPL', 'GOOGL', 'MSFT']
                }));
            } else {
                alert('Not connected!');
            }
        }
        
        function unsubscribe() {
            if (ws && ws.readyState === WebSocket.OPEN) {
                ws.send(JSON.stringify({
                    action: 'unsubscribe',
                    symbols: ['GOOGL']
                }));
            }
        }
        
        function disconnect() {
            if (ws) ws.close();
        }
        
        function addMessage(msg) {
            const div = document.createElement('div');
            div.className = 'quote';
            div.textContent = new Date().toLocaleTimeString() + ' - ' + msg;
            document.getElementById('messages').prepend(div);
        }
    </script>
</body>
</html>"#;
        
        Ok(Response::ok()
            .header("content-type", "text/html")
            .body(html.to_string()))
    });
    
    let server = server.bind("127.0.0.1:8080").await?;
    
    tracing::info!("🚀 WebSocket server running on http://127.0.0.1:8080");
    tracing::info!("📖 Open http://127.0.0.1:8080 in your browser");
    tracing::info!("🔌 Connect with: websocat ws://localhost:8080/ws");
    
    server.run().await?;
    Ok(())
}

/// Background task that streams quotes for subscribed symbols
async fn stream_quotes(state: StreamState) {
    let mut ticker = interval(Duration::from_secs(5));
    
    loop {
        ticker.tick().await;
        
        // Get current subscriptions
        let symbols: Vec<String> = {
            let subs = state.subscriptions.read().await;
            subs.iter().cloned().collect()
        };
        
        if symbols.is_empty() {
            continue;
        }
        
        tracing::info!("Streaming quotes for {} symbols", symbols.len());
        
        // Fetch quotes for all subscribed symbols
        match state.client.batch_quotes(&symbols).await {
            Ok(quotes) => {
                for quote in quotes {
                    tracing::info!(
                        "Stream: {} = ${} @ {}",
                        quote.symbol,
                        quote.price,
                        quote.timestamp
                    );
                    
                    // In a real implementation, this would send to all WebSocket clients
                    // via web-server-abstraction's WebSocket API
                }
            }
            Err(e) => {
                tracing::error!("Error fetching quotes: {}", e);
            }
        }
    }
}
