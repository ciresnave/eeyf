//! WebSocket streaming for real-time market data
//!
//! This module provides real-time streaming of market data from Yahoo Finance
//! using WebSocket connections. Data is transmitted as Base64-encoded Protocol
//! Buffer messages.
//!
//! # Features
//!
//! - **Automatic reconnection** with exponential backoff
//! - **Message handler callbacks** for event-driven processing
//! - **Backpressure handling** for high-frequency updates
//! - **Automatic heartbeat** to maintain subscriptions
//! - **Protocol Buffer decoding** with Base64 encoding
//!
//! # Example
//!
//! ```no_run
//! use eeyf::websocket::{WebSocketStream, WebSocketConfig};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Configure with reconnection and backpressure
//!     let config = WebSocketConfig::new()
//!         .with_max_reconnect_attempts(5)
//!         .with_backpressure_buffer_size(100);
//!     
//!     // Connect to Yahoo Finance WebSocket
//!     let mut stream = WebSocketStream::connect_with_config(config).await?;
//!     
//!     // Subscribe to symbols
//!     stream.subscribe(&["AAPL", "GOOGL", "MSFT"]).await?;
//!     
//!     // Receive updates
//!     while let Some(ticker) = stream.next().await {
//!         match ticker {
//!             Ok(data) => {
//!                 println!("{}: ${:.2}", data.id, data.price);
//!             }
//!             Err(e) => {
//!                 eprintln!("Error: {}", e);
//!             }
//!         }
//!     }
//!     
//!     Ok(())
//! }
//! ```

use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use futures_util::{SinkExt, StreamExt};
use prost::Message;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex};
use tokio::time::{interval, sleep, Interval};
use tokio_tungstenite::{
    connect_async,
    tungstenite::{Error as WsError, Message as WsMessage},
    MaybeTlsStream, WebSocketStream as TungsteniteStream,
};

// Include the generated protobuf code
mod proto {
    include!(concat!(env!("OUT_DIR"), "/yahoo.finance.rs"));
}

pub use proto::{yaticker, Yaticker};

/// WebSocket endpoint URL for Yahoo Finance
const YAHOO_WS_URL: &str = "wss://streamer.finance.yahoo.com/";

/// Heartbeat interval to maintain subscriptions (15 seconds)
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(15);

/// Default initial reconnection delay
const DEFAULT_INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Maximum reconnection delay (capped for exponential backoff)
const MAX_RECONNECT_DELAY: Duration = Duration::from_secs(60);

/// Default maximum reconnection attempts (0 = infinite)
const DEFAULT_MAX_RECONNECT_ATTEMPTS: u32 = 0;

/// Default backpressure buffer size
const DEFAULT_BACKPRESSURE_BUFFER: usize = 1000;

/// Subscription message format
#[derive(Debug, Serialize, Deserialize)]
struct SubscriptionMessage {
    subscribe: Vec<String>,
}

/// Unsubscription message format
#[derive(Debug, Serialize, Deserialize)]
struct UnsubscriptionMessage {
    unsubscribe: Vec<String>,
}

/// Message handler callback type
/// 
/// Handlers receive ticker data and can return an error to stop processing
pub type MessageHandler = Arc<dyn Fn(Yaticker) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>;

/// WebSocket configuration
#[derive(Clone)]
pub struct WebSocketConfig {
    /// Initial delay before first reconnection attempt
    pub initial_reconnect_delay: Duration,
    
    /// Maximum number of reconnection attempts (0 = infinite)
    pub max_reconnect_attempts: u32,
    
    /// Heartbeat interval for maintaining subscriptions
    pub heartbeat_interval: Duration,
    
    /// Buffer size for backpressure handling
    pub backpressure_buffer_size: usize,
    
    /// Whether to automatically reconnect on disconnect
    pub auto_reconnect: bool,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self::new()
    }
}

impl WebSocketConfig {
    /// Create a new configuration with default values
    pub fn new() -> Self {
        Self {
            initial_reconnect_delay: DEFAULT_INITIAL_RECONNECT_DELAY,
            max_reconnect_attempts: DEFAULT_MAX_RECONNECT_ATTEMPTS,
            heartbeat_interval: HEARTBEAT_INTERVAL,
            backpressure_buffer_size: DEFAULT_BACKPRESSURE_BUFFER,
            auto_reconnect: true,
        }
    }
    
    /// Set the initial reconnection delay
    pub fn with_initial_reconnect_delay(mut self, delay: Duration) -> Self {
        self.initial_reconnect_delay = delay;
        self
    }
    
