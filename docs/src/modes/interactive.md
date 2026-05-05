# Interactive Mode

Running `autogpt` with no arguments launches **GenericGPT**, a full-featured conversational AI shell with session persistence, runtime model switching, and multi-provider support.

```sh
autogpt
```

This is the default mode and requires no subcommands or flags.

## Shell Commands

Once inside the shell, the following commands are available:

| Command         | Description                                                             |
| --------------- | ----------------------------------------------------------------------- |
| `<your prompt>` | Send a message to the active AI agent                                   |
| `/help`         | Show all available shell commands                                       |
| `/provider`     | Switch LLM provider at runtime (Gemini, OpenAI, Anthropic, XAI, Cohere) |
| `/models`       | Browse and switch between models supported by the active provider       |
| `/sessions`     | List and resume previous conversation sessions                          |
| `/status`       | Show the current model, provider, and workspace directory               |
| `/workspace`    | Print the current workspace path                                        |
| `/clear`        | Clear the terminal screen                                               |
| `exit` / `quit` | Save the session and exit                                               |

<div class="callout callout-info">
<strong>ℹ️ Note</strong>
Press <code>ESC</code> at any time to interrupt a running generation without closing the shell.
</div>

## Session Management

Every conversation is automatically saved. When you type `/sessions`, AutoGPT lists all previous sessions. Selecting one restores the full conversation history, so the agent retains context from where you left off.

Sessions are stored in the configured workspace directory and indexed by a UUID, creation timestamp, and a short automatic summary.

## Switching Providers at Runtime

You can switch between LLM providers without restarting the shell:

```
> /provider
  1. Gemini (active)
  2. OpenAI
  3. Anthropic
  4. XAI
  5. Cohere
Select provider: 2
✓ Switched to OpenAI
```

The selected provider must have its API key set as an environment variable and its corresponding Cargo feature enabled at compile time.

## Switching Models

`/models` lists all models available for the active provider.

## How GenericGPT Works

GenericGPT is implemented as an `AgentGPT` instance configured with a general-purpose system persona. Each user message is appended to the conversation history, sent to the LLM, and the response is streamed back to the terminal using `termimad` for Markdown rendering.

The `cli` feature flag must be enabled to use the interactive shell:

```sh
cargo install autogpt --features cli,gem
```
