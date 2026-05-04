// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use autogpt::agents::git::GitGPT;
use autogpt::common::utils::{Status, Task};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::AsyncFunctions;
use autogpt::traits::functions::Functions;
use std::env;
use std::fs;
use std::path::Path;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

#[tokio::test]
async fn test_git_gpt_execute() {
    let filter = filter::LevelFilter::DEBUG;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let test_workspace = "workspace/";
    unsafe { env::set_var("AUTOGPT_WORKSPACE", "workspace/") };

    if Path::new(test_workspace).exists() {
        fs::remove_dir_all(test_workspace).unwrap();
    }

    let mut git_agent = GitGPT::new("GitGPT", "Commit all changes").await;

    let dummy_file_path = format!("{test_workspace}/hello.txt");
    fs::create_dir_all(test_workspace).unwrap();
    fs::write(&dummy_file_path, "Hello, GitGPT!").unwrap();

    let mut task = Task {
        description: "Initial commit - Added hello.txt".into(),
        scope: None,
        urls: None,
        frontend_code: None,
        backend_code: None,
        api_schema: None,
    };

    let result = git_agent.execute(&mut task, true, false, 1).await;

    assert!(result.is_ok());
    assert_eq!(git_agent.get_agent().status(), &Status::Completed);

    let repo = git2::Repository::open(test_workspace).unwrap();
    let mut revwalk = repo.revwalk().unwrap();
    revwalk.push_head().unwrap();

    let commit_count = revwalk.count();
    assert!(commit_count >= 1);

    fs::remove_dir_all(test_workspace).unwrap();
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
