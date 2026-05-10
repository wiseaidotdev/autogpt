// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # `FrontendGPT` agent.
//!
//! This module provides functionality for generating frontend code based on prompts
//! using Gemini API. The `FrontendGPT` agent is capable of understanding user requests
//! and producing code snippets in various programming languages and frameworks commonly
//! used for web development.
//!
//! # Example - Generating frontend code:
//!
//! ```rust
//! use autogpt::agents::frontend::FrontendGPT;
//! use autogpt::common::utils::Task;
//! use autogpt::traits::functions::Functions;
//! use autogpt::traits::functions::AsyncFunctions;
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut frontend_agent = FrontendGPT::new(
//!         "Frontend Developer",
//!         "Generate frontend code",
//!         "rust",
//!     ).await;
//!
//!     let mut task = Task {
//!         description: "Create a landing page with a sign-up form".into(),
//!         scope: None,
//!         urls: None,
//!         frontend_code: None,
//!         backend_code: None,
//!         api_schema: None,
//!     };
//!
//!     if let Err(err) = frontend_agent.execute(&mut task, true, false, 3).await {
//!         eprintln!("Error executing frontend tasks: {:?}", err);
//!     }
//! }
//! ```
//!
#![allow(unreachable_code)]

use crate::agents::agent::AgentGPT;
#[cfg(feature = "net")]
use crate::collaboration::Collaborator;
#[cfg(feature = "cli")]
use crate::common::utils::spinner;
#[allow(unused_imports)]
use crate::common::utils::{
    Capability, ClientType, ContextManager, GenerationOutput, Goal, Knowledge, Message, OutputKind,
    Persona, Planner, Reflection, Route, Scope, Status, Task, TaskScheduler, Tool, extract_array,
    strip_code_blocks,
};
#[allow(unused_imports)]
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
use crate::prompts::frontend::{
    FIX_CODE_PROMPT, FRONTEND_CODE_PROMPT, IMPROVED_FRONTEND_CODE_PROMPT,
};
use crate::traits::agent::Agent;
use crate::traits::functions::{AsyncFunctions, Executor, Functions};
use anyhow::{Result, anyhow};
use auto_derive::Auto;
use colored::*;
use reqwest::Client as ReqClient;
use std::borrow::Cow;
use std::env::var;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use tokio::fs;
use tokio::io::AsyncReadExt;
use tokio::process::Child;
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

use async_trait::async_trait;

/// Struct representing a `FrontendGPT`, which manages frontend code generation and testing using Gemini API.
#[derive(Debug, Clone, Default, Auto)]
#[allow(unused)]
pub struct FrontendGPT {
    /// Represents the workspace directory path for `FrontendGPT`.
    workspace: Cow<'static, str>,
    /// Represents the GPT agent responsible for handling frontend tasks.
    agent: AgentGPT,
    /// Represents an OpenAI or Gemini client for interacting with their API.
    client: ClientType,
    /// Represents a client for sending HTTP requests.
    req_client: ReqClient,
    /// Represents the bugs found in the code.
    bugs: Option<Cow<'static, str>>,
    /// Represents the programming language used for frontend development.
    language: &'static str,
    /// Represents the number of bugs found in the code.
    nb_bugs: u64,
}

impl FrontendGPT {
    /// Constructor function to create a new instance of FrontendGPT.
    ///
    /// # Arguments
    ///
    /// * `behavior` - behavior description for FrontendGPT.
    /// * `position` - Position description for FrontendGPT.
    /// * `language` - Programming language used for frontend development.
    ///
    /// # Returns
    ///
    /// (`FrontendGPT`): A new instance of FrontendGPT.
    ///
    /// # Business Logic
    ///
    /// - Constructs the workspace directory path for FrontendGPT.
    /// - Initializes the GPT agent with the given persona, behavior, and language.
    /// - Creates clients for interacting with Gemini API
    pub async fn new(
        persona: &'static str,
        behavior: &'static str,
        language: &'static str,
    ) -> Self {
        let workspace = var("AUTOGPT_WORKSPACE")
            .unwrap_or("workspace/".to_string())
            .to_owned()
            + "frontend";

        if !fs::try_exists(&workspace).await.unwrap_or(false) {
            match fs::create_dir_all(&workspace).await {
                Ok(_) => debug!("Directory '{}' created successfully!", workspace),
                Err(e) => error!("Error creating directory '{}': {}", workspace, e),
            }
        } else {
            debug!("Workspace directory '{}' already exists.", workspace);
        }

        match language {
            "rust" => {
                let cargo_new = Command::new("cargo")
                    .arg("init")
                    .arg(workspace.clone())
                    .spawn();

                match cargo_new {
                    Ok(_) => debug!("Cargo project initialized successfully!"),
                    Err(e) => error!("Error initializing Cargo project: {}", e),
                }
                match fs::write(workspace.clone() + "/src/template.rs", "").await {
                    Ok(_) => debug!("File 'template.rs' created successfully!"),
                    Err(e) => error!("Error creating file 'template.rs': {}", e),
                };
            }
            "python" => {
                match fs::write(workspace.clone() + "/main.py", "").await {
                    Ok(_) => debug!("File 'main.py' created successfully!"),
                    Err(e) => error!("Error creating file 'main.py': {}", e),
                }
                match fs::write(workspace.clone() + "/template.py", "").await {
                    Ok(_) => debug!("File 'template.py' created successfully!"),
                    Err(e) => error!("Error creating file 'template.py': {}", e),
                };
            }
            "javascript" => {
                let npx_install = Command::new("npx")
                    .arg("create-react-app")
                    .arg(workspace.clone())
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit())
                    .spawn();

                match npx_install {
                    Ok(mut child) => match child.wait().await {
                        Ok(status) => {
                            if status.success() {
                                debug!("React JS project initialized successfully!");
                            } else {
                                error!("Failed to initialize React JS project");
                            }
                        }
                        Err(e) => {
                            error!("Error waiting for process: {}", e);
                        }
                    },
                    Err(e) => {
                        error!("Error initializing React JS project: {}", e);
                    }
                }
                match fs::write(workspace.clone() + "/src/template.js", "").await {
                    Ok(_) => debug!("File 'template.js' created successfully!"),
                    Err(e) => error!("Error creating file 'template.js': {}", e),
                };
            }
            _ => panic!("Unsupported language, consider open an Issue/PR"),
        };
        #[allow(unused)]
        let mut agent: AgentGPT = AgentGPT::new_borrowed(persona, behavior);
        agent.id = agent.persona().to_string().into();

        #[allow(unused)]
        let client = ClientType::from_env();

        info!(
            "{}",
            format!("[*] {:?}: 🛠️  Getting ready!", agent.persona(),)
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
            bugs: None,
            language,
            nb_bugs: 0,
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

        let request: String = format!(
            "{}\n\nTask Description: {}\nPrevious Conversation: {:?}",
            prompt,
            task.description,
            self.agent.memory(),
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

    /// Asynchronously generates frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `tasks` - A mutable reference to tasks containing the project description.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the generated frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in generating frontend code.
    ///
    /// # Business Logic
    ///
    /// - Determines the file path based on the programming language.
    /// - Reads the template code from the specified file.
    /// - Logs messages throughout the code generation process.
    /// - Constructs a request for generating frontend code using the template and project description.
    /// - Sends the request to the Gemini API to generate frontend code.
    /// - Writes the generated frontend code to the appropriate file.
    pub async fn generate_frontend_code(&mut self, task: &mut Task) -> Result<String> {
        let path = self.workspace.clone();

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/template.rs"),
            "python" => format!("{}/{}", path, "template.py"),
            "javascript" => format!("{}/{}", path, "src/template.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        let template = fs::read_to_string(&frontend_path).await?;

        let prompt = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            FRONTEND_CODE_PROMPT, template, task.description
        );

        self.agent.add_message(Message {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to generate frontend code for project: {}",
                task.description
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to generate frontend code for project: {}",
                        task.description
                    )),
                })
                .await;
        }

        let output = self.build_request(&prompt, task, OutputKind::Text).await?;

        let code = match output {
            GenerationOutput::Text(code) => code,
            _ => {
                return Err(anyhow!("Expected text output for frontend code generation"));
            }
        };

        let frontend_main_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_main_path, &code).await?;

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Frontend code generated and saved to '{frontend_main_path}'"
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Frontend code generated and saved to '{frontend_main_path}'"
                    )),
                })
                .await;
        }

        task.frontend_code = Some(code.clone().into());
        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);

        Ok(code)
    }

    /// Asynchronously improves existing frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `task` - A mutable reference to the task containing the project description and existing code.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the improved frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in improving frontend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request for improving existing frontend code using project description and current code.
    /// - Logs message entries for tracing user intent and AI response.
    /// - Sends the request to the Gemini API to improve the frontend code.
    /// - Writes the improved frontend code to the appropriate file.
    pub async fn improve_frontend_code(&mut self, task: &mut Task) -> Result<String> {
        let path = self.workspace.clone();

        self.agent.add_message(Message {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to improve frontend code for project: {}",
                task.description
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to improve frontend code for project: {}",
                        task.description
                    )),
                })
                .await;
        }

        let existing_code = task.clone().frontend_code.unwrap_or_default();

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Improving existing frontend code using project description...".to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Improving existing frontend code using project description...".to_string(),
                    ),
                })
                .await;
        }

        let prompt = format!(
            "{}\n\nCode Template: {}\nProject Description: {}",
            IMPROVED_FRONTEND_CODE_PROMPT, existing_code, task.description
        );

        let output = self.build_request(&prompt, task, OutputKind::Text).await?;

        let improved_code = match output {
            GenerationOutput::Text(code) => code,
            _ => {
                return Err(anyhow!(
                    "Expected text output for improved frontend code generation"
                ));
            }
        };

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_path, &improved_code).await?;

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!("Improved frontend code saved to '{frontend_path}'",)),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Improved frontend code saved to '{frontend_path}'"
                    )),
                })
                .await;
        }

        task.frontend_code = Some(improved_code.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);

        Ok(improved_code)
    }

    /// Asynchronously fixes bugs in the frontend code based on tasks.
    ///
    /// # Arguments
    ///
    /// * `task` - A mutable reference to the task containing the project description and existing code.
    ///
    /// # Returns
    ///
    /// (`Result<String>`): Result containing the fixed frontend code.
    ///
    /// # Errors
    ///
    /// Returns an error if there's a failure in fixing bugs in the frontend code.
    ///
    /// # Business Logic
    ///
    /// - Constructs a request for fixing bugs in the frontend code using project description and existing code.
    /// - Logs messages throughout the process.
    /// - Sends the request to the Gemini API to fix bugs in the frontend code.
    /// - Writes the fixed frontend code to the appropriate file.
    pub async fn fix_code_bugs(&mut self, task: &mut Task) -> Result<String> {
        let path = self.workspace.clone();

        let bugs_description = self
            .bugs
            .clone()
            .unwrap_or_else(|| "No bug description provided.".into());

        self.agent.add_message(Message {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(format!(
                "Request to fix bugs in frontend code. Known bugs: {bugs_description}"
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("user"),
                    content: Cow::Owned(format!(
                        "Request to fix bugs in frontend code. Known bugs: {bugs_description}"
                    )),
                })
                .await;
        }

        let buggy_code = task.clone().frontend_code.unwrap_or_default();

        let prompt = format!(
            "{FIX_CODE_PROMPT}\n\nBuggy Code: {buggy_code}\nBugs: {bugs_description}\n\nFix all bugs."
        );

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(
                "Attempting to fix bugs in the provided frontend code...".to_string(),
            ),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(
                        "Attempting to fix bugs in the provided frontend code...".to_string(),
                    ),
                })
                .await;
        }

        let output = self.build_request(&prompt, task, OutputKind::Text).await?;

        let fixed_code = match output {
            GenerationOutput::Text(code) => code,
            _ => {
                return Err(anyhow!(
                    "Expected text output for bug-fixed code generation"
                ));
            }
        };

        let frontend_path = match self.language {
            "rust" => format!("{}/{}", path, "src/main.rs"),
            "python" => format!("{}/{}", path, "main.py"),
            "javascript" => format!("{}/{}", path, "src/index.js"),
            _ => panic!("Unsupported language, consider opening an Issue/PR"),
        };

        fs::write(&frontend_path, &fixed_code).await?;

        self.agent.add_message(Message {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(format!(
                "Bugs fixed. Updated code saved to '{frontend_path}'"
            )),
        });

        #[cfg(feature = "mem")]
        {
            let _ = self
                .save_ltm(Message {
                    role: Cow::Borrowed("assistant"),
                    content: Cow::Owned(format!(
                        "Bugs fixed. Updated code saved to '{frontend_path}'"
                    )),
                })
                .await;
        }

        task.frontend_code = Some(fixed_code.clone().into());

        self.agent.update(Status::Completed);
        debug!("[*] {:?}: {:?}", self.agent.persona(), self.agent);

        Ok(fixed_code)
    }
    pub fn think(&self) -> String {
        let behavior = self.agent.behavior();
        format!("How do I build and test the frontend for '{behavior}'")
    }

    pub fn plan(&mut self, context: String) -> Goal {
        let mut goals = vec![
            Goal {
                description: "Generate initial frontend code".into(),
                priority: 1,
                completed: false,
            },
            Goal {
                description: "Improve code quality".into(),
                priority: 2,
                completed: false,
            },
            Goal {
                description: "Run unit tests".into(),
                priority: 3,
                completed: false,
            },
            Goal {
                description: "Fix build/test bugs".into(),
                priority: 4,
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
            description: format!("Fallback task from context: {context}"),
            priority: 99,
            completed: false,
        }
    }
    pub async fn act(
        &mut self,
        goal: Goal,
        task: &mut Task,
        _execute: bool,
        max_tries: u64,
    ) -> Result<()> {
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

        match goal.description.to_lowercase() {
            desc if desc.contains("generate") => {
                let _ = self.generate_frontend_code(task).await;
                self.agent.update(Status::Active);
            }
            desc if desc.contains("improve") => {
                let _ = self.improve_frontend_code(task).await;
                self.agent.update(Status::InUnitTesting);
            }
            desc if desc.contains("test") => {
                let path = &self.workspace.to_string();
                let _ = self.unit_test_and_build(path, task, max_tries).await;
            }
            desc if desc.contains("fix") => {
                let _ = self.fix_code_bugs(task).await;
                self.agent.update(Status::InUnitTesting);
            }
            _ => {
                warn!(
                    "{}",
                    format!(
                        "[*] {:?}: Unknown goal: {}",
                        self.agent.persona(),
                        goal.description
                    )
                    .yellow()
                );
            }
        }
        Ok(())
    }
    pub async fn reflect(&mut self) {
        let summary = format!(
            "Reflection: Reviewing progress on '{}'",
            self.agent.behavior()
        );

        self.agent.memory_mut().push(Message {
            role: Cow::Borrowed("assistant"),
            content: summary.clone().into(),
        });

        self.agent.context_mut().recent_messages.push(Message {
            role: Cow::Borrowed("assistant"),
            content: summary.into(),
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
        info!(
            "{}",
            format!("[*] {:?}: Executing task:", self.agent.persona())
                .bright_white()
                .bold()
        );
        for task in task.clone().description.clone().split("- ") {
            if !task.trim().is_empty() {
                info!("{} {}", "•".bright_white().bold(), task.trim().cyan());
            }
        }
    }
    pub async fn unit_test_and_build(
        &mut self,
        path: &str,
        task: &mut Task,
        max_tries: u64,
    ) -> Result<()> {
        for attempt in 1..=max_tries {
            info!(
                "{}",
                format!(
                    "[*] {:?}: Attempting to build frontend...",
                    self.agent.persona()
                )
                .bright_white()
                .bold()
            );

            let result = self.run_build_command(path).await;

            match result {
                Ok(mut child) => {
                    let mut stderr = String::new();
                    let _ = child
                        .stderr
                        .as_mut()
                        .expect("stderr not captured")
                        .read_to_string(&mut stderr)
                        .await;

                    if stderr.trim().is_empty() {
                        info!(
                            "{}",
                            format!("[*] {:?}: Build succeeded!", self.agent.persona())
                                .bright_green()
                                .bold()
                        );
                        self.agent.update(Status::Completed);
                        break;
                    } else {
                        self.nb_bugs += 1;
                        self.bugs = Some(stderr.clone().into());

                        error!(
                            "{}",
                            format!("[*] {:?}: Build failed: {}", self.agent.persona(), stderr)
                                .bright_red()
                        );

                        if attempt == max_tries {
                            error!(
                                "{}",
                                format!(
                                    "[*] {:?}: Max build attempts reached. Aborting...",
                                    self.agent.persona()
                                )
                                .bright_red()
                            );
                        } else {
                            info!(
                                "{}",
                                format!(
                                    "[*] {:?}: Retrying build... ({}/{})",
                                    self.agent.persona(),
                                    attempt,
                                    max_tries
                                )
                                .yellow()
                            );
                            let _ = self.fix_code_bugs(task).await;
                        }
                    }
                }
                Err(e) => {
                    error!(
                        "{}",
                        format!(
                            "[*] {:?}: Build command execution failed: {}",
                            self.agent.persona(),
                            e
                        )
                        .bright_red()
                    );
                }
            }
        }

        Ok(())
    }
    async fn run_build_command(&self, path: &str) -> Result<Child> {
        match self.language {
            "rust" => Ok(Command::new("timeout")
                .arg("10s")
                .arg("cargo")
                .arg("build")
                .arg("--release")
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?),

            "python" => {
                let venv_path = format!("{path}/.venv");
                let pip_path = format!("{venv_path}/bin/pip");
                let venv_exists = Path::new(&venv_path).exists();

                if !venv_exists {
                    let create_venv = Command::new("python3")
                        .arg("-m")
                        .arg("venv")
                        .arg(&venv_path)
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .status();

                    if let Ok(status) = create_venv.await
                        && status.success()
                    {
                        let main_py_path = format!("{path}/main.py");
                        let main_py_content = fs::read_to_string(&main_py_path)
                            .await
                            .expect("Failed to read main.py");

                        let mut packages = vec![];

                        for line in main_py_content.lines() {
                            if line.starts_with("from ") || line.starts_with("import ") {
                                let parts: Vec<&str> = line.split_whitespace().collect();

                                if let Some(pkg) = parts.get(1) {
                                    let root_pkg = pkg.split('.').next().unwrap_or(pkg);
                                    if !packages.contains(&root_pkg) {
                                        packages.push(root_pkg);
                                    }
                                }
                            }
                        }
                        if !packages.is_empty() {
                            if !packages.contains(&"uvicorn") {
                                packages.push("uvicorn");
                            }
                            if !packages.contains(&"httpx") {
                                packages.push("httpx");
                            }
                            for pkg in &packages {
                                let install_status = Command::new(&pip_path)
                                    .arg("install")
                                    .arg(pkg)
                                    .stdout(Stdio::null())
                                    .stderr(Stdio::null())
                                    .status();

                                match install_status.await {
                                    Ok(status) if status.success() => {
                                        info!(
                                                "{}",
                                                format!(
                                                    "[*] {:?}: Successfully installed Python package '{}'",
                                                    self.agent.persona(),
                                                    pkg
                                                )
                                                .bright_white()
                                                .bold()
                                            );
                                    }
                                    Err(e) => {
                                        error!(
                                                "{}",
                                                format!(
                                                    "[*] {:?}: Failed to install Python package '{}': {}",
                                                    self.agent.persona(),
                                                    pkg,
                                                    e
                                                )
                                                .bright_red()
                                                .bold()
                                            );
                                    }
                                    _ => {
                                        error!(
                                                "{}",
                                                format!(
                                                    "[*] {:?}: Installation of package '{}' exited with an error",
                                                    self.agent.persona(),
                                                    pkg
                                                )
                                                .bright_red()
                                                .bold()
                                            );
                                    }
                                }
                            }
                        }
                    }
                }

                let run_output = Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "timeout {} '.venv/bin/python' -m uvicorn main:app --host 0.0.0.0 --port 8000",
                        10
                    ))
                    .current_dir(path)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()
                    .expect("Failed to run the backend application");

                Ok(run_output)
            }

            "javascript" => Ok(Command::new("timeout")
                .arg("10s")
                .arg("npm")
                .arg("run")
                .arg("build")
                .current_dir(path)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?),

            _ => panic!("Unsupported language: {}", self.language),
        }
    }
    /// Updates the bugs found in the codebase.
    ///
    /// # Arguments
    ///
    /// * `bugs` - Optional description of bugs found in the codebase.
    ///
    /// # Business Logic
    ///
    /// - Updates the bugs field with the provided description.
    ///
    pub fn update_bugs(&mut self, bugs: Option<Cow<'static, str>>) {
        self.bugs = bugs;
    }
}

