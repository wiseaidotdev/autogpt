// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[macro_export]
macro_rules! agents {
    ( $($agent:expr),* $(,)? ) => {
        vec![
            $(
                std::sync::Arc::new(tokio::sync::Mutex::new(Box::new($agent) as Box<dyn AgentFunctions>))
            ),*
        ]
    };
}
