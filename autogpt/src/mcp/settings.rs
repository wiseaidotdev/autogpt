// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP server configuration types.
//!
//! Defines [`McpServerConfig`], [`McpTransport`], and [`McpOAuthConfig`]: the
//! core data structures used to register and persist MCP server connections.
//!
//! These types are serialisable so they can be stored in
//! `~/.autogpt/settings.json` (via the `cli` feature) or defined inline in
//! code (requires only the `mcp` feature).

use {
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
};

/// Supported transport mechanisms for MCP.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum McpTransport {
    /// Spawn a subprocess and communicate via stdin / stdout.
    #[default]
    Stdio,
    /// Connect to a Server-Sent Events endpoint.
    Sse,
    /// Connect using streamable HTTP.
    Http,
}

impl std::fmt::Display for McpTransport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stdio => write!(f, "stdio"),
            Self::Sse => write!(f, "sse"),
            Self::Http => write!(f, "http"),
        }
    }
}

/// OAuth 2.0 configuration for remote MCP servers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct McpOAuthConfig {
    /// Whether OAuth authentication is enabled for this server.
    #[serde(default)]
    pub enabled: bool,

    /// OAuth client identifier (optional when dynamic registration is used).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    /// OAuth client secret (optional for public clients).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    /// Custom redirect URI; defaults to a random OS-assigned localhost port.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redirect_uri: Option<String>,

    /// Required OAuth scopes.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scopes: Vec<String>,

    /// OAuth authorization endpoint (auto-discovered when omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub authorization_url: Option<String>,

    /// OAuth token endpoint (auto-discovered when omitted).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token_url: Option<String>,
}

/// Full configuration for a single registered MCP server.
///
/// Exactly one of `command` (stdio), `url` (SSE), or `http_url` (HTTP) must be set.
///
/// # Examples
///
/// ```rust
/// use autogpt::mcp::settings::{McpServerConfig, McpTransport};
/// use std::collections::HashMap;
///
/// let cfg = McpServerConfig {
///     name: "github".to_string(),
///     transport: McpTransport::Stdio,
///     command: Some("docker".to_string()),
///     args: vec!["run".into(), "-i".into(), "ghcr.io/github/github-mcp-server".into()],
///     url: None, http_url: None,
///     headers: HashMap::new(),
///     env: HashMap::new(),
///     cwd: None,
///     timeout_ms: 30_000,
///     trust: false,
///     include_tools: vec![],
///     exclude_tools: vec![],
///     description: Some("GitHub MCP server".to_string()),
///     oauth: None,
/// };
/// assert_eq!(cfg.name, "github");
/// ```
#[derive(Default, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Human-readable label shown in UIs. Mirrors the map key.
    pub name: String,

    /// Transport type derived from which connection field is populated.
    #[serde(default)]
    pub transport: McpTransport,

    /// Executable path for stdio transport.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,

    /// Command-line arguments for stdio transport.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,

    /// SSE endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Streamable HTTP endpoint URL.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_url: Option<String>,

    /// Custom HTTP headers sent with every request (SSE / HTTP transports).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub headers: HashMap<String, String>,

    /// Extra environment variables passed to the server process.
    ///
    /// Values may reference variables using `$VAR_NAME` or `${VAR_NAME}`.
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub env: HashMap<String, String>,

    /// Working directory for stdio transport processes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,

    /// Connection timeout in milliseconds (default: `30_000`).
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// When `true` all tool-call confirmation prompts are bypassed.
    #[serde(default)]
    pub trust: bool,

    /// Allowlist of tool names to expose from this server. Empty means all.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include_tools: Vec<String>,

    /// Blocklist of tool names to hide from this server.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub exclude_tools: Vec<String>,

    /// Optional short description shown in the list / inspect views.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// OAuth configuration for remote servers that require it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oauth: Option<McpOAuthConfig>,
}

fn default_timeout() -> u64 {
    30_000
}

impl McpServerConfig {
    /// Returns the connection URL for display purposes regardless of transport type.
    pub fn connection_display(&self) -> String {
        if let Some(ref cmd) = self.command {
            let args = self.args.join(" ");
            if args.is_empty() {
                cmd.clone()
            } else {
                format!("{cmd} {args}")
            }
        } else if let Some(ref url) = self.url {
            url.clone()
        } else if let Some(ref url) = self.http_url {
            url.clone()
        } else {
            "(unconfigured)".to_string()
        }
    }
}
