# OptimizerGPT

<span class="badge badge-orange">Feature: gpt</span>

OptimizerGPT refactors messy or monolithic code files into clean, well-organized, modular structures. It reads a single large source file, identifies logical separation of concerns, and rewrites it as a set of focused modules following framework conventions.

## What OptimizerGPT Solves

Agent-generated code (from BackendGPT or FrontendGPT) often starts as a single large file to satisfy the LLM context window. OptimizerGPT acts as a post-processor to enforce real-world code organization, splitting routes from handlers, models from services, and utilities from application logic.

## How It Works

1. OptimizerGPT receives a file path pointing to a monolithic source file
2. It sends the file contents to the LLM with instructions to identify logical modules
3. The LLM returns a restructured file layout following standard conventions for the target stack
4. OptimizerGPT writes the new module files to the workspace, preserving the original

## Supported Stacks

OptimizerGPT works with any language. It applies framework-idiomatic conventions when it recognizes the stack:

| Stack            | Resulting Structure                                 |
| ---------------- | --------------------------------------------------- |
| FastAPI (Python) | `routes/`, `models/`, `services/`, `utils/`         |
| Axum (Rust)      | `routes.rs`, `models.rs`, `handlers.rs`, `state.rs` |
| React (JS)       | `components/`, `hooks/`, `services/`, `utils/`      |
| Express (JS)     | `routes/`, `controllers/`, `middleware/`, `models/` |

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Senior Software Engineer";
    let behavior = "Refactor workspace/backend/main.py into a clean FastAPI project structure.";

    let agent = OptimizerGPT::new(persona, behavior).await;

    AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

## Pipeline Integration

OptimizerGPT pairs naturally after BackendGPT or FrontendGPT in an `agents!` list. Because AutoGPT runs agents concurrently, chain them sequentially when order matters:

```rust
let backend = BackendGPT::new("Dev", "Build a user auth API in Rust with axum.", "rust").await;

AutoGPT::default()
    .with(agents![backend])
    .build().unwrap()
    .run().await.unwrap();

let optimizer = OptimizerGPT::new(
    "Refactor Expert",
    "Modularize workspace/backend/main.rs into idiomatic axum structure.",
).await;

AutoGPT::default()
    .with(agents![optimizer])
    .build().unwrap()
    .run().await.unwrap();
```
