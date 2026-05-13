// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # MCP settings serialization tests.

#![cfg(all(feature = "cli", feature = "mcp"))]

use autogpt::cli::settings::{GlobalSettings, SettingsManager};
use autogpt::mcp::settings::{McpOAuthConfig, McpServerConfig, McpTransport};
use std::collections::HashMap;
use tempfile::TempDir;

fn stdio_config(name: &str) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        transport: McpTransport::Stdio,
        command: Some("docker".to_string()),
        args: vec![
            "run".to_string(),
            "-i".to_string(),
            format!("ghcr.io/{name}/mcp-server"),
        ],
        url: None,
        http_url: None,
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 500_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec![],
        description: Some(format!("{name} MCP server")),
        oauth: None,
    }
}

fn sse_config(name: &str, url: &str) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        transport: McpTransport::Sse,
        command: None,
        args: vec![],
        url: Some(url.to_string()),
        http_url: None,
        headers: {
            let mut m = HashMap::new();
            m.insert("Authorization".to_string(), "Bearer tok123".to_string());
            m
        },
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 30_000,
        trust: true,
        include_tools: vec!["search".to_string()],
        exclude_tools: vec![],
        description: None,
        oauth: None,
    }
}

fn http_config(name: &str, url: &str) -> McpServerConfig {
    McpServerConfig {
        name: name.to_string(),
        transport: McpTransport::Http,
        command: None,
        args: vec![],
        url: None,
        http_url: Some(url.to_string()),
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 500_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec!["internal_debug".to_string()],
        description: Some("HTTP endpoint".to_string()),
        oauth: None,
    }
}

fn settings_manager_in(tmp: &TempDir) -> SettingsManager {
    SettingsManager::with_path(tmp.path().join("settings.json"))
}

#[test]
fn transport_default_is_stdio() {
    assert_eq!(McpTransport::default(), McpTransport::Stdio);
}

#[test]
fn transport_display() {
    assert_eq!(McpTransport::Stdio.to_string(), "stdio");
    assert_eq!(McpTransport::Sse.to_string(), "sse");
    assert_eq!(McpTransport::Http.to_string(), "http");
}

#[test]
fn transport_roundtrip_serde() {
    for variant in [McpTransport::Stdio, McpTransport::Sse, McpTransport::Http] {
        let serialized = serde_json::to_string(&variant).unwrap();
        let restored: McpTransport = serde_json::from_str(&serialized).unwrap();
        assert_eq!(restored, variant);
    }
}

#[test]
fn stdio_config_connection_display_includes_command_and_args() {
    let cfg = stdio_config("github");
    let display = cfg.connection_display();
    assert!(display.starts_with("docker"), "got: {display}");
    assert!(display.contains("run"), "got: {display}");
}

#[test]
fn sse_config_connection_display_returns_url() {
    let url = "https://api.wiseai.dev/mcp/sse";
    let cfg = sse_config("myapi", url);
    assert_eq!(cfg.connection_display(), url);
}

#[test]
fn http_config_connection_display_returns_http_url() {
    let url = "https://api.wiseai.dev/mcp";
    let cfg = http_config("myhttp", url);
    assert_eq!(cfg.connection_display(), url);
}

#[test]
fn unconfigured_config_connection_display_fallback() {
    let cfg = McpServerConfig {
        name: "empty".to_string(),
        transport: McpTransport::Stdio,
        command: None,
        args: vec![],
        url: None,
        http_url: None,
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 500_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec![],
        description: None,
        oauth: None,
    };
    assert_eq!(cfg.connection_display(), "(unconfigured)");
}

#[test]
fn config_roundtrip_serde_stdio() {
    let cfg = stdio_config("github");
    let serialized = serde_json::to_string_pretty(&cfg).unwrap();
    let restored: McpServerConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored.name, cfg.name);
    assert_eq!(restored.transport, McpTransport::Stdio);
    assert_eq!(restored.command, cfg.command);
    assert_eq!(restored.args, cfg.args);
}

#[test]
fn config_roundtrip_serde_sse_with_headers() {
    let cfg = sse_config("search", "https://search.wiseai.dev/mcp/sse");
    let serialized = serde_json::to_string_pretty(&cfg).unwrap();
    let restored: McpServerConfig = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored.transport, McpTransport::Sse);
    assert_eq!(restored.url, cfg.url);
    assert_eq!(
        restored.headers.get("Authorization"),
        Some(&"Bearer tok123".to_string())
    );
    assert!(restored.trust);
    assert_eq!(restored.include_tools, vec!["search".to_string()]);
}

#[test]
fn config_default_timeout_is_500_000() {
    let cfg = stdio_config("test");
    assert_eq!(cfg.timeout_ms, 500_000);
}

#[test]
fn config_oauth_roundtrip() {
    let mut cfg = stdio_config("oauth-server");
    cfg.oauth = Some(McpOAuthConfig {
        enabled: true,
        client_id: Some("client123".to_string()),
        client_secret: Some("secret456".to_string()),
        redirect_uri: None,
        scopes: vec!["read".to_string(), "write".to_string()],
        authorization_url: Some("https://auth.wiseai.dev/oauth/authorize".to_string()),
        token_url: Some("https://auth.wiseai.dev/oauth/token".to_string()),
    });
    let serialized = serde_json::to_string_pretty(&cfg).unwrap();
    let restored: McpServerConfig = serde_json::from_str(&serialized).unwrap();
    let oauth = restored.oauth.unwrap();
    assert!(oauth.enabled);
    assert_eq!(oauth.client_id, Some("client123".to_string()));
    assert_eq!(oauth.scopes, vec!["read", "write"]);
}

