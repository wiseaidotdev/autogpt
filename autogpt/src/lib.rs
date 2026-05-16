// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/wiseaidotdev/autogpt/refs/heads/main/assets/logo.png",
    html_favicon_url = "https://raw.githubusercontent.com/wiseaidotdev/autogpt/refs/heads/main/assets/favicon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

extern crate self as autogpt;

#[doc(hidden)]
pub use anyhow;
#[cfg(feature = "hf")]
#[doc(hidden)]
pub use api_huggingface;
#[doc(hidden)]
pub use futures;
#[doc(hidden)]
pub use serde_json;

pub mod agents;
pub mod common;
pub mod macros;
pub mod prelude;
#[cfg(any(feature = "gpt", feature = "cli"))]
pub mod prompts;
pub mod traits;

#[cfg(feature = "net")]
pub mod collaboration;

#[cfg(feature = "cli")]
pub mod cli;

#[cfg(feature = "cli")]
pub mod tui;

#[cfg(feature = "mcp")]
pub mod mcp;

#[cfg(all(feature = "cli", feature = "net", feature = "gpt"))]
pub mod orchestrator;

#[cfg(feature = "cli")]
pub mod message;

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
