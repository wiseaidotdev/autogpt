// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for generating frontend code for any language and framework.
pub(crate) const FRONTEND_CODE_PROMPT: &str = r#"<role>You are a senior frontend engineer. Generate complete, production-ready frontend code in the requested language and framework.</role>

<rules>
- Base your implementation on the provided code template; modify it to match the project description.
- Write components and functions appropriate for the request.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<context>
<project>{TASK_DESCRIPTION}</project>
<template>{CODE_TEMPLATE}</template>
</context>"#;

/// Prompt for improving existing frontend code.
pub(crate) const IMPROVED_FRONTEND_CODE_PROMPT: &str = r#"<role>You are a senior frontend engineer. Improve the provided frontend code.</role>

<rules>
- Fix any bugs and add any missing functionality required by the project description.
- Output only raw source code. No backticks, no fences, no commentary.
</rules>

<context>
<project>{TASK_DESCRIPTION}</project>
<current_code>{CODE_TEMPLATE}</current_code>
</context>"#;

/// Prompt for fixing bugs in frontend code.
pub(crate) const FIX_CODE_PROMPT: &str = r#"<role>You are a senior frontend engineer. Fix the bugs in the provided code.</role>

<rules>
- Fix all identified bugs. Do not add unrelated changes.
- Output only the corrected source code. No backticks, no fences, no commentary.
</rules>"#;

/// Prompt for determining environment setup commands and entry point for any requested frontend language.
pub(crate) const ENV_SETUP_PROMPT: &str = r#"<role>You are a senior DevOps and frontend architect. Given a programming language, provide the shell commands to scaffold a new project and the relative path to the primary source entry file.</role>

<schema>
Return a JSON object:
{
  "commands": ["command1", "command2"],
  "entry_point": "path/to/main/file"
}
</schema>

<rules>
- Use standard, minimalist scaffolding (e.g., `npx create-vite-app@latest ./ --template vanilla` for JS/TS).
- Commands must be non-interactive and suitable for a Linux shell.
- The entry point should be the main file that will hold the frontend logic.
- For Python, always use `.venv` as the virtual environment folder name.
- Output ONLY the raw JSON object. No backticks, no commentary.
</rules>

<language>{LANGUAGE}</language>"#;
