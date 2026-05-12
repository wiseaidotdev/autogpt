# MCP Authentication

AutoGPT supports authentication for both **stdio** (via environment variables) and **HTTP/SSE** (via custom headers) MCP servers.

## Stdio Servers: Environment Variables

Stdio-based MCP servers receive secrets through their environment. Pass them using the `--env` flag or the `env` block in `settings.json`. Use variable references to keep raw values out of your config file.

```sh
autogpt mcp add github \
  --command npx -- -y @modelcontextprotocol/server-github \
  --env 'GITHUB_TOKEN=${GITHUB_TOKEN}'
```

The resulting `settings.json` entry:

```json
"mcp": {
  "github": {
    "name": "github",
    "transport": "stdio",
    "command": "npx",
    "args": [
      "-y",
      "@modelcontextprotocol/server-github",
      "--env",
      "GITHUB_TOKEN=${GITHUB_TOKEN}"
    ],
    "timeout_ms": 30000,
    "trust": false
  }
}
```

`${GITHUB_TOKEN}` is expanded at spawn time from your shell environment or `.env` file. See [Secrets & Env →](./mcp-env.md).

## HTTP/SSE Servers: Custom Headers

HTTP and SSE transports support arbitrary HTTP headers. This is the standard mechanism for passing `Authorization`, API keys, or tenant IDs to remote MCP servers.

### Bearer Token

```sh
autogpt mcp add secure-api \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --header 'Authorization: Bearer $MY_API_TOKEN'
```

`settings.json`:

```json
"headers": {
  "Authorization": "Bearer $MY_API_TOKEN"
}
```

The header value is expanded before each HTTP request.

### API Key in a custom header

```sh
autogpt mcp add analytics \
  --transport http \
  --command https://analytics.wiseai.dev/mcp \
  --header 'X-Api-Key: $ANALYTICS_KEY'
```

### Multiple headers

Repeat `--header` for each header:

```sh
autogpt mcp add multi-auth \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --header 'Authorization: Bearer $TOKEN' \
  --header 'X-Tenant-ID: ${TENANT_ID}' \
  --header "Content-Type: application/json"
```

## Verifying Authentication

Use `autogpt mcp inspect` to check whether the server accepted the auth:

```sh
autogpt mcp inspect secure-api
```

## Security Notes

- **Never hard-code tokens** in `settings.json`. Always use `$VAR` or `${VAR}`.
- Store secrets in your shell profile, CI/CD secret manager, or a `.env` file that is listed in `.gitignore`.
- HTTP headers are transmitted in plain text. Use `https://` endpoints whenever passing auth headers to remote servers.
- For stdio servers, the [redaction policy](./mcp-env.md#security-redaction-policy) prevents other inherited secrets from leaking into the subprocess.
