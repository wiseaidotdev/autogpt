// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cli::autogpt::ast::AgentConfig;
use anyhow::Result;
use convert_case::{Case, Casing};
use std::fs;
use std::path::Path;

pub fn generate_code(config: &AgentConfig, out: &Path) -> Result<()> {
    let struct_name = config.name.to_case(Case::UpperCamel);
    let code = format!(
        r#"#![allow(unused)]

use autogpt::prelude::*;

#[derive(Debug, Default, Auto)]
pub struct {name} {{
    agent: AgentGPT,
    client: ClientType,
}}

#[async_trait]
impl Executor for {name} {{
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {{
        let prompt = self.agent.behavior().clone();
        let response = self.generate(prompt.as_ref()).await?;

        self.agent.add_message(Message {{
            role: "{role}".into(),
            content: response.clone().into(),
        }});

        println!("{{}}", response);
        Ok(())
    }}
}}

#[tokio::main]
async fn main() {{
    let agent = {name}::new(
        "{persona}".into(),
        "{prompt}".into()
    );

    let autogpt = AutoGPT::default()
        .with(agents![agent])
        .build()
        .expect("Build failed");

    match autogpt.run().await {{
        Ok(response) => println!("{{}}", response),
        Err(err) => eprintln!("Agent error: {{:?}}", err),
    }}
}}
"#,
        name = struct_name,
        role = config.role,
        prompt = config.prompt,
        persona = config.persona,
    );

    fs::write(out, code)?;
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
