//! # MCP Custom Agent Example
//!
//! Demonstrates how to create a custom agent (via `#[derive(Auto)]`) that
//! has MCP servers pre-configured and can use their tools during execution.

#![allow(unused)]

use autogpt::prelude::*;
use std::collections::HashMap;

/// A custom research agent with MCP servers pre-loaded.
///
/// The `Auto` derive macro automatically delegates the `Agent`, `Functions`,
/// and `AsyncFunctions` traits, including MCP accessors, to the inner
/// `AgentGPT` field.
#[derive(Debug, Default, Auto)]
pub struct ResearchAgent {
    agent: AgentGPT,
    client: ClientType,
}

#[async_trait]
impl Executor for ResearchAgent {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        let servers = self.agent.mcp_servers();
        println!("ResearchAgent has {} MCP server(s):", servers.len());
        for srv in servers {
            println!(
                "  • {} ({}) - {}",
                srv.name,
                srv.transport,
                srv.description.as_deref().unwrap_or("no description")
            );
        }

        // Here, you would:
        // 1. Create an McpManager from these configs
        // 2. Call manager.connect_all() to discover tools
        // 3. Use the tools as part of your agent's LLM tool-use loop

        let prompt = self.agent.behavior().clone();
        let response = self.generate(prompt.as_ref()).await?;

        self.agent.add_message(Message {
            role: "assistant".into(),
            content: response.clone().into(),
        });

        println!("\nAgent response:\n{}", response);
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let persona = "Senior Research Analyst";
    let behavior = r#"You are a research agent with access to GitHub and web search tools.
    List 3 trending open-source AI projects from this week."#;

    let mut agent = ResearchAgent::new(persona.into(), behavior.into());

    agent
        .agent
        .with_mcp_server(McpServerConfig {
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
            exclude_tools: vec![],
            description: Some("GitHub tools for repository search".to_string()),
            oauth: None,
        })
        .with_mcp_server(McpServerConfig {
            name: "brave-search".to_string(),
            transport: McpTransport::Stdio,
            command: Some("npx".to_string()),
            args: vec![
                "-y".into(),
                "@modelcontextprotocol/server-brave-search".into(),
            ],
            url: None,
            http_url: None,
            headers: HashMap::new(),
            env: {
                let mut e = HashMap::new();
                e.insert("BRAVE_API_KEY".to_string(), "$BRAVE_API_KEY".to_string());
                e
            },
            cwd: None,
            timeout_ms: 30_000,
            trust: true,
            include_tools: vec![],
            exclude_tools: vec![],
            description: Some("Brave web search".to_string()),
            oauth: None,
        });

    let trait_servers = <ResearchAgent as Agent>::mcp_servers(&agent);
    println!("Servers visible via Agent trait: {}", trait_servers.len());

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT");

    match autogpt.run().await {
        Ok(response) => println!("\nFinal output:\n{}", response),
        Err(err) => eprintln!("Agent error: {:?}", err),
    }
}
