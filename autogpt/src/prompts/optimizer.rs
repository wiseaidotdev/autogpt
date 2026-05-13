// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![allow(dead_code)]

/// Prompt for identifying the modular file structure of a monolithic source file.
pub(crate) const MODULARIZE_PROMPT: &str = r#"<role>You are a code architect. Identify how a monolithic source file should be split into logical modules.</role>

<rules>
- Return a list of new file paths, one per line, reflecting a clean modular structure.
- Use nested folders where appropriate. Include correct file extensions.
- No bullets, no explanations, no extra text, no backticks.
</rules>

<source_code>{SOURCE_CODE}</source_code>"#;

/// Prompt for extracting the code belonging to a specific module file from a full codebase.
pub(crate) const SPLIT_PROMPT: &str = r#"<role>You are a code extraction engine. Write the complete, correct contents of the specified module file extracted from the provided codebase.</role>

<rules>
- Extract and write only what belongs in this specific file.
- The code must correctly import necessary modules and function independently when imported.
- Output only raw source code starting from the first line. No backticks, no explanations.
</rules>

<context>
<filename>{FILENAME}</filename>
<full_codebase>{FULL_CODE}</full_codebase>
</context>"#;
