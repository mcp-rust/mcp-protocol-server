//! # MCP Protocol Server
//!
//! Lightweight server library for the Model Context Protocol (MCP).
//!
//! This crate provides a high-level, ergonomic API for building MCP servers in Rust.
//! It handles the JSON-RPC protocol details, capability negotiation, and transport
//! management, allowing you to focus on implementing your tools, resources, and prompts.
//!
//! ## Features
//!
//! - **Pure Rust** - Zero-cost abstractions with memory safety
//! - **Type-Safe** - Compile-time guarantees using mcp-protocol-types
//! - **Async/Await** - Built on Tokio for high performance
//! - **Multiple Transports** - STDIO transport with extensible design
//! - **Complete MCP Support** - Tools, resources, prompts, logging
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use mcp_protocol_server::{Server, ServerBuilder};
//! use mcp_protocol_types::*;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create server
//!     let server = ServerBuilder::new("my-server", "1.0.0")
//!         .with_tool(
//!             Tool::new("echo", "Echo back the input")
//!                 .with_parameter("message", "Message to echo", true)
//!         )
//!         .build();
//!
//!     // Set up tool handler
//!     server.set_tool_handler("echo", |request| async move {
//!         let message = request.arguments
//!             .as_ref()
//!             .and_then(|args| args.get("message"))
//!             .and_then(|v| v.as_str())
//!             .unwrap_or("Hello, World!");
//!
//!         Ok(CallToolResult {
//!             content: vec![ToolResultContent::text(message)],
//!             is_error: None,
//!         })
//!     });
//!
//!     // Run server with STDIO transport
//!     server.run_stdio().await?;
//!     Ok(())
//! }
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs, rust_2018_idioms)]
#![deny(unsafe_code)]

pub mod server;
pub mod transport;
pub mod error;
pub mod handlers;

#[cfg(feature = "stdio")]
pub mod stdio;

pub use server::{Server, ServerBuilder};
pub use error::ServerError;
pub use handlers::*;
pub use transport::Transport;

#[cfg(feature = "stdio")]
pub use stdio::StdioTransport;

// Re-export commonly used types
pub use mcp_protocol_types::*;