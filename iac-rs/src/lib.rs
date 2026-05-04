// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/wiseaidotdev/autogpt/refs/heads/main/iac-rs/assets/logo.webp",
    html_favicon_url = "https://github.com/wiseaidotdev/autogpt/blob/main/iac-rs/assets/favicon.png"
)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]

// TODO: Add non std support
// #![no_std]
// extern crate alloc;

pub mod client;
pub mod crypto;
pub mod message;
pub mod prelude;
pub mod server;
pub mod traits;
pub mod transport;

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
