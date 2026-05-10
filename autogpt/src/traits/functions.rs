// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # `Functions` and `AsyncFunctions` traits.
//!
//! These traits define special functions for agents.
//!
//! # Examples
//!
//! ```rust
//! use autogpt::agents::agent::AgentGPT;
//! use autogpt::common::utils::Task;
//! use anyhow::Result;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//! use autogpt::traits::functions::ReqResponse;
//! use autogpt::common::utils::Message;
//! use autogpt::prelude::*;
//! use std::borrow::Cow;
//!
//!
//! /// A struct implementing the `Functions` trait.
//! struct SpecialFunctions {
//!     agent: AgentGPT,
//! }
//!
//! impl SpecialFunctions {
//!     /// Creates a new instance of `SpecialFunctions`.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `agent` - The agent to associate with the functions.
//!     fn new(agent: AgentGPT) -> Self {
//!         SpecialFunctions { agent }
//!     }
//! }
//!
//! impl Functions for SpecialFunctions {
//!     /// Get fields from an agent.
//!     ///
//!     /// # Returns
//!     ///
//!     /// A reference to the agent.
//!     fn get_agent(&self) -> &AgentGPT {
//!         &self.agent
//!     }
//! }
//!
//! #[async_trait]
//! impl AsyncFunctions for SpecialFunctions {
//!     /// Execute special functions for an agent.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `task` - The task associated with the agent.
//!     /// * `execute` - A boolean indicating whether to execute the generated code by the agent.
//!     /// * `max_tries` - A integer indicating the max number of tries fixing code bugs.
//!     ///
//!     /// # Returns
//!     ///
//!     /// A result indicating success or failure.
//!     async fn execute<'a>(
//!        &'a mut self,
//!        task: &'a mut Task,
//!        _execute: bool,
//!        _browse: bool,
//!        _max_tries: u64,
//!     ) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     /// Saves a message to long-term memory for the agent.
//!     ///
//!     /// # Arguments
//!     ///
//!     /// * `message` - The message to save, which contains the role and content.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`Result<()>`): Result indicating the success or failure of saving the message.
//!     async fn save_ltm(&mut self, _message: Message) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     /// Retrieves all messages stored in the agent's long-term memory.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`Result<Vec<Message>>`): A result containing a vector of messages retrieved from the agent's long-term memory.
//!     async fn get_ltm(&self) -> Result<Vec<Message>> {
//!         Ok(vec![
//!             Message {
//!                 role: Cow::Borrowed("system"),
//!                 content: Cow::Borrowed("System initialized."),
//!             },
//!             Message {
//!                 role: Cow::Borrowed("user"),
//!                 content: Cow::Borrowed("Hello, autogpt!"),
//!             },
//!         ])
//!     }
//!
//!     /// Retrieves the concatenated context of all messages in the agent's long-term memory.
//!     ///
//!     /// # Returns
//!     ///
//!     /// (`String`): A string containing the concatenated role and content of all messages stored in the agent's long-term memory.
//!     async fn ltm_context(&self) -> String {
//!         let messages = [
//!             Message {
//!                 role: Cow::Borrowed("system"),
//!                 content: Cow::Borrowed("System initialized."),
//!             },
//!             Message {
//!                 role: Cow::Borrowed("user"),
//!                 content: Cow::Borrowed("Hello, autogpt!"),
//!             },
//!         ];
//!
//!         messages
//!             .iter()
//!             .map(|c| format!("{}: {}", c.role, c.content))
//!             .collect::<Vec<_>>()
//!             .join("\n")
//!     }
//!
//!     async fn generate(&mut self, _request: &str) -> Result<String> {
//!         Ok("".to_string())
//!     }
//!
//!     async fn imagen(&mut self, _request: &str) -> Result<Vec<u8>> {
//!         // TODO: Impl
//!         Ok(Default::default())
//!     }
//!
//!     async fn stream(&mut self, _request: &str) -> Result<ReqResponse> {
//!         // TODO: Impl
//!         Ok(ReqResponse(None))
//!     }
//! }
//!

use crate::agents::agent::AgentGPT;
#[cfg(feature = "mem")]
use crate::common::utils::Message;
use crate::common::utils::{AgentMessage, Task};
use anyhow::Result;
use async_trait::async_trait;
use tokio::sync::mpsc::Receiver;

#[derive(Default)]
pub struct ReqResponse(pub Option<Receiver<String>>);

/// Trait to retrieve an agent.
pub trait Functions {
    /// Get attributes from an agent.
    ///
    /// # Returns
    ///
    /// A reference to the agent.
    fn get_agent(&self) -> &AgentGPT;
}

/// Trait defining special functions for agents.\
#[async_trait]
pub trait AsyncFunctions: Send + Sync {
    /// Execute special functions for an agent.
    ///
    /// # Arguments
    ///
    /// * `task` - The task associated with the agent.
    /// * `execute` - A boolean indicating whether to execute the generated code by the agent.
    /// * `browse` - Whether to open a browser.
    /// * `max_tries` - A integer indicating the max number of tries fixing code bugs.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()>;

    /// Save a message into long-term memory.
    ///
    /// # Arguments
    ///
    /// * `message` - The message to save.
    ///
    /// # Returns
    ///
    /// A result indicating success or failure.
    #[cfg(feature = "mem")]
    async fn save_ltm<'a>(&'a mut self, message: Message) -> Result<()>;

    /// Get the long-term memory of an agent.
    ///
    /// # Returns
    ///
    /// A result containing a vector of messages.
    #[cfg(feature = "mem")]
    async fn get_ltm<'a>(&'a self) -> Result<Vec<Message>>;

    /// Retrieve the long-term memory context as a string.
    ///
    /// # Returns
    ///
    /// A string containing the concatenated context of the agent's memory.
    #[cfg(feature = "mem")]
    async fn ltm_context<'a>(&'a self) -> String;

    async fn generate(&mut self, request: &str) -> Result<String>;

    async fn imagen(&mut self, request: &str) -> Result<Vec<u8>>;

    async fn stream(&mut self, request: &str) -> Result<ReqResponse>;
}

#[async_trait]
pub trait Executor {
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()>;
}

#[async_trait]
pub trait Collaborate: Send + Sync {
    async fn handle_task(&mut self, task: Task) -> Result<()>;
    async fn receive_message(&mut self, message: AgentMessage) -> Result<()>;
    fn get_id(&self) -> &str;
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
