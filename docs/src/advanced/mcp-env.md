# Secrets, Environment Variables & `.env` Files

AutoGPT's MCP integration is designed so that **no raw secrets are ever stored in `settings.json`**. Instead, use variable references that are resolved at runtime.

## Variable Expansion

All values in the `env` and `headers` blocks of an MCP server config support three expansion syntaxes:

| Syntax   | Platform | Example         |
| -------- | -------- | --------------- |
| `$VAR`   | All      | `$MY_API_KEY`   |
| `${VAR}` | All      | `${MY_API_KEY}` |
| `%VAR%`  | Windows  | `%MY_API_KEY%`  |

If the referenced variable is not set, it expands to an empty string.

### Example: `env` block

```json
{
  "mcp": {
    "github": {
      "command": "npx",
      "args": ["-y", "@modelcontextprotocol/server-github"],
      "env": {
        "GITHUB_TOKEN": "${GITHUB_TOKEN}",
        "LOG_LEVEL": "$LOG_LEVEL"
      }
    }
  }
}
```

When `autogpt` spawns the server, it replaces `${GITHUB_TOKEN}` with the actual value from your shell environment (or `.env` file) before passing it to the subprocess.

### Example: `headers` block

HTTP and SSE transports support the same expansion in header values:

```json
{
  "mcp": {
    "my-api": {
      "transport": "http",
      "command": "https://api.wiseai.dev/mcp",
      "headers": {
        "Authorization": "Bearer $MY_API_TOKEN"
      }
    }
  }
}
```

### Adding via CLI (keeps references intact)

The CLI preserves the literal reference string in `settings.json`:

```sh
autogpt mcp add github \
  --command npx -- -y @modelcontextprotocol/server-github \
  --env "GITHUB_TOKEN=${GITHUB_TOKEN}"

autogpt mcp add my-api \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --header "Authorization: Bearer $MY_API_TOKEN"
```

> The single quotes in shell commands prevent premature expansion by your terminal. The literal string `$MY_API_TOKEN` is stored in `settings.json` and expanded when autogpt runs.

## `.env` File Support

`autogpt` automatically loads `.env` files on startup. Variables from `.env` are set in the process environment **only if they are not already set**, so your shell environment always takes precedence.

### Lookup order

1. `.env` in the **current working directory**
2. `$HOME/.env` (home directory)
3. `$HOME/.autogpt/.env` (autogpt config directory)

### `.env` file format

```sh
GITHUB_TOKEN=ghp_...
MY_API_KEY=sk-...
LOG_LEVEL=info
```

Quoted values are unquoted automatically:

```sh
DATABASE_URL="postgresql://user:pass@localhost/db"
GREETING='hello world'
```

### Workflow example

Create a `.env` file in your project root:

```sh
echo 'MY_API_TOKEN=sk-abc123' >> .env
echo 'GITHUB_TOKEN=ghp_xyz' >> .env
```

Register your servers using references:

```sh
autogpt mcp add my-api \
  --transport http \
  --command https://api.wiseai.dev/mcp \
  --header "Authorization: Bearer $MY_API_TOKEN"
```

The stored `settings.json` will contain:

```json
"headers": {
  "Authorization": "Bearer $MY_API_TOKEN"
}
```

When you next run `autogpt`, it loads `.env`, sets `MY_API_TOKEN`, and the header is expanded before the HTTP request is sent.

## Security Redaction Policy

By default, AutoGPT applies a **security redaction filter** when spawning MCP stdio subprocesses. This prevents third-party servers from accidentally inheriting sensitive secrets from your shell.

### What gets redacted

Environment variables whose **names** match any of the following patterns are removed from the inherited environment before the server process is started:

| Pattern                                  | Examples                                  |
| ---------------------------------------- | ----------------------------------------- |
| `*API_KEY*`                              | `GEMINI_API_KEY`, `OPENAI_API_KEY`        |
| `*TOKEN*`                                | `GITHUB_TOKEN`, `OAUTH_ACCESS_TOKEN`      |
| `*SECRET*`                               | `AWS_SECRET_ACCESS_KEY`, `JWT_SECRET`     |
| `*PASSWORD*`                             | `DB_PASSWORD`, `REDIS_PASSWORD`           |
| `*AUTH*`                                 | `BEARER_AUTH`, `X_AUTH_HEADER`            |
| `*CREDENTIAL*`                           | `GOOGLE_CREDENTIALS`, `AZURE_CREDENTIALS` |
| `GOOGLE_APPLICATION_CREDENTIALS` (exact) | -                                         |

### Explicit overrides bypass redaction

Variables you **explicitly list** in the `env` block of a server config are always passed through, even if their name matches a redaction pattern. This follows the principle of **informed consent**: if you deliberately configure a variable for a server, it is trusted.

```json
"env": {
  "OPENAI_API_KEY": "$OPENAI_API_KEY"
}
```

This is the **only safe way** to pass a secret to an MCP server.

### Verification example

You can verify which variables actually reach a server using the `get-env` tool from `@modelcontextprotocol/server-everything`:

```sh
autogpt mcp add test-server \
  --command npx -- -y @modelcontextprotocol/server-everything \
  --env "MY_KEY=${MY_SECRET}"

autogpt mcp call test-server get-env
```

You should see `MY_KEY` in the output (expanded), but not `MY_SECRET` directly as an inherited variable.

## Summary

| Concern                      | Recommendation                                      |
| ---------------------------- | --------------------------------------------------- |
| Storing secrets              | Never, use `$VAR` references                        |
| Loading secrets at runtime   | Shell env or `.env` files                           |
| Passing secrets to a server  | Only via `env` block with `"KEY": "$VAR"` pattern   |
| Protecting inherited secrets | Automatic, redaction policy removes them by default |
