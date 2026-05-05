# Orchestrated Mode

Orchestrated Mode connects `autogpt` agents to an external `orchgpt` orchestrator over a secure TLS-encrypted TCP channel. This enables distributed, multi-agent collaboration across machines, not just locally.

## Starting the Orchestrator

On the host that will coordinate agents, start `orchgpt`:

```sh
orchgpt
```

By default it binds to `0.0.0.0:8443`. To use a different address:

```sh
export ORCHESTRATOR_ADDRESS=0.0.0.0:9443
orchgpt
```

## Connecting an Agent

In a separate terminal (or on another machine), connect `autogpt` to the running orchestrator:

```sh
autogpt --net
```

`autogpt` resolves the orchestrator address from `ORCHESTRATOR_ADDRESS` and establishes a TLS session using the IAC protocol.

## Sending Commands

Once connected, you interact with agents using this command syntax:

```
/<agent_name> <action> "<payload>" | <language>
```

**Examples:**

```sh
# Create an ArchitectGPT agent for a FastAPI app using Python
/arch create "fastapi app" | python

# Create a BackendGPT agent for a REST API in Rust
/back create "user auth REST API" | rust

# Create a FrontendGPT agent for a React app
/front create "dashboard UI with charts" | javascript
```

Each command sends an IAC `Message` with:

| Field          | Value                                |
| -------------- | ------------------------------------ |
| `msg_type`     | `create`                             |
| `to`           | `ArchitectGPT` / `BackendGPT` / etc. |
| `payload_json` | Your project goal string             |
| `language`     | Target programming language          |

## Communication Flow

```
┌─────────┐   TLS+TCP (IAC)   ┌──────────────┐   IAC   ┌────────────┐
│ autogpt │ ────────────────▶ │   orchgpt    │ ──────▶ │ AgentGPTs  │
│  (CLI)  │ ◀──────────────── │(Orchestrator)│ ◀────── │ (workers)  │
└─────────┘    Responses      └──────────────┘         └────────────┘
```

All messages travel over **TLS 1.3** and are encoded in **Protocol Buffers**. Every message is signed with **Ed25519** to verify agent identity. See [IAC Protocol](../advanced/iac-protocol.md) for full details.

## Running via Docker Compose

The included `docker-compose.yml` sets up both services with a shared network:

```sh
docker compose up --build
```

This builds and starts:

- `autogpt` container, the agent.
- `orchgpt` container, the orchestrator.

Docker Compose configures the internal network automatically, so `autogpt` can reach `orchgpt` by container name.

## Using the SDK for Networked Agents

Enable the `net` feature flag and use the `Collaborator` type to register remote agents:

```rust
use autogpt::prelude::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "127.0.0.1:4555";
    let signer = Signer::new(KeyPair::generate());
    let client = Client::connect(addr, signer.clone()).await?;

    let mut clients = HashMap::new();
    clients.insert("frontend".into(), Arc::new(Mutex::new(client.clone())));

    let mut agent = AgentGPT::new("Design software".into(), "designer".into());
    agent.signer = signer;
    agent.clients = clients;
    agent.addr = addr.into();
    agent.capabilities.insert(Capability::CodeGen);

    agent.broadcast_capabilities().await?;
    Ok(())
}
```

<div class="callout callout-tip">
<strong>💡 Tip</strong>
See the <a href="https://github.com/wiseaidotdev/autogpt/tree/main/examples/gemini-iac-client-server">gemini-iac-client-server</a> example for a complete working client/server setup with a designer agent and frontend agent communicating through an IAC server.
</div>