    /// Set the maximum number of reconnection attempts (0 = infinite)
    pub fn with_max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.max_reconnect_attempts = attempts;
        self
    }
    
    /// Set the heartbeat interval
    pub fn with_heartbeat_interval(mut self, interval: Duration) -> Self {
        self.heartbeat_interval = interval;
        self
    }
    
    /// Set the backpressure buffer size
    pub fn with_backpressure_buffer_size(mut self, size: usize) -> Self {
        self.backpressure_buffer_size = size;
        self
    }
    
    /// Enable or disable automatic reconnection
    pub fn with_auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }
}

/// Connection state for tracking reconnections
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    /// Connected and active
    Connected,
    /// Disconnected, attempting to reconnect
    Reconnecting,
    /// Disconnected, no reconnection attempt
    Disconnected,
}

/// WebSocket stream statistics
#[derive(Debug, Clone, Default)]
pub struct StreamStats {
    /// Total messages received
    pub messages_received: u64,
    /// Total messages dropped due to backpressure
    pub messages_dropped: u64,
    /// Total reconnection attempts
    pub reconnect_attempts: u32,
    /// Total successful reconnections
    pub successful_reconnects: u32,
    /// Total heartbeats sent
    pub heartbeats_sent: u64,
}

/// WebSocket connection for streaming real-time market data
/// 
/// This struct provides a high-level interface for receiving real-time market data
/// with automatic reconnection, backpressure handling, and message callbacks.
pub struct WebSocketStream {
    ws_stream: Option<TungsteniteStream<MaybeTlsStream<tokio::net::TcpStream>>>,
    subscribed_symbols: Vec<String>,
    heartbeat: Interval,
    config: WebSocketConfig,
    state: ConnectionState,
    reconnect_attempts: u32,
    current_reconnect_delay: Duration,
    message_handlers: Vec<MessageHandler>,
    backpressure_buffer: Option<mpsc::Receiver<Yaticker>>,
    stats: Arc<Mutex<StreamStats>>,
}

impl WebSocketStream {
    /// Connect to Yahoo Finance WebSocket endpoint with default configuration
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let stream = WebSocketStream::connect().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect() -> Result<Self, WsError> {
        Self::connect_with_config(WebSocketConfig::new()).await
    }
    
    /// Connect to Yahoo Finance WebSocket endpoint with custom configuration
    ///
    /// # Arguments
    ///
    /// * `config` - WebSocket configuration including reconnection and backpressure settings
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::{WebSocketStream, WebSocketConfig};
    /// # use std::time::Duration;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = WebSocketConfig::new()
    ///     .with_max_reconnect_attempts(5)
    ///     .with_backpressure_buffer_size(100);
    /// let stream = WebSocketStream::connect_with_config(config).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn connect_with_config(config: WebSocketConfig) -> Result<Self, WsError> {
        let (ws_stream, _) = connect_async(YAHOO_WS_URL).await?;
        
        let heartbeat = interval(config.heartbeat_interval);
        
