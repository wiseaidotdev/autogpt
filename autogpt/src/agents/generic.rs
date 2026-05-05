use crate::agents::agent::AgentGPT;
use crate::common::utils::{
    Capability, ClientType, ContextManager, Knowledge, Persona, Planner, Reflection, Status, Task,
    TaskScheduler, Tool,
};
#[cfg(feature = "cli")]
use crate::prelude::*;
use crate::traits::agent::Agent;
use auto_derive::Auto;
use std::borrow::Cow;

#[cfg(feature = "net")]
use crate::collaboration::Collaborator;

#[cfg(feature = "mem")]
use {
    crate::common::memory::load_long_term_memory, crate::common::memory::long_term_memory_context,
    crate::common::memory::save_long_term_memory,
};

#[cfg(feature = "oai")]
use {openai_dive::v1::models::Gpt4Model, openai_dive::v1::resources::chat::*};

#[cfg(feature = "cld")]
use anthropic_ai_sdk::types::message::{
    ContentBlock, CreateMessageParams, Message as AnthMessage, MessageClient,
    RequiredMessageParams, Role,
};

#[cfg(feature = "gem")]
use gems::{
    chat::ChatBuilder, imagen::ImageGenBuilder, messages::Content, models::Model,
    stream::StreamBuilder, traits::CTrait,
};

#[cfg(any(
    feature = "co",
    feature = "oai",
    feature = "gem",
    feature = "cld",
    feature = "xai"
))]
use crate::traits::functions::ReqResponse;

#[cfg(feature = "xai")]
use x_ai::{
    chat_compl::{ChatCompletionsRequestBuilder, Message as XaiMessage},
    traits::ChatCompletionsFetcher,
};

#[cfg(feature = "cli")]
use {
    crate::cli::session::{Session, SessionManager, SessionTask, TaskStatus as SessionTaskStatus},
    crate::cli::tui::{
        TaskStatus as TuiTaskStatus, create_spinner, print_agent_msg, print_error, print_section,
        print_success, print_task_item, print_warning, render_markdown,
    },
    crate::prompts::generic::{
        GENERIC_SYSTEM_PROMPT, IMPLEMENTATION_PLAN_PROMPT, REFLECTION_PROMPT,
        TASK_EXECUTION_PROMPT, TASK_SYNTHESIS_PROMPT, WALKTHROUGH_PROMPT,
    },
    colored::Colorize,
    serde::{Deserialize, Serialize},
    std::io::{self, BufRead, Write},
    std::time::Duration,
    termimad::crossterm::event::{self, Event, KeyCode},
    tracing::{error, info, warn},
};

/// The operational phase of the generic agent within a session lifecycle.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, PartialEq)]
pub enum PhaseState {
    Idle,
    Synthesizing,
    Planning,
    AwaitingApproval,
    Executing(usize),
    Reflecting,
    Complete,
}

/// A single structured action directive emitted by the LLM.
///
/// The LLM outputs a JSON array of these during task execution. Each variant maps to
/// a concrete filesystem or shell operation that AutoGPT's runtime carries out directly.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ActionRequest {
    CreateDir {
        path: String,
    },
    CreateFile {
        path: String,
        content: String,
    },
    WriteFile {
        path: String,
        content: String,
    },
    RunCommand {
        cmd: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
    GitCommit {
        message: String,
    },
}

/// Result of executing a single action directive.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct ActionResult {
    pub action_type: String,
    pub path: Option<String>,
    pub stdout: String,
    pub stderr: String,
    pub success: bool,
}

/// The reflection verdict returned by the LLM after verifying task output.
#[cfg(feature = "cli")]
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReflectionOutcome {
    Success,
    Retry,
    Skip,
}

#[cfg(feature = "cli")]
#[derive(Debug, Deserialize)]
pub struct ReflectionResult {
    pub outcome: ReflectionOutcome,
    pub reasoning: String,
    pub corrective_actions: Vec<ActionRequest>,
}

