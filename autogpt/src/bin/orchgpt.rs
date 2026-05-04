// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

/// The main entry point of `orchgpt`.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(all(feature = "cli", feature = "net"))]
    {
        use autogpt::cli::orchgpt::Cli;
        use autogpt::common::utils::setup_logging;
        use autogpt::orchestrator::Orchestrator;
        use clap::Parser;
        use iac_rs::prelude::*;
        use tracing::error;

        let _args: Cli = Cli::parse();

        setup_logging()?;

        let signer = Signer::new(KeyPair::generate());
        let verifier = Verifier::new(vec![]);

        let mut orchestrator =
            Orchestrator::new("orchestrator".to_string(), signer, verifier).await?;

        if let Err(e) = orchestrator.run().await {
            error!("Orchestrator error: {:?}", e);
        }
    }

    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
