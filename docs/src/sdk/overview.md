# SDK Overview

The AutoGPT SDK lets you embed the full agent framework as a library in your own Rust applications. You compose agents, configure the runner, and call `run()`, all with async/await and Tokio.

## Adding the Dependency

```toml
[dependencies]
autogpt = { version = "0.4.2", features = ["gem", "gpt"] }
tokio   = { version = "1",   features = ["full"] }
```

All types needed for typical usage are re-exported through `autogpt::prelude::*`.

## The `AutoGPT` Struct

`AutoGPT` is the top-level orchestration handle. It holds a list of agents and controls how they run:

```rust
use autogpt::prelude::*;

let autogpt = AutoGPT::default()
    .with(agents![agent_a, agent_b])  // register agents
    .execute(true)         // enable task execution (default: true)
    .browse(false)         // disable browser access (default: false)
    .max_tries(3)          // retry failed agents up to 3 times
    .crud(true)            // allow CRUD operations in scope
    .auth(false)           // no auth-related code generation
    .external(true)        // allow external API calls
    .build()
    .expect("Failed to build AutoGPT");
```

## Running Agents

`run()` spawns one Tokio task per agent and runs them **concurrently**:

```rust
match autogpt.run().await {
    Ok(msg) => println!("{}", msg),    // "All agents executed successfully."
    Err(e)  => eprintln!("{:?}", e),
}
```

If any agent fails all its retry attempts, `run()` returns an `Err` containing all failure messages joined into a single string.

## Inspecting Agent State After Execution

After `run()` returns, you can inspect each agent's in-memory state:

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let agent = ArchitectGPT::new("Architect", "Design a Kubernetes cluster.").await;

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .unwrap();

    autogpt.run().await.unwrap();

    let guard = autogpt.agents[0].lock().await;
    let agent_state = guard.get_agent();

    println!("Status:  {:?}", agent_state.status());
    println!("Memory:  {} messages", agent_state.memory().len());

    for msg in agent_state.memory() {
        println!("[{}] {}", msg.role, msg.content);
    }
}
```

## Accessing Long-Term Memory

With the `mem` feature, retrieve persisted Pinecone memories:

```rust
let memories = guard.get_ltm().await.unwrap();
println!("LTM messages: {}", memories.len());
```

See [Long-Term Memory](../advanced/memory-pinecone.md) for full setup.
