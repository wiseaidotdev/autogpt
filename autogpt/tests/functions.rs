// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use anyhow::Result;
use async_trait::async_trait;
use autogpt::agents::agent::AgentGPT;
use autogpt::common::utils::{Message, Route, Scope, Task};
use autogpt::traits::agent::Agent;
use autogpt::traits::functions::AsyncFunctions;
use autogpt::traits::functions::Functions;
use autogpt::traits::functions::ReqResponse;
use serde_json::json;
use std::borrow::Cow;
use tracing::info;
use tracing_subscriber::{filter, fmt, prelude::*, reload};

pub struct MockFunctions {
    agent: AgentGPT,
}

impl Functions for MockFunctions {
    fn get_agent(&self) -> &AgentGPT {
        &self.agent
    }
}

#[async_trait]
impl AsyncFunctions for MockFunctions {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        _execute: bool,
        _browse: bool,
        _max_tries: u64,
    ) -> Result<()> {
        info!(
            "[*] {:?}: Executing tasks: {:?}",
            self.agent.persona(),
            task.clone()
        );

        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;

        task.description = "Updated description".into();

        Ok(())
    }

    async fn save_ltm(&mut self, _message: Message) -> Result<()> {
        Ok(())
    }

    async fn get_ltm(&self) -> Result<Vec<Message>> {
        Ok(vec![
            Message {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("System initialized."),
            },
            Message {
                role: Cow::Borrowed("user"),
                content: Cow::Borrowed("Hello, autogpt!"),
            },
        ])
    }

    async fn ltm_context(&self) -> String {
        let messages = [
            Message {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("System initialized."),
            },
            Message {
                role: Cow::Borrowed("user"),
                content: Cow::Borrowed("Hello, autogpt!"),
            },
        ];

        messages
            .iter()
            .map(|c| format!("{}: {}", c.role, c.content))
            .collect::<Vec<_>>()
            .join("\n")
    }

    async fn generate(&mut self, _request: &str) -> Result<String> {
        Ok("".to_string())
    }

    async fn imagen(&mut self, _request: &str) -> Result<Vec<u8>> {
        // TODO: Impl
        Ok(Default::default())
    }
    async fn stream(&mut self, _request: &str) -> Result<ReqResponse> {
        // TODO: Impl
        Ok(ReqResponse(None))
    }
}

#[tokio::test]
async fn test_functions_execution() {
    let filter = filter::LevelFilter::INFO;
    let (filter, _reload_handle) = reload::Layer::new(filter);
    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::Layer::default())
        .init();

    let persona = "Persona";
    let behavior = "Behavior";
    let agent = AgentGPT::new_borrowed(persona, behavior);

    let mut task = Task {
        description: Cow::Borrowed("Task"),
        scope: Some(Scope {
            crud: true,
            auth: false,
            external: true,
        }),
        urls: Some(vec![Cow::Borrowed("https://wiseai.dev")]),
        backend_code: Some(Cow::Borrowed("fn main() {}")),
        frontend_code: None,
        api_schema: Some(vec![
            Route {
                dynamic: Cow::Borrowed("no"),
                method: Cow::Borrowed("GET"),
                body: json!({}),
                response: json!({}),
                path: Cow::Borrowed("/path"),
            },
            Route {
                dynamic: Cow::Borrowed("yes"),
                method: Cow::Borrowed("POST"),
                body: json!({"key": "value"}),
                response: json!({"success": true}),
                path: Cow::Borrowed("/path"),
            },
        ]),
    };

    let mut functions = MockFunctions { agent };

    let result = functions.execute(&mut task, true, false, 3).await;

    assert!(result.is_ok());
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
