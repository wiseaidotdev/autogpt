# Standalone Mode

In Standalone Mode you run a single specialized agent directly via a CLI subcommand. Each agent operates independently, no orchestrator, no network, no IAC protocol. This is the simplest way to use the specialized built-in agents.

## Available Subcommands

| Subcommand       | Agent        | Purpose                              |
| ---------------- | ------------ | ------------------------------------ |
| `autogpt arch`   | ArchitectGPT | Generate architecture diagrams       |
| `autogpt back`   | BackendGPT   | Generate backend source code         |
| `autogpt front`  | FrontendGPT  | Generate frontend UI code            |
| `autogpt design` | DesignerGPT  | Generate UI design images            |
| `autogpt man`    | ManagerGPT   | Orchestrate all other agents locally |

## Example: Architecture Diagram

```sh
autogpt arch
```

AutoGPT prompts you for a project goal, passes it to ArchitectGPT, which generates Python `diagrams`-library code in `workspace/architect/`. Run the output to produce a PNG:

```sh
./workspace/architect/.venv/bin/python ./workspace/architect/diagram.py
```

## Example: Full-Stack Code Generation via ManagerGPT

```sh
autogpt man
```

Enter your project goal when prompted:

```
> Develop a full-stack todo app in Rust using Axum for the backend and Yew for the frontend.
```

ManagerGPT decomposes the goal and dispatches subtasks to BackendGPT, FrontendGPT, and ArchitectGPT in parallel. The generated code lands in:

```
workspace/
├── architect/   # architecture diagram
├── backend/     # Rust Axum server code
└── frontend/    # Yew component code
```

## Task Lifecycle

When running in Standalone Mode, each agent follows this lifecycle:

```
User Input → Task Construction → Agent Execution → File Write → Done
```

1. **User Input**: You provide a project goal as a prompt.
2. **Task Construction**: AutoGPT wraps the goal into a `Task` struct with a `Scope` controlling CRUD, auth, and external access permissions.
3. **Agent Execution**: The agent calls the configured LLM, processes the response, and handles retries (`max_tries`).
4. **File Write**: Generated artifacts are written to the workspace directory.
5. **Done**: The process exits cleanly.

## Scope Permissions

When using the SDK in Standalone Mode, you can restrict what agents are allowed to do:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let agent = BackendGPT::new("Backend Dev", "Write a Rust HTTP server.", "rust").await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .crud(true)
        .auth(false)      // No auth-related code
        .external(true)   // Allow external API calls
        .max_tries(3)     // Retry up to 3 times on failure
        .build()
        .expect("Failed to build AutoGPT");

    autogpt.run().await.unwrap();
}
```

<div class="callout callout-tip">
<strong>💡 Tip</strong>
Increase <code>max_tries</code> when working with complex prompts. Agents automatically detect and fix compilation or logic errors across retry attempts.
</div>