/// Implementation of the trait `Executor` for FrontendGPT.
/// Contains additional methods related to frontend tasks.
///
/// This trait provides methods for:
///
/// - Retrieving the GPT agent associated with FrontendGPT.
/// - Executing frontend tasks asynchronously.
///
/// # Business Logic
///
/// - Provides access to the GPT agent associated with the FrontendGPT instance.
/// - Executes frontend tasks asynchronously based on the current status of the agent.
/// - Handles task execution including code generation, improvement, bug fixing, and testing.
/// - Manages retries and error handling during task execution.
#[async_trait]
impl Executor for FrontendGPT {
    /// Asynchronously executes frontend tasks associated with FrontendGPT.
    ///
    /// # Arguments
    ///
    /// * `task` - A mutable reference to the task to be executed.
    /// * `execute` - A boolean indicating whether to execute the task.
    /// * `max_tries` - Maximum number of attempts to execute the task.
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
    /// - Executes frontend tasks asynchronously based on the current status of the agent.
    /// - Handles task execution including code generation, improvement, bug fixing, and testing.
    /// - Manages retries and error handling during task execution.
    ///
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        _browse: bool,
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

        while self.agent.status() != &Status::Completed {
            #[cfg(feature = "cli")]
            let pb = spinner("Thinking...");
            let context = self.think();
            #[cfg(feature = "cli")]
            pb.finish_with_message("Thinking complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Planning...");
            let goal = self.plan(context);
            #[cfg(feature = "cli")]
            pb.finish_with_message("Planning complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Acting on goal...");
            self.act(goal.clone(), task, execute, max_tries).await?;
            #[cfg(feature = "cli")]
            pb.finish_with_message("Action complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Marking goal complete...");
            self.mark_goal_complete(goal);
            #[cfg(feature = "cli")]
            pb.finish_with_message("Goal marked complete!");

            #[cfg(feature = "cli")]
            let pb = spinner("Reflecting...");
            self.reflect().await;
            #[cfg(feature = "cli")]
            pb.finish_with_message("Reflection complete!");

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

        self.agent.update(Status::Idle);
        Ok(())
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
