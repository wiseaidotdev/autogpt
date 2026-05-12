# MCP TUI Commands

The interactive AutoGPT shell (`autogpt` with no arguments) exposes all MCP management commands as TUI **slash commands**. These work identically to the CLI equivalents but without leaving the shell.

## Slash Command Reference

| Command                               | Description                          |
| ------------------------------------- | ------------------------------------ |
| `/mcp list`                           | List all servers and their status    |
| `/mcp inspect <name>`                 | Show tools and config for one server |
| `/mcp remove <name>`                  | Remove a server from settings        |
| `/mcp call <server> <tool> [args...]` | Manually invoke a tool               |

> Type `/help` inside the shell to see all available commands, including MCP ones.

## `/mcp list`

Show all registered MCP servers and whether they are reachable.

```sh
> /mcp list
```

## `/mcp inspect <name>`

Display detailed information about a single server, including all discovered tools.

```sh
> /mcp inspect everything
```

**Output:**

```sh
▸ MCP Server: everything
  Transport:  stdio
  Connection:  npx -y @modelcontextprotocol/server-everything
  Status:     CONNECTED
  Trust:      no
  Timeout:    10000ms

▸ Available Tools (13):
  echo  Echoes back the input string
    * message: string - Message to echo
  get-annotated-message  Demonstrates how annotations can be used to provide metadata about content.
    * messageType: string - Type of message to demonstrate different annotation patterns
      includeImage: boolean - Whether to include an example image
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
      duration: number - Duration of the operation in seconds
      steps: number - Number of steps in the operation
  simulate-research-query  Simulates a deep research operation that gathers, analyzes, and synthesizes information. Demonstrates MCP task-based operations with progress through multiple stages. If 'ambiguous' is true and client supports elicitation, sends an elicitation request for clarification.
      ambiguous: boolean - Simulate an ambiguous query that requires clarification (triggers input_required status)
    * topic: string - The research topic to investigate
```

## `/mcp call <server> <tool> [args...]`

Invoke any tool directly from the shell. This is useful for testing and debugging your server configuration without going through the agent.

Arguments can be passed as `key=value` pairs:

```sh
> /mcp call everything echo message="Hello from the TUI!"
```

Or as a raw JSON object:

```sh
> /mcp call everything get-sum '{"a": 10, "b": 32}'
```

**Example output:**

```sh
✓  Tool 'get-sum' called successfully on 'everything'
The sum of 10 and 32 is 42.
```

Calling a no-argument tool:

```sh
> /mcp call everything get-env
```

## `/mcp remove <name>`

Remove a server from the configuration. The change is persisted to `~/.autogpt/settings.json` immediately.

```sh
> /mcp remove everything
✓  MCP server 'everything' removed.
```

## Adding Servers from the TUI

Server registration is a CLI-only operation (it requires flag-based input). Add servers with the [`autogpt mcp add`](./mcp-cli.md) CLI command before starting the shell, or in a separate terminal while the shell is running.

Changes to `~/.autogpt/settings.json` are picked up on the next `/mcp list` or `/mcp inspect` call.

## Getting Help

```sh
> /help
```

**Output:**

```sh
MCP Commands
  /mcp list  Show all configured MCP servers and their status
  /mcp inspect <name>  Inspect a server and list its tools
  /mcp remove <name>  Remove a server registration
  /mcp call <srv> <tool> [args]  Call an MCP tool with JSON args or key=val pairs
```
