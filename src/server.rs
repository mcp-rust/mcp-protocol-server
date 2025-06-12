//! Server implementation.

use crate::{ServerError, ToolHandler, ResourceHandler, PromptHandler, Transport};
use mcp_protocol_types::*;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP Server implementation
pub struct Server {
    info: Implementation,
    capabilities: ServerCapabilities,
    tools: Vec<Tool>,
    resources: Vec<Resource>,
    prompts: Vec<Prompt>,
    tool_handlers: Arc<RwLock<HashMap<String, ToolHandler>>>,
    resource_handlers: Arc<RwLock<HashMap<String, ResourceHandler>>>,
    prompt_handlers: Arc<RwLock<HashMap<String, PromptHandler>>>,
}

/// Builder for creating MCP servers
pub struct ServerBuilder {
    name: String,
    version: String,
    description: Option<String>,
    instructions: Option<String>,
    tools: Vec<Tool>,
    resources: Vec<Resource>,
    prompts: Vec<Prompt>,
}

impl ServerBuilder {
    /// Create a new server builder
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: None,
            instructions: None,
            tools: Vec::new(),
            resources: Vec::new(),
            prompts: Vec::new(),
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add instructions
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Add a tool
    pub fn with_tool(mut self, tool: Tool) -> Self {
        self.tools.push(tool);
        self
    }

    /// Add a resource
    pub fn with_resource(mut self, resource: Resource) -> Self {
        self.resources.push(resource);
        self
    }

    /// Add a prompt
    pub fn with_prompt(mut self, prompt: Prompt) -> Self {
        self.prompts.push(prompt);
        self
    }

    /// Build the server
    pub fn build(self) -> Server {
        let capabilities = ServerCapabilities {
            tools: if self.tools.is_empty() { None } else { Some(ToolsCapability { list_changed: None }) },
            resources: if self.resources.is_empty() { None } else { Some(ResourcesCapability { subscribe: None, list_changed: None }) },
            prompts: if self.prompts.is_empty() { None } else { Some(PromptsCapability { list_changed: None }) },
            logging: Some(LoggingCapability {}),
            experimental: None,
        };

        Server {
            info: Implementation {
                name: self.name,
                version: self.version,
            },
            capabilities,
            tools: self.tools,
            resources: self.resources,
            prompts: self.prompts,
            tool_handlers: Arc::new(RwLock::new(HashMap::new())),
            resource_handlers: Arc::new(RwLock::new(HashMap::new())),
            prompt_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Server {
    /// Set a tool handler
    pub async fn set_tool_handler<F, Fut>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn(CallToolRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<CallToolResult, McpError>> + Send + 'static,
    {
        let boxed_handler: ToolHandler = Box::new(move |req| Box::pin(handler(req)));
        self.tool_handlers.write().await.insert(name.into(), boxed_handler);
    }

    /// Set a resource handler
    pub async fn set_resource_handler<F, Fut>(&self, handler: F)
    where
        F: Fn(ReadResourceRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<ReadResourceResult, McpError>> + Send + 'static,
    {
        let boxed_handler: ResourceHandler = Box::new(move |req| Box::pin(handler(req)));
        self.resource_handlers.write().await.insert("default".to_string(), boxed_handler);
    }

    /// Set a prompt handler
    pub async fn set_prompt_handler<F, Fut>(&self, name: impl Into<String>, handler: F)
    where
        F: Fn(GetPromptRequest) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<GetPromptResult, McpError>> + Send + 'static,
    {
        let boxed_handler: PromptHandler = Box::new(move |req| Box::pin(handler(req)));
        self.prompt_handlers.write().await.insert(name.into(), boxed_handler);
    }

    /// Run the server with STDIO transport
    #[cfg(feature = "stdio")]
    pub async fn run_stdio(&self) -> Result<(), ServerError> {
        use crate::StdioTransport;
        let mut transport = StdioTransport::new();
        self.run(transport).await
    }

    /// Run the server with a custom transport
    pub async fn run<T: Transport>(&self, mut transport: T) -> Result<(), ServerError> {
        loop {
            let request = transport.receive_request().await?;
            let response = self.handle_request(request).await;
            transport.send_response(response).await?;
        }
    }

    async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => self.handle_initialize(request).await,
            "tools/list" => self.handle_list_tools(request).await,
            "tools/call" => self.handle_call_tool(request).await,
            "resources/list" => self.handle_list_resources(request).await,
            "resources/read" => self.handle_read_resource(request).await,
            "prompts/list" => self.handle_list_prompts(request).await,
            "prompts/get" => self.handle_get_prompt(request).await,
            _ => JsonRpcResponse::error(request.id, McpError::method_not_found(&request.method)),
        }
    }

    async fn handle_initialize(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = InitializeResult {
            protocol_version: MCP_VERSION.to_string(),
            capabilities: self.capabilities.clone(),
            server_info: self.info.clone(),
            instructions: None,
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    async fn handle_list_tools(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = ListToolsResult {
            tools: self.tools.clone(),
            next_cursor: None,
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    async fn handle_call_tool(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let tool_request: CallToolRequest = match request.params.as_ref().and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(req) => req,
            None => return JsonRpcResponse::error(request.id, McpError::invalid_params("Invalid tool request")),
        };

        let handlers = self.tool_handlers.read().await;
        match handlers.get(&tool_request.name) {
            Some(handler) => {
                match handler(tool_request).await {
                    Ok(result) => JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
                    Err(error) => JsonRpcResponse::error(request.id, error),
                }
            }
            None => JsonRpcResponse::error(request.id, McpError::method_not_found(&format!("Tool not found: {}", tool_request.name))),
        }
    }

    async fn handle_list_resources(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = ListResourcesResult {
            resources: self.resources.clone(),
            next_cursor: None,
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    async fn handle_read_resource(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let resource_request: ReadResourceRequest = match request.params.as_ref().and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(req) => req,
            None => return JsonRpcResponse::error(request.id, McpError::invalid_params("Invalid resource request")),
        };

        let handlers = self.resource_handlers.read().await;
        match handlers.get("default") {
            Some(handler) => {
                match handler(resource_request).await {
                    Ok(result) => JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
                    Err(error) => JsonRpcResponse::error(request.id, error),
                }
            }
            None => JsonRpcResponse::error(request.id, McpError::method_not_found("No resource handler configured")),
        }
    }

    async fn handle_list_prompts(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let result = ListPromptsResult {
            prompts: self.prompts.clone(),
            next_cursor: None,
        };
        JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap())
    }

    async fn handle_get_prompt(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        let prompt_request: GetPromptRequest = match request.params.as_ref().and_then(|p| serde_json::from_value(p.clone()).ok()) {
            Some(req) => req,
            None => return JsonRpcResponse::error(request.id, McpError::invalid_params("Invalid prompt request")),
        };

        let handlers = self.prompt_handlers.read().await;
        match handlers.get(&prompt_request.name) {
            Some(handler) => {
                match handler(prompt_request).await {
                    Ok(result) => JsonRpcResponse::success(request.id, serde_json::to_value(result).unwrap()),
                    Err(error) => JsonRpcResponse::error(request.id, error),
                }
            }
            None => JsonRpcResponse::error(request.id, McpError::method_not_found(&format!("Prompt not found: {}", prompt_request.name))),
        }
    }
}