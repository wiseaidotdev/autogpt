use crate::agents::agent::AgentGPT;
use crate::common::utils::{
    Capability, ClientType, ContextManager, Knowledge, Persona, Planner, Reflection, Status, Task,
    TaskScheduler, Tool, is_yes, strip_code_blocks,
};
#[allow(unused_imports)]
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
#[cfg(feature = "cli")]
use crate::prelude::*;
use crate::traits::agent::Agent;
use auto_derive::Auto;
use std::borrow::Cow;

#[cfg(feature = "net")]
use crate::collaboration::Collaborator;

#[cfg(feature = "mop")]
use crate::agents::mop::run_mixture;

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

#[cfg(feature = "cli")]
use {
    crate::cli::models::{default_model, default_provider, model_index, provider_models},
    crate::cli::session::{Session, SessionManager, SessionTask, TaskStatus as SessionTaskStatus},
    crate::cli::skills::SkillStore,
    crate::cli::tui::{
        TaskStatus as TuiTaskStatus, create_spinner, print_agent_msg, print_banner, print_error,
        print_greeting, print_section, print_status_bar, print_success, print_task_item,
        print_warning, render_help_table, render_input_box_hint, render_markdown,
        render_model_selector, render_warning_box,
    },
    crate::prompts::generic::{
        FOLLOWUP_SYNTHESIS_PROMPT, GENERIC_SYSTEM_PROMPT, IMPLEMENTATION_PLAN_PROMPT,
        LESSON_EXTRACTION_PROMPT, REASONING_PROMPT, REFLECTION_PROMPT, TASK_EXECUTION_PROMPT,
        TASK_SYNTHESIS_PROMPT, WALKTHROUGH_PROMPT,
    },
    anyhow::anyhow,
    colored::Colorize,
    serde::{Deserialize, Serialize},
    std::env,
    std::fs,
    std::io::{self, BufRead, Write as IoWrite},
    std::path::{Path, PathBuf},
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

/// A single structured action directive emitted by the LLM during task execution.
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
    ReadFile {
        path: String,
    },
    PatchFile {
        path: String,
        old_text: String,
        new_text: String,
    },
    AppendFile {
        path: String,
        content: String,
    },
    ListDir {
        path: String,
    },
    FindInFile {
        path: String,
        pattern: String,
    },
    RunCommand {
        cmd: String,
        args: Vec<String>,
        cwd: Option<String>,
    },
    GitCommit {
        message: String,
    },
    GlobFiles {
        pattern: String,
    },
    MultiPatch {
        path: String,
        patches: Vec<(String, String)>,
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

/// Structured inner monologue emitted by the LLM before executing each task.
#[cfg(feature = "cli")]
#[derive(Debug, Deserialize, Default)]
pub struct ReasoningResult {
    pub thought: String,
    pub approach: String,
    #[serde(default)]
    pub risks: Vec<String>,
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
            env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".autogpt")
                    .join("workspace")
                    .to_string_lossy()
                    .to_string()
            })
        } else {
            self.workspace.clone()
        };
        fs::create_dir_all(&workspace)?;
        let workspace_path = PathBuf::from(&workspace);

        let skills_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("skills");

        let skills = SkillStore::load_for_domain(&prompt, skills_dir.clone())
            .unwrap_or_else(|_| SkillStore::new(skills_dir.clone()));
        let skills_context = skills.to_prompt_context();

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

        let workspace_snapshot = self.scan_workspace(&workspace_path).await;

        let tasks = match self
            .synthesize_tasks(&prompt, "", &skills_context, &workspace_snapshot)
            .await
        {
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
            info!(
                "{}  Approve this plan and begin execution? {} ",
                "?".bright_cyan().bold(),
                "(yes / no)".bright_black()
            );
            print!("> ");
            io::stdout().flush()?;

            let mut approval = String::new();
            io::stdin().lock().read_line(&mut approval)?;

            if !is_yes(approval.trim()) {
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

            let reasoning = self
                .reason_about_task(
                    task_item,
                    &plan,
                    &completed_refs,
                    idx + 1,
                    total,
                    &workspace,
                )
                .await;
            session.add_reasoning(&reasoning.thought);

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
                    &reasoning,
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

        let build_succeeded = self
            .build_and_verify(&workspace_path, &mut session, 3)
            .await;
        if !build_succeeded {
            print_warning("Build verification failed after all attempts.");
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

        let wt_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            WALKTHROUGH_PROMPT
                .replace("{SESSION_ID}", &session.id)
                .replace(
                    "{DATE}",
                    &session.created_at.format("%Y-%m-%d %H:%M UTC").to_string()
                )
                .replace("{MODEL}", &model)
                .replace("{WORKSPACE}", &workspace)
                .replace("{PROMPT}", &prompt)
                .replace("{TASK_LIST_WITH_STATUSES}", &tasks_status)
                .replace("{FILES_CREATED}", &files_str)
        );

        let walkthrough = match self.generate(&wt_prompt).await {
            Ok(w) if !w.trim().is_empty() => w,
            _ => SessionManager::generate_walkthrough(&session),
        };

        wt_spinner.finish_and_clear();
        session.set_walkthrough(&walkthrough);
        session_mgr.save(&session)?;

        let wt_path = session_mgr.base_dir.join("walkthrough.md");
        fs::write(&wt_path, &walkthrough)?;

        if let Some((domain, lessons, anti_patterns)) = self
            .extract_lessons(&prompt, &session.tasks, &tasks_status)
            .await
            && (!lessons.is_empty() || !anti_patterns.is_empty())
        {
            let mut store = SkillStore::new(skills_dir);
            for lesson in &lessons {
                let _ = store.save_lesson(&domain, lesson, None);
            }
            for ap in &anti_patterns {
                let _ = store.save_lesson(&domain, "", Some(ap));
            }
        }

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
    async fn synthesize_tasks(
        &mut self,
        prompt: &str,
        history: &str,
        skills_context: &str,
        workspace_snapshot: &str,
    ) -> Result<Vec<SessionTask>> {
        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            TASK_SYNTHESIS_PROMPT
                .replace("{PROMPT}", prompt)
                .replace("{HISTORY}", history)
                .replace("{WORKSPACE_SNAPSHOT}", workspace_snapshot)
                .replace("{SKILLS_CONTEXT}", skills_context)
        );

        let raw: String = self.generate(&full_prompt).await?;

        let numbered: Vec<SessionTask> = raw
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
            .filter(|t| Self::is_valid_task_desc(&t.description))
            .collect();

        if !numbered.is_empty() {
            return Ok(numbered);
        }

        let clean = strip_code_blocks(&raw);
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(clean.trim()) {
            let from_json: Vec<SessionTask> = arr
                .into_iter()
                .filter(|s| Self::is_valid_task_desc(s))
                .map(|s| SessionTask {
                    description: s,
                    status: SessionTaskStatus::Pending,
                })
                .collect();
            if !from_json.is_empty() {
                return Ok(from_json);
            }
        }

        Err(anyhow!(
            "LLM returned malformed task list. Raw output: {}",
            &raw[..raw.len().min(200)]
        ))
    }

    fn is_valid_task_desc(desc: &str) -> bool {
        let words: Vec<&str> = desc.split_whitespace().collect();
        words.len() >= 3 && desc.len() >= 15
    }

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

    async fn reason_about_task(
        &mut self,
        task: &SessionTask,
        plan: &str,
        completed: &[&str],
        task_num: usize,
        task_total: usize,
        workspace: &str,
    ) -> ReasoningResult {
        let completed_str = completed
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {}", i + 1, t))
            .collect::<Vec<_>>()
            .join("\n");

        let plan_lines: Vec<&str> = plan.lines().collect();
        let search_key = &task.description[..task.description.len().min(40)];
        let plan_excerpt: String = plan_lines
            .iter()
            .skip_while(|l| !l.to_lowercase().contains(&search_key.to_lowercase()))
            .take(12)
            .cloned()
            .collect::<Vec<_>>()
            .join("\n");

        let effective_excerpt = if plan_excerpt.is_empty() {
            plan.lines().take(15).collect::<Vec<_>>().join("\n")
        } else {
            plan_excerpt
        };

        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            REASONING_PROMPT
                .replace("{TASK_NUM}", &task_num.to_string())
                .replace("{TASK_TOTAL}", &task_total.to_string())
                .replace("{TASK_DESCRIPTION}", &task.description)
                .replace("{PLAN_EXCERPT}", &effective_excerpt)
                .replace("{COMPLETED_TASKS}", &completed_str)
                .replace("{WORKSPACE}", workspace)
        );

        let raw = self.generate(&full_prompt).await.unwrap_or_default();
        let clean = strip_code_blocks(&raw);

        let parsed = serde_json::from_str::<ReasoningResult>(clean.trim());
        match parsed {
            Ok(r) if !r.thought.trim().is_empty() => r,
            _ => {
                let thought = if raw.trim().is_empty() {
                    format!(
                        "Executing task {task_num}/{task_total}: {}.",
                        task.description
                    )
                } else {
                    raw.chars().take(400).collect()
                };
                ReasoningResult {
                    thought,
                    approach: format!("Emit action directives for: {}", task.description),
                    risks: vec![],
                }
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    async fn execute_task(
        &mut self,
        prompt: &str,
        task: &SessionTask,
        task_num: usize,
        task_total: usize,
        plan: &str,
        completed: &[&str],
        workspace: &Path,
        session: &mut Session,
        reasoning: &ReasoningResult,
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

        let reasoning_text = if reasoning.thought.is_empty() {
            String::new()
        } else {
            format!(
                "Thought: {}\nApproach: {}\nRisks: {}",
                reasoning.thought,
                reasoning.approach,
                reasoning.risks.join(", ")
            )
        };

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
            .replace("{COMPLETED_TASKS}", &completed_str)
            .replace("{REASONING}", &reasoning_text);

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

    /// Dispatches a single `ActionRequest` to the appropriate operation.
    pub async fn run_action(
        action: &ActionRequest,
        workspace: &Path,
        session: &mut Session,
    ) -> ActionResult {
        match action {
            ActionRequest::CreateDir { path } => {
                let abs = workspace.join(path);
                match fs::create_dir_all(&abs) {
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
                    Err(e) => ActionResult {
                        action_type: "CreateDir".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
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
                    let _ = fs::create_dir_all(parent);
                }
                match fs::write(&abs, content) {
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
                    Err(e) => ActionResult {
                        action_type: action_type.into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }

            ActionRequest::ReadFile { path } => {
                let abs = workspace.join(path);
                match fs::read_to_string(&abs) {
                    Ok(content) => {
                        info!("  {} {}", "📖".bright_cyan(), path.bright_blue());
                        ActionResult {
                            action_type: "ReadFile".into(),
                            path: Some(path.clone()),
                            stdout: content,
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "ReadFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }

            ActionRequest::PatchFile {
                path,
                old_text,
                new_text,
            } => {
                let abs = workspace.join(path);
                match fs::read_to_string(&abs) {
                    Ok(content) => {
                        if !content.contains(old_text.as_str()) {
                            return ActionResult {
                                action_type: "PatchFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: format!(
                                    "patch anchor not found in {path}. \
                                     Use ReadFile first to confirm the exact text."
                                ),
                                success: false,
                            };
                        }
                        let patched = content.replacen(old_text.as_str(), new_text.as_str(), 1);
                        match fs::write(&abs, &patched) {
                            Ok(_) => {
                                info!("  {} {}", "✏️ ".bright_cyan(), path.bright_blue());
                                session.record_file(path, "PatchFile");
                                ActionResult {
                                    action_type: "PatchFile".into(),
                                    path: Some(path.clone()),
                                    stdout: format!("Patched {path} successfully."),
                                    stderr: String::new(),
                                    success: true,
                                }
                            }
                            Err(e) => ActionResult {
                                action_type: "PatchFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            },
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "PatchFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }

            ActionRequest::AppendFile { path, content } => {
                let abs = workspace.join(path);
                if let Some(parent) = abs.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                match fs::OpenOptions::new().create(true).append(true).open(&abs) {
                    Ok(mut file) => match file.write_all(content.as_bytes()) {
                        Ok(_) => {
                            info!("  {} {}", "➕".bright_cyan(), path.bright_blue());
                            session.record_file(path, "AppendFile");
                            ActionResult {
                                action_type: "AppendFile".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: String::new(),
                                success: true,
                            }
                        }
                        Err(e) => ActionResult {
                            action_type: "AppendFile".into(),
                            path: Some(path.clone()),
                            stdout: String::new(),
                            stderr: e.to_string(),
                            success: false,
                        },
                    },
                    Err(e) => ActionResult {
                        action_type: "AppendFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }

            ActionRequest::ListDir { path } => {
                let abs = workspace.to_path_buf();
                match fs::read_dir(&abs) {
                    Ok(entries) => {
                        let mut lines = Vec::new();
                        for entry in entries.flatten() {
                            let entry: std::fs::DirEntry = entry;
                            let name = entry.file_name().to_string_lossy().to_string();
                            let is_dir = entry
                                .file_type()
                                .map(|t: std::fs::FileType| t.is_dir())
                                .unwrap_or(false);
                            lines.push(if is_dir { format!("{name}/") } else { name });
                        }
                        lines.sort();
                        ActionResult {
                            action_type: "ListDir".into(),
                            path: Some(path.clone()),
                            stdout: lines.join("\n"),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "ListDir".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }

            ActionRequest::FindInFile { path, pattern } => {
                let abs = workspace.join(path);
                match fs::read_to_string(&abs) {
                    Ok(content) => {
                        let matches: Vec<String> = content
                            .lines()
                            .enumerate()
                            .filter(|(_, line)| line.contains(pattern.as_str()))
                            .map(|(i, line)| format!("{}:{}", i + 1, line))
                            .collect();
                        ActionResult {
                            action_type: "FindInFile".into(),
                            path: Some(path.clone()),
                            stdout: matches.join("\n"),
                            stderr: String::new(),
                            success: true,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "FindInFile".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
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

            ActionRequest::GlobFiles { pattern } => {
                let mut matched: Vec<String> = Vec::new();
                Self::walk_glob(workspace, workspace, pattern, &mut matched);
                matched.sort();
                ActionResult {
                    action_type: "GlobFiles".into(),
                    path: None,
                    stdout: matched.join("\n"),
                    stderr: String::new(),
                    success: true,
                }
            }

            ActionRequest::MultiPatch { path, patches } => {
                let abs = workspace.join(path);
                match fs::read_to_string(&abs) {
                    Ok(original) => {
                        let mut content = original;
                        let mut applied = 0usize;
                        let mut errors: Vec<String> = Vec::new();
                        for (old_text, new_text) in patches {
                            if content.contains(old_text.as_str()) {
                                content = content.replacen(old_text.as_str(), new_text.as_str(), 1);
                                applied += 1;
                            } else {
                                errors.push(format!(
                                    "anchor not found: {:?}",
                                    &old_text[..old_text.len().min(60)]
                                ));
                            }
                        }
                        if let Err(e) = fs::write(&abs, &content) {
                            return ActionResult {
                                action_type: "MultiPatch".into(),
                                path: Some(path.clone()),
                                stdout: String::new(),
                                stderr: e.to_string(),
                                success: false,
                            };
                        }
                        let success = errors.is_empty();
                        if success {
                            info!(
                                "  {} {} ({} patches)",
                                "✏️ ".bright_cyan(),
                                path.bright_blue(),
                                applied
                            );
                            session.record_file(path, "MultiPatch");
                        }
                        ActionResult {
                            action_type: "MultiPatch".into(),
                            path: Some(path.clone()),
                            stdout: format!("{applied} patches applied."),
                            stderr: errors.join("\n"),
                            success,
                        }
                    }
                    Err(e) => ActionResult {
                        action_type: "MultiPatch".into(),
                        path: Some(path.clone()),
                        stdout: String::new(),
                        stderr: e.to_string(),
                        success: false,
                    },
                }
            }
        }
    }

    fn walk_glob(workspace: &Path, dir: &Path, pattern: &str, results: &mut Vec<String>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                Self::walk_glob(workspace, &path, pattern, results);
            } else if let Ok(rel) = path.strip_prefix(workspace) {
                let rel_str = rel.to_string_lossy();
                if pattern_matches(pattern, &rel_str) {
                    results.push(rel_str.to_string());
                }
            }
        }
    }

    fn detect_build_system(workspace: &Path) -> Option<ActionRequest> {
        if workspace.join("Cargo.toml").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "cargo".into(),
                args: vec!["build".into()],
                cwd: None,
            });
        }
        if workspace.join("package.json").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "npm".into(),
                args: vec!["run".into(), "build".into()],
                cwd: None,
            });
        }
        if workspace.join("pyproject.toml").exists()
            || workspace.join("setup.py").exists()
            || workspace.join("requirements.txt").exists()
        {
            return Some(ActionRequest::RunCommand {
                cmd: "python3".into(),
                args: vec!["-m".into(), "compileall".into(), "-q".into(), ".".into()],
                cwd: None,
            });
        }
        if workspace.join("go.mod").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "go".into(),
                args: vec!["build".into(), "./...".into()],
                cwd: None,
            });
        }
        if workspace.join("Makefile").exists() || workspace.join("makefile").exists() {
            return Some(ActionRequest::RunCommand {
                cmd: "make".into(),
                args: vec![],
                cwd: None,
            });
        }
        None
    }

    async fn build_and_verify(
        &mut self,
        workspace: &Path,
        session: &mut Session,
        max_attempts: u8,
    ) -> bool {
        let build_action = match Self::detect_build_system(workspace) {
            Some(a) => a,
            None => return true,
        };

        print_section("🔨 Verifying Build");

        for attempt in 0..max_attempts {
            session.increment_build_attempt();
            let result = Self::run_action(&build_action, workspace, session).await;

            if result.success {
                print_success(&format!("Build passed on attempt {}", attempt + 1));
                return true;
            }

            if attempt + 1 >= max_attempts {
                break;
            }

            print_warning(&format!(
                "Build attempt {} failed. Asking LLM for fix...",
                attempt + 1
            ));

            let error_context = format!(
                "Build failed:\nstdout: {}\nstderr: {}",
                &result.stdout[..result.stdout.len().min(800)],
                &result.stderr[..result.stderr.len().min(800)]
            );

            let fix_prompt = format!(
                "{}\n\nYou are operating in workspace: {}\n\n\
                The project build just failed. Analyze the error output and emit a JSON array \
                of action directives that fix the compilation/syntax errors.\n\n\
                Error context:\n{error_context}\n\n\
                Output only a valid JSON array starting with `[` and ending with `]`.",
                GENERIC_SYSTEM_PROMPT,
                workspace.display()
            );

            let raw = self.generate(&fix_prompt).await.unwrap_or_default();
            let clean = strip_code_blocks(&raw);
            let fix_actions: Vec<ActionRequest> =
                serde_json::from_str(clean.trim()).unwrap_or_default();

            for action in &fix_actions {
                let _ = Self::run_action(action, workspace, session).await;
            }
        }

        false
    }

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
                    r.stdout.chars().take(600).collect::<String>(),
                    r.stderr.chars().take(400).collect::<String>()
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

    async fn extract_lessons(
        &mut self,
        prompt: &str,
        tasks: &[SessionTask],
        results_summary: &str,
    ) -> Option<(String, Vec<String>, Vec<String>)> {
        let tasks_str = tasks
            .iter()
            .enumerate()
            .map(|(i, t)| format!("{}. {} [{}]", i + 1, t.description, t.status.as_str()))
            .collect::<Vec<_>>()
            .join("\n");

        let full_prompt = LESSON_EXTRACTION_PROMPT
            .replace("{ORIGINAL_PROMPT}", prompt)
            .replace("{TASKS}", &tasks_str)
            .replace("{RESULTS}", results_summary);

        let raw = self.generate(&full_prompt).await.ok()?;
        let clean = crate::common::utils::strip_code_blocks(&raw);

        serde_json::from_str::<LessonOutput>(clean.trim())
            .ok()
            .map(|l| (l.domain, l.lessons, l.anti_patterns))
    }

    /// Scans the workspace up to two directory levels deep and reads common config files.
    ///
    /// Returns a compact string suitable for injection into synthesis prompts so the LLM
    /// knows what already exists before emitting task directives. The total output is
    /// capped at roughly 2 000 characters to limit token overhead.
    async fn scan_workspace(&self, workspace: &Path) -> String {
        let mut lines: Vec<String> = Vec::new();
        Self::list_tree(workspace, workspace, 0, 2, &mut lines);

        let config_names = [
            "Cargo.toml",
            "package.json",
            "pyproject.toml",
            "requirements.txt",
            "go.mod",
            "Makefile",
            "README.md",
        ];
        let mut config_snippets: Vec<String> = Vec::new();
        for name in &config_names {
            let path = workspace.join(name);
            if let Ok(content) = fs::read_to_string(&path) {
                let preview: String = content.lines().take(12).collect::<Vec<_>>().join("\n");
                if !preview.trim().is_empty() {
                    config_snippets.push(format!("### {name}\n```\n{preview}\n```"));
                }
            }
        }

        let tree = lines.join("\n");
        let configs = config_snippets.join("\n\n");
        let combined = if configs.is_empty() {
            tree
        } else {
            format!("{tree}\n\n{configs}")
        };

        if combined.is_empty() {
            String::new()
        } else {
            combined.chars().take(2000).collect()
        }
    }

    fn list_tree(
        workspace: &Path,
        dir: &Path,
        depth: usize,
        max_depth: usize,
        out: &mut Vec<String>,
    ) {
        if depth > max_depth {
            return;
        }
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        let mut items: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                !p.file_name()
                    .and_then(|n| n.to_str())
                    .map(|n| n.starts_with('.'))
                    .unwrap_or(false)
            })
            .collect();
        items.sort();
        for path in items {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            let rel = path
                .strip_prefix(workspace)
                .map(|r| r.to_string_lossy().to_string())
                .unwrap_or_else(|_| name.to_string());
            let indent = "  ".repeat(depth);
            if path.is_dir() {
                out.push(format!("{indent}{rel}/"));
                Self::list_tree(workspace, &path, depth + 1, max_depth, out);
            } else {
                out.push(format!("{indent}{rel}"));
            }
        }
    }

    /// Synthesises a delta task list for a follow-up request on an existing session.
    ///
    /// Uses `FOLLOWUP_SYNTHESIS_PROMPT` which instructs the LLM to emit only the
    /// _new_ tasks required to satisfy the request, without re-scaffolding anything
    /// already built.
    async fn synthesize_followup_tasks(
        &mut self,
        user_request: &str,
        prior_session: &Session,
        skills_context: &str,
        workspace_snapshot: &str,
    ) -> Result<Vec<SessionTask>> {
        let prior_context = prior_session.session_context_summary();

        let full_prompt = format!(
            "{}\n\n{}",
            GENERIC_SYSTEM_PROMPT,
            FOLLOWUP_SYNTHESIS_PROMPT
                .replace("{USER_REQUEST}", user_request)
                .replace("{PRIOR_CONTEXT}", &prior_context)
                .replace("{WORKSPACE_SNAPSHOT}", workspace_snapshot)
                .replace("{SKILLS_CONTEXT}", skills_context)
        );

        let raw: String = self.generate(&full_prompt).await?;

        let numbered: Vec<SessionTask> = raw
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
            .filter(|t| Self::is_valid_task_desc(&t.description))
            .collect();

        if !numbered.is_empty() {
            return Ok(numbered);
        }

        let clean = strip_code_blocks(&raw);
        if let Ok(arr) = serde_json::from_str::<Vec<String>>(clean.trim()) {
            let from_json: Vec<SessionTask> = arr
                .into_iter()
                .filter(|s| Self::is_valid_task_desc(s))
                .map(|s| SessionTask {
                    description: s,
                    status: SessionTaskStatus::Pending,
                })
                .collect();
            if !from_json.is_empty() {
                return Ok(from_json);
            }
        }

        Err(anyhow::anyhow!(
            "LLM returned an empty follow-up task list. Raw: {}",
            &raw[..raw.len().min(200)]
        ))
    }
}

/// Lesson data extracted at the end of a session for the skill store.
#[cfg(feature = "cli")]
#[derive(Deserialize)]
struct LessonOutput {
    domain: String,
    #[serde(default)]
    lessons: Vec<String>,
    #[serde(default)]
    anti_patterns: Vec<String>,
}

/// Returns `true` when a file path matches a simple glob pattern.
///
/// Supports `*` (any characters within one path segment) and `**` (any path).
/// Case-sensitive. Used by the `GlobFiles` action handler.
#[cfg(feature = "cli")]
fn pattern_matches(pattern: &str, path: &str) -> bool {
    if pattern == "**" {
        return true;
    }
    if !pattern.contains('*') {
        return path.ends_with(pattern) || path == pattern;
    }
    let parts: Vec<&str> = pattern.splitn(2, '*').collect();
    let prefix = parts[0];
    let suffix = if parts.len() > 1 { parts[1] } else { "" };
    path.starts_with(prefix)
        && (suffix.is_empty() || path.ends_with(suffix))
        && path.len() >= prefix.len() + suffix.len()
}

/// Runs the interactive AutoGPT CLI loop.
///
/// This is a thin REPL shell that:
///   1. Prints the TUI banner, tips, and startup warnings.
///   2. Reads the user's slash commands or prompts in a loop.
///   3. On a real prompt, builds a `Task` and calls `agent.execute()` - which contains
///      the full synthesize → plan → approve → execute → reflect → walkthrough pipeline.
///   4. On follow-up prompts in the same session, passes the prior session context
///      to `synthesize_followup_tasks` so the LLM works on top of existing code.
///
/// Slash commands (`/help`, `/sessions`, `/models`, `/clear`, `/status`, `/workspace`,
/// `/provider`) are handled here and never reach the executor.
#[cfg(feature = "cli")]
pub async fn run_generic_agent_loop(
    yolo: bool,
    session_id: Option<&str>,
    _mixture: bool,
) -> anyhow::Result<()> {
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

    let mut active_session: Option<Session> = None;

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

    let workspace = env::var("AUTOGPT_WORKSPACE").unwrap_or_else(|_| {
        PathBuf::from(".")
            .join("workspace")
            .to_string_lossy()
            .to_string()
    });
    fs::create_dir_all(&workspace)?;

    if matches!((std::env::current_dir(), dirs::home_dir()), (Ok(cwd), Some(home)) if cwd == home) {
        render_warning_box(
            "You are running AutoGPT in your home directory.\n\
             It is recommended to run AutoGPT from a project-specific directory\n\
             so that generated files are scoped correctly.",
        );
    }

    let mut current_provider = default_provider();
    let mut current_model = default_model(&current_provider);
    let mut available_models = provider_models(&current_provider);
    let mut current_model_idx = model_index(&available_models, &current_model);

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
        #[allow(unused_mut)]
        let mut input = line.trim().to_string();

        #[cfg(feature = "mop")]
        if _mixture
            && !input.starts_with('/')
            && !is_yes(&input)
            && let Some((provider, response)) = run_mixture(&input).await
        {
            print_success(&format!("MoP selected response from: {provider}"));
            render_markdown(&response);
            input = format!("High-quality context from {provider}: {response}\n\nTask: {input}");
        }

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
            let cwd = env::current_dir()
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

                available_models = provider_models(&current_provider);
                current_model = default_model(&current_provider);
                current_model_idx = model_index(&available_models, &current_model);
                agent.model = current_model.clone();

                print_success(&format!("Switched to provider: {current_provider}"));
            }
            continue;
        }

        if input.eq_ignore_ascii_case("/models") {
            if available_models.is_empty() {
                print_warning(
                    "No models available for the current provider. Try setting the appropriate API key.",
                );
                continue;
            }
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

        if let Some(ref prior) = active_session {
            info!(
                "  {} {}",
                "↩".bright_cyan(),
                format!("Follow-up on: {}", prior.title).bright_black()
            );
        }

        agent.agent.behavior = input.clone().into();

        let workspace_path = PathBuf::from(&workspace);
        let skills_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".autogpt")
            .join("skills");
        let skills = SkillStore::load_for_domain(&input, skills_dir).unwrap_or_else(|_| {
            SkillStore::new(
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join(".autogpt")
                    .join("skills"),
            )
        });
        let skills_context = skills.to_prompt_context();

        if let Some(ref prior) = active_session {
            let snapshot = agent.scan_workspace(&workspace_path).await;
            let tasks = match agent
                .synthesize_followup_tasks(&input, prior, &skills_context, &snapshot)
                .await
            {
                Ok(t) if !t.is_empty() => t as Vec<SessionTask>,
                Ok(_) => {
                    print_error("LLM returned an empty follow-up task list.");
                    continue;
                }
                Err(e) => {
                    print_error(&format!("Follow-up synthesis failed: {e}"));
                    continue;
                }
            };

            print_section("📋 Follow-up Task Plan");
            for t in &tasks {
                print_task_item(&t.description, TuiTaskStatus::Pending);
            }

            if !yolo {
                info!(
                    "{}  Approve and execute these tasks? {} ",
                    "?".bright_cyan().bold(),
                    "(yes / no)".bright_black()
                );
                print!("> ");
                io::stdout().flush()?;
                let mut approval = String::new();
                io::stdin().lock().read_line(&mut approval)?;
                if !is_yes(approval.trim()) {
                    print_warning("Tasks not approved.");
                    continue;
                }
            }

            let model = if agent.model.is_empty() {
                "gemini-2.5-flash".to_string()
            } else {
                agent.model.clone()
            };
            let provider = if agent.provider.is_empty() {
                "gemini".to_string()
            } else {
                agent.provider.clone()
            };

            let mut new_session = Session::new(&input, &workspace, &model, &provider);
            new_session.add_message("user", &input);
            new_session.set_tasks(tasks.clone());

            let plan = prior.plan.clone().unwrap_or_default();
            print_section("⚙️  Executing Follow-up Tasks");
            let total = tasks.len();
            let tasks_snap = tasks.clone();
            for (idx, task_item) in tasks_snap.iter().enumerate() {
                print_task_item(&task_item.description, TuiTaskStatus::InProgress);
                new_session.update_task_status(idx, SessionTaskStatus::InProgress);

                let completed_descs: Vec<String> = tasks_snap
                    .iter()
                    .take(idx)
                    .map(|t| t.description.clone())
                    .collect();
                let completed_refs: Vec<&str> =
                    completed_descs.iter().map(|s| s.as_str()).collect();

                let reasoning = agent
                    .reason_about_task(
                        task_item,
                        &plan,
                        &completed_refs,
                        idx + 1,
                        total,
                        &workspace,
                    )
                    .await;
                new_session.add_reasoning(&reasoning.thought);

                let results = agent
                    .execute_task(
                        &input,
                        task_item,
                        idx + 1,
                        total,
                        &plan,
                        &completed_refs,
                        &workspace_path,
                        &mut new_session,
                        &reasoning,
                    )
                    .await
                    .unwrap_or_default();

                let reflection = agent
                    .reflect_on_task(task_item, &results, 0)
                    .await
                    .unwrap_or(ReflectionResult {
                        outcome: ReflectionOutcome::Success,
                        reasoning: "Assumed success.".into(),
                        corrective_actions: vec![],
                    });

                match reflection.outcome {
                    ReflectionOutcome::Success => {
                        print_task_item(&task_item.description, TuiTaskStatus::Completed);
                        new_session.update_task_status(idx, SessionTaskStatus::Completed);
                    }
                    _ => {
                        print_task_item(&task_item.description, TuiTaskStatus::Skipped);
                        new_session.update_task_status(idx, SessionTaskStatus::Skipped);
                    }
                }

                session_mgr.save(&new_session)?;
            }

            let _ = agent
                .build_and_verify(&workspace_path, &mut new_session, 3)
                .await;

            session_mgr.save(&new_session)?;
            active_session = Some(new_session);
            continue;
        }

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
                    Ok(msg) => {
                        info!("{}", msg.bright_green());
                        if let Ok(entries) = session_mgr.list()
                            && let Some(entry) = entries.first()
                                && let Ok(s) = session_mgr.load(&entry.id) {
                                    active_session = Some(s);
                            }
                    }
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
