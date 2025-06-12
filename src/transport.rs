//! Transport trait and implementations.

use async_trait::async_trait;
use mcp_protocol_types::{JsonRpcRequest, JsonRpcResponse};
use crate::error::ServerError;

/// Transport trait for MCP communication
#[async_trait]
pub trait Transport {
    /// Receive a request from the transport
    async fn receive_request(&mut self) -> Result<JsonRpcRequest, ServerError>;
    
    /// Send a response through the transport
    async fn send_response(&mut self, response: JsonRpcResponse) -> Result<(), ServerError>;
}