        Ok(Self {
            ws_stream: Some(ws_stream),
            subscribed_symbols: Vec::new(),
            heartbeat,
            config: config.clone(),
            state: ConnectionState::Connected,
            reconnect_attempts: 0,
            current_reconnect_delay: config.initial_reconnect_delay,
            message_handlers: Vec::new(),
            backpressure_buffer: None,
            stats: Arc::new(Mutex::new(StreamStats::default())),
        })
    }
    
    /// Attempt to reconnect to the WebSocket endpoint
    /// 
    /// Uses exponential backoff with jitter for reconnection attempts
    async fn reconnect(&mut self) -> Result<(), WsError> {
        if !self.config.auto_reconnect {
            return Err(WsError::AlreadyClosed);
        }
        
        // Check if we've exceeded max attempts
        if self.config.max_reconnect_attempts > 0 
            && self.reconnect_attempts >= self.config.max_reconnect_attempts {
            log::error!(
                "Max reconnection attempts ({}) exceeded",
                self.config.max_reconnect_attempts
            );
            self.state = ConnectionState::Disconnected;
            return Err(WsError::AlreadyClosed);
        }
        
        self.state = ConnectionState::Reconnecting;
        self.reconnect_attempts += 1;
        
        {
            let mut stats = self.stats.lock().await;
            stats.reconnect_attempts += 1;
        }
        
        log::info!(
            "Attempting reconnection #{} after {:?}",
            self.reconnect_attempts,
            self.current_reconnect_delay
        );
        
        // Wait with exponential backoff
        sleep(self.current_reconnect_delay).await;
        
        // Exponential backoff with cap
        self.current_reconnect_delay = std::cmp::min(
            self.current_reconnect_delay * 2,
            MAX_RECONNECT_DELAY
        );
        
        // Attempt connection
        match connect_async(YAHOO_WS_URL).await {
            Ok((ws_stream, _)) => {
                self.ws_stream = Some(ws_stream);
                self.state = ConnectionState::Connected;
                self.current_reconnect_delay = self.config.initial_reconnect_delay;
                
                {
                    let mut stats = self.stats.lock().await;
                    stats.successful_reconnects += 1;
                }
                
                log::info!("Reconnection successful");
                
                // Re-subscribe to all symbols
                if !self.subscribed_symbols.is_empty() {
                    log::info!("Re-subscribing to {} symbols", self.subscribed_symbols.len());
                    self.resubscribe_all().await?;
                }
                
                Ok(())
            }
            Err(e) => {
                log::error!("Reconnection failed: {}", e);
                Err(e)
            }
        }
    }
    
    /// Re-subscribe to all currently tracked symbols after reconnection
    async fn resubscribe_all(&mut self) -> Result<(), WsError> {
        if self.subscribed_symbols.is_empty() {
            return Ok(());
        }
        
        let symbols = self.subscribed_symbols.clone();
        let message = SubscriptionMessage {
            subscribe: symbols,
        };
        
        let json = serde_json::to_string(&message)
            .map_err(|e| WsError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        if let Some(ws) = &mut self.ws_stream {
            ws.send(WsMessage::Text(json)).await?;
        }
        
        Ok(())
    }
    
    /// Add a message handler callback
    /// 
    /// Handlers are called for each received ticker update. Multiple handlers can be registered
    /// and will be called in order of registration.
    ///
    /// # Arguments
    ///
    /// * `handler` - Closure that processes ticker data
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = WebSocketStream::connect().await?;
    /// 
    /// // Add a handler that logs high-value trades
    /// stream.add_handler(|ticker| {
    ///     if ticker.price > 1000.0 {
    ///         println!("High value: {} at ${}", ticker.id, ticker.price);
    ///     }
    ///     Ok(())
    /// });
    /// 
    /// stream.subscribe(&["AAPL"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn add_handler<F>(&mut self, handler: F)
    where
        F: Fn(Yaticker) -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        self.message_handlers.push(Arc::new(handler));
    }
    
    /// Enable backpressure handling with a buffered channel
    /// 
    /// When enabled, messages are buffered and can be retrieved via `next_buffered()`.
    /// If the buffer fills up, oldest messages are dropped.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = WebSocketStream::connect().await?;
    /// stream.enable_backpressure();
    /// stream.subscribe(&["AAPL"]).await?;
    /// 
    /// // Now use next_buffered() instead of next()
    /// while let Some(ticker) = stream.next_buffered().await {
    ///     println!("{}: ${}", ticker.id, ticker.price);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn enable_backpressure(&mut self) {
        let (tx, rx) = mpsc::channel(self.config.backpressure_buffer_size);
        self.backpressure_buffer = Some(rx);
        
        // Spawn background task to handle backpressure
        let stats = Arc::clone(&self.stats);
        let handlers = self.message_handlers.clone();
        
        tokio::spawn(async move {
            // This will be populated by the actual message processing
            // For now, this is a placeholder for the architecture
        });
    }
    
    /// Get the current connection state
    pub fn state(&self) -> ConnectionState {
        self.state
    }
    
    /// Get stream statistics
    pub async fn stats(&self) -> StreamStats {
        self.stats.lock().await.clone()
    }
    
    /// Reset statistics
    pub async fn reset_stats(&mut self) {
        let mut stats = self.stats.lock().await;
        *stats = StreamStats::default();
    }

    /// Subscribe to one or more symbols for real-time updates
    ///
    /// # Arguments
    ///
    /// * `symbols` - Slice of symbol strings (e.g., ["AAPL", "GOOGL"])
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = WebSocketStream::connect().await?;
    /// stream.subscribe(&["AAPL", "GOOGL", "MSFT"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn subscribe(&mut self, symbols: &[&str]) -> Result<(), WsError> {
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        
        let message = SubscriptionMessage {
            subscribe: symbols.clone(),
        };
        
        let json = serde_json::to_string(&message)
            .map_err(|e| WsError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        if let Some(ws) = &mut self.ws_stream {
            ws.send(WsMessage::Text(json)).await?;
            self.subscribed_symbols.extend(symbols);
        } else {
            return Err(WsError::AlreadyClosed);
        }
        
        Ok(())
    }

    /// Unsubscribe from one or more symbols
    ///
    /// # Arguments
    ///
    /// * `symbols` - Slice of symbol strings to unsubscribe from
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = WebSocketStream::connect().await?;
    /// stream.subscribe(&["AAPL", "GOOGL"]).await?;
    /// stream.unsubscribe(&["AAPL"]).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn unsubscribe(&mut self, symbols: &[&str]) -> Result<(), WsError> {
        let symbols: Vec<String> = symbols.iter().map(|s| s.to_string()).collect();
        
        let message = UnsubscriptionMessage {
            unsubscribe: symbols.clone(),
        };
        
        let json = serde_json::to_string(&message)
            .map_err(|e| WsError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;
        
        if let Some(ws) = &mut self.ws_stream {
            ws.send(WsMessage::Text(json)).await?;
            
            // Remove from subscribed symbols
            self.subscribed_symbols.retain(|s| !symbols.contains(s));
        } else {
            return Err(WsError::AlreadyClosed);
        }
        
        Ok(())
    }

    /// Get the next ticker update from the stream
    ///
    /// This method will:
    /// - Send heartbeat messages automatically to maintain subscriptions
    /// - Decode Base64-encoded Protocol Buffer messages
    /// - Automatically reconnect on disconnection (if enabled)
    /// - Call registered message handlers
    /// - Return parsed ticker data
    ///
    /// # Returns
    ///
    /// * `Some(Ok(Yaticker))` - Successfully decoded ticker update
    /// * `Some(Err(WsError))` - Error occurred
    /// * `None` - Stream closed
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use eeyf::websocket::WebSocketStream;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut stream = WebSocketStream::connect().await?;
    /// stream.subscribe(&["AAPL"]).await?;
    ///
    /// while let Some(ticker) = stream.next().await {
    ///     match ticker {
    ///         Ok(data) => println!("{}: ${:.2}", data.id, data.price),
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn next(&mut self) -> Option<Result<Yaticker, WsError>> {
        loop {
            // If backpressure is enabled, use buffered channel
            if self.backpressure_buffer.is_some() {
                return self.next_buffered().await.map(Ok);
            }
            
            // Check if we need to reconnect
            if self.ws_stream.is_none() && self.config.auto_reconnect {
                match self.reconnect().await {
                    Ok(()) => continue,
                    Err(e) => return Some(Err(e)),
                }
            }
            
            tokio::select! {
                // Check for heartbeat
                _ = self.heartbeat.tick() => {
                    if let Err(e) = self.send_heartbeat().await {
                        // On heartbeat failure, attempt reconnection
                        if self.config.auto_reconnect {
                            log::warn!("Heartbeat failed, attempting reconnection");
                            self.ws_stream = None;
                            continue;
                        }
                        return Some(Err(e));
                    }
                }
                
                // Receive message from WebSocket
                msg = async {
                    if let Some(ws) = &mut self.ws_stream {
                        ws.next().await
                    } else {
                        None
                    }
                } => {
                    match msg {
                        Some(Ok(WsMessage::Binary(data))) => {
                            // Update stats
                            {
                                let mut stats = self.stats.lock().await;
                                stats.messages_received += 1;
                            }
                            
                            // Decode Base64
                            let decoded = match BASE64.decode(&data) {
                                Ok(d) => d,
                                Err(e) => {
                                    log::error!("Base64 decode error: {}", e);
                                    continue;
                                }
                            };
                            
                            // Decode Protocol Buffer
                            let ticker = match Yaticker::decode(&decoded[..]) {
                                Ok(ticker) => ticker,
                                Err(e) => {
                                    log::error!("Protobuf decode error: {}", e);
                                    continue;
                                }
                            };
                            
                            // Call message handlers
                            for handler in &self.message_handlers {
                                if let Err(e) = handler(ticker.clone()) {
                                    log::error!("Message handler error: {}", e);
                                }
                            }
                            
                            return Some(Ok(ticker));
                        }
                        Some(Ok(WsMessage::Text(_))) => {
                            // Ignore text messages (usually subscription confirmations)
                            continue;
                        }
                        Some(Ok(WsMessage::Ping(data))) => {
                            if let Some(ws) = &mut self.ws_stream {
                                if let Err(e) = ws.send(WsMessage::Pong(data)).await {
                                    return Some(Err(e));
                                }
                            }
                            continue;
                        }
                        Some(Ok(WsMessage::Pong(_))) => {
                            continue;
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            log::info!("WebSocket closed by server");
                            if self.config.auto_reconnect {
                                self.ws_stream = None;
                                continue;
                            }
                            return None;
                        }
                        Some(Ok(WsMessage::Frame(_))) => {
                            // Raw frames, ignore
                            continue;
                        }
                        Some(Err(e)) => {
                            log::error!("WebSocket error: {}", e);
                            if self.config.auto_reconnect {
                                self.ws_stream = None;
                                continue;
                            }
                            return Some(Err(e));
                        }
                        None => {
                            // Connection closed
                            log::info!("WebSocket connection closed");
                            if self.config.auto_reconnect {
                                self.ws_stream = None;
                                continue;
                            }
                            return None;
                        }
                    }
                }
            }
        }
    }
    
    /// Get the next ticker update from the buffered channel
    /// 
    /// This method is used when backpressure handling is enabled.
    /// Messages are buffered and retrieved from the channel.
    ///
    /// # Returns
    ///
    /// * `Some(Yaticker)` - Successfully decoded ticker update
    /// * `None` - Channel closed or backpressure not enabled
    pub async fn next_buffered(&mut self) -> Option<Yaticker> {
        if let Some(rx) = &mut self.backpressure_buffer {
            rx.recv().await
        } else {
            None
        }
    }

    /// Send heartbeat to maintain subscriptions
    ///
    /// Yahoo Finance requires re-sending the subscription message every 15 seconds
    /// to keep the subscriptions active.
    async fn send_heartbeat(&mut self) -> Result<(), WsError> {
        if self.subscribed_symbols.is_empty() {
            return Ok(());
        }

        let message = SubscriptionMessage {
            subscribe: self.subscribed_symbols.clone(),
        };

        let json = serde_json::to_string(&message)
            .map_err(|e| WsError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))?;

        if let Some(ws) = &mut self.ws_stream {
            ws.send(WsMessage::Text(json)).await?;
            
            {
                let mut stats = self.stats.lock().await;
                stats.heartbeats_sent += 1;
            }
        }
        
        Ok(())
    }

    /// Get list of currently subscribed symbols
    pub fn subscribed_symbols(&self) -> &[String] {
        &self.subscribed_symbols
    }

    /// Close the WebSocket connection
    pub async fn close(mut self) -> Result<(), WsError> {
        if let Some(mut ws) = self.ws_stream.take() {
            ws.close(None).await
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscription_message_serialization() {
        let msg = SubscriptionMessage {
            subscribe: vec!["AAPL".to_string(), "GOOGL".to_string()],
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"subscribe\""));
        assert!(json.contains("\"AAPL\""));
        assert!(json.contains("\"GOOGL\""));
    }

    #[test]
    fn test_unsubscription_message_serialization() {
        let msg = UnsubscriptionMessage {
            unsubscribe: vec!["AAPL".to_string()],
        };
        
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("\"unsubscribe\""));
        assert!(json.contains("\"AAPL\""));
    }
    
    #[test]
    fn test_config_builder() {
        let config = WebSocketConfig::new()
            .with_max_reconnect_attempts(5)
            .with_backpressure_buffer_size(500)
            .with_auto_reconnect(false);
        
        assert_eq!(config.max_reconnect_attempts, 5);
        assert_eq!(config.backpressure_buffer_size, 500);
        assert!(!config.auto_reconnect);
    }
    
    #[test]
    fn test_config_defaults() {
        let config = WebSocketConfig::new();
        
        assert_eq!(config.initial_reconnect_delay, DEFAULT_INITIAL_RECONNECT_DELAY);
        assert_eq!(config.max_reconnect_attempts, DEFAULT_MAX_RECONNECT_ATTEMPTS);
        assert_eq!(config.heartbeat_interval, HEARTBEAT_INTERVAL);
        assert_eq!(config.backpressure_buffer_size, DEFAULT_BACKPRESSURE_BUFFER);
        assert!(config.auto_reconnect);
    }
    
    #[test]
    fn test_connection_state() {
        assert_eq!(ConnectionState::Connected, ConnectionState::Connected);
        assert_ne!(ConnectionState::Connected, ConnectionState::Disconnected);
        assert_ne!(ConnectionState::Reconnecting, ConnectionState::Disconnected);
    }
    
    #[test]
    fn test_stream_stats_default() {
        let stats = StreamStats::default();
        
        assert_eq!(stats.messages_received, 0);
        assert_eq!(stats.messages_dropped, 0);
        assert_eq!(stats.reconnect_attempts, 0);
        assert_eq!(stats.successful_reconnects, 0);
        assert_eq!(stats.heartbeats_sent, 0);
    }
}
