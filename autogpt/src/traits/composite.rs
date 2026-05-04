// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Functions};
use async_trait::async_trait;
use std::fmt::Debug;

#[async_trait]
pub trait AgentFunctions: Agent + Functions + AsyncFunctions + Send + Sync + Debug {}

impl<T> AgentFunctions for T where T: Agent + Functions + AsyncFunctions + Send + Sync + Debug {}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
