# 🤖 MCP Custom Agent Example

This example demonstrates how to build a **custom agent** with the `#[derive(Auto)]` macro that has MCP servers pre-configured and accessible via the `Agent` trait.

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
   cd autogpt/examples/mcp-custom-agent
   ```

1. Set the following environment variables:

   ```sh
   export GEMINI_API_KEY=<your_gemini_api_key>
   ```

   Generate an api key from [Google AI Studio](https://aistudio.google.com/app/apikey).

1. Run the app:

   ```sh
   cargo run --features=gem
   ```

## 📖 What This Demonstrates

- **Custom agents via `#[derive(Auto)]`**: The derive macro automatically delegates MCP accessors to the inner `AgentGPT`.
- **Pre-loaded MCP servers**: Servers are attached before the agent enters its execution loop.
- **Agent trait integration**: Shows that `<ResearchAgent as Agent>::mcp_servers()` returns the registered configs.
- **Real-world config**: GitHub MCP server and Brave Search configured with environment variable expansion.
