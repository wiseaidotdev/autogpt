// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # `ArchitectGPT` agent.
//!
//! This module provides functionality for creating innovative website designs
//! and architectural diagrams based on prompts using Gemini API and diagrams library.
//! The `ArchitectGPT` agent understands user requirements and generates architectural diagrams
//! for your web applications.
//!
//! # Example - Generating website designs:
//!
//! ```rust
//! use autogpt::agents::architect::ArchitectGPT;
//! use autogpt::common::utils::Task;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut architect_agent = ArchitectGPT::new(
//!         "Web wireframes and UIs",
//!         "Create innovative website designs",
//!     ).await;
//!
//!     let mut task = Task {
//!         description: "Design an architectural diagram for a modern chat application".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = architect_agent.execute(&mut task, true, false, 3).await {
//!         eprintln!("Error executing architect tasks: {:?}", err);
//!     }
//! }
//! ```
#![allow(unreachable_code)]

use crate::agents::agent::AgentGPT;
#[cfg(feature = "net")]
use crate::collaboration::Collaborator;
#[allow(unused_imports)]
use crate::common::utils::{
    Capability, ClientType, ContextManager, GenerationOutput, Goal, Knowledge, Message, OutputKind,
    Persona, Planner, Reflection, Scope, Status, Task, TaskScheduler, Tool, extract_array,
    extract_json_string, strip_code_blocks,
};
#[allow(unused_imports)]
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
use crate::prompts::architect::{
    ARCHITECT_DIAGRAM_PROMPT, ARCHITECT_ENDPOINTS_PROMPT, ARCHITECT_SCOPE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Executor, Functions};
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use colored::*;
// use duckduckgo::browser::Browser;
// use duckduckgo::user_agents::get;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{debug, error, info, warn};

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
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

use auto_derive::Auto;

/// Struct representing an ArchitectGPT, which orchestrates tasks related to architectural design using GPT.
#[derive(Debug, Clone, Default, Auto)]
#[allow(dead_code)]
pub struct ArchitectGPT {
    /// Represents the workspace directory path for ArchitectGPT.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling architectural tasks.
    agent: AgentGPT,
    /// Represents a client for interacting with an AI API (OpenAI or Gemini).
    client: ClientType,
    /// Represents a client for making HTTP requests.
    req_client: ReqClient,
}

