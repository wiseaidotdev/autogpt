// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg(all(feature = "cli", feature = "mcp"))]

use autogpt::cli::settings::SettingsManager;
use autogpt::mcp::client::McpClient;
use autogpt::mcp::settings::{McpServerConfig, McpTransport};
use autogpt::mcp::types::McpServerStatus;
use std::collections::HashMap;
use std::process::Command;
use tempfile::TempDir;

fn has_npx() -> bool {
    Command::new("npx")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[tokio::test]
async fn test_live_mcp_everything_stdio() {
    if !has_npx() {
        eprintln!("npx not found, skipping live mcp test");
        return;
    }

    let tmp = TempDir::new().unwrap();
    let settings_path = tmp.path().join("settings.json");
    let mgr = SettingsManager::with_path(settings_path.clone());

    let config = McpServerConfig {
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
        description: Some("Everything MCP server".to_string()),
        oauth: None,
    };

    mgr.add_mcp_server(config.clone()).unwrap();
    let settings = mgr.load().unwrap();
    assert!(settings.mcp.contains_key("everything"));

    let mut client = McpClient::new("everything".to_string());
    client
        .connect(&config)
        .expect("Failed to connect to everything server");

    let info = client.to_server_info("Everything MCP server");
    assert_eq!(info.status, McpServerStatus::Connected);
    assert!(
        !info.tools.is_empty(),
        "No tools found on everything server"
    );

    let echo_tool = info.tools.iter().find(|t| t.name == "echo");
    assert!(echo_tool.is_some(), "echo tool not found");

    let mut args = HashMap::new();
    args.insert("message".to_string(), serde_json::json!("hello world"));
    let result = client
        .call_tool("echo", args)
        .expect("Failed to call echo tool");

    let result_str = serde_json::to_string(&result).unwrap();
    assert!(result_str.contains("hello world"));

    let (_, existed) = mgr.remove_mcp_server("everything").unwrap();
    assert!(existed);
    let settings = mgr.load().unwrap();
    assert!(!settings.mcp.contains_key("everything"));
}

#[tokio::test]
async fn test_mcp_connection_timeout() {
    let mut client = McpClient::new("slow".to_string());
    let config = McpServerConfig {
        name: "slow".to_string(),
        transport: McpTransport::Stdio,
        command: Some("sleep".to_string()),
        args: vec!["10".into()],
        url: None,
        http_url: None,
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 100,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec![],
        description: None,
        oauth: None,
    };

    let res = client.connect(&config);
    assert!(res.is_err(), "Connection should have timed out");
    let err = res.unwrap_err().to_string();
    println!("Actual error: {}", err);
    assert!(
        err.contains("timeout") || err.contains("timed out"),
        "Error should mention timeout but was: {}",
        err
    );
}
