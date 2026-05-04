// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use autogpt::agents::manager::ManagerGPT;
use tracing::debug;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_manager_gpt() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let persona = "Manager";
    let behavior = "Expertise at managing projects at scale";
    let request = "Develop a full stack app that fetches today's weather in python using FastAPI.";
    let language = "python";

    let mut manager = ManagerGPT::new(persona, behavior, request, language);

    let _ = manager.execute(true, false, 3).await;

    debug!("{:?}", manager);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
