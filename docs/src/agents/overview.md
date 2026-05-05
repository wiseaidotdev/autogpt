# Built-in Agents Overview

AutoGPT 0.2 ships with **9 built-in autonomous agents**, each specializing in a different domain. They are independently usable via CLI subcommands and composable via the SDK's `agents!` macro.

## Agent Roster

| Agent          | Feature Flag | CLI Subcommand             | Role                                    |
| -------------- | ------------ | -------------------------- | --------------------------------------- |
| `GenericGPT`   | `cli`        | _(default, no subcommand)_ | General-purpose conversational AI shell |
| `ManagerGPT`   | `gpt`        | `autogpt manage`           | Task orchestration across all agents    |
| `ArchitectGPT` | `gpt`        | `autogpt arch`             | System architecture diagram generation  |
| `BackendGPT`   | `gpt`        | `autogpt back`             | Backend source code generation          |
| `FrontendGPT`  | `gpt`        | `autogpt front`            | Frontend UI code generation             |
| `DesignerGPT`  | `img`        | `autogpt design`           | AI image and UI mockup generation       |
| `GitGPT`       | `git`        | _(automatic)_              | Atomic Git commits from agent output    |
| `MailerGPT`    | `mail`       | _(SDK only)_               | Email reading and automated sending     |
| `OptimizerGPT` | `gpt`        | _(SDK only)_               | Codebase modularization and refactoring |

## Agent Architecture

Every agent in AutoGPT is built from three composable layers:

```
┌─────────────────────────────────────────┐
│              Agent Struct               │
│  (e.g. ArchitectGPT, BackendGPT, etc.)  │
├─────────────────────────────────────────┤
│              AgentGPT Core              │
│  persona · behavior · memory · status   │
│  tools · knowledge · planner · context  │
├─────────────────────────────────────────┤
│          ClientType (LLM Client)        │
│  Gemini · OpenAI · Claude · XAI · Co    │
└─────────────────────────────────────────┘
```

- **`AgentGPT`**: the universal agent state: persona string, behavior string, conversation memory, status, tools, planner, and context manager
- **`ClientType`**: an enum wrapping the active LLM client (Gemini Flash, GPT-4o, Claude 3.5, etc.)
- **`Executor` impl**: the agent-specific logic: what to do with the LLM response (write files, commit code, send email, etc.)

## Composing Multiple Agents

All agents conform to the `AgentFunctions` trait, making them interchangeable inside the `agents!` macro:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let architect = ArchitectGPT::new(
        "Software Architect",
        "Design a microservices diagram for a payment gateway.",
    ).await;

    let backend = BackendGPT::new(
        "Backend Engineer",
        "Generate a payment processing API in Rust with axum.",
        "rust",
    ).await;

    let frontend = FrontendGPT::new(
        "Frontend Developer",
        "Build a React checkout form that calls the payment API.",
        "javascript",
    ).await;

    AutoGPT::default()
        .with(agents![architect, backend, frontend])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

All three agents run **concurrently** using Tokio tasks. There is no sequential bottleneck.

## Agent Status Lifecycle

Every agent transitions through these states:

| Status      | Meaning                                         |
| ----------- | ----------------------------------------------- |
| `Idle`      | Initialized, waiting to run                     |
| `Active`    | Executing its task                              |
| `Completed` | Task finished successfully                      |
| `Error`     | Task failed (triggers retry if `max_tries > 1`) |

You can inspect status programmatically after a run:

```rust
let guard = autogpt.agents[0].lock().await;
let agent = guard.get_agent();
println!("Status: {:?}", agent.status());
println!("Memory length: {}", agent.memory().len());
```
