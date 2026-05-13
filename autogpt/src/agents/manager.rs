#![allow(unused)]
// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # `ManagerGPT` agent.
//!

use crate::agents::agent::AgentGPT;
use crate::agents::architect::ArchitectGPT;
use crate::agents::backend::BackendGPT;
#[cfg(feature = "img")]
use crate::agents::designer::DesignerGPT;
use crate::agents::frontend::FrontendGPT;
#[cfg(feature = "git")]
use crate::agents::git::GitGPT;
use crate::agents::types::AgentType;
#[cfg(feature = "net")]
use crate::collaboration::Collaborator;
use crate::common::utils::{
    Capability, ClientType, ContextManager, Knowledge, Message, Persona, Planner, Reflection,
    Status, Task, TaskScheduler, Tool, strip_code_blocks,
};
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
use crate::prompts::manager::{FRAMEWORK_MANAGER_PROMPT, LANGUAGE_MANAGER_PROMPT, MANAGER_PROMPT};
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Executor, Functions};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use auto_derive::Auto;
use colored::*;
#[cfg(feature = "gem")]
use gems::Client;
use std::borrow::Cow;
use std::env::var;
use tracing::{debug, info};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};
#[cfg(feature = "oai")]
use {
    openai_dive::v1::models::Gpt4Model, openai_dive::v1::resources::chat::*,
    openai_dive::v1::resources::model::*,
};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient,
    RequiredMessageParams, Role,
};

#[cfg(feature = "gem")]
use gems::{
    chat::ChatBuilder,
    imagen::ImageGenBuilder,
    messages::{Content, Message as GemMessage},
    models::Model,
    stream::StreamBuilder,
    traits::CTrait,
};

#[cfg(any(
    feature = "co",
    feature = "oai",
    feature = "gem",
    feature = "cld",
    feature = "xai",
    feature = "hf",
    feature = "gpt"
))]
use crate::traits::functions::ReqResponse;

#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

#[cfg(feature = "co")]
use {cohere_rust::api::GenerateModel, cohere_rust::api::generate::GenerateRequest};

/// Struct representing a ManagerGPT, responsible for managing different types of GPT agents.
#[derive(Debug, Clone, Default, Auto)]
#[allow(unused)]
pub struct ManagerGPT {
    /// Represents the GPT agent associated with the manager.
    agent: AgentGPT,
    /// Represents the task to be executed by the manager.
    task: Task,
    /// Represents the programming language used in the tasks.
    language: &'static str,
    /// Represents a collection of GPT agents managed by the manager.
    agents: Vec<AgentType>,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
}

impl ManagerGPT {
    /// Constructor function to create a new instance of ManagerGPT.
    ///
    /// # Arguments
    ///
    /// * `behavior` - behavior description for ManagerGPT.
    /// * `position` - Position description for ManagerGPT.
    /// * `request` - Description of the user's request.
    /// * `language` - Programming language used in the tasks.
    ///
    /// # Returns
    ///
    /// (`ManagerGPT`): A new instance of ManagerGPT.
    ///
    /// # Business Logic
    ///
    /// - Initializes the GPT agent with the given persona and behavior.
    /// - Initializes an empty collection of agents.
    /// - Initializes tasks with the provided description.
    /// - Initializes a Gemini client for interacting with Gemini API.
    ///
    pub fn new(
        persona: &'static str,
        behavior: &'static str,
        request: &str,
        language: &'static str,
    ) -> Self {
        let mut agent = AgentGPT::new_borrowed(persona, behavior);
        agent.id = agent.persona().to_string().into();

        let agents: Vec<AgentType> = Vec::new();

        // let request = format!("{}\n\nUser Request: {}", MANAGER_PROMPT, request);

        let task: Task = Task {
            description: request.to_string().into(),
            scope: None,
            urls: None,
            frontend_code: None,
            backend_code: None,
            api_schema: None,
        };

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.persona(),)
                .bright_white()
                .bold()
        );

        let client = ClientType::from_env();

        Self {
            agent,
            task,
            language,
            agents,
            client,
        }
    }

    /// Adds an agent to the manager.
    ///
    /// # Arguments
    ///
    /// * `agent` - The agent to be added.
    ///
    /// # Business Logic
    ///
    /// - Adds the specified agent to the collection of agents managed by the manager.
    ///
    fn add_agent(&mut self, agent: AgentType) {
        self.agents.push(agent);
    }

    async fn spawn_default_agents(&mut self) {
        self.add_agent(AgentType::Architect(
            ArchitectGPT::new(
                "ArchitectGPT",
                "Creates innovative website designs and user experiences",
            )
            .await,
        ));
        #[cfg(feature = "img")]
        self.add_agent(AgentType::Designer(
            DesignerGPT::new(
                "DesignerGPT",
                "Creates innovative website designs and user experiences",
            )
            .await,
        ));
        self.add_agent(AgentType::Backend(
            BackendGPT::new(
                "BackendGPT",
                "Expertise lies in writing backend code for web servers and JSON databases",
                self.language,
            )
            .await,
        ));
        self.add_agent(AgentType::Frontend(
            FrontendGPT::new(
                "FrontendGPT",
                "Expertise lies in writing frontend code for Yew rust framework",
                self.language,
            )
            .await,
        ));
        #[cfg(feature = "git")]
        self.add_agent(AgentType::Git(
            GitGPT::new(
                "GitGPT",
                "Handles git operations like staging and committing code",
            )
            .await,
        ));
    }

    /// Sends a prompt to the configured LLM and returns the full response text.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The user's prompt.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): The AI's response text.
    pub async fn execute_prompt(&mut self, prompt: String) -> Result<String, anyhow::Error> {
        let response = self.generate(&prompt).await?;
        Ok(strip_code_blocks(&response))
    }
}

