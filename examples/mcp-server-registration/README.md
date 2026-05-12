# 🔌 MCP Server Registration Example

This example demonstrates how to register MCP (Model Context Protocol) servers on AutoGPT agents using both the **builder API** and **convenience macros**.

## 🛠️ Pre-requisites:

### 🐧 **Linux Users**

1. **Install [`rustup`](https://www.rust-lang.org/tools/install)**:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

### 🪟 **Windows Users**

1. **Download and install `rustup`**: Follow the installation instructions [here](https://forge.rust-lang.org/infra/other-installation-methods.html).

## 🚀 Building and Running

1. Fork/Clone the GitHub repository.

   ```sh
   git clone https://github.com/wiseaidotdev/autogpt
   ```

1. Navigate to the example directory.

   ```sh
   cd autogpt/examples/mcp-server-registration
   ```

1. Run the app:

   ```sh
   cargo run
   ```

   You should see output:

   ```sh
   Builder API: 2 MCP servers registered
   • github (stdio): GitHub tools via MCP
   • search (sse): Web search via SSE

   Macro API: 3 MCP servers registered
   • postgres (stdio): PostgreSQL via MCP
   • filesystem (stdio): Filesystem access
   • memory (stdio): In-memory knowledge graph
   ```

## 📖 What This Demonstrates

- **Builder pattern**: `agent.with_mcp_server(config)` for fluent chaining.
- **`mcp_server!` macro**: Single server registration in one call.
- **`with_mcp_servers!` macro**: Batch registration of multiple servers.
- **Transport types**: Stdio, SSE, and HTTP transport configurations.
- **Config options**: Environment variables, tool filtering, trust mode, timeouts.
