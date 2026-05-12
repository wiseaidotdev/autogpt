//! # MCP Server Registration Example
//!
//! Demonstrates how to register MCP servers on agents using the builder API
//! and convenience macros (`mcp_server!`, `with_mcp_servers!`).

use autogpt::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() {
    let github_config = McpServerConfig {
        name: "github".to_string(),
        transport: McpTransport::Stdio,
        command: Some("docker".to_string()),
        args: vec![
            "run".into(),
            "-i".into(),
            "ghcr.io/github/github-mcp-server:latest".into(),
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
        timeout_ms: 30_000,
        trust: false,
        include_tools: vec![],
        exclude_tools: vec!["delete_file".to_string()],
        description: Some("GitHub tools via MCP".to_string()),
        oauth: None,
    };

    let search_config = McpServerConfig {
        name: "search".to_string(),
        transport: McpTransport::Sse,
        command: None,
        args: vec![],
        url: Some("https://search.wiseai.dev/mcp/sse".to_string()),
        http_url: None,
        headers: {
            let mut h = HashMap::new();
            h.insert("Authorization".to_string(), "Bearer my-api-key".to_string());
            h
        },
        env: HashMap::new(),
        cwd: None,
        timeout_ms: 30_000,
        trust: true,
        include_tools: vec!["web_search".to_string()],
        exclude_tools: vec![],
        description: Some("Web search via SSE".to_string()),
        oauth: None,
    };

    let mut agent = AgentGPT::new_borrowed("Dev", "Write code using external tools");

    agent
        .with_mcp_server(github_config)
        .with_mcp_server(search_config);

    println!(
        "Builder API: {} MCP servers registered",
        agent.mcp_servers().len()
    );
    for srv in agent.mcp_servers() {
        println!(
            "  • {} ({}): {}",
            srv.name,
            srv.transport,
            srv.description.as_deref().unwrap_or("-")
        );
    }

    let mut agent2 = AgentGPT::new_borrowed("Analyst", "Analyze data from multiple sources");

    autogpt::mcp_server!(
        agent2,
        McpServerConfig {
            name: "postgres".to_string(),
            transport: McpTransport::Stdio,
            command: Some("npx".to_string()),
            args: vec!["-y".into(), "@modelcontextprotocol/server-postgres".into()],
            url: None,
            http_url: None,
            headers: HashMap::new(),
            env: {
                let mut e = HashMap::new();
                e.insert("POSTGRES_URL".to_string(), "$DATABASE_URL".to_string());
                e
            },
            cwd: None,
            timeout_ms: 30_000,
            trust: false,
            include_tools: vec![],
            exclude_tools: vec![],
            description: Some("PostgreSQL via MCP".to_string()),
            oauth: None,
        }
    );

    autogpt::with_mcp_servers!(
        agent2,
        [
            McpServerConfig {
                name: "filesystem".to_string(),
                transport: McpTransport::Stdio,
                command: Some("npx".to_string()),
                args: vec![
                    "-y".into(),
                    "@modelcontextprotocol/server-filesystem".into(),
                    "/tmp".into()
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
                description: Some("Filesystem access".to_string()),
                oauth: None,
            },
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
        ]
    );

    println!(
        "\nMacro API: {} MCP servers registered",
        agent2.mcp_servers().len()
    );
    for srv in agent2.mcp_servers() {
        println!(
            "  • {} ({}): {}",
            srv.name,
            srv.transport,
            srv.description.as_deref().unwrap_or("-")
        );
    }
}
