// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Common module.
//!
//! This module contains sub-modules for utility functions and common utilities that can be used across various parts of the project.
//!
//! ## Sub-modules
//!
//! - `utils`: Contains definitions and implementations of various utility functions and helpers that can be used throughout the project.
//!

pub mod utils;

#[cfg(feature = "mem")]
pub mod memory;

#[cfg(feature = "cli")]
pub mod input;

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
