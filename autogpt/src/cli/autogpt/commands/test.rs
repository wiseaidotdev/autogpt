// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::cli::autogpt::parser::parse_yaml;
use crate::cli::autogpt::utils::*;
use anyhow::Result;

pub fn handle_test() -> Result<()> {
    spinner("Validating YAML file", || {
        let config = parse_yaml("agent.yaml")?;
        println!("🔍 Agent(s) parsed:\n{config:#?}");
        Ok(())
    })?;

    success("✅ YAML is valid");
    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
