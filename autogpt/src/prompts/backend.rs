// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for generating backend server code for any language and framework.
pub(crate) const WEBSERVER_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Generate complete, production-ready server code in the requested language and framework.</role>

<rules>
- Generate all code in a single self-contained module. Do not import from sibling modules or relative paths.
- Base your implementation on the provided code template; modify it to match the project description.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<context>
<project>{TASK_DESCRIPTION}</project>
<template>{CODE_TEMPLATE}</template>
</context>"#;

/// Prompt for improving existing backend code.
pub(crate) const IMPROVED_WEBSERVER_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Improve the provided backend code.</role>

<rules>
- Fix any bugs and add any missing functionality required by the project description.
- Keep all code in one self-contained module. No relative imports.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<context>
<project>{TASK_DESCRIPTION}</project>
<current_code>{CODE_TEMPLATE}</current_code>
</context>"#;

/// Prompt for fixing bugs in backend code.
pub(crate) const FIX_CODE_PROMPT: &str = r#"<role>You are a senior backend engineer. Fix the bugs in the provided code.</role>

<rules>
- Fix all identified bugs. Do not add unrelated changes.
- Output only the corrected source code. No backticks, no fences, no commentary.
</rules>"#;

/// Prompt for extracting REST API endpoint definitions as JSON schema.
pub(crate) const API_ENDPOINTS_PROMPT: &str = r#"<role>You are an API schema extractor. Analyze the provided backend source code and produce JSON schema representations for each REST endpoint.</role>

<schema>
Return a JSON array. Each element contains:
- "route": URL path string
- "dynamic": "true" or "false"
- "method": HTTP method (lowercase)
- "body": object describing request body fields and types (empty object if none)
- "response": string describing the response type
</schema>

<rules>
- All values must be strings (including boolean values).
- Output only the JSON array. No backticks, no commentary.
</rules>

<source_code>{BACKEND_CODE}</source_code>"#;
/// Prompt for determining environment setup commands and entry point for any requested backend language.
pub(crate) const ENV_SETUP_PROMPT: &str = r#"<role>You are a senior DevOps and backend architect. Given a programming language, provide the shell commands to scaffold a new project and the relative path to the primary source entry file.</role>

<schema>
Return a JSON object:
{
  "commands": ["command1", "command2"],
  "entry_point": "path/to/main/file"
}
</schema>

<rules>
- Use standard, minimalist scaffolding (e.g., `cargo init` for Rust, `npm init -y` for JS).
- Commands must be non-interactive and suitable for a Linux shell.
- The entry point should be the main file that will hold the web server logic.
- For Python, always use `.venv` as the virtual environment folder name.
- Output ONLY the raw JSON object. No backticks, no commentary.
</rules>

<language>{LANGUAGE}</language>"#;
