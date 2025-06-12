// Copyright (c) 2025 MCP Rust Contributors
// SPDX-License-Identifier: MIT

//! # MCP Protocol Server
//!
//! Lightweight server library for building MCP servers in Rust.
//!
//! This crate provides a high-level, ergonomic API for building MCP servers.
//! It handles the JSON-RPC protocol details, capability negotiation, and
//! transport management.

pub mod builder;
pub mod error;
pub mod handler;
pub mod server;
pub mod transport;

pub use builder::ServerBuilder;
pub use error::{ServerError, ServerResult};
pub use server::Server;

/// Re-export commonly used types
pub use mcp_protocol_types::*;
