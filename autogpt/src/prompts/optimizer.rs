// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for identifying the modular file structure of a monolithic source file.
pub(crate) const MODULARIZE_PROMPT: &str = r#"<role>You are a surgical code architect. Divide a monolithic source file into logical nested modules.</role>

<rules>
- Output ONLY a JSON array of strings representing the new file paths: `["foo/bar.rs", "foo/baz.rs"]`.
- The new paths must reflect a clean, standard modular structure.
- Include proper file extensions.
- Do not output markdown fencing, bullets, explanations or extra text.
</rules>

<source_code>{SOURCE_CODE}</source_code>"#;

/// Prompt for extracting the code belonging to a specific module file from a full codebase.
pub(crate) const SPLIT_PROMPT: &str = r#"<role>You are a precise code extraction engine. Extract the code belonging to a specified module from a monolithic codebase.</role>

<rules>
- Extract only code that belongs strictly to the requested file path.
- Include all necessary module imports and ensure the code functions independently.
- Output ONLY the raw source code starting from the first line. 
- Do not provide backticks, explanations, or commentary.
</rules>

<context>
<filename>{FILENAME}</filename>
<full_codebase>{FULL_CODE}</full_codebase>
</context>"#;
