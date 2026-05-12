// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Agent MCP integration tests.

#![cfg(feature = "mcp")]

use autogpt::agents::agent::AgentGPT;
use autogpt::mcp::settings::{McpServerConfig, McpTransport};
use autogpt::traits::agent::Agent;
use std::collections::HashMap;

fn github_stdio() -> McpServerConfig {
    McpServerConfig {
        name: "github".to_string(),
        transport: McpTransport::Stdio,
        command: Some("docker".to_string()),
        args: vec![
            "run".to_string(),
            "-i".to_string(),
            "ghcr.io/github/github-mcp-server".to_string(),
        ],
        url: None,
        http_url: None,
        headers: HashMap::new(),
        env: {
            let mut e = HashMap::new();
            e.insert(
                "GITHUB_PERSONAL_ACCESS_TOKEN".to_string(),
                "$GITHUB_TOKEN".to_string(),
            );
            e
        },
        cwd: None,
        timeout_ms: 500_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec!["delete_file".to_string()],
        description: Some("GitHub tools via MCP".to_string()),
        oauth: None,
    }
}

fn search_sse(url: &str) -> McpServerConfig {
    McpServerConfig {
        name: "search".to_string(),
        transport: McpTransport::Sse,
        command: None,
        args: vec![],
        url: Some(url.to_string()),
        http_url: None,
        headers: {
            let mut h = HashMap::new();
            h.insert("Authorization".to_string(), "Bearer my-key".to_string());
            h
        },
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 30_000,
        trust: true,
        include_tools: vec!["web_search".to_string()],
        exclude_tools: vec![],
        description: Some("Search engine via SSE".to_string()),
        oauth: None,
    }
}

fn stripe_http(url: &str) -> McpServerConfig {
    McpServerConfig {
        name: "stripe".to_string(),
        transport: McpTransport::Http,
        command: None,
        args: vec![],
        url: None,
        http_url: Some(url.to_string()),
        headers: HashMap::new(),
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 60_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec![],
        description: Some("Stripe payments via HTTP".to_string()),
        oauth: None,
    }
}

#[test]
fn agent_initially_has_no_mcp_servers() {
    let agent = AgentGPT::new_borrowed("Dev", "Write code");
    assert!(agent.mcp_servers().is_empty());
}

#[test]
fn with_mcp_server_appends_config() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(github_stdio());
    assert_eq!(agent.mcp_servers().len(), 1);
    assert_eq!(agent.mcp_servers()[0].name, "github");
}

#[test]
fn with_mcp_server_is_chainable() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent
        .with_mcp_server(github_stdio())
        .with_mcp_server(search_sse("https://search.wiseai.dev/mcp/sse"))
        .with_mcp_server(stripe_http("https://mcp.stripe.com"));
    assert_eq!(agent.mcp_servers().len(), 3);
}

#[test]
fn with_mcp_server_preserves_transport_type() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent
        .with_mcp_server(github_stdio())
        .with_mcp_server(search_sse("https://s.wiseai.dev"))
        .with_mcp_server(stripe_http("https://api.stripe.com/mcp"));
    assert_eq!(agent.mcp_servers()[0].transport, McpTransport::Stdio);
    assert_eq!(agent.mcp_servers()[1].transport, McpTransport::Sse);
    assert_eq!(agent.mcp_servers()[2].transport, McpTransport::Http);
}

#[test]
fn with_mcp_server_preserves_env_tokens() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(github_stdio());
    assert_eq!(
        agent.mcp_servers()[0]
            .env
            .get("GITHUB_PERSONAL_ACCESS_TOKEN"),
        Some(&"$GITHUB_TOKEN".to_string())
    );
}

#[test]
fn with_mcp_server_preserves_exclude_tools() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(github_stdio());
    assert_eq!(agent.mcp_servers()[0].exclude_tools, vec!["delete_file"]);
}

#[test]
fn with_mcp_server_preserves_include_tools() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(search_sse("https://s.wiseai.dev"));
    assert_eq!(agent.mcp_servers()[0].include_tools, vec!["web_search"]);
}

#[test]
fn with_mcp_server_preserves_trust_flag() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(search_sse("https://s.wiseai.dev"));
    assert!(agent.mcp_servers()[0].trust);
}

#[test]
fn with_mcp_server_preserves_description() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(github_stdio());
    assert_eq!(
        agent.mcp_servers()[0].description,
        Some("GitHub tools via MCP".to_string())
    );
}

#[test]
fn with_mcp_server_preserves_headers() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(search_sse("https://s.wiseai.dev"));
    assert_eq!(
        agent.mcp_servers()[0].headers.get("Authorization"),
        Some(&"Bearer my-key".to_string())
    );
}

#[test]
fn mcp_server_macro_appends_config() {
    use autogpt::mcp_server;
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    mcp_server!(agent, github_stdio());
    assert_eq!(agent.mcp_servers().len(), 1);
    assert_eq!(agent.mcp_servers()[0].name, "github");
}

#[test]
fn mcp_server_macro_can_be_called_multiple_times() {
    use autogpt::mcp_server;
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    mcp_server!(agent, github_stdio());
    mcp_server!(agent, search_sse("https://s.wiseai.dev"));
    assert_eq!(agent.mcp_servers().len(), 2);
}

#[test]
fn with_mcp_servers_macro_registers_multiple_at_once() {
    use autogpt::with_mcp_servers;
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    with_mcp_servers!(
        agent,
        [
            github_stdio(),
            search_sse("https://s.wiseai.dev"),
            stripe_http("https://api.stripe.com/mcp"),
        ]
    );
    assert_eq!(agent.mcp_servers().len(), 3);
}

#[test]
fn with_mcp_servers_macro_preserves_order() {
    use autogpt::with_mcp_servers;
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    with_mcp_servers!(agent, [github_stdio(), search_sse("https://s.wiseai.dev"),]);
    assert_eq!(agent.mcp_servers()[0].name, "github");
    assert_eq!(agent.mcp_servers()[1].name, "search");
}

#[test]
fn agent_trait_mcp_servers_returns_registered_configs() {
    let mut agent = AgentGPT::new_borrowed("Dev", "Write code");
    agent.with_mcp_server(github_stdio());
    let servers = <AgentGPT as Agent>::mcp_servers(&agent);
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "github");
}

#[test]
fn two_agents_have_independent_mcp_server_lists() {
    let mut agent_a = AgentGPT::new_borrowed("A", "Task A");
    let mut agent_b = AgentGPT::new_borrowed("B", "Task B");
    agent_a.with_mcp_server(github_stdio());
    assert_eq!(agent_a.mcp_servers().len(), 1);
    assert!(agent_b.mcp_servers().is_empty());
    agent_b.with_mcp_server(stripe_http("https://api.stripe.com/mcp"));
    assert_eq!(agent_a.mcp_servers().len(), 1);
    assert_eq!(agent_b.mcp_servers().len(), 1);
}

#[test]
fn new_owned_starts_with_empty_mcp_list() {
    let agent = AgentGPT::new_owned("Dev".to_string(), "Write code".to_string());
    assert!(agent.mcp_servers().is_empty());
}

#[test]
fn new_owned_with_mcp_server_appends() {
    let mut agent = AgentGPT::new_owned("Dev".to_string(), "Write code".to_string());
    agent.with_mcp_server(github_stdio());
    assert_eq!(agent.mcp_servers().len(), 1);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
