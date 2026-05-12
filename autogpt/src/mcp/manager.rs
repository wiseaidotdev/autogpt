// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP manager.
//!
//! Loads all registered MCP servers from configuration, connects to each one in
//! turn, and provides a unified interface for looking up tools and dispatching
//! tool-call requests.

use {
    crate::mcp::client::McpClient,
    crate::mcp::settings::McpServerConfig,
    crate::mcp::types::{McpServerInfo, McpServerStatus, McpTool, McpToolResult},
    anyhow::{Result, anyhow},
    serde_json::Value,
    std::collections::HashMap,
    tracing::{info, warn},
};

/// Orchestrates connections to all configured MCP servers and exposes their tools.
///
/// # Example
///
/// ```rust
/// use autogpt::mcp::manager::McpManager;
/// use autogpt::mcp::settings::McpServerConfig;
/// use std::collections::HashMap;
///
/// let mut servers = HashMap::new();
/// let mut mgr = McpManager::new(servers, vec![], vec![]);
/// mgr.connect_all();
/// ```
pub struct McpManager {
    clients: HashMap<String, McpClient>,
    configs: HashMap<String, McpServerConfig>,
}

impl McpManager {
    /// Builds a manager pre-populated with the given server map, respecting the
    /// allow/exclude lists.
    pub fn new(
        servers: HashMap<String, McpServerConfig>,
        allowed: Vec<String>,
        excluded: Vec<String>,
    ) -> Self {
        let configs: HashMap<String, McpServerConfig> = servers
            .into_iter()
            .filter(|(name, _)| {
                let is_excluded = excluded.contains(name);
                let is_allowed = allowed.is_empty() || allowed.contains(name);
                !is_excluded && is_allowed
            })
            .collect();

        let clients = configs
            .keys()
            .map(|name| (name.clone(), McpClient::new(name.clone())))
            .collect();
        Self { clients, configs }
    }

    /// Attempts to connect to every registered server and discover its tools.
    ///
    /// Errors are logged as warnings and do not abort the process.
    pub fn connect_all(&mut self) {
        let names: Vec<String> = self.configs.keys().cloned().collect();
        for name in names {
            if let Some(config) = self.configs.get(&name).cloned()
                && let Some(client) = self.clients.get_mut(&name)
            {
                match client.connect(&config) {
                    Ok(_) => info!(
                        "MCP server '{}' connected ({} tools)",
                        name,
                        client.tools.len()
                    ),
                    Err(e) => warn!("MCP server '{}' failed to connect: {}", name, e),
                }
            }
        }
    }

    /// Returns a summary for every server (connected or not).
    pub fn server_infos(&self) -> Vec<McpServerInfo> {
        self.clients
            .values()
            .map(|c| {
                let desc = self
                    .configs
                    .get(&c.name)
                    .and_then(|cfg| cfg.description.clone())
                    .unwrap_or_default();
                c.to_server_info(&desc)
            })
            .collect()
    }

    /// Returns the `McpServerInfo` for a specific server by name.
    pub fn server_info(&self, name: &str) -> Option<McpServerInfo> {
        self.clients.get(name).map(|c| {
            let desc = self
                .configs
                .get(name)
                .and_then(|cfg| cfg.description.clone())
                .unwrap_or_default();
            c.to_server_info(&desc)
        })
    }

    /// Returns all tools from all connected servers as a flat list.
    pub fn all_tools(&self) -> Vec<McpTool> {
        self.clients
            .values()
            .filter(|c| c.status == McpServerStatus::Connected)
            .flat_map(|c| c.tools.clone())
            .collect()
    }

    /// Looks up a tool by its fully-qualified name (`mcp_{server}_{tool}`).
    pub fn find_tool(&self, fqn: &str) -> Option<(&str, &McpTool)> {
        for (name, client) in &self.clients {
            if let Some(tool) = client.tools.iter().find(|t| t.fqn == fqn) {
                return Some((name.as_str(), tool));
            }
        }
        None
    }

    /// Executes a tool call identified by its FQN.
    pub fn call_tool(&mut self, fqn: &str, args: HashMap<String, Value>) -> Result<McpToolResult> {
        let server_name = self
            .clients
            .iter()
            .find(|(_, c)| c.tools.iter().any(|t| t.fqn == fqn))
            .map(|(name, _)| name.clone())
            .ok_or_else(|| anyhow!("No MCP server found for tool FQN: {fqn}"))?;

        let tool_name = self
            .clients
            .get(&server_name)
            .and_then(|c| c.tools.iter().find(|t| t.fqn == fqn))
            .map(|t| t.name.clone())
            .ok_or_else(|| anyhow!("Tool not found: {fqn}"))?;

        self.clients
            .get_mut(&server_name)
            .ok_or_else(|| anyhow!("Client not found for server: {server_name}"))?
            .call_tool(&tool_name, args)
    }

    /// Returns the number of currently connected servers.
    pub fn connected_count(&self) -> usize {
        self.clients
            .values()
            .filter(|c| c.status == McpServerStatus::Connected)
            .count()
    }

    /// Returns the total number of configured servers.
    pub fn total_count(&self) -> usize {
        self.clients.len()
    }

    /// Returns reference to the internal configs map.
    pub fn configs(&self) -> &HashMap<String, McpServerConfig> {
        &self.configs
    }
}
