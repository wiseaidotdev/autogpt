// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # `mcp` CLI command handlers.

#[cfg(all(feature = "cli", feature = "mcp"))]
use {
    crate::cli::settings::SettingsManager,
    crate::cli::tui::{print_success, print_warning, render_mcp_inspect, render_mcp_list},
    crate::mcp::client::McpClient,
    crate::mcp::settings::{McpServerConfig, McpTransport},
    anyhow::{Result, anyhow},
    colored::Colorize,
    std::collections::HashMap,
    tracing::info,
};

/// Adds a new MCP server to `~/.autogpt/settings.json`.
///
/// # Arguments
///
/// * `name`         - Unique server label used as the map key.
/// * `command_or_url` - Command path (stdio) **or** URL (http / sse).
/// * `args`         - Extra command-line arguments for stdio servers.
/// * `transport`    - Transport type string: `"stdio"`, `"http"`, or `"sse"`.
/// * `env_pairs`    - `KEY=VALUE` strings for the server's environment variables.
/// * `headers`      - `Key: Value` strings for HTTP headers.
/// * `timeout_ms`   - Connection timeout in milliseconds.
/// * `trust`        - Whether to bypass all tool-call confirmation prompts.
/// * `description`  - Optional human-readable description.
/// * `include_tools` - Comma-separated allowlist of tool names.
/// * `exclude_tools` - Comma-separated blocklist of tool names.
#[cfg(all(feature = "cli", feature = "mcp"))]
#[allow(clippy::too_many_arguments)]
pub fn cmd_mcp_add(
    name: &str,
    command_or_url: &str,
    args: Vec<String>,
    transport: &str,
    env_pairs: Vec<String>,
    headers: Vec<String>,
    timeout_ms: u64,
    trust: bool,
    description: Option<String>,
    include_tools: Vec<String>,
    exclude_tools: Vec<String>,
) -> Result<()> {
    let transport_kind = match transport.to_lowercase().as_str() {
        "sse" => McpTransport::Sse,
        "http" => McpTransport::Http,
        _ => McpTransport::Stdio,
    };

    let mut env: HashMap<String, String> = HashMap::new();
    for pair in &env_pairs {
        if let Some((k, v)) = pair.split_once('=') {
            env.insert(k.to_string(), v.to_string());
        }
    }

    let mut header_map: HashMap<String, String> = HashMap::new();
    for header in &headers {
        if let Some((k, v)) = header.split_once(':') {
            header_map.insert(k.trim().to_string(), v.trim().to_string());
        }
    }

    let (command, url, http_url) = match transport_kind {
        McpTransport::Stdio => (Some(command_or_url.to_string()), None, None),
        McpTransport::Sse => (None, Some(command_or_url.to_string()), None),
        McpTransport::Http => (None, None, Some(command_or_url.to_string())),
    };

    let include: Vec<String> = include_tools
        .iter()
        .flat_map(|s| s.split(',').map(|t| t.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect();

    let exclude: Vec<String> = exclude_tools
        .iter()
        .flat_map(|s| s.split(',').map(|t| t.trim().to_string()))
        .filter(|s| !s.is_empty())
        .collect();

    let config = McpServerConfig {
        name: name.to_string(),
        transport: transport_kind,
        command,
        args,
        url,
        http_url,
        headers: header_map,
        env,
        cwd: None,
        timeout_ms,
        trust,
        include_tools: include,
        exclude_tools: exclude,
        description,
        oauth: None,
    };

    let mgr = SettingsManager::new();
    mgr.add_mcp_server(config)?;

    print_success(&format!(
        "MCP server '{}' added to ~/.autogpt/settings.json",
        name
    ));
    info!(
        "  {}  transport: {}  connection: {}",
        name.bright_magenta().bold(),
        transport.bright_cyan(),
        command_or_url.bright_black()
    );
    Ok(())
}

/// Lists all registered MCP servers, connecting to them to show live status.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn cmd_mcp_list() -> Result<()> {
    let mgr = SettingsManager::new();
    let settings = mgr.load()?;

    if settings.mcp.is_empty() {
        print_warning("No MCP servers configured. Use `autogpt mcp add` to register one.");
        return Ok(());
    }

    let mut infos = Vec::new();
    for (name, config) in &settings.mcp {
        let mut client = McpClient::new(name.clone());
        let _ = client.connect(config);
        let desc = config.description.clone().unwrap_or_default();
        infos.push(client.to_server_info(&desc));
    }

    render_mcp_list(&infos);
    Ok(())
}

/// Removes an MCP server from `~/.autogpt/settings.json`.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn cmd_mcp_remove(name: &str) -> Result<()> {
    let mgr = SettingsManager::new();
    let (_, existed) = mgr.remove_mcp_server(name)?;
    if existed {
        print_success(&format!("MCP server '{}' removed.", name));
    } else {
        print_warning(&format!("MCP server '{}' was not found in settings.", name));
    }
    Ok(())
}

/// Displays detailed information about a single registered MCP server.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn cmd_mcp_inspect(name: &str) -> Result<()> {
    let mgr = SettingsManager::new();
    let settings = mgr.load()?;

    let config = settings
        .mcp
        .get(name)
        .ok_or_else(|| {
            anyhow!(
                "MCP server '{}' is not registered. Run `autogpt mcp list`.",
                name
            )
        })?
        .clone();

    let mut client = McpClient::new(name.to_string());
    let _ = client.connect(&config);
    let desc = config.description.clone().unwrap_or_default();
    let info = client.to_server_info(&desc);

    render_mcp_inspect(&info, &config);
    Ok(())
}

/// Calls a specific tool on a registered MCP server.
#[cfg(all(feature = "cli", feature = "mcp"))]
pub fn cmd_mcp_call(server_name: &str, tool_name: &str, args_raw: Vec<String>) -> Result<()> {
    let mgr = SettingsManager::new();
    let settings = mgr.load()?;

    let config = settings
        .mcp
        .get(server_name)
        .ok_or_else(|| {
            anyhow!(
                "MCP server '{}' is not registered. Run `autogpt mcp list`.",
                server_name
            )
        })?
        .clone();

    let mut client = McpClient::new(server_name.to_string());
    client.connect(&config)?;

    use serde_json::Value;

    let args_str = args_raw.join(" ");
    let args_str_trimmed = args_str.trim().trim_matches('\'').trim_matches('"');
    let args: HashMap<String, Value> = if args_str_trimmed.is_empty() {
        HashMap::new()
    } else {
        match serde_json::from_str(args_str_trimmed) {
            Ok(v) => v,
            Err(_) => {
                let mut map = HashMap::new();
                for pair in args_raw {
                    if let Some((k, v)) = pair.split_once('=') {
                        let val = match serde_json::from_str(v) {
                            Ok(json_val) => json_val,
                            Err(_) => Value::String(v.to_string()),
                        };
                        map.insert(k.to_string(), val);
                    }
                }
                map
            }
        }
    };

    let result = client.call_tool(tool_name, args)?;
    if result.success {
        print_success(&format!(
            "Tool '{}' called successfully on '{}'",
            tool_name, server_name
        ));
        println!("{}", result.content);
        if let Some(ref data) = result.data {
            println!("{}", serde_json::to_string_pretty(data)?);
        }
    } else {
        let err_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
        print_warning(&format!(
            "Tool '{}' failed on '{}': {}",
            tool_name, server_name, err_msg
        ));
    }

    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
