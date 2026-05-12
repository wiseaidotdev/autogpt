// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP (Model Context Protocol) module.
//!
//! Provides types, client, and manager for connecting to and using
//! external MCP tool servers from AutoGPT agents.
//!
//! Enabled with the `mcp` feature flag.

pub mod client;
pub mod manager;
pub mod settings;
pub mod types;

pub use client::McpClient;
pub use manager::McpManager;
pub use settings::{McpOAuthConfig, McpServerConfig, McpTransport};
pub use types::{
    McpServerInfo, McpServerStatus, McpTool, McpToolCall, McpToolParam, McpToolResult,
};
