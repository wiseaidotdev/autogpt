// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

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

#[cfg(all(feature = "cli", feature = "net"))]
pub mod orchestrator;

#[cfg(feature = "cli")]
pub mod message;

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