#[async_trait]
impl Executor for ManagerGPT {
    /// Asynchronously executes the tasks described by the user request.
    ///
    /// # Arguments
    ///
    /// * `task` - A mutable reference to the task to be executed.
    /// * `execute` - A boolean indicating whether to execute the tasks.
    /// * `browse` - Whether to open a browser.
    /// * `max_tries` - Maximum number of attempts to execute tasks.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of task execution.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in executing tasks.
    ///
    /// # Business Logic
    ///
    /// - Executes tasks described by the user request using the collection of agents managed by the manager.
    /// - Logs user request, system decisions, and assistant responses.
    /// - Manages retries and error handling during task execution.
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        self.agent.add_message(Message {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Execute tasks with description: '{}'",
                self.task.description.clone()
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Execute tasks with description: '{}'",
                        self.task.description.clone()
                    )),
                })
                .await;
        }
        info!(
            "{}",
            format!(
                "[*] {:?}: Executing task: {:?}",
                self.agent.persona(),
                self.task.description.clone()
            )
            .bright_white()
            .bold()
        );

        let language_request = format!(
            "{}\n\nUser Request: {}",
            LANGUAGE_MANAGER_PROMPT,
            self.task.description.clone()
        );

        let framework_request = format!(
            "{}\n\nUser Request: {}",
            FRAMEWORK_MANAGER_PROMPT,
            self.task.description.clone()
        );

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Analyzing user request to determine programming language and framework..."
                    .to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Analyzing user request to determine programming language and framework..."
                            .to_string(),
                    ),
                })
                .await;
        }
        let language = self.execute_prompt(language_request).await?;
        let framework = self.execute_prompt(framework_request).await?;

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Identified Language: '{language}', Framework: '{framework}'"
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Identified Language: '{language}', Framework: '{framework}'"
                    )),
                })
                .await;
        }
        if self.agents.is_empty() {
            self.spawn_default_agents().await;
            self.agent.add_message(Message {
                role: Cow::Borrowed("system"),
                content: Cow::Borrowed("No agents were available. Spawned default agents."),
            });
        }

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("system"),
                    content: Cow::Borrowed("No agents were available. Spawned default agents."),
                })
                .await;
        }

        for mut agent in self.agents.clone() {
            let request_prompt = format!(
                "{}\n\n\n\nUser Request: {}\n\nAgent Role: {}\nProgramming Language: {}\nFramework: {}\n",
                MANAGER_PROMPT,
                self.task.description.clone(),
                agent.persona(),
                language,
                framework
            );

            let refined_task = self.execute_prompt(request_prompt).await?;

            self.agent.add_message(Message {
                role: Cow::Borrowed("assistant"),
                content: Cow::Owned(format!(
                    "Refined task for '{}': {}",
                    agent.persona(),
                    refined_task
                )),
            });

            #[cfg(feature = "mem")]
            {
                let _ = self
                    .save_ltm(Message {
                        role: Cow::Borrowed("assistant"),
                        content: Cow::Owned(format!(
                            "Refined task for '{}': {}",
                            agent.persona(),
                            refined_task
                        )),
                    })
                    .await;
            }

            self.task = Task {
                description: refined_task.into(),
                scope: None,
                urls: None,
                frontend_code: None,
                backend_code: None,
                api_schema: None,
            };

            let _agent_res = agent
                .execute(&mut self.task, execute, browse, max_tries)
                .await;
        }

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Borrowed("Task execution completed by all agents."),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Borrowed("Task execution completed by all agents."),
                })
                .await;
        }
        info!(
            "{}",
            format!("[*] {:?}: Completed Task:", self.agent.persona())
                .bright_white()
                .bold()
        );

        Ok(())
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
