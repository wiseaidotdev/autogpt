# Long-Term Memory (Pinecone)

AutoGPT can persist agent conversation history to a [Pinecone](https://www.pinecone.io/) vector database, giving agents durable memory across separate runs. This enables agents that learn from prior interactions, recall past decisions, and improve over time.

## Enabling Long-Term Memory

Add the `mem` feature flag:

```toml
[dependencies]
autogpt = { version = "0.2", features = ["gem", "gpt", "mem"] }
```

Set the Pinecone environment variables:

```sh
export PINECONE_API_KEY=<your_pinecone_api_key>
export PINECONE_INDEX_URL=<your_pinecone_index_url>
```

The index URL has the form `https://<index-name>-<project-id>.svc.<region>.pinecone.io`.

Follow the [Pinecone setup guide](https://github.com/wiseaidotdev/autogpt/blob/main/PINECONE.md) for step-by-step index creation instructions.

## How It Works

When a `mem`-enabled agent runs, after each LLM response it automatically calls `save_ltm` to upsert the message into Pinecone as a vector embedding. On subsequent runs, `get_ltm` retrieves the most semantically relevant prior messages and prepends them to the conversation context.

## API Reference

### `save_ltm`

```rust
async fn save_ltm(&mut self, message: Message) -> Result<()>
```

Upserts a `Message` into the Pinecone index. Called automatically by built-in agents; call manually in custom `Executor` implementations:

```rust
self.save_ltm(Message {
    role: "assistant".into(),
    content: response.clone().into(),
}).await?;
```

### `get_ltm`

```rust
async fn get_ltm(&self) -> Result<Vec<Message>>
```

Retrieves all stored messages for this agent from Pinecone.

### `ltm_context`

```rust
async fn ltm_context(&self) -> String
```

Returns all stored messages formatted as `"role: content\n"`, ready to be prepended to a new prompt.

## Inspecting Memory After a Run

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
    let state = guard.get_agent();

    println!("In-memory messages: {}", state.memory().len());
    println!("First role:  {}", state.memory()[0].role);
    println!("Second role: {}", state.memory()[1].role);
    println!("Status: {:?}", state.status());
}
```

Expected output when memory is populated:

```
All agents executed successfully.
In-memory messages: 3
First role:  user
Second role: assistant
Status: Completed
```
