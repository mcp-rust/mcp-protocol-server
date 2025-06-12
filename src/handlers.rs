//! Handler types and traits.

use async_trait::async_trait;
use mcp_protocol_types::*;
use std::future::Future;
use std::pin::Pin;

/// Tool handler function type
pub type ToolHandler = Box<dyn Fn(CallToolRequest) -> Pin<Box<dyn Future<Output = Result<CallToolResult, McpError>> + Send>> + Send + Sync>;

/// Resource handler function type  
pub type ResourceHandler = Box<dyn Fn(ReadResourceRequest) -> Pin<Box<dyn Future<Output = Result<ReadResourceResult, McpError>> + Send>> + Send + Sync>;

/// Prompt handler function type
pub type PromptHandler = Box<dyn Fn(GetPromptRequest) -> Pin<Box<dyn Future<Output = Result<GetPromptResult, McpError>> + Send>> + Send + Sync>;