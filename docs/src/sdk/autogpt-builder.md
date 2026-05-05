# AutoGPT Builder

The `AutoGPT` struct uses a **builder pattern** to configure the runtime before executing agents. All builder methods consume and return `Self`, enabling method chaining.

## Builder Methods

| Method            | Type                  | Default | Description                             |
| ----------------- | --------------------- | ------- | --------------------------------------- |
| `.with(agents)`   | `impl Into<Vec<...>>` | -       | Register agents to run                  |
| `.execute(bool)`  | `bool`                | `true`  | Enable agent task execution             |
| `.browse(bool)`   | `bool`                | `false` | Allow agents to open browser windows    |
| `.max_tries(u64)` | `u64`                 | `1`     | Max retry attempts per agent on failure |
| `.crud(bool)`     | `bool`                | `true`  | Allow CRUD file operations in scope     |
| `.auth(bool)`     | `bool`                | `false` | Allow auth-related code generation      |
| `.external(bool)` | `bool`                | `true`  | Allow calls to external APIs            |
| `.id(Uuid)`       | `Uuid`                | random  | Override the AutoGPT instance UUID      |
| `.build()`        | `Result<Self>`        | -       | Validate and build the instance         |

## Full Builder Example

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let agent = BackendGPT::new(
        "Senior Engineer",
        "Generate a secure user authentication API in Rust.",
        "rust",
    ).await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .execute(true)
        .browse(false)
        .max_tries(3)
        .crud(true)
        .auth(true)       // auth-related code allowed
        .external(false)  // no external network calls
        .build()
        .expect("Failed to build AutoGPT");

    autogpt.run().await.unwrap();
}
```

## Scope: How Permissions Flow to Agents

When `.build()` is called, a `Scope` struct is constructed from the `crud`, `auth`, and `external` values:

```rust
pub struct Scope {
    pub crud:     bool,
    pub auth:     bool,
    pub external: bool,
}
```

This `Scope` is included in every `Task` dispatched to each agent. Agents can inspect it in their `Executor::execute` implementation to decide whether certain operations are permitted.

## Running Multiple Agent Groups Sequentially

Each `AutoGPT` instance is independent. Chain `.run()` calls to enforce ordering:

```rust
// Phase 1: generate backend
let phase1 = AutoGPT::default()
    .with(agents![BackendGPT::new("Dev", "REST API in Rust.", "rust").await])
    .build().unwrap();
phase1.run().await.unwrap();

// Phase 2: optimize the output of phase 1
let phase2 = AutoGPT::default()
    .with(agents![OptimizerGPT::new("Refactorer", "Modularize workspace/backend/main.rs.").await])
    .build().unwrap();
phase2.run().await.unwrap();
```

## Error Handling

`.build()` returns `Result<AutoGPT, anyhow::Error>`. Currently validation is lightweight, it succeeds as long as required feature flags are compiled in. `.run()` returns `Result<String, anyhow::Error>` where the `Err` variant contains all agent failure messages joined with newlines.
