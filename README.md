# MCP Protocol Server

[![Crates.io](https://img.shields.io/crates/v/mcp-protocol-server.svg)](https://crates.io/crates/mcp-protocol-server)
[![Documentation](https://docs.rs/mcp-protocol-server/badge.svg)](https://docs.rs/mcp-protocol-server)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**Lightweight server library for the Model Context Protocol (MCP)**

This crate provides a high-level, ergonomic API for building MCP servers in Rust. It handles the JSON-RPC protocol details, capability negotiation, and transport management, allowing you to focus on implementing your tools, resources, and prompts.

## ✨ Features

- 🦀 **Pure Rust** - Zero-cost abstractions with memory safety
- 🎯 **Type-Safe** - Compile-time guarantees using mcp-protocol-types
- 🚀 **Async/Await** - Built on Tokio for high performance
- 🔌 **Multiple Transports** - STDIO transport with extensible design
- 🛠️ **Complete MCP Support** - Tools, resources, prompts, logging
- 📦 **Lightweight** - Minimal dependencies for fast builds
- 🧪 **Well Tested** - Comprehensive test suite
- 📖 **Great Documentation** - Examples and guides

## 🚀 Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
mcp-protocol-server = "0.1.0"
mcp-protocol-types = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
serde_json = "1.0"
```

### Basic Server Example

```rust
use mcp_protocol_server::{Server, ServerBuilder};
use mcp_protocol_types::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create server
    let server = ServerBuilder::new("my-server", "1.0.0")
        .with_tool(
            Tool::new("echo", "Echo back the input")
                .with_parameter("message", "Message to echo", true)
        )
        .build();

    // Set up tool handler
    server.set_tool_handler("echo", |request| async move {
        let message = request.arguments
            .as_ref()
            .and_then(|args| args.get("message"))
            .and_then(|v| v.as_str())
            .unwrap_or("Hello, World!");

        Ok(CallToolResult {
            content: vec![ToolResultContent::text(message)],
            is_error: None,
        })
    });

    // Run server with STDIO transport
    server.run_stdio().await?;
    Ok(())
}
```

### Resource Server Example

```rust
use mcp_protocol_server::{Server, ServerBuilder};
use mcp_protocol_types::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut files = HashMap::new();
    files.insert("config.json".to_string(), r#"{"debug": true}"#.to_string());
    files.insert("README.md".to_string(), "# My Project\n\nDocumentation here.".to_string());

    let server = ServerBuilder::new("file-server", "1.0.0")
        .with_resource(Resource {
            uri: "file://config.json".to_string(),
            name: Some("Configuration".to_string()),
            description: Some("Application configuration file".to_string()),
            mime_type: Some("application/json".to_string()),
        })
        .with_resource(Resource {
            uri: "file://README.md".to_string(),
            name: Some("Documentation".to_string()),
            description: Some("Project documentation".to_string()),
            mime_type: Some("text/markdown".to_string()),
        })
        .build();

    // Set up resource handler
    server.set_resource_handler(move |request| {
        let files = files.clone();
        async move {
            let file_name = request.uri.strip_prefix("file://").unwrap_or(&request.uri);
            
            if let Some(content) = files.get(file_name) {
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(request.uri.clone(), content)],
                })
            } else {
                Err(McpError::invalid_params("File not found"))
            }
        }
    });

    server.run_stdio().await?;
    Ok(())
}
```

## 🏗️ Architecture

The server library provides a layered architecture:

```
┌─────────────────────────────────────────────┐
│           Your Application Logic             │
├─────────────────────────────────────────────┤
│         MCP Protocol Server                 │ ← This crate
├─────────────────────────────────────────────┤
│         MCP Protocol Types                  │
├─────────────────────────────────────────────┤
│           Transport Layer                   │ (STDIO, HTTP, WebSocket)
└─────────────────────────────────────────────┘
```

## 📋 Core Concepts

### Server Builder

The `ServerBuilder` provides a fluent API for configuring your server:

```rust
use mcp_protocol_server::ServerBuilder;
use mcp_protocol_types::*;

let server = ServerBuilder::new("my-server", "1.0.0")
    .with_description("An awesome MCP server")
    .with_instructions("Use this server to perform amazing tasks")
    .with_tool(Tool::new("calculate", "Perform calculations"))
    .with_resource(Resource {
        uri: "data://numbers".to_string(),
        name: Some("Numbers Dataset".to_string()),
        description: Some("Collection of interesting numbers".to_string()),
        mime_type: Some("application/json".to_string()),
    })
    .build();
```

### Handler Functions

Register async handlers for different MCP operations:

```rust
// Tool handler
server.set_tool_handler("my-tool", |request| async move {
    // Process the tool request
    Ok(CallToolResult {
        content: vec![ToolResultContent::text("Result")],
        is_error: None,
    })
});

// Resource handler
server.set_resource_handler(|request| async move {
    // Read and return resource content
    Ok(ReadResourceResult {
        contents: vec![ResourceContents::text(request.uri, "content")],
    })
});

