//! Error types for the MCP server.

use thiserror::Error;

/// Server error types
#[derive(Debug, Error)]
pub enum ServerError {
    /// Transport error
    #[error("Transport error: {0}")]
    Transport(String),
    
    /// Protocol error
    #[error("Protocol error: {0}")]
    Protocol(String),
    
    /// Handler error
    #[error("Handler error: {0}")]
    Handler(String),
    
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}