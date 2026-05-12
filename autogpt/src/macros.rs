// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_export]
macro_rules! agents {
    ( $($agent:expr),* $(,)? ) => {
        vec![
            $(
                std::sync::Arc::new(tokio::sync::Mutex::new(Box::new($agent) as Box<dyn AgentFunctions>))
            ),*
        ]
    };
}

/// Registers a single MCP server on an agent.
///
/// # Usage
///
/// ```rust
/// use autogpt::mcp_server;
/// use autogpt::agents::agent::AgentGPT;
/// use autogpt::cli::settings::{McpServerConfig, McpTransport};
/// use std::collections::HashMap;
///
/// let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
/// mcp_server!(agent, McpServerConfig {
///     name: "github".to_string(), transport: McpTransport::Stdio,
///     command: Some("docker".to_string()),
///     args: vec!["run".into(), "-i".into(), "ghcr.io/github/github-mcp-server".into()],
///     url: None, http_url: None, headers: Default::default(), env: Default::default(),
///     cwd: None, timeout_ms: 500_000, trust: false,
///     include_tools: vec![], exclude_tools: vec![],
///     description: Some("GitHub tools".to_string()), oauth: None,
/// });
/// ```
#[cfg(feature = "mcp")]
#[macro_export]
macro_rules! mcp_server {
    ($agent:expr, $config:expr) => {
        $agent.with_mcp_server($config)
    };
}

/// Registers multiple MCP servers on an agent at once.
///
/// # Usage
///
/// ```rust
/// use autogpt::with_mcp_servers;
/// use autogpt::agents::agent::AgentGPT;
/// use autogpt::cli::settings::{McpServerConfig, McpTransport};
/// use std::collections::HashMap;
///
/// let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
/// with_mcp_servers!(agent, [
///     McpServerConfig {
///         name: "github".to_string(), transport: McpTransport::Stdio,
///         command: Some("docker".to_string()),
///         args: vec!["run".into()], url: None, http_url: None,
///         headers: Default::default(), env: Default::default(),
///         cwd: None, timeout_ms: 500_000, trust: false,
///         include_tools: vec![], exclude_tools: vec![],
///         description: None, oauth: None,
///     },
/// ]);
/// ```
#[cfg(feature = "mcp")]
#[macro_export]
macro_rules! with_mcp_servers {
    ($agent:expr, [ $($config:expr),* $(,)? ]) => {
        {
            $( $agent.with_mcp_server($config); )*
        }
    };
}
