# IAC Protocol

The **IAC (Inter/Intra-Agent Communication)** protocol is AutoGPT's purpose-built communication layer for distributed multi-agent systems. It is the transport backbone of [Orchestrated Mode](../modes/orchestrated.md).

## Why IAC Exists

Traditional inter-process communication methods, REST, gRPC, bare TCP, suffer under demanding multi-agent workloads:

- **TCP + TLS handshake cost**: every new connection incurs at least 1 RTT of latency.
- **Head-of-line blocking**: HTTP/2 multiplexing still stalls when a single packet is lost.
- **Token-based authentication**: bearer tokens lack cryptographic agent identity and granular revocation.
- **Centralized topologies**: REST and RPC assume a hub; they have no native peer-to-peer delegation.

IAC addresses all of these with a single coherent protocol.

## Transport: QUIC over TLS 1.3

IAC defaults to QUIC (currently implemented as TLS 1.3 over TCP in the Rust implementation, with full QUIC as a roadmap item):

| Property                    | Benefit                                                           |
| --------------------------- | ----------------------------------------------------------------- |
| 0-RTT reconnection          | Reuse prior session parameters for near-instant reconnect         |
| Stream multiplexing         | Multiple concurrent message streams with no head-of-line blocking |
| Adaptive congestion control | BBR-based pacing keeps throughput smooth at scale                 |
| Graceful fallback           | Degrades to TLS/TCP or UNIX domain sockets when needed            |

## Cryptographic Agent Identity

Each agent generates an **Ed25519 keypair** at runtime. The public key is the agent's identity, it replaces opaque tokens and API keys.

Every IAC message is signed:

1. Clone the message and zero the `signature` field
2. Serialize via Protocol Buffers
3. Sign the serialized bytes with the agent's Ed25519 private key
4. Write the signature bytes back to `signature`

The receiver reverses the process to verify. This guarantees **unforgeable message provenance**, only the private-key holder can produce a valid signature.

## Message Format (Protobuf Schema)

```protobuf
syntax = "proto3";
package iac;

enum MessageType {
  UNKNOWN        = 0;
  PING           = 1;
  BROADCAST      = 2;
  FILE_TRANSFER  = 3;
  COMMAND        = 4;
  DELEGATE_TASK  = 5;
}

message Message {
  string from         = 1;  // sender identity (hex public key or alias)
  string to           = 2;  // recipient identity
  MessageType msg_type = 3;
  string payload_json = 4;  // UTF-8 JSON metadata or control object
  uint64 timestamp    = 5;  // microsecond precision
  uint64 msg_id       = 6;  // monotonic counter for deduplication
  uint64 session_id   = 7;  // unique per connection
  bytes  signature    = 8;  // Ed25519 signature over the serialized message
  bytes  extra_data   = 9;  // binary payload (file chunks, compressed frames)
}
```

## Communication Topologies

| Topology               | Description                                                                 |
| ---------------------- | --------------------------------------------------------------------------- |
| **Inter-Agent**        | Global QUIC mesh; agents broadcast, delegate, or proxy messages to peers    |
| **Agent-Orchestrator** | Hierarchical; orchestrator acts as bootstrap registry and command router    |
| **Intra-Agent**        | CPU-local using shared IPC or UNIX domain sockets for minimal-latency pings |

## Efficiency Features

**Frame compression**: zstd is negotiated per session. Dictionary-based compression yields up to 85% payload reduction on repetitive agent messages.

**Deduplication**: receivers track `msg_id` and `session_id` to drop duplicate messages from network retransmissions.

**Batching**: low-overhead messages are flushed together, reducing per-message overhead.

**File transfer**: `msg_type = FILE_TRANSFER` chunks large files with index, total, and checksum metadata in `payload_json`; raw bytes in `extra_data`.

## Comparison: IAC vs. REST/gRPC

| Feature             | REST / gRPC | IAC (TLS/TCP) | IAC (QUIC)             |
| ------------------- | ----------- | ------------- | ---------------------- |
| Zero-RTT setup      | ❌          | ❌            | ✅                     |
| Stream multiplexing | ❌          | ❌            | ✅ (no HOL blocking)   |
| Agent identity      | API tokens  | Bearer tokens | ✅ Ed25519 keypair     |
| Mesh / P2P support  | ❌          | Partial       | ✅ Native delegation   |
| Frame compression   | ❌          | ❌            | ✅ zstd + dictionaries |
| File transfer       | ❌          | ❌            | ✅ Chunked + checksum  |
| Dedup / reorder     | ❌          | ❌            | ✅ Protocol-level      |

## SDK: Sending an IAC Message

```rust
use autogpt::prelude::*;

async fn send_task(client: &Client, signer: &Signer, to: &str, payload: &str) -> Result<()> {
    let mut msg = IacMessage {
        from: "my-agent".into(),
        to: to.into(),
        msg_type: MessageType::DelegateTask,
        payload_json: payload.into(),
        ..Default::default()
    };
    msg.sign(signer)?;
    client.send(msg).await
}
```

For a complete working example see [`gemini-iac-client-server`](https://github.com/wiseaidotdev/autogpt/tree/main/examples/gemini-iac-client-server).
