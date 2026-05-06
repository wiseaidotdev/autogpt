# Orchestrator (`orchgpt`)

`orchgpt` is the orchestration binary that manages agent lifecycles, routes IAC commands, and enables rich inter-agent collaboration in [Orchestrated Mode](../modes/orchestrated.md). It runs as a standalone long-lived TLS server.

## Starting the Orchestrator

```sh
orchgpt
```

By default it binds to `0.0.0.0:8443`. Override the address:

```sh
export ORCHESTRATOR_ADDRESS=0.0.0.0:9443
orchgpt
```

The orchestrator will print a confirmation and wait for agent connections.

## What the Orchestrator Does

1. **Listens** for inbound TLS connections from `autogpt` agents on the configured address
2. **Verifies** each connecting agent's Ed25519 public key against its registry
3. **Routes** IAC `Message` frames to the correct registered agent based on the `to` field
4. **Manages** agent lifecycle, registration, heartbeat, and deregistration
5. **Broadcasts** capability advertisements so agents can discover each other

## Connecting Agents

From any terminal (or remote machine), connect an agent:

```sh
autogpt --net
```

Once connected the interactive prompt accepts IAC commands:

```sh
/arch create "e-commerce backend" | python
/back create "payment processing API" | rust
/front create "checkout form UI" | javascript
```

Each command is serialized as an IAC `Message` with `msg_type = create` and sent to the orchestrator, which instantiates the appropriate agent.

## TLS Certificate Setup

The orchestrator generates ephemeral TLS credentials at startup using the `iac-rs` crate. No manual certificate provisioning is required for local development.

For production deployments behind a reverse proxy (e.g., nginx, Caddy), terminate TLS at the proxy and point the orchestrator at an internal plaintext address.

## Architecture Inside the Orchestrator

```
┌─────────────────────────────────────────┐
│              orchgpt process            │
│                                         │
│  TLS Listener                           │
│    │                                    │
│    ├── Agent Registry (HashMap)         │
│    │    ├── ArchitectGPT (id, key)      │
│    │    ├── BackendGPT   (id, key)      │
│    │    └── FrontendGPT  (id, key)      │
│    │                                    │
│    └── Message Router                   │
│         └── Dispatch by `to` field      │
└─────────────────────────────────────────┘
```

## Building the Orchestrator from Source

```sh
cd autogpt/autogpt
cargo build --release --features net,cli --bin orchgpt
./target/release/orchgpt
```

## Docker

```sh
docker run -it \
  -e GEMINI_API_KEY=<key> \
  -p 8443:8443 \
  --rm --name orchgpt kevinrsdev/orchgpt
```
