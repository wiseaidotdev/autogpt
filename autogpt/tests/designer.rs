// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(unused)]

use anyhow::Result;
#[cfg(feature = "img")]
use autogpt::agents::designer::DesignerGPT;
use autogpt::common::utils::{Status, Task};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::AsyncFunctions;
use autogpt::traits::functions::Functions;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
#[cfg(feature = "img")]
async fn test_generate_image_from_text() -> Result<()> {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let persona = "Web Designer";
    let behavior = "Crafts stunning web design layouts";

    let mut designer_agent = DesignerGPT::new(persona, behavior).await;

    let mut task = Task {
        description: "Generate a kanban-style task management board. The board is divided into three columns: To Do, In Progress, and Done. Each column contains a list of tasks. The tasks in the To Do column are prioritized from highest to lowest, with the highest priority task at the top. The tasks in the In Progress column are listed in the order in which they were started. The tasks in the Done column are listed in the order in which they were completed.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    designer_agent.generate_image_from_text(&task).await?;
    assert_eq!(designer_agent.get_agent().memory().len(), 3);
    assert_eq!(designer_agent.get_agent().memory()[0].role, "user");
    assert_eq!(designer_agent.get_agent().memory()[1].role, "assistant");

    Ok(())
}

#[tokio::test]
#[cfg(feature = "img")]
async fn test_execute_agent() -> Result<()> {
    let persona = "Web Designer";
    let behavior = "Crafts stunning web design layouts";

    let mut designer_agent = DesignerGPT::new(persona, behavior).await;

    let mut task = Task {
        description: "A kanban-style task management board. The board is divided into three columns: To Do, In Progress, and Done. Each column contains a list of tasks. The tasks in the To Do column are prioritized from highest to lowest, with the highest priority task at the top. The tasks in the In Progress column are listed in the order in which they were started. The tasks in the Done column are listed in the order in which they were completed.".into(),
        scope: None,
        urls: None,
        backend_code: None,
        frontend_code: None,
        api_schema: None,
    };

    designer_agent.execute(&mut task, true, false, 3).await?;
    assert_eq!(designer_agent.get_agent().memory().len(), 3);
    assert_eq!(designer_agent.get_agent().memory()[0].role, "user");
    assert_eq!(designer_agent.get_agent().memory()[1].role, "assistant");

    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