impl ArchitectGPT {
    /// Creates a new instance of `ArchitectGPT`.
    ///
    /// # Arguments
    ///
    /// * `behavior` - The behavior of the agent.
    /// * `position` - The position of the agent.
    ///
    /// # Returns
    ///
    /// (`ArchitectGPT`): A new instance of ArchitectGPT.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for ArchitectGPT.
    /// - Creates the workspace directory if it does not exist.
    /// - Initializes the GPT agent with the given persona and behavior.
    /// - Creates clients for interacting with Gemini or OpenAI API and making HTTP requests.
    #[allow(unused)]
    pub async fn new(persona: &'static str, behavior: &'static str) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "architect";

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Directory '{}' already exists.", workspace);
        }
        let file_path = format!("{workspace}/diagram.py");
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&file_path)
            .await
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(b"").await {
                    error!("Failed to write to 'diagram.py': {}", e);
                } else {
                    debug!("File 'diagram.py' created successfully!");
                }
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    debug!("File 'diagram.py' already exists, skipping creation.");
                } else {
                    error!("Error creating file 'diagram.py': {}", e);
                }
            }
        }
        let create_venv = Command::new("python3")
            .arg("-m")
            .arg("venv")
            .arg(workspace.clone() + "/.venv")
            .status();

        if let Ok(status) = create_venv.await
            && status.success()
        {
            let pip_path = format!("{}/bin/pip", workspace.clone() + "/.venv");
            let pip_install = Command::new(pip_path)
                .arg("install")
                .arg("diagrams")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();

            match pip_install {
                Ok(_) => info!(
                    "{}",
                    format!("[*] {persona:?}: Diagrams installed successfully!")
                        .bright_white()
                        .bold()
                ),
                Err(e) => error!(
                    "{}",
                    format!("[*] {persona:?}: Error installing Diagrams: {e}!")
                        .bright_red()
                        .bold()
                ),
            }
        }

        let mut agent: AgentGPT = AgentGPT::new_borrowed(persona, behavior);
        agent.id = agent.persona().to_string().into();

        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.persona())
                .bright_white()
                .bold()
        );

        let req_client: ReqClient = ReqClient::builder()
            .timeout(Duration::from_secs(3))
            .build()
            .unwrap();

        Self {
            workspace: workspace.into(),
            agent,
            client,
            req_client,
        }
    }

    pub async fn build_request(
        &mut self,
        prompt: &str,
        task: &mut Task,
        output_type: OutputKind,
    ) -> Result<GenerationOutput> {
        #[cfg(feature = "mem")]
        {
            self.agent.memory = self.get_ltm().await?;
        }

        let current_code = fs::read_to_string(&format!("{}/diagram.py", self.workspace)).await?;
        let request: String = format!(
            "{}\n\nTask Description: {}\nPrevious Conversation: {:?}\nCurrent Architecture: {:?}",
            prompt,
            task.description,
            self.agent.memory(),
            current_code
        );

        self.agent.add_message(Message {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(request.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(request.clone()),
                })
                .await;
        }

        #[allow(unused)]
        let mut response_text = String::new();

        #[cfg(any(
            feature = "co",
            feature = "oai",
            feature = "gem",
            feature = "cld",
            feature = "xai"
        ))]
        {
            response_text = self.generate(&request).await?;
        }

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response_text.clone()),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(response_text.clone()),
                })
                .await;
        }

        debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);

        match output_type {
            OutputKind::Text => Ok(GenerationOutput::Text(strip_code_blocks(&response_text))),
            OutputKind::UrlList => {
                let urls: Vec<Cow<'static, str>> =
                    serde_json::from_str(&extract_array(&response_text).unwrap_or_default())?;
                task.urls = Some(urls.clone());
                self.agent.update(Status::InUnitTesting);
                Ok(GenerationOutput::UrlList(urls))
            }
            OutputKind::Scope => {
                let scope: Scope = serde_json::from_str(&strip_code_blocks(&response_text))?;
                Ok(GenerationOutput::Scope(scope))
            }
        }
    }

    /// Retrieves the scope based on task description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<Scope>`): The scope generated based on the task description.
    ///
    /// # Side Effects
    ///
    /// - Updates the agent status to `Status::Completed` upon successful completion.
    /// - Adds both the user prompt and AI response (or error message) to the agent's message memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided task.
    /// - Sends the request to the Gemini or OpenAI API to generate content.
    /// - Logs the request as a user message.
    /// - Parses the response into a Scope object.
    /// - Logs the response (or error) as an assistant message.
    /// - Updates the task with the retrieved scope.
    /// - Updates the agent status to `Completed`.
    pub async fn get_scope(&mut self, task: &mut Task) -> Result<Scope> {
        match self
            .build_request(ARCHITECT_SCOPE_PROMPT, task, OutputKind::Scope)
            .await?
        {
            GenerationOutput::Scope(scope) => {
                self.agent.update(Status::Completed);
                debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);
                Ok(scope)
            }
            _ => Err(anyhow::anyhow!("Expected scope from generation.")),
        }
    }

    /// Retrieves URLs based on task description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of the operation.
    ///
    /// # Side Effects
    ///
    /// - Updates the agent status to `Status::InUnitTesting` upon successful completion.
    /// - Adds both the user prompt and AI response (or error message) to the agent's message memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided task.
    /// - Sends the request to the GPT client to generate content.
    /// - Logs the request as a user message.
    /// - Parses the response into a vector of URLs.
    /// - Logs the response (or error) as an assistant message.
    /// - Updates the task with the retrieved URLs.
    /// - Updates the agent status to `InUnitTesting`.
    pub async fn get_urls(&mut self, task: &mut Task) -> Result<()> {
        match self
            .build_request(ARCHITECT_ENDPOINTS_PROMPT, task, OutputKind::UrlList)
            .await?
        {
            GenerationOutput::UrlList(urls) => {
                task.urls = Some(urls.clone());
                self.agent.update(Status::InUnitTesting);
                debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);
                Ok(())
            }
            _ => Err(anyhow::anyhow!("Expected URL list from generation.")),
        }
    }

    /// Generates a diagram based on task description and logs the interaction in agent memory.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to be performed.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): The generated diagram content.
    ///
    /// # Side Effects
    ///
    /// - Adds both the user prompt and AI response (or error message) to the agent's message memory.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request based on the provided task.
    /// - Sends the request to the GPT client to generate content.
    /// - Logs the request as a user message.
    /// - Logs the response (or error) as an assistant message.
    /// - Processes the response to strip code blocks.
    /// - Returns the cleaned-up diagram content.
    pub async fn generate_diagram(&mut self, task: &mut Task) -> Result<String> {
        match self
            .build_request(ARCHITECT_DIAGRAM_PROMPT, task, OutputKind::Text)
            .await?
        {
            GenerationOutput::Text(diagram) => Ok(diagram),
            _ => Err(anyhow::anyhow!("Expected diagram text from generation.")),
        }
    }

    pub fn think(&self) -> String {
        let behavior = self.agent.behavior();
        format!("What steps should I take to achieve '{behavior}'")
    }

    pub fn plan(&mut self, context: String) -> Goal {
        let mut goals = vec![
            Goal {
                description: "Identify system components".into(),
                priority: 1,
                completed: false,
            },
            Goal {
                description: "Determine message between components".into(),
                priority: 2,
                completed: false,
            },
            Goal {
                description: "Generate diagram for architecture".into(),
                priority: 3,
                completed: false,
            },
        ];

        goals.sort_by_key(|g| g.priority);

        if let Some(planner) = self.agent.planner_mut() {
            if planner.current_plan.is_empty() {
                for g in goals.into_iter().rev() {
                    planner.current_plan.push(g);
                }
            }

            if let Some(next_goal) = planner.current_plan.iter().rev().find(|g| !g.completed) {
                return next_goal.clone();
            }
        }

        Goal {
            description: format!("Default task from context: {context}"),
            priority: 1,
            completed: false,
        }
    }

    pub fn act(&mut self, goal: Goal) {
        info!(
            "{}",
            format!(
                "[*] {:?}: Executing goal: {}",
                self.agent.persona(),
                goal.description
            )
            .cyan()
            .bold()
        );

        for tool in self.agent.tools() {
            if goal
                .description
                .to_lowercase()
                .contains(&format!("{:?}", tool.name).to_lowercase())
            {
                let result = (tool.invoke)(&goal.description);
                info!(
                    "{}",
                    format!(
                        "[*] {:?}: Tool [{:?}] executed: {}",
                        self.agent.persona(),
                        tool.name,
                        result
                    )
                    .green()
                );
                self.agent.memory_mut().push(Message {
                    role: goal.description.into(),
                    content: result.into(),
                });
                return;
            }
        }

        warn!(
            "{}",
            format!(
                "[*] {:?}: No tool matched for goal: {}",
                self.agent.persona(),
                goal.description
            )
            .yellow()
        );
    }

    pub fn reflect(&mut self) {
        let entry = format!("Reflection on step toward '{}'", self.agent.behavior());

        self.agent.memory_mut().push(Message {
            role: Cow::Borrowed("assistant"),
            content: entry.clone().into(),
        });

        self.agent.context_mut().recent_messages.push(Message {
            role: Cow::Borrowed("assistant"),
            content: entry.into(),
        });

        if let Some(reflection) = self.agent.reflection() {
            let feedback = (reflection.evaluation_fn)(&self.agent);
            info!(
                "{}",
                format!(
                    "[*] {:?}: Self Reflection: {}",
                    self.agent.persona(),
                    feedback
                )
                .blue()
            );
        }
    }
    pub fn has_completed_behavior(&self) -> bool {
        self.planner()
            .map(|p| p.current_plan.iter().all(|g| g.completed))
            .unwrap_or(false)
    }
    pub fn mark_goal_complete(&mut self, goal: Goal) {
        if let Some(planner) = self.planner_mut() {
            for g in &mut planner.current_plan {
                if g.description == goal.description {
                    g.completed = true;
                }
            }
        }
    }

    fn display_task_info(&self, task: &Task) {
        for task in task.clone().description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }
    }

    async fn idle(&mut self, task: &mut Task) -> Result<()> {
        debug!(
            "{}",
            format!("[*] {:?}: Idle", self.agent.persona())
                .bright_white()
                .bold()
        );

        let scope = self.get_scope(task).await?;
        if scope.external {
            let _ = self.get_urls(task).await;
        }

        self.agent.update(Status::InUnitTesting);
        Ok(())
    }

    async fn unit_test_and_generate(
        &mut self,
        path: &str,
        task: &mut Task,
        max_tries: u64,
    ) -> Result<()> {
        self.filter_urls(task).await;

        let mut python_code = self.generate_diagram(task).await?;

        self.write_code_to_file(path, &python_code).await?;

        for attempt in 1..=max_tries {
            let run_result = self.run_python_script().await;

            match run_result {
                Ok(_) => {
                    info!(
                        "{}",
                        format!(
                            "[*] {:?}: Diagram generated successfully!",
                            self.agent.persona()
                        )
                        .green()
                        .bold()
                    );
                    self.agent.update(Status::Completed);
                    break;
                }
                Err(e) => {
                    error!(
                        "{}",
                        format!(
                            "[*] {:?}: Error generating diagram: {}",
                            self.agent.persona(),
                            e
                        )
                        .bright_red()
                        .bold()
                    );

                    if attempt < max_tries {
                        info!(
                            "{}",
                            format!(
                                "[*] {:?}: Retrying... ({}/{})",
                                self.agent.persona(),
                                attempt,
                                max_tries
                            )
                            .yellow()
                            .bold()
                        );

                        task.description =
                            (task.description.to_string() + " Got an error: " + &e.to_string())
                                .into();

                        python_code = self.search_solution_and_regenerate(task).await?;
                        self.write_code_to_file(path, &python_code).await?;
                    } else {
                        error!(
                            "{}",
                            format!(
                                "[*] {:?}: Maximum retries reached. Exiting...",
                                self.agent.persona()
                            )
                            .bright_red()
                            .bold()
                        );
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    async fn filter_urls(&self, task: &mut Task) {
        let mut exclude = Vec::new();

        let urls = task.urls.as_ref().map_or_else(Vec::new, |url| url.to_vec());

        for url in &urls {
            info!(
                "{}",
                format!(
                    "[*] {:?}: Testing URL Endpoint: {}",
                    self.agent.persona(),
                    url
                )
                .bright_white()
                .bold()
            );

            match self.req_client.get(url.to_string()).send().await {
                Ok(response) if response.status() != reqwest::StatusCode::OK => {
                    exclude.push(url.clone());
                }
                Err(err) => {
                    let url = err
                        .url()
                        .map(|u| u.to_string())
                        .unwrap_or_else(|| "unknown URL".to_string());

                    error!(
                        "{}",
                        format!(
                            "[*] {:?}: Failed to request URL {}. Check connection.",
                            self.agent.persona(),
                            url
                        )
                        .bright_red()
                        .bold()
                    );
                }
                _ => {}
            }
        }

        if !exclude.is_empty() {
            let filtered: Vec<Cow<'static, str>> = task
                .urls
                .as_ref()
                .unwrap()
                .iter()
                .filter(|url| !exclude.contains(url))
                .cloned()
                .collect();
            task.urls = Some(filtered);
        }
    }

    async fn write_code_to_file(&self, path: &str, code: &str) -> Result<()> {
        match fs::write(path, code).await {
            Ok(_) => {
                debug!(
                    "{}",
                    format!(
                        "[*] {:?}: Wrote diagram.py successfully!",
                        self.agent.persona()
                    )
                    .green()
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "{}",
                    format!(
                        "[*] {:?}: Failed writing diagram.py: {}",
                        self.agent.persona(),
                        e
                    )
                    .bright_red()
                );
                Err(anyhow!("File write error"))
            }
        }
    }

    async fn run_python_script(&self) -> Result<()> {
        let result = Command::new("sh")
            .arg("-c")
            .arg(format!("timeout {} .venv/bin/python ./diagram.py", 10))
            .current_dir(self.workspace.to_string())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output();

        match result.await {
            Ok(output) if output.status.success() => Ok(()),
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(anyhow!("Python error: {}", stderr))
            }
            Err(e) => Err(anyhow!("Execution error: {}", e)),
        }
    }

    async fn search_solution_and_regenerate(&mut self, task: &mut Task) -> Result<String> {
        // TODO: remove `req_client` arg in duckduckgo
        // let browser = Browser::new(self.req_client.clone());
        // let user_agent = get("firefox").unwrap();

        let query = format!("Python error handling for: {}", task.description);
        info!(
            "{}",
            format!("[*] {:?}: Searching: {}", self.agent.persona(), query)
                .blue()
                .bold()
        );

        // let results = browser
        //     .lite_search(&query, "wt-wt", Some(3), user_agent)
        //     .await?;
        let results = vec!["".to_string()];

        for result in &results {
            info!(
                "{}",
                format!(
                    "[*] {:?}: DuckDuckGo result: {}",
                    self.agent.persona(),
                    // result.title
                    result
                )
                .bright_cyan()
            );
        }

        self.generate_diagram(task).await
    }
}

/// Implementation of the trait `Executor` for `ArchitectGPT`.
/// Contains additional methods related to architectural tasks.
///
/// This trait provides methods for:
/// - Retrieving the agent associated with `ArchitectGPT`.
/// - Executing tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the agent associated with the `ArchitectGPT` instance.
/// - Executes the task asynchronously based on the current status of the agent.
/// - Handles task execution including scope retrieval, URL retrieval, and diagram generation.
/// - Manages retries in case of failures during task execution.
#[async_trait]
impl Executor for ArchitectGPT {
    /// Executes the task asynchronously.
    ///
    /// # Arguments
    ///
    /// * `task` - The task to be executed.
    /// * `execute` - Flag indicating whether to execute the task.
    /// * `max_tries` - Maximum number of retry attempts.
    ///
    /// # Returns
    ///
    /// (`Result<()>`): Result indicating success or failure of the operation.
    ///
    /// # Business Logic
    ///
    /// - Executes the task asynchronously based on the current status of the agent.
    /// - Handles task execution including scope retrieval, URL retrieval, and diagram generation.
    /// - Manages retries in case of failures during task execution.
    ///
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        self.agent.update(Status::Idle);
        info!(
            "{}",
            format!("[*] {:?}: Executing task:", self.agent.persona())
                .bright_white()
                .bold()
        );

        self.display_task_info(task);
        let path = &(self.workspace.to_string() + "/diagram.py");

        while self.agent.status() != &Status::Completed {
            let context = self.think();
            let goal = self.plan(context);

            if browse {
                // no execute = no unit testing -> max_tries = 1
                self.idle(task).await?;
            } else {
                self.agent.update(Status::InUnitTesting);
            }

            if execute {
                self.unit_test_and_generate(path, task, max_tries).await?;
            } else {
                // no execute = no unit testing -> max_tries = 1
                self.unit_test_and_generate(path, task, 1).await?;
            }

            self.mark_goal_complete(goal);

            self.reflect();

            if self.has_completed_behavior() {
                info!(
                    "{}",
                    format!("[*] {:?}: behavior complete!", self.agent.persona())
                        .green()
                        .bold()
                );
                self.agent.update(Status::Completed);
                break;
            }
        }

        Ok(())
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
