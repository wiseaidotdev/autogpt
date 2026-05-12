// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP manager unit tests.

#![cfg(feature = "mcp")]

use autogpt::mcp::manager::McpManager;
use autogpt::mcp::settings::{McpServerConfig, McpTransport};
use autogpt::mcp::types::McpServerStatus;
use std::collections::HashMap;

fn bad_stdio(name: &str) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        transport: McpTransport::Stdio,
        command: Some("/nonexistent/mcp-server-binary-that-does-not-exist".to_string()),
        args: vec![],
        url: None,
        http_url: None,
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 1_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec![],
        description: Some(format!("{name} (unreachable)")),
        oauth: None,
    }
}

fn servers_from(pairs: &[(&str, McpServerConfig)]) -> HashMap<String, McpServerConfig> {
    pairs
        .iter()
        .map(|(name, cfg)| (name.to_string(), cfg.clone()))
        .collect()
}

fn mgr_from(pairs: &[(&str, McpServerConfig)]) -> McpManager {
    McpManager::new(servers_from(pairs), vec![], vec![])
}

#[test]
fn manager_from_empty_settings_is_empty() {
    let mgr = McpManager::new(HashMap::new(), vec![], vec![]);
    assert_eq!(mgr.total_count(), 0);
    assert_eq!(mgr.connected_count(), 0);
}

#[test]
fn manager_creates_one_client_per_server() {
    let mgr = mgr_from(&[("a", bad_stdio("a")), ("b", bad_stdio("b"))]);
    assert_eq!(mgr.total_count(), 2);
}

#[test]
fn manager_respects_mcp_allowed_list() {
    let servers = servers_from(&[
        ("allowed", bad_stdio("allowed")),
        ("blocked", bad_stdio("blocked")),
    ]);
    let mgr = McpManager::new(servers, vec!["allowed".to_string()], vec![]);
    assert_eq!(mgr.total_count(), 1);
}

#[test]
fn manager_respects_mcp_excluded_list() {
    let servers = servers_from(&[("good", bad_stdio("good")), ("bad", bad_stdio("bad"))]);
    let mgr = McpManager::new(servers, vec![], vec!["bad".to_string()]);
    assert_eq!(mgr.total_count(), 1);
}

#[test]
fn excluded_takes_priority_over_allowed() {
    let servers = servers_from(&[("srv", bad_stdio("srv"))]);
    let mgr = McpManager::new(servers, vec!["srv".to_string()], vec!["srv".to_string()]);
    assert_eq!(mgr.total_count(), 0);
}

#[test]
fn connect_all_unreachable_stays_disconnected() {
    let mut mgr = mgr_from(&[("srv", bad_stdio("srv"))]);
    mgr.connect_all();
    assert_eq!(mgr.connected_count(), 0);
}

#[test]
fn server_infos_sets_error_on_failure() {
    let mut mgr = mgr_from(&[("broken", bad_stdio("broken"))]);
    mgr.connect_all();
    let info = mgr
        .server_infos()
        .into_iter()
        .find(|i| i.name == "broken")
        .unwrap();
    assert_eq!(info.status, McpServerStatus::Disconnected);
    assert!(info.error.is_some());
}

#[test]
fn find_tool_returns_none_when_not_connected() {
    let mut mgr = mgr_from(&[("srv", bad_stdio("srv"))]);
    mgr.connect_all();
    assert!(mgr.find_tool("mcp_srv_some_tool").is_none());
}

#[test]
fn all_tools_empty_when_not_connected() {
    let mut mgr = mgr_from(&[("a", bad_stdio("a")), ("b", bad_stdio("b"))]);
    mgr.connect_all();
    assert!(mgr.all_tools().is_empty());
}

#[test]
fn call_tool_errors_on_unknown_fqn() {
    let mut mgr = mgr_from(&[("srv", bad_stdio("srv"))]);
    mgr.connect_all();
    let result = mgr.call_tool("mcp_srv_nonexistent", HashMap::new());
    assert!(result.is_err());
}

#[test]
fn server_info_some_for_known_server() {
    let mgr = mgr_from(&[("myserver", bad_stdio("myserver"))]);
    assert!(mgr.server_info("myserver").is_some());
}

#[test]
fn server_info_none_for_unknown_server() {
    let mgr = McpManager::new(HashMap::new(), vec![], vec![]);
    assert!(mgr.server_info("unknown").is_none());
}

#[test]
fn server_info_description_from_config() {
    let mut cfg = bad_stdio("described");
    cfg.description = Some("Custom description".to_string());
    let mgr = mgr_from(&[("described", cfg)]);
    let info = mgr.server_info("described").unwrap();
    assert_eq!(info.description, "Custom description");
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