/// A general-purpose autonomous agent that operates interactively from the CLI.
///
/// `GenericAgent` is the default agent launched when running `autogpt` with no subcommand
/// (and without `--net`). It is invoked by the `run_generic_agent_loop` REPL - which reads
/// the user's prompt, creates a `Task` with it, and calls `Executor::execute`.
///
/// Inside `execute`, the agent runs the full pipeline:
///   1. Synthesise a numbered task list from the prompt.
///   2. Generate a markdown implementation plan.
///   3. Optionally gate on user approval (skipped in yolo mode via `_execute = false`).
///   4. Execute each task by prompting the LLM for `ActionRequest` JSON and running each action.
///   5. Reflect on every task's output; retry up to `max_tries` times before skipping.
///   6. Write a session walkthrough document on completion.
#[cfg(feature = "cli")]
#[derive(Debug, Default, Clone, Auto)]
pub struct GenericAgent {
    pub agent: AgentGPT,
    pub client: ClientType,
    /// Whether to skip the approval gate (mirrors the `--yolo` flag).
    pub yolo: bool,
    /// Workspace root directory for generated files.
    pub workspace: String,
    /// Human-readable model name (for session metadata).
    pub model: String,
    /// Provider name (gemini / openai / etc.) for session metadata.
    pub provider: String,
}

#[cfg(feature = "cli")]
use {
    crate::traits::functions::{AsyncFunctions, Functions},
    anyhow::Result,
    async_trait::async_trait,
};

