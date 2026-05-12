//! # MCP Manager Example
//!
//! Demonstrates direct use of `McpManager` for connecting to MCP servers,
//! discovering tools, and dispatching tool calls, without using any LLM.

use autogpt::prelude::*;
use std::collections::HashMap;

fn main() {
    let demo_path = "/tmp/mcp-demo";
    std::fs::create_dir_all(demo_path).ok();
    std::fs::write(
        format!("{demo_path}/hello.txt"),
        "Hello from MCP Manager Example!",
    )
    .ok();

    let mut servers: HashMap<String, McpServerConfig> = HashMap::new();

    servers.insert(
        "everything".to_string(),
        McpServerConfig {
            name: "everything".to_string(),
            transport: McpTransport::Stdio,
            command: Some("npx".to_string()),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-everything".into(),
            ],
            url: None,
            http_url: None,
            headers: HashMap::new(),
            env: HashMap::new(),
            cwd: None,
            timeout_ms: 30_000,
            trust: true,
            include_tools: vec![],
            exclude_tools: vec![],
            description: Some("Miscellaneous tool collection".to_string()),
            oauth: None,
        },
    );

    servers.insert(
        "memory".to_string(),
        McpServerConfig {
            name: "memory".to_string(),
            transport: McpTransport::Stdio,
            command: Some("npx".to_string()),
            args: vec!["-y".into(), "@modelcontextprotocol/server-memory".into()],
            url: None,
            http_url: None,
            headers: HashMap::new(),
            env: HashMap::new(),
            cwd: None,
            timeout_ms: 30_000,
            trust: true,
            include_tools: vec![],
            exclude_tools: vec![],
            description: Some("In-memory knowledge graph".to_string()),
            oauth: None,
        },
    );

    let allowed: Vec<String> = vec![];
    let excluded: Vec<String> = vec![];

    let mut mgr = McpManager::new(servers, allowed, excluded);
    println!("Configured {} server(s)", mgr.total_count());

    mgr.connect_all();
    println!("Connected to {} server(s)\n", mgr.connected_count());

    for info in mgr.server_infos() {
        let status = if info.error.is_some() { "✗" } else { "✓" };
        println!(
            "{} {} - {} tool(s) - {}",
            status,
            info.name,
            info.tools.len(),
            info.description,
        );

        for tool in &info.tools {
            println!("  └─ {} (FQN: {})", tool.name, tool.fqn);
            println!("     {}", tool.description);
            for (pname, param) in &tool.params {
                let req = if param.required { "*" } else { " " };
                println!(
                    "     {}{}: {} - {}",
                    req, pname, param.param_type, param.description
                );
            }
        }

        if let Some(ref err) = info.error {
            println!("  ⚠ Error: {}", err);
        }
        println!();
    }

    let fqn = "mcp_everything_echo";
    match mgr.find_tool(fqn) {
        Some((server_name, tool)) => {
            println!("Found tool '{}' on server '{}'", tool.name, server_name);
        }
        None => {
            println!("Tool '{}' not found (servers may be offline)", fqn);
        }
    }

    let mut args = HashMap::new();
    args.insert(
        "message".to_string(),
        serde_json::json!("Hello from MCP Manager Example!"),
    );

    match mgr.call_tool(fqn, args) {
        Ok(result) => {
            if result.success {
                println!("Tool result: {}", result.content);
            } else {
                eprintln!("Tool error: {}", result.error.unwrap_or_default());
            }
        }
        Err(e) => {
            eprintln!("Call failed: {} (server may be offline)", e);
        }
    }
}
