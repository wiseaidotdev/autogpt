# Creating Custom Agents

AutoGPT's most powerful feature is the ability to compose **any agent** from scratch using the `Auto` derive macro and the `Executor` trait. Your custom agent participates in the same runtime as the built-in agents; It can be passed to `agents!`, run concurrently, and use the full LLM client API.

## The Minimum Required Shape

A custom agent struct needs exactly two fields:

```rust
use autogpt::prelude::*;

#[derive(Debug, Default, Auto)]
pub struct MyAgent {
    pub agent: AgentGPT,    // Core agent state
    pub client: ClientType, // LLM client (Gemini, OpenAI, etc.)
}
```

The `Auto` derive macro implements `Agent`, `Functions`, `AsyncFunctions`, and `AgentFunctions` for you automatically. You only need to implement `Executor` with your custom logic.

## Implementing Executor

`Executor` has one required method, `execute`. This is where your agent does its work:

```rust
use autogpt::prelude::*;

#[derive(Debug, Default, Auto)]
pub struct SummarizerAgent {
    pub agent: AgentGPT,
    pub client: ClientType,
}

#[async_trait]
impl Executor for SummarizerAgent {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        let prompt = self.agent.behavior().clone();

        let response = self.generate(prompt.as_ref()).await?;

        self.agent.add_message(Message {
            role: "assistant".into(),
            content: response.clone().into(),
        });

        let _ = self.save_ltm(Message {
            role: "assistant".into(),
            content: response.clone().into(),
        }).await;

        println!("{}", response);
        Ok(())
    }
}
```

## Running Your Custom Agent

```rust
#[tokio::main]
async fn main() {
    let persona = "Technical Writer";
    let behavior = "Summarize the Rust ownership model for a beginner audience.";

    let agent = SummarizerAgent::new(persona.into(), behavior.into());

    AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

## Mixing Custom and Built-in Agents

Custom agents are fully interchangeable with built-in ones:

```rust
let architect  = ArchitectGPT::new("Architect", "Design a microservices diagram.").await;
let summarizer = SummarizerAgent::new("Writer".into(), "Summarize the architecture decisions.".into());

AutoGPT::default()
    .with(agents![architect, summarizer])
    .build()
    .unwrap()
    .run()
    .await
    .unwrap();
```

Both run concurrently. If you need sequential ordering, run two separate `AutoGPT` instances back-to-back.

## Using `generate` vs `stream`

The `AsyncFunctions` trait (derived by `Auto`) exposes two generation methods:

| Method                  | Return                | Use When                                      |
| ----------------------- | --------------------- | --------------------------------------------- |
| `self.generate(prompt)` | `Result<String>`      | You need the full response as a string        |
| `self.stream(prompt)`   | `Result<ReqResponse>` | You want to stream tokens progressively       |
| `self.imagen(prompt)`   | `Result<Vec<u8>>`     | You need image bytes (requires `img` feature) |

## Working with Task Scope

The `task` argument carries the `Scope` configured on `AutoGPT`:

```rust
async fn execute<'a>(&'a mut self, task: &'a mut Task, ...) -> Result<()> {
    if let Some(scope) = &task.scope {
        if scope.crud {
            // write files to disk
        }
        if scope.external {
            // call external APIs
        }
    }
    Ok(())
}
```

This makes your agent scope-aware and compatible with access control policies at the `AutoGPT` level.

## Complete Working Example

See the [`gemini-custom-agent`](https://github.com/wiseaidotdev/autogpt/tree/main/examples/gemini-custom-agent) example in the repository for a minimal end-to-end Cargo project:

```sh
cd examples/gemini-custom-agent
export GEMINI_API_KEY=<your_key>
cargo run --features=gem
```