#[cfg(feature = "cli")]
#[async_trait]
impl Executor for GenericAgent {
    /// Runs the full AutoGPT pipeline for a single user prompt.
    ///
    /// `task.description` is the user's prompt.
    /// `execute`  - when `false` the plan-approval gate is skipped (yolo mode).
    /// `_browse`  - reserved for future web-search integration.
    /// `max_tries` - maximum retry attempts per sub-task on reflection failure.
    async fn execute<'a>(
        &'a mut self,
        task: &'a mut Task,
        execute: bool,
        _browse: bool,
        max_tries: u64,
    ) -> Result<()> {
        let prompt = task.description.to_string();
        if prompt.is_empty() {
            return Ok(());
        }

        let max_retries = max_tries.clamp(1, 5) as u8;

        let session_mgr = SessionManager::default();
        session_mgr.ensure_dirs()?;

        let workspace = if self.workspace.is_empty() {
            std::env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| std::path::PathBuf::from("."))
                    .join(".autogpt")
                    .join("workspace")
                    .to_string_lossy()
                    .to_string()
            })
        } else {
            self.workspace.clone()
        };
        std::fs::create_dir_all(&workspace)?;
        let workspace_path = std::path::PathBuf::from(&workspace);

        let model = if self.model.is_empty() {
            "gemini-2.5-flash".to_string()
        } else {
            self.model.clone()
        };
        let provider = if self.provider.is_empty() {
            "gemini".to_string()
        } else {
            self.provider.clone()
        };

        let mut session = Session::new(&prompt, &workspace, &model, &provider);
        session.add_message("user", &prompt);

        task.description = format!("[AutoGPT] {prompt}").into();

        print_section("🔬 Synthesizing Task List");
        let task_spinner = create_spinner("Decomposing your request into actionable tasks...");

        let tasks = match self.synthesize_tasks(&prompt, "").await {
            Ok(t) if !t.is_empty() => t,
            Ok(_) => {
                task_spinner.finish_and_clear();
                print_error("LLM returned an empty task list.");
                return Ok(());
            }
            Err(e) => {
                task_spinner.finish_and_clear();
                print_error(&format!("Task synthesis failed: {e}"));
                return Err(e);
            }
        };
        task_spinner.finish_and_clear();

        session.set_tasks(tasks.clone());
        session.add_message(
            "assistant",
            &tasks
                .iter()
                .enumerate()
                .map(|(i, t)| format!("{}. {}", i + 1, t.description))
                .collect::<Vec<_>>()
                .join("\n"),
        );
        session_mgr.save(&session)?;

        print_section("📋 Task Plan");
        for t in &tasks {
            print_task_item(&t.description, TuiTaskStatus::Pending);
        }

        print_section("🏗️  Generating Implementation Plan");
        let plan_spinner = create_spinner("Architecting a production-grade solution...");

        let plan = match self.generate_plan(&prompt, &tasks).await {
            Ok(p) => p,
            Err(e) => {
                plan_spinner.finish_and_clear();
                print_error(&format!("Plan generation failed: {e}"));
                return Err(e);
            }
        };
        plan_spinner.finish_and_clear();

        session.set_plan(&plan);
        session.add_message("assistant", &plan);

        print_section("📑 Implementation Plan");
        render_markdown(&plan);

        session_mgr.save(&session)?;

        if execute {
            info!("");
            info!(
                "{}  Approve this plan and begin execution? {} ",
                "?".bright_cyan().bold(),
                "(yes / no)".bright_black()
            );
            print!("> ");
            io::stdout().flush()?;

            let mut approval = String::new();
            io::stdin().lock().read_line(&mut approval)?;

            if !crate::common::utils::is_yes(approval.trim()) {
                print_warning("Plan not approved. Ready for next prompt.");
                session.add_message("user", "Plan rejected.");
                session_mgr.save(&session)?;
                return Ok(());
            }
        }

        print_section("⚙️  Executing Tasks via AutoGPT");

        let tasks_snapshot = session.tasks.clone();
        let total = tasks_snapshot.len();

        for (idx, task_item) in tasks_snapshot.iter().enumerate() {
            print_task_item(&task_item.description, TuiTaskStatus::InProgress);
            session.update_task_status(idx, SessionTaskStatus::InProgress);
            session_mgr.save(&session)?;

            let exec_spinner = create_spinner(&format!(
                "Task {}/{}: {}",
                idx + 1,
                total,
                &task_item.description[..task_item.description.len().min(55)]
            ));

            let completed_descs: Vec<String> = session
                .tasks
                .iter()
                .take(idx)
                .map(|t| t.description.clone())
                .collect();
            let completed_refs: Vec<&str> = completed_descs.iter().map(|s| s.as_str()).collect();

            let mut results = self
                .execute_task(
                    &prompt,
                    task_item,
                    idx + 1,
                    total,
                    &plan,
                    &completed_refs,
                    &workspace_path,
                    &mut session,
                )
                .await
                .unwrap_or_default();

            exec_spinner.finish_and_clear();

            let mut retry_count: u8 = 0;
            loop {
                let reflection = self
                    .reflect_on_task(task_item, &results, retry_count)
                    .await
                    .unwrap_or(ReflectionResult {
                        outcome: ReflectionOutcome::Success,
                        reasoning: "Assumed success.".into(),
                        corrective_actions: vec![],
                    });

                match reflection.outcome {
                    ReflectionOutcome::Success => {
                        print_task_item(&task_item.description, TuiTaskStatus::Completed);
                        session.update_task_status(idx, SessionTaskStatus::Completed);
                        break;
                    }
                    ReflectionOutcome::Retry if retry_count < max_retries => {
                        retry_count += 1;
                        print_warning(&format!(
                            "Retry {retry_count}/{max_retries} - {}",
                            reflection.reasoning
                        ));
                        results = Vec::new();
                        for action in &reflection.corrective_actions {
                            let r = GenericAgent::run_action(action, &workspace_path, &mut session)
                                .await;
                            results.push(r);
                        }
                    }
                    ReflectionOutcome::Retry | ReflectionOutcome::Skip => {
                        print_task_item(&task_item.description, TuiTaskStatus::Skipped);
                        session.update_task_status(idx, SessionTaskStatus::Skipped);
                        warn!("  ⊘  Skipped: {}", reflection.reasoning.bright_black());
                        break;
                    }
                }
            }

            session_mgr.save(&session)?;
        }

        print_section("📓 Generating Session Walkthrough");
        let wt_spinner = create_spinner("Composing walkthrough document...");

        let tasks_status = session
            .tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {} [{}]", i + 1, t.description, t.status.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        let files_str = session
            .files_created
            .iter()
            .map(|f| format!("{} ({})", f.path, f.action))
            .collect::<Vec<_>>()
            .join("\n");

        let wt_prompt = WALKTHROUGH_PROMPT
            .replace("{SESSION_ID}", &session.id)
            .replace(
                "{DATE}",
                &session.created_at.format("%Y-%m-%d %H:%M UTC").to_string(),
            )
            .replace("{MODEL}", &model)
            .replace("{WORKSPACE}", &workspace)
            .replace("{PROMPT}", &prompt)
            .replace("{TASK_LIST_WITH_STATUSES}", &tasks_status)
            .replace("{FILES_CREATED}", &files_str);

        let walkthrough = self
            .generate(&wt_prompt)
            .await
            .unwrap_or_else(|_| SessionManager::generate_walkthrough(&session));

        wt_spinner.finish_and_clear();

        session.set_walkthrough(&walkthrough);
        session_mgr.save(&session)?;

        let wt_path = session_mgr.base_dir.join("walkthrough.md");
        std::fs::write(&wt_path, &walkthrough)?;

        print_section("✅ Session Complete");
        print_success(&format!("Walkthrough → {}", wt_path.display()));
        print_success(&format!(
            "Session     → {}",
            session_mgr.session_dir(&session.id).display()
        ));
        print_success(&format!("Workspace   → {workspace}"));
        print_agent_msg(
            "AutoGPT",
            "All tasks complete. Ready for your next request.",
        );

        Ok(())
    }
}

