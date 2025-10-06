//! EEYF Web Server Integration
//!
//! Helpers and utilities for integrating EEYF with web-server-abstraction.
//! Provides ready-to-use route handlers, WebSocket streaming, and database
//! helpers.

pub mod database;
pub mod helpers;

pub use database::*;
pub use eeyf::{EEYFClient, HistoricalDataPoint, Quote};
pub use helpers::*;
// Re-export commonly used types
pub use web_server_abstraction::{
    Handler, HttpMethod, Request, Response, Result as WebResult, StatusCode, WebServer,
};
