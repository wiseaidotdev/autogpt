# MCP CLI Reference

The `autogpt mcp` command family lets you manage MCP server configurations from your terminal without editing `settings.json` by hand.

## `autogpt mcp add`

Register a new MCP server.

```sh
autogpt mcp add [OPTIONS] <NAME> [-- <SERVER_ARGS>...]
```

| Option                  | Short | Description                                           |
| ----------------------- | ----- | ----------------------------------------------------- |
| `--command <CMD>`       | `-c`  | Executable path (stdio) or endpoint URL (http/sse)    |
| `--transport <TYPE>`    | `-t`  | `stdio` (default), `http`, or `sse`                   |
| `--env <KEY=VALUE>`     | `-e`  | Environment variable (repeatable)                     |
| `--header <KEY: VALUE>` | `-H`  | HTTP header (repeatable, http/sse only)               |
| `--timeout <MS>`        |       | Connection timeout in milliseconds (default: `30000`) |
| `--trust`               |       | Bypass tool-call confirmation prompts for this server |
| `--description <TEXT>`  |       | Human-readable label shown in `list` and `inspect`    |
| `--include-tools <...>` |       | Comma-separated allowlist of tool names               |
| `--exclude-tools <...>` |       | Comma-separated blocklist of tool names               |

### Adding a stdio server

```sh
autogpt mcp add everything --command npx -- -y @modelcontextprotocol/server-everything
```

### Adding a stdio server with environment variables

Values are stored as-is in `settings.json`; they are expanded from the shell or `.env` at runtime:

```sh
autogpt mcp add github \
  --command npx -- -y @modelcontextprotocol/server-github \
  --env 'GITHUB_TOKEN=${GITHUB_TOKEN}'
```

### Adding an HTTP server

```sh
autogpt mcp add my-api \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --timeout 10000
```

### Adding an HTTP server with an auth header

```sh
autogpt mcp add secure-api \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --header 'Authorization: Bearer $MY_API_TOKEN'
```

### Adding an SSE server

```sh
autogpt mcp add sse-stream \
  --transport sse \
  --command https://wiseai.dev/mcp/sse \
  --header 'X-Api-Key: $STREAM_KEY'
```

### Filtering exposed tools

```sh
autogpt mcp add safe-server \
  --command npx -- -y @modelcontextprotocol/server-everything \
  --include-tools echo,add \
  --trust
```

## `autogpt mcp list`

Show all configured servers with their live connection status.

```sh
autogpt mcp list
```

## `autogpt mcp inspect <NAME>`

Show detailed information and all available tools for a specific server.

```sh
autogpt mcp inspect everything
```

**Example output:**

```
▸ MCP Server: everything
  Transport:  stdio
  Connection:  npx -y @modelcontextprotocol/server-everything
  Status:     CONNECTED
  Trust:      no
  Timeout:    30000ms

▸ Available Tools (13):
  echo  Echoes back the input string
    * message: string - Message to echo
  get-annotated-message  Demonstrates how annotations can be used to provide metadata about content.
      includeImage: boolean - Whether to include an example image
    * messageType: string - Type of message to demonstrate different annotation patterns
  get-env  Returns all environment variables, helpful for debugging MCP server configuration
  get-resource-links  Returns up to ten resource links that reference different types of resources
      count: number - Number of resource links to return (1-10)
  get-resource-reference  Returns a resource reference that can be used by MCP clients
      resourceId: number - ID of the text resource to fetch
      resourceType: string -
  get-structured-content  Returns structured content along with an output schema for client data validation
    * location: string - Choose city
  get-sum  Returns the sum of two numbers
    * b: number - Second number
    * a: number - First number
  get-tiny-image  Returns a tiny MCP logo image.
  gzip-file-as-resource  Compresses a single file using gzip compression. Depending upon the selected output type, returns either the compressed data as a gzipped resource or a resource link, allowing it to be downloaded in a subsequent request during the current session.
      name: string - Name of the output file
      outputType: string - How the resulting gzipped file should be returned. 'resourceLink' returns a link to a resource that can be read later, 'resource' returns a full resource object.
      data: string - URL or data URI of the file content to compress
  toggle-simulated-logging  Toggles simulated, random-leveled logging on or off.
  toggle-subscriber-updates  Toggles simulated resource subscription updates on or off.
  trigger-long-running-operation  Demonstrates a long running operation with progress updates.
      steps: number - Number of steps in the operation
      duration: number - Duration of the operation in seconds
  simulate-research-query  Simulates a deep research operation that gathers, analyzes, and synthesizes information. Demonstrates MCP task-based operations with progress through multiple stages. If 'ambiguous' is true and client supports elicitation, sends an elicitation request for clarification.
      ambiguous: boolean - Simulate an ambiguous query that requires clarification (triggers input_required status)
    * topic: string - The research topic to investigate
```

## `autogpt mcp call <SERVER> <TOOL> [ARGS...]`

Manually invoke a tool on a registered MCP server. Useful for testing and debugging.

```sh
autogpt mcp call <SERVER> <TOOL> [ARGS...]
```

Arguments can be passed as `key=value` pairs or as a JSON string.

### Calling with key=value pairs

```sh
autogpt mcp call everything echo message="Hello, World"

Echo: Hello, World
```

### Calling with a JSON argument string

```sh
autogpt mcp call everything get-sum '{"a": 5, "b": 3}'

The sum of 5 and 3 is 8.
```

### Calling a tool with no arguments

```sh
autogpt mcp call everything get-env
```

**Example output:**

```json
{
  "CARGO_PKG_NAME": "autogpt",
  "CARGO_PKG_DESCRIPTION": "🦀 A Pure Rust Framework For Building AGIs.\n",
  "EDITOR": "vim"
}
```

## `autogpt mcp remove <NAME>`

Permanently remove a server from `settings.json`.

```sh
autogpt mcp remove everything
```

**Output:**

```
✓  MCP server 'everything' removed.
```

## Settings Storage

All configurations are stored in `~/.autogpt/settings.json`. You can edit this file directly if needed; the CLI is a convenience wrapper.

```json
{
  "mcp": {
    "everything": {
      "name": "everything",
      "transport": "stdio",
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-everything"],
      "env": {},
      "headers": {},
      "timeout_ms": 30000,
      "trust": false,
      "include_tools": [],
      "exclude_tools": [],
      "description": "Everything test server"
    }
  }
}
```