// Prompt handler
server.set_prompt_handler("my-prompt", |request| async move {
    // Generate prompt messages
    Ok(GetPromptResult {
        description: Some("Generated prompt".to_string()),
        messages: vec![PromptMessage {
            role: PromptRole::User,
            content: PromptContent::Text {
                text: "Hello!".to_string(),
            },
        }],
    })
});
```

### Transport Support

Currently supports STDIO transport (perfect for Claude Desktop):

```rust
// Run with STDIO (for Claude Desktop integration)
server.run_stdio().await?;

// Or use custom transport implementation
let transport = MyCustomTransport::new();
server.run(transport).await?;
```

## 🔧 Feature Flags

| Feature | Description | Default |
|---------|-------------|---------|
| `stdio` | STDIO transport support | ✅ |

## 📊 Usage Patterns

### Claude Desktop Integration

Perfect for extending Claude Desktop with custom capabilities:

```rust
use mcp_protocol_server::{Server, ServerBuilder};
use mcp_protocol_types::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let server = ServerBuilder::new("claude-tools", "1.0.0")
        .with_instructions("I provide helpful tools for development tasks")
        .with_tool(
            Tool::new("git-status", "Check git repository status")
                .with_parameter("path", "Repository path", false)
        )
        .with_tool(
            Tool::new("run-tests", "Execute project tests")
                .with_parameter("test_name", "Specific test to run", false)
        )
        .build();

    // Implement handlers...
    server.run_stdio().await?;
    Ok(())
}
```

### File System Tools

```rust
use mcp_protocol_server::ServerBuilder;
use mcp_protocol_types::*;
use std::fs;

let server = ServerBuilder::new("file-tools", "1.0.0")
    .with_tool(
        Tool::new("read-file", "Read file contents")
            .with_parameter("path", "File path to read", true)
    )
    .with_tool(
        Tool::new("list-dir", "List directory contents")
            .with_parameter("path", "Directory path", true)
    )
    .build();

server.set_tool_handler("read-file", |request| async move {
    let path = request.arguments
        .as_ref()
        .and_then(|args| args.get("path"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing path parameter"))?;

    match fs::read_to_string(path) {
        Ok(content) => Ok(CallToolResult {
            content: vec![ToolResultContent::text(content)],
            is_error: None,
        }),
        Err(e) => Ok(CallToolResult {
            content: vec![ToolResultContent::text(format!("Error: {}", e))],
            is_error: Some(true),
        }),
    }
});
```

### Database Integration

```rust
// Example with a hypothetical database
server.set_tool_handler("query-db", |request| async move {
    let sql = request.arguments
        .as_ref()
        .and_then(|args| args.get("sql"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing sql parameter"))?;

    // Execute query (implement your database logic)
    let results = execute_query(sql).await?;
    
    Ok(CallToolResult {
        content: vec![ToolResultContent::text(format!("Results: {:?}", results))],
        is_error: None,
    })
});
```

## 🧪 Testing

```rust
use mcp_protocol_server::testing::MockTransport;

#[tokio::test]
async fn test_tool_handler() {
    let server = ServerBuilder::new("test-server", "1.0.0")
        .with_tool(Tool::new("test-tool", "Test tool"))
        .build();

    server.set_tool_handler("test-tool", |_| async move {
        Ok(CallToolResult {
            content: vec![ToolResultContent::text("test result")],
            is_error: None,
        })
    });

    let mut transport = MockTransport::new();
    
    // Send test request
    transport.send_request("tools/call", json!({
        "name": "test-tool",
        "arguments": {}
    })).await;

    // Verify response
    let response = transport.receive_response().await;
    // Assert response content...
}
```

## 🛠️ Development

```bash
# Build the crate
cargo build

# Run tests
cargo test

# Run with all features
cargo check --all-features

# Generate documentation
cargo doc --open
```

## 🔗 Related Crates

- [`mcp-protocol-types`](https://github.com/mcp-rust/mcp-protocol-types) - Core protocol types
- [`mcp-protocol-client`](https://github.com/mcp-rust/mcp-protocol-client) - Client library
- [`mcp-protocol-sdk`](https://github.com/mcp-rust/mcp-protocol-sdk) - Full-featured SDK

## 🤝 Contributing

This crate is part of the [MCP Rust ecosystem](https://github.com/mcp-rust). Contributions are welcome!

### Guidelines
- **API Design** - Keep the API simple and ergonomic
- **Performance** - Optimize for low latency and memory usage
- **Documentation** - All public APIs need examples
- **Testing** - Comprehensive test coverage required

## 📋 Protocol Compliance

✅ **MCP 2024-11-05 Specification**

This library implements the complete MCP server specification:
- JSON-RPC 2.0 protocol handling
- Capability negotiation and initialization
- Tool calling and parameter validation
- Resource access and content delivery
- Prompt template processing
- Logging and debugging support
- Error handling and recovery

## 📄 License

Licensed under the [MIT License](./LICENSE).

## 🙏 Acknowledgments

- **Anthropic** - For creating the MCP specification
- **Tokio Team** - For the excellent async runtime
- **Rust Community** - For the amazing ecosystem

---

*Lightweight MCP server library for Rust 🦀*