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

Every conversation is automatically saved as a YAML file inside your workspace. When you type `/sessions`, AutoGPT lists all previous sessions. Selecting one restores the full conversation history, so the agent retains context from where you left off.

Sessions are stored under the `.autogpt/sessions/` directory and indexed by a UUID, creation timestamp, and a short automatic summary. See the [GenericGPT agent page](../agents/generic-gpt.md) for full detail on the `.autogpt` directory layout.

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

Switching providers also reloads the model list for the new provider and resets the active model to that provider's default.

The selected provider must have its API key set as an environment variable and its corresponding Cargo feature enabled at compile time.

## Switching Models

`/models` lists all models available from the active provider, sourced directly from each provider's crate, no hardcoded strings. Use arrow keys to navigate and `Enter` to confirm:

```
> /models
─────────────────────────────────────────────────────────────
  ● Flash3Preview
    Pro31Preview
    Flash31LitePreview
    ...
─────────────────────────────────────────────────────────────
✓ Model set to: Flash3Preview
```

Override the default model using environment variables instead of the interactive selector:

```sh
export GEMINI_MODEL=gemini-2.5-pro-preview-05-06
```

## How GenericGPT Works

GenericGPT is a production-hardened autonomous agent that goes well beyond simple chat:

1. **Reasoning pre-step**: Before acting, the agent emits a structured internal monologue (stored in the session log) to plan its approach.
2. **Task synthesis**: The plan is decomposed into an ordered list of typed actions (`CreateFile`, `PatchFile`, `RunCommand`, etc.).
3. **Execution**: Each action is applied against the workspace. `PatchFile` performs anchor-text surgical edits; `RunCommand` runs shell commands.
4. **Build-and-verify**: After code changes, the agent detects the project's build system (`cargo`, `npm`, etc.) and runs it. On failure, it feeds the error back and retries automatically (up to 3 attempts).
5. **Reflection**: After completion the agent reviews what worked and what didn't.
6. **Skill extraction**: Lessons are written to provider-specific TOML files in `.autogpt/skills/` and injected into future sessions automatically.

The `cli` feature flag must be enabled to use the interactive shell:

```sh
cargo install autogpt --features cli,gem
```
