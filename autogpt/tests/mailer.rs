#[cfg(feature = "mail")]
use autogpt::agents::mailer::MailerGPT;
use autogpt::common::utils::Scope;
use autogpt::common::utils::Task;
use autogpt::traits::functions::AsyncFunctions;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
#[ignore]
#[cfg(feature = "mail")]
async fn test_mailer_gpt() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let persona = "Mailer";
    let behavior = "Expertise at summarizing emails";
    let request = "Summarize the content of the 5 recent email messages";

    let mut mailer = MailerGPT::new(persona, behavior).await;
    let mut task = Task {
        description: request.into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };
    let _ = mailer.execute(&mut task, true, false, 3).await;
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
