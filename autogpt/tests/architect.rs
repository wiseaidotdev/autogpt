// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use autogpt::agents::architect::ArchitectGPT;
use autogpt::common::utils::{Scope, Status, Task};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::{AsyncFunctions, Functions};
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
#[ignore]
async fn test_get_scope() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let persona = "Lead UX/UI Designer";
    let behavior = "Creates innovative website designs and user experiences";

    let mut architect_agent = ArchitectGPT::new(persona, behavior).await;

    let mut task = Task {
        description: "Create a blog platform for publishing articles and comments.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    let scope = architect_agent.get_scope(&mut task).await.unwrap();

    assert!(
        scope
            == Scope {
                crud: true,
                auth: true,
                external: false
            }
            || scope
                == Scope {
                    crud: true,
                    auth: true,
                    external: true
                },
        "Unexpected scope value: {scope:?}",
    );

    assert_eq!(architect_agent.get_agent().status(), &Status::Completed);
}

#[tokio::test]
async fn test_get_urls() {
    let persona = "Lead UX/UI Designer";
    let behavior = "Creates innovative website designs and user experiences";

    let mut architect_agent = ArchitectGPT::new(persona, behavior).await;

    let mut task = Task {
        description: "Create a weather forecast website for global cities.".into(),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: Some(Vec::new()),
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    let _ = architect_agent.get_urls(&mut task).await;
    // 1 msg from user and 1 msg from assistant -> 2
    assert!(architect_agent.get_agent().memory().len() >= 2);
    // assert_eq!(architect_agent.get_agent().memory()[0].role, "user");
    // assert_eq!(architect_agent.get_agent().memory()[1].role, "assistant");

    // assert!(!task.urls.unwrap().is_empty());
    // assert_eq!(architect_agent.get_agent().status(), &Status::InUnitTesting);
}

#[tokio::test]
async fn test_architect_agent() {
    let persona = "Lead UX/UI Designer";
    let behavior = "Creates innovative website designs and user experiences";

    let mut architect_agent = ArchitectGPT::new(persona, behavior).await;

    let mut task = Task {
        description: "Create a weather forecast website for global cities.".into(),
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

    architect_agent
        .execute(&mut task, true, false, 1)
        .await
        .unwrap();
    assert!(architect_agent.get_agent().memory().len() >= 3);
    // assert_eq!(architect_agent.get_agent().memory()[0].role, "user");
    // assert_eq!(architect_agent.get_agent().memory()[1].role, "usassistanter");

    assert!(task.scope.is_some());
    // assert!(task.urls.is_some());

    dbg!(task);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
