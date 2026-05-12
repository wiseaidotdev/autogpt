# 🧩 MCP Manager Example

This example demonstrates **direct usage of `McpManager`** for connecting to MCP servers, discovering tools, and dispatching tool calls, without any LLM integration.

## 🛠️ Pre-requisites:

### 🐧 **Linux Users**

1. **Install [`rustup`](https://www.rust-lang.org/tools/install)**:

   ```sh
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

1. **Install Node.js** (required for `npx`-based MCP servers):

   ```sh
   curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
   sudo apt-get install -y nodejs
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
   cd autogpt/examples/mcp-manager
   ```

1. Run the app:

   ```sh
   cargo run
   ```

   The output will show discovered servers, their tools, and attempt a sample tool call.

   ```sh
   Configured 2 server(s)
   Connected to 2 server(s)

   ✓ everything - 13 tool(s) - Miscellaneous tool collection
   └─ echo (FQN: mcp_everything_echo)
      Echoes back the input string
      *message: string - Message to echo
   └─ get-annotated-message (FQN: mcp_everything_get-annotated-message)
      Demonstrates how annotations can be used to provide metadata about content.
         includeImage: boolean - Whether to include an example image
      *messageType: string - Type of message to demonstrate different annotation patterns
   └─ get-env (FQN: mcp_everything_get-env)
      Returns all environment variables, helpful for debugging MCP server configuration
   └─ get-resource-links (FQN: mcp_everything_get-resource-links)
      Returns up to ten resource links that reference different types of resources
         count: number - Number of resource links to return (1-10)
   └─ get-resource-reference (FQN: mcp_everything_get-resource-reference)
      Returns a resource reference that can be used by MCP clients
         resourceId: number - ID of the text resource to fetch
         resourceType: string -
   └─ get-structured-content (FQN: mcp_everything_get-structured-content)
      Returns structured content along with an output schema for client data validation
      *location: string - Choose city
   └─ get-sum (FQN: mcp_everything_get-sum)
      Returns the sum of two numbers
      *a: number - First number
      *b: number - Second number
   └─ get-tiny-image (FQN: mcp_everything_get-tiny-image)
      Returns a tiny MCP logo image.
   └─ gzip-file-as-resource (FQN: mcp_everything_gzip-file-as-resource)
      Compresses a single file using gzip compression. Depending upon the selected output type, returns either the compressed data as a gzipped resource or a resource link, allowing it to be downloaded in a subsequent request during the current session.
         data: string - URL or data URI of the file content to compress
         name: string - Name of the output file
         outputType: string - How the resulting gzipped file should be returned. 'resourceLink' returns a link to a resource that can be read later, 'resource' returns a full resource object.
   └─ toggle-simulated-logging (FQN: mcp_everything_toggle-simulated-logging)
      Toggles simulated, random-leveled logging on or off.
   └─ toggle-subscriber-updates (FQN: mcp_everything_toggle-subscriber-updates)
      Toggles simulated resource subscription updates on or off.
   └─ trigger-long-running-operation (FQN: mcp_everything_trigger-long-running-operation)
      Demonstrates a long running operation with progress updates.
         duration: number - Duration of the operation in seconds
         steps: number - Number of steps in the operation
   └─ simulate-research-query (FQN: mcp_everything_simulate-research-query)
      Simulates a deep research operation that gathers, analyzes, and synthesizes information. Demonstrates MCP task-based operations with progress through multiple stages. If 'ambiguous' is true and client supports elicitation, sends an elicitation request for clarification.
         ambiguous: boolean - Simulate an ambiguous query that requires clarification (triggers input_required status)
      *topic: string - The research topic to investigate

   ✓ memory - 9 tool(s) - In-memory knowledge graph
   └─ create_entities (FQN: mcp_memory_create_entities)
      Create multiple new entities in the knowledge graph
      *entities: array -
   └─ create_relations (FQN: mcp_memory_create_relations)
      Create multiple new relations between entities in the knowledge graph. Relations should be in active voice
      *relations: array -
   └─ add_observations (FQN: mcp_memory_add_observations)
      Add new observations to existing entities in the knowledge graph
      *observations: array -
   └─ delete_entities (FQN: mcp_memory_delete_entities)
      Delete multiple entities and their associated relations from the knowledge graph
      *entityNames: array - An array of entity names to delete
   └─ delete_observations (FQN: mcp_memory_delete_observations)
      Delete specific observations from entities in the knowledge graph
      *deletions: array -
   └─ delete_relations (FQN: mcp_memory_delete_relations)
      Delete multiple relations from the knowledge graph
      *relations: array - An array of relations to delete
   └─ read_graph (FQN: mcp_memory_read_graph)
      Read the entire knowledge graph
   └─ search_nodes (FQN: mcp_memory_search_nodes)
      Search for nodes in the knowledge graph based on a query
      *query: string - The search query to match against entity names, types, and observation content
   └─ open_nodes (FQN: mcp_memory_open_nodes)
      Open specific nodes in the knowledge graph by their names
      *names: array - An array of entity names to retrieve

   Found tool 'echo' on server 'everything'
   Tool result: Echo: Hello from MCP Manager Example!
   ```

## 📖 What This Demonstrates

- **`McpManager::new()`**: Creating a manager from a `HashMap<String, McpServerConfig>`.
- **`connect_all()`**: Connecting to all servers and discovering their tools.
- **`server_infos()`**: Inspecting the live status and tool lists of all servers.
- **`find_tool(fqn)`**: Looking up a tool by its fully-qualified name (`mcp_{server}_{tool}`).
- **`call_tool(fqn, args)`**: Dispatching a JSON-RPC tool call to the correct server.
- **Allow/Exclude policies**: Filtering which servers to connect to.
- **No LLM required**: Uses only the `mcp` feature, no API keys needed.
