// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP runtime types.
//!
//! Defines the data structures that describe discovered MCP tools, their schemas,
//! and the live connection status of each registered server.

use {
    serde::{Deserialize, Serialize},
    serde_json::Value,
    std::collections::HashMap,
};

/// Live connection state of an MCP server.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum McpServerStatus {
    /// Server has not been connected yet.
    Disconnected,
    /// Connection is currently being established.
    Connecting,
    /// Server is reachable and tools have been discovered.
    Connected,
}

impl std::fmt::Display for McpServerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disconnected => write!(f, "DISCONNECTED"),
            Self::Connecting => write!(f, "CONNECTING"),
            Self::Connected => write!(f, "CONNECTED"),
        }
    }
}

/// JSON Schema style parameter definition for a single MCP tool input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolParam {
    /// JSON Schema `type` string (e.g. `"string"`, `"integer"`, `"object"`).
    #[serde(rename = "type", default)]
    pub param_type: String,

    /// Human-readable description of the parameter.
    #[serde(default)]
    pub description: String,

    /// Whether this parameter must be provided.
    #[serde(default)]
    pub required: bool,

    /// Allowed enumeration values (when the parameter is an enum).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub enum_values: Vec<Value>,
}

/// Complete description of a tool exposed by an MCP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name as returned by the MCP server.
    pub name: String,

    /// Fully-qualified name used internally: `mcp_{server_name}_{tool_name}`.
    pub fqn: String,

    /// Human-readable description of what the tool does.
    #[serde(default)]
    pub description: String,

    /// Parameters accepted by this tool, keyed by parameter name.
    #[serde(default)]
    pub params: HashMap<String, McpToolParam>,

    /// Whether required-parameter validation is enforced client-side.
    #[serde(default)]
    pub validate_required: bool,
}

/// Summary information about a connected MCP server, including its live tools.
#[derive(Debug, Clone)]
pub struct McpServerInfo {
    /// Unique server name (matches the key in `settings.mcp`).
    pub name: String,

    /// Current connection state.
    pub status: McpServerStatus,

    /// Short description of the server's purpose (from config or introspection).
    pub description: String,

    /// Tools discovered from this server after successful connection.
    pub tools: Vec<McpTool>,

    /// Error message when the server is in `Disconnected` state due to a failure.
    pub error: Option<String>,
}

/// A request to call a specific MCP tool with the given arguments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolCall {
    /// `fqn` of the target tool.
    pub tool_fqn: String,

    /// Arguments matching the tool's parameter schema.
    pub args: HashMap<String, Value>,
}

/// The result returned by executing an MCP tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    /// Whether the execution completed without protocol-level errors.
    pub success: bool,

    /// Primary textual output from the tool (may contain JSON).
    pub content: String,

    /// Optional structured data returned alongside text content.
    pub data: Option<Value>,

    /// Error description when `success` is `false`.
    pub error: Option<String>,
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