#[cfg(feature = "cli")]
impl GenericAgent {
    /// Synthesises a numbered task list for the given prompt via the LLM.
    async fn synthesize_tasks(&mut self, prompt: &str, history: &str) -> Result<Vec<SessionTask>> {
        let full_prompt = TASK_SYNTHESIS_PROMPT
            .replace("{PROMPT}", prompt)
            .replace("{HISTORY}", history);

        let raw = self.generate(&full_prompt).await?;

        let tasks = raw
            .lines()
            .filter(|l| {
                let t = l.trim();
                !t.is_empty()
                    && t.chars()
                        .next()
                        .map(|c| c.is_ascii_digit())
                        .unwrap_or(false)
            })
            .map(|l| {
                let desc = l
                    .trim()
                    .trim_start_matches(|c: char| c.is_ascii_digit())
                    .trim_start_matches('.')
                    .trim()
                    .to_string();
                SessionTask {
                    description: desc,
                    status: SessionTaskStatus::Pending,
                }
            })
            .collect();

        Ok(tasks)
    }

    /// Generates a detailed markdown implementation plan via the LLM.
    async fn generate_plan(&mut self, prompt: &str, tasks: &[SessionTask]) -> Result<String> {
        let task_list = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t.description))
            .collect::<Vec<_>>()
            .join("\n");

        let title: String = prompt.chars().take(60).collect();

        let full_prompt = IMPLEMENTATION_PLAN_PROMPT
            .replace("{TITLE}", &title)
            .replace("{PROMPT}", prompt)
            .replace("{TASK_LIST}", &task_list);

        self.generate(&full_prompt).await
    }

    /// Executes a single task by asking the LLM to emit ActionRequest JSON and running it.
    #[allow(clippy::too_many_arguments)]
    async fn execute_task(
        &mut self,
        prompt: &str,
        task: &SessionTask,
        task_num: usize,
        task_total: usize,
        plan: &str,
        completed: &[&str],
        workspace: &std::path::Path,
        session: &mut Session,
    ) -> Result<Vec<ActionResult>> {
        let completed_str = completed
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        let plan_lines: Vec<&str> = plan.lines().collect();
        let plan_excerpt: String = plan_lines
            .iter()
            .skip_while(|l| !l.contains(&task.description[..task.description.len().min(30)]))
            .take(15)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let execution_prompt = TASK_EXECUTION_PROMPT
            .replace("{WORKSPACE}", &workspace.to_string_lossy())
            .replace("{PROMPT}", prompt)
            .replace("{TASK_NUM}", &task_num.to_string())
            .replace("{TASK_TOTAL}", &task_total.to_string())
            .replace("{TASK_DESCRIPTION}", &task.description)
            .replace(
                "{PLAN_EXCERPT}",
                if plan_excerpt.is_empty() {
                    plan
                } else {
                    &plan_excerpt
                },
            )
            .replace("{COMPLETED_TASKS}", &completed_str);

        let combined = format!(
            "{}\n\nYou are operating inside workspace: {}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            workspace.display(),
            execution_prompt
        );

        let raw = self.generate(&combined).await.unwrap_or_default();
        let clean = crate::common::utils::strip_code_blocks(&raw);

        let actions: Vec<ActionRequest> = serde_json::from_str(clean.trim()).unwrap_or_default();

        let mut results = Vec::new();
        for action in &actions {
            let result = Self::run_action(action, workspace, session).await;
            results.push(result);
        }

        Ok(results)
    }

    /// Dispatches a single `ActionRequest` to the appropriate filesystem or shell operation.
    pub async fn run_action(
        action: &ActionRequest,
        workspace: &std::path::Path,
        session: &mut Session,
    ) -> ActionResult {
        match action {
            ActionRequest::CreateDir { path } => {
                let abs = workspace.join(path);
                match std::fs::create_dir_all(&abs) {
                    Ok(_) => {
                        info!("  {} {}", "📁".bright_cyan(), path.bright_blue());
                        ActionResult {
                            action_type: "CreateDir".into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => {
                        error!(
                            "  {} Failed to create dir {}: {}",
                            "✗".bright_red(),
                            path,
                            e
                        );
                        ActionResult {
                            action_type: "CreateDir".into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        }
                    }
                }
            }

            ActionRequest::CreateFile { path, content }
            | ActionRequest::WriteFile { path, content } => {
                let abs = workspace.join(path);
                let action_type = match action {
                    ActionRequest::CreateFile { .. } => "CreateFile",
                    _ => "WriteFile",
                };

                if let Some(parent) = abs.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }

                match std::fs::write(&abs, content) {
                    Ok(_) => {
                        info!("  {} {}", "📄".bright_cyan(), path.bright_blue());
                        session.record_file(path, action_type);
                        ActionResult {
                            action_type: action_type.into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => {
                        error!("  {} Failed to write {}: {}", "✗".bright_red(), path, e);
                        ActionResult {
                            action_type: action_type.into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        }
                    }
                }
            }

            ActionRequest::RunCommand { cmd, args, cwd } => {
                let working_dir = cwd
                    .as_ref()
                    .map(|c| workspace.join(c))
                    .unwrap_or_else(|| workspace.to_path_buf());

                info!(
                    "  {} {} {}",
                    "⚡".bright_magenta(),
                    cmd.bright_cyan().bold(),
                    args.join(" ").bright_white()
                );

                match tokio::process::Command::new(cmd)
                    .args(args)
                    .current_dir(&working_dir)
                    .output()
                    .await
                {
                    Ok(out) => {
                        let stdout = String::from_utf8_lossy(&out.stdout).to_string();
                        let stderr = String::from_utf8_lossy(&out.stderr).to_string();
                        let success = out.status.success();

                        if !stdout.trim().is_empty() {
                            for line in stdout.lines().take(20) {
                                info!("    {}", line.bright_black());
                            }
                        }

                        if !success && !stderr.trim().is_empty() {
                            for line in stderr.lines().take(10) {
                                error!("    {}", line.bright_red());
                            }
                        }

                        ActionResult {
                            action_type: "RunCommand".into(),
                            path: None,
                            stdout,
                            stderr,
                            success,
                        }
                    }
                    Err(e) => {
                        error!("  {} Command failed: {}", "✗".bright_red(), e);
                        ActionResult {
                            action_type: "RunCommand".into(),
                            path: None,
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        }
                    }
                }
            }

            ActionRequest::GitCommit { message } => {
                let _ = tokio::process::Command::new("git")
                    .args(["add", "-A"])
                    .current_dir(workspace)
                    .output()
                    .await;

                match tokio::process::Command::new("git")
                    .args(["commit", "-m", message])
                    .current_dir(workspace)
                    .output()
                    .await
                {
                    Ok(out) => {
                        let success = out.status.success();
                        if success {
                            info!("  {} git: {}", "🔖".bright_cyan(), message.bright_black());
                        }
                        ActionResult {
                            action_type: "GitCommit".into(),
                            path: None,
                            stdout: String::from_utf8_lossy(&out.stdout).to_string(),
                            stderr: String::from_utf8_lossy(&out.stderr).to_string(),
                            success,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "GitCommit".into(),
                        path: None,
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }
        }
    }

    /// Asks the LLM to reflect on task execution results and return a verdict.
    async fn reflect_on_task(
        &mut self,
        task: &SessionTask,
        results: &[ActionResult],
        retry_attempt: u8,
    ) -> Result<ReflectionResult> {
        let actions_str = results
            .iter()
            .map(|r| format!("- {} {:?}", r.action_type, r.path))
            .collect::<Vec<_>>()
            .join("\n");

        let outputs_str = results
            .iter()
            .map(|r| {
                format!(
                    "[{}] success={}\nstdout: {}\nstderr: {}",
                    r.action_type,
                    r.success,
                    r.stdout.chars().take(500).collect::<String>(),
                    r.stderr.chars().take(500).collect::<String>()
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        let full_prompt = REFLECTION_PROMPT
            .replace("{TASK_DESCRIPTION}", &task.description)
            .replace("{ACTIONS_EXECUTED}", &actions_str)
            .replace("{COMMAND_OUTPUTS}", &outputs_str)
            .replace("{RETRY_ATTEMPT}", &retry_attempt.to_string());

        let raw = self.generate(&full_prompt).await.unwrap_or_default();
        let clean = crate::common::utils::strip_code_blocks(&raw);

        match serde_json::from_str(clean.trim()) {
            Ok(result) => Ok(result),
            Err(_) => Ok(ReflectionResult {
                outcome: ReflectionOutcome::Success,
                reasoning: "Could not parse reflection - assuming success.".into(),
                corrective_actions: vec![],
            }),
        }
    }
}

/// Runs the interactive AutoGPT CLI loop.
///
/// This is a thin REPL shell that:
///   1. Prints the TUI banner, tips, and startup warnings.
///   2. Reads the user's slash commands or prompts in a loop.
///   3. On a real prompt, builds a `Task` and calls `agent.execute()` - which contains
///      the full synthesize → plan → approve → execute → reflect → walkthrough pipeline.
///
/// Slash commands (`/help`, `/sessions`, `/models`, `/clear`, `/status`, `/workspace`,
/// `/provider`) are handled here and never reach the executor.
#[cfg(feature = "cli")]
pub async fn run_generic_agent_loop(yolo: bool, session_id: Option<&str>) -> anyhow::Result<()> {
    use crate::cli::tui::{
        ModelEntry, print_banner, print_greeting, print_status_bar, render_help_table,
        render_input_box_hint, render_model_selector, render_warning_box,
    };

    print_banner();
    print_greeting();

    if yolo {
        warn!(
            "{}",
            "⚡  YOLO mode active - all plans will be auto-approved."
                .bright_yellow()
                .bold()
        );
        info!("");
    }

    let session_mgr = SessionManager::default();
    session_mgr.ensure_dirs()?;

    if let Some(id) = session_id {
        match session_mgr.load(id) {
            Ok(session) => {
                print_section("📂 Resuming Session");
                info!(
                    "  {} {} - {} tasks, {} messages",
                    "▸".bright_cyan(),
                    session.title.white().bold(),
                    session.tasks.len(),
                    session.messages.len()
                );
                info!("");
                for msg in &session.messages {
                    info!(
                        "  {} {}",
                        format!("[{}]", msg.role).bright_magenta(),
                        msg.content
                            .chars()
                            .take(120)
                            .collect::<String>()
                            .bright_white()
                    );
                }
                info!("");
            }
            Err(e) => print_warning(&format!("Could not load session {id}: {e}")),
        }
    }

    let workspace = std::env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
        dirs::home_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join(".autogpt")
            .join("workspace")
            .to_string_lossy()
            .to_string()
    });
    std::fs::create_dir_all(&workspace)?;

    if matches!((std::env::current_dir(), dirs::home_dir()), (Ok(cwd), Some(home)) if cwd == home) {
        render_warning_box(
            "You are running AutoGPT in your home directory.\n\
             It is recommended to run AutoGPT from a project-specific directory\n\
             so that generated files are scoped correctly.",
        );
    }

    let mut current_model = "gemini-2.5-flash".to_string();
    let mut current_provider = "gemini".to_string();

    let available_models = vec![
        ModelEntry {
            id: "gemini-2.5-flash".into(),
            display_name: "Flash 2.0".into(),
            description: "Fastest model, ideal for iterative task execution".into(),
        },
        ModelEntry {
            id: "gemini-2.5-pro-preview-05-06".into(),
            display_name: "Gemini 2.5 Pro".into(),
            description: "Most capable - deep reasoning and complex architectures".into(),
        },
        ModelEntry {
            id: "gemini-2.5-flash-lite".into(),
            display_name: "Flash 2.0 Lite".into(),
            description: "Ultra-fast, efficient for simple tasks".into(),
        },
    ];
    let mut current_model_idx: usize = 0;

    let mut agent = GenericAgent {
        yolo,
        workspace: workspace.clone(),
        model: current_model.clone(),
        provider: current_provider.clone(),
        ..Default::default()
    };

    loop {
        if let Ok(cwd) = std::env::current_dir() {
            print_status_bar(&cwd.to_string_lossy(), &current_model, &current_provider);
        }

        render_input_box_hint();
        info!(
            "{} {}",
            ">".bright_blue().bold(),
            "Type your request, or /help for commands".bright_black()
        );
        print!("> ");
        io::stdout().flush()?;

        let stdin = io::stdin();
        let mut line = String::new();
        stdin.lock().read_line(&mut line)?;
        let input = line.trim().to_string();

        if input.is_empty() {
            print_warning("Please enter a prompt to work on.");
            continue;
        }

        if input.eq_ignore_ascii_case("exit") || input.eq_ignore_ascii_case("quit") {
            print_success("Session saved. Goodbye!");
            break;
        }

        if input.eq_ignore_ascii_case("/help") {
            render_help_table();
            continue;
        }

        if input.eq_ignore_ascii_case("/clear") {
            print!("\x1B[2J\x1B[1;1H");
            io::stdout().flush()?;
            print_banner();
            print_greeting();
            continue;
        }

        if input.eq_ignore_ascii_case("/workspace") {
            info!(
                "{} {}",
                "Workspace:".bright_cyan(),
                workspace.bright_white()
            );
            continue;
        }

        if input.eq_ignore_ascii_case("/status") {
            let cwd = std::env::current_dir()
                .map(|p| p.to_string_lossy().to_string())
                .unwrap_or_default();
            info!("{} {}", "Directory:".bright_cyan(), cwd.bright_white());
            info!(
                "{} {}",
                "Model:".bright_cyan(),
                current_model.bright_magenta()
            );
            info!(
                "{} {}",
                "Provider:".bright_cyan(),
                current_provider.bright_white()
            );
            info!(
                "{} {}",
                "Workspace:".bright_cyan(),
                workspace.bright_white()
            );
            continue;
        }

        if input.eq_ignore_ascii_case("/provider") {
            let providers = ["gemini", "openai", "anthropic", "xai", "cohere"];
            info!("");
            info!("{}", "Select Provider:".bright_cyan().bold());
            for (i, p) in providers.iter().enumerate() {
                if *p == current_provider {
                    info!(
                        "  {} {} {}",
                        format!("{}.", i + 1).bright_cyan(),
                        p.bright_white().bold(),
                        "(active)".bright_green()
                    );
                } else {
                    info!(
                        "  {} {}",
                        format!("{}.", i + 1).bright_black(),
                        p.bright_white()
                    );
                }
            }
            info!("");
            print!("> Enter number: ");
            io::stdout().flush()?;
            let mut pick = String::new();
            io::stdin().lock().read_line(&mut pick)?;
            if let (Ok(n), current_len) = (pick.trim().parse::<usize>(), providers.len())
                && n >= 1
                && n <= current_len
            {
                current_provider = providers[n - 1].to_string();
                agent.provider = current_provider.clone();
                print_success(&format!("Switched to provider: {current_provider}"));
            }
            continue;
        }

        if input.eq_ignore_ascii_case("/models") {
            let selected = render_model_selector(&available_models, current_model_idx);
            current_model_idx = selected;
            current_model = available_models[selected].id.clone();
            agent.model = current_model.clone();
            print_success(&format!(
                "Model set to: {}",
                available_models[selected].display_name
            ));
            continue;
        }

        if input.eq_ignore_ascii_case("/sessions") {
            match session_mgr.list() {
                Ok(entries) if !entries.is_empty() => {
                    print_section("📁 Recent Sessions");
                    for (i, entry) in entries.iter().enumerate() {
                        info!(
                            "  {} {} {} {}",
                            format!("{}.", i + 1).bright_cyan(),
                            entry.title.white().bold(),
                            format!("({}/{})", entry.completed_count, entry.task_count)
                                .bright_green(),
                            entry
                                .updated_at
                                .format("%Y-%m-%d %H:%M")
                                .to_string()
                                .bright_black()
                        );
                        info!("     {} {}", "↳".bright_black(), entry.id.bright_black());
                    }
                    info!("");
                    print!("> Enter number to resume (or press Enter to skip): ");
                    io::stdout().flush()?;

                    let mut pick = String::new();
                    io::stdin().lock().read_line(&mut pick)?;
                    let pick = pick.trim();

                    if let Some(entry) = pick.parse::<usize>().ok().and_then(|n| {
                        if n >= 1 && n <= entries.len() {
                            Some(&entries[n - 1])
                        } else {
                            None
                        }
                    }) {
                        match session_mgr.load(&entry.id) {
                            Ok(s) => {
                                print_section("📂 Resumed Session");
                                info!("  {} {}", "▸".bright_cyan(), s.title.white().bold());
                                for msg in &s.messages {
                                    info!(
                                        "  {} {}",
                                        format!("[{}]", msg.role).bright_magenta(),
                                        msg.content
                                            .chars()
                                            .take(120)
                                            .collect::<String>()
                                            .bright_white()
                                    );
                                }
                            }
                            Err(e) => print_error(&format!("Failed to load session: {e}")),
                        }
                    }
                }
                Ok(_) => print_warning("No previous sessions found."),
                Err(e) => print_error(&format!("Failed to list sessions: {e}")),
            }
            continue;
        }

        agent.agent.behavior = input.clone().into();

        let arc_agent: Arc<Mutex<Box<dyn AgentFunctions>>> =
            Arc::new(Mutex::new(Box::new(agent.clone())));

        let autogpt = AutoGPT::default()
            .execute(!yolo)
            .max_tries(2)
            .with(vec![arc_agent])
            .build()
            .expect("Failed to build AutoGPT");

        let mut interrupt_handle = tokio::spawn(async move {
            loop {
                if matches!(event::poll(Duration::from_millis(100)), Ok(true))
                    && let Ok(Event::Key(key)) = event::read()
                    && key.code == KeyCode::Esc
                {
                    return;
                }
                tokio::time::sleep(Duration::from_millis(50)).await;
            }
        });

        tokio::select! {
            res = autogpt.run() => {
                interrupt_handle.abort();
                match res {
                    Ok(msg) => info!("{}", msg.bright_green()),
                    Err(e) => print_error(&format!("Agent error: {e:?}")),
                }
            }
            _ = &mut interrupt_handle => {
                print_warning("Execution interrupted by user (ESC pressed).");
            }
        }
    }

    Ok(())
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