#[test]
fn global_settings_default_values() {
    let s = GlobalSettings::default();
    assert!(!s.yolo);
    assert!(s.session.is_none());
    assert!(!s.mixture);
    assert_eq!(s.provider, "gemini");
    assert!(s.mcp.is_empty());
    assert!(!s.verbose);
    assert_eq!(s.max_retries, 3);
    assert!(s.auto_browse);
}

#[test]
fn global_settings_roundtrip_with_mcp() {
    let mut settings = GlobalSettings::default();
    settings
        .mcp
        .insert("github".to_string(), stdio_config("github"));
    settings.mcp.insert(
        "search".to_string(),
        sse_config("search", "https://s.wiseai.dev"),
    );
    let serialized = serde_json::to_string_pretty(&settings).unwrap();
    let restored: GlobalSettings = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored.mcp.len(), 2);
    assert!(restored.mcp.contains_key("github"));
    assert!(restored.mcp.contains_key("search"));
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn global_settings_mcp_allowed_excluded_roundtrip() {
    let mut settings = GlobalSettings::default();
    settings.mcp_allowed = vec!["github".to_string()];
    settings.mcp_excluded = vec!["dangerous".to_string()];
    let serialized = serde_json::to_string_pretty(&settings).unwrap();
    let restored: GlobalSettings = serde_json::from_str(&serialized).unwrap();
    assert_eq!(restored.mcp_allowed, ["github"]);
    assert_eq!(restored.mcp_excluded, ["dangerous"]);
}

#[test]
fn settings_manager_creates_file_with_defaults_when_missing() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    assert!(!mgr.path().exists());
    let settings = mgr.load().unwrap();
    assert!(mgr.path().exists());
    assert!(settings.mcp.is_empty());
    assert_eq!(settings.max_retries, 3);
}

#[test]
#[allow(clippy::field_reassign_with_default)]
fn settings_manager_persists_and_reloads() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    let mut s = GlobalSettings::default();
    s.yolo = true;
    s.provider = "anthropic".to_string();
    mgr.save(&s).unwrap();
    let loaded = mgr.load().unwrap();
    assert!(loaded.yolo);
    assert_eq!(loaded.provider, "anthropic");
}

#[test]
fn settings_manager_add_mcp_server() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    let updated = mgr.add_mcp_server(stdio_config("github")).unwrap();
    assert!(updated.mcp.contains_key("github"));
    let reloaded = mgr.load().unwrap();
    assert!(reloaded.mcp.contains_key("github"));
}

#[test]
fn settings_manager_add_overwrites_existing() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    mgr.add_mcp_server(stdio_config("github")).unwrap();
    let mut updated_cfg = stdio_config("github");
    updated_cfg.trust = true;
    updated_cfg.description = Some("Updated".to_string());
    let s = mgr.add_mcp_server(updated_cfg).unwrap();
    assert!(s.mcp["github"].trust);
    assert_eq!(s.mcp["github"].description, Some("Updated".to_string()));
}

#[test]
fn settings_manager_remove_existing_server() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    mgr.add_mcp_server(stdio_config("github")).unwrap();
    let (s, existed) = mgr.remove_mcp_server("github").unwrap();
    assert!(existed);
    assert!(!s.mcp.contains_key("github"));
}

#[test]
fn settings_manager_remove_nonexistent_returns_false() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    let (_, existed) = mgr.remove_mcp_server("nope").unwrap();
    assert!(!existed);
}

#[test]
fn settings_manager_multiple_servers_persist() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    mgr.add_mcp_server(stdio_config("server-a")).unwrap();
    mgr.add_mcp_server(sse_config("server-b", "https://b.wiseai.dev"))
        .unwrap();
    mgr.add_mcp_server(http_config("server-c", "https://c.wiseai.dev"))
        .unwrap();
    let s = mgr.load().unwrap();
    assert_eq!(s.mcp.len(), 3);
    assert_eq!(s.mcp["server-b"].transport, McpTransport::Sse);
    assert_eq!(s.mcp["server-c"].transport, McpTransport::Http);
}

#[test]
fn settings_manager_rejects_malformed_json() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("settings.json");
    std::fs::write(&path, b"{ not valid json }").unwrap();
    let mgr = SettingsManager::with_path(path);
    assert!(mgr.load().is_err());
}

#[test]
fn settings_manager_env_pairs_preserved() {
    let tmp = TempDir::new().unwrap();
    let mgr = settings_manager_in(&tmp);
    let mut cfg = stdio_config("tools");
    cfg.env
        .insert("GITHUB_TOKEN".to_string(), "$MY_GH_TOKEN".to_string());
    mgr.add_mcp_server(cfg).unwrap();
    let s = mgr.load().unwrap();
    assert_eq!(
        s.mcp["tools"].env.get("GITHUB_TOKEN"),
        Some(&"$MY_GH_TOKEN".to_string())
    );
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
