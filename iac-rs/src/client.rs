// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::crypto::Signer;
use crate::message::Message;
use crate::transport::connect;
use anyhow::Result;
use quinn::Connection;
use tracing::{debug, instrument};
use zstd::decode_all;
use zstd::stream::encode_all;

#[derive(Clone, Debug)]
pub struct Client {
    conn: Connection,
    signer: Signer,
}

impl PartialEq for Client {
    fn eq(&self, other: &Self) -> bool {
        // TODO: File an issue to allow quinn Connection to implement PartialEq.
        // For now, let's compare the signer
        self.signer == other.signer
    }
}

impl Client {
    #[instrument(skip_all, fields(addr))]
    pub async fn connect(addr: &str, signer: Signer) -> Result<Self> {
        debug!(%addr, "🌐 Connecting to server...");
        let conn = connect(addr).await?;
        debug!("✅ Client connected to {}", addr);
        Ok(Self { conn, signer })
    }

    #[instrument(skip_all, fields(to = %msg.to, from = %msg.from, msg_id = msg.msg_id))]
    pub async fn send(&self, mut msg: Message) -> Result<()> {
        msg.sign(&self.signer)?;
        debug!("🖋️ Message signed");

        let data = msg.serialize()?;
        debug!(original_len = data.len(), "📦 Message serialized");

        let compressed = encode_all(&data[..], 0)?;
        debug!(compressed_len = compressed.len(), "📉 Message compressed");

        debug!("🔓 Opening unidirectional stream");
        let mut stream = self.conn.open_uni().await?;

        debug!("✍️ Writing {} bytes to stream", compressed.len());
        stream.write_all(&compressed).await?;

        debug!("✅ Write complete, finalizing stream...");
        stream.finish()?;

        debug!("📤 Stream finished successfully");
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn receive(&self) -> Result<Option<Message>> {
        debug!("📥 Waiting for incoming unidirectional stream...");
        match self.conn.accept_uni().await {
            Ok(mut recv) => {
                debug!("🔓 Stream accepted, reading message...");

                let mut compressed = Vec::new();
                recv.read_to_end(usize::MAX).await.inspect(|data| {
                    compressed.extend_from_slice(data);
                    debug!(
                        compressed_len = compressed.len(),
                        "📦 Compressed data received"
                    );
                })?;

                debug!("📈 Decompressing message...");
                let decompressed = decode_all(&compressed[..])?;
                debug!(
                    decompressed_len = decompressed.len(),
                    "✅ Decompression complete"
                );

                debug!("🧠 Deserializing message...");
                let msg = Message::deserialize(&decompressed)?;
                debug!("📬 Message deserialized: {:?}", msg.msg_type);

                Ok(Some(msg))
            }
            Err(e) => {
                debug!("❌ Failed to receive stream: {:?}", e);
                Ok(None)
            }
        }
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
