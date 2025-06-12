//! STDIO transport implementation.

#[cfg(feature = "stdio")]
use crate::{Transport, ServerError};
#[cfg(feature = "stdio")]
use async_trait::async_trait;
#[cfg(feature = "stdio")]
use mcp_protocol_types::{JsonRpcRequest, JsonRpcResponse};
#[cfg(feature = "stdio")]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, stdin, stdout};

#[cfg(feature = "stdio")]
/// STDIO transport for MCP communication
pub struct StdioTransport {
    reader: BufReader<tokio::io::Stdin>,
    writer: tokio::io::Stdout,
}

#[cfg(feature = "stdio")]
impl StdioTransport {
    /// Create a new STDIO transport
    pub fn new() -> Self {
        Self {
            reader: BufReader::new(stdin()),
            writer: stdout(),
        }
    }
}

#[cfg(feature = "stdio")]
#[async_trait]
impl Transport for StdioTransport {
    async fn receive_request(&mut self) -> Result<JsonRpcRequest, ServerError> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await?;
        let request: JsonRpcRequest = serde_json::from_str(&line)?;
        Ok(request)
    }

    async fn send_response(&mut self, response: JsonRpcResponse) -> Result<(), ServerError> {
        let json = serde_json::to_string(&response)?;
        self.writer.write_all(json.as_bytes()).await?;
        self.writer.write_all(b"\n").await?;
        self.writer.flush().await?;
        Ok(())
    }
}