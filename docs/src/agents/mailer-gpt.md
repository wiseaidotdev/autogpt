# MailerGPT

<span class="badge badge-blue">Feature: mail</span>

MailerGPT integrates with the [Nylas](https://developer.nylas.com/) email API to read, parse, and send emails autonomously. It extracts relevant information from incoming messages and generates personalized outgoing email content based on user-defined goals.

## What MailerGPT Solves

Automating email workflows typically requires complex IMAP/SMTP integrations with fragile parsing logic. MailerGPT uses the Nylas unified API to abstract email providers and the LLM to handle content understanding and generation, turning natural language goals into fully composed emails.

## Enabling MailerGPT

MailerGPT requires the `mail` feature flag and three Nylas environment variables:

```sh
cargo install autogpt --features mail,gem

export NYLAS_SYSTEM_TOKEN=<your_nylas_system_token>
export NYLAS_CLIENT_ID=<your_nylas_client_id>
export NYLAS_CLIENT_SECRET=<your_nylas_client_secret>
```

Follow the [Nylas setup guide](https://github.com/wiseaidotdev/autogpt/blob/main/NYLAS.md) to obtain these credentials.

## How It Works

1. MailerGPT connects to the Nylas API using the configured credentials
2. It reads emails from the configured inbox and parses their content
3. Based on the agent's `behavior` goal, it composes reply or new outbound email content using the LLM
4. It sends the generated email via the Nylas API

## SDK Usage

```rust
use autogpt::prelude::*;

#[tokio::main]
async fn main() {
    let persona = "Customer Support Agent";
    let behavior = "Read incoming support emails and draft polite, helpful replies.";

    let agent = MailerGPT::new(persona, behavior).await;

    AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Failed to build AutoGPT")
        .run()
        .await
        .unwrap();
}
```

<div class="callout callout-warning">
<strong>⚠️ Warning</strong>
MailerGPT will send real emails via the configured Nylas account. Always test with a sandbox account before using production credentials.
</div>
