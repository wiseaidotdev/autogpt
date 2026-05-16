// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use chrono::{DateTime, Utc};
#[cfg(feature = "cli")]
use dirs::home_dir;
#[cfg(feature = "cli")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "cli")]
use serde_json::{from_str, to_string_pretty};
#[cfg(feature = "cli")]
use std::cmp;
#[cfg(feature = "cli")]
use std::fs;
#[cfg(feature = "cli")]
use std::path::PathBuf;
#[cfg(feature = "cli")]
use uuid::Uuid;

/// Completion state of a single session task.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

#[cfg(feature = "cli")]
impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn icon(self) -> &'static str {
        match self {
            Self::Pending => "○",
            Self::InProgress => "●",
            Self::Completed => "✓",
            Self::Failed => "✗",
            Self::Skipped => "⊘",
        }
    }
}

/// A single message exchanged during a session.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMessage {
    pub role: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
}

/// A task item tracked within a session, including its execution outcome.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTask {
    pub description: String,
    pub status: TaskStatus,
}

/// A file that was created or written during task execution.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionFile {
    pub path: String,
    pub action: String,
}

/// Cumulative token and request statistics for a session.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub tokens_sent: u64,
    pub tokens_received: u64,
    pub requests: u32,
    pub responses: u32,
}

/// Persistent session data stored under `~/.autogpt/sessions/<id>/`.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub model: String,
    pub provider: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub messages: Vec<SessionMessage>,
    pub tasks: Vec<SessionTask>,
    pub files_created: Vec<SessionFile>,
    pub plan: Option<String>,
    pub walkthrough: Option<String>,
    pub workspace: String,
    pub reasoning_log: Vec<String>,
    pub build_attempts: u8,
    pub stats: SessionStats,
}

#[cfg(feature = "cli")]
impl Session {
    pub fn new(prompt: &str, workspace: &str, model: &str, provider: &str) -> Self {
        let now = Utc::now();
        let title = prompt.chars().take(60).collect::<String>();
        Self {
            id: Uuid::new_v4().to_string(),
            title,
            prompt: prompt.to_string(),
            model: model.to_string(),
            provider: provider.to_string(),
            created_at: now,
            updated_at: now,
            messages: Vec::new(),
            tasks: Vec::new(),
            files_created: Vec::new(),
            plan: None,
            walkthrough: None,
            workspace: workspace.to_string(),
            reasoning_log: Vec::new(),
            build_attempts: 0,
            stats: SessionStats::default(),
        }
    }

    pub fn record_request(&mut self, prompt_len: usize) {
        self.stats.tokens_sent += (prompt_len / 4).max(1) as u64;
        self.stats.requests += 1;
        self.updated_at = Utc::now();
    }

    pub fn record_response(&mut self, response_len: usize) {
        self.stats.tokens_received += (response_len / 4).max(1) as u64;
        self.stats.responses += 1;
        self.updated_at = Utc::now();
    }

    pub fn add_reasoning(&mut self, thought: &str) {
        self.reasoning_log.push(thought.to_string());
        self.updated_at = Utc::now();
    }

    pub fn increment_build_attempt(&mut self) {
        self.build_attempts = self.build_attempts.saturating_add(1);
        self.updated_at = Utc::now();
    }

    pub fn add_message(&mut self, role: &str, content: &str) {
        self.messages.push(SessionMessage {
            role: role.to_string(),
            content: content.to_string(),
            timestamp: Utc::now(),
        });
        self.updated_at = Utc::now();
    }

    pub fn set_tasks(&mut self, tasks: Vec<SessionTask>) {
        self.tasks = tasks;
        self.updated_at = Utc::now();
    }

    pub fn update_task_status(&mut self, index: usize, status: TaskStatus) {
        if let Some(task) = self.tasks.get_mut(index) {
            task.status = status;
            self.updated_at = Utc::now();
        }
    }

    pub fn record_file(&mut self, path: &str, action: &str) {
        self.files_created.push(SessionFile {
            path: path.to_string(),
            action: action.to_string(),
        });
        self.updated_at = Utc::now();
    }

    pub fn set_plan(&mut self, plan: &str) {
        self.plan = Some(plan.to_string());
        self.updated_at = Utc::now();
    }

    pub fn set_walkthrough(&mut self, walkthrough: &str) {
        self.walkthrough = Some(walkthrough.to_string());
        self.updated_at = Utc::now();
    }

    /// Produces a compact, token-efficient summary of the session's prior state.
    ///
    /// The output is injected into `{HISTORY}` and `{PREVIOUS_CONTEXT}` placeholders in
    /// follow-up prompts so the LLM knows exactly what was already built without receiving
    /// the full, potentially large session JSON. Capped at roughly 1200 tokens.
    pub fn session_context_summary(&self) -> String {
        let mut parts: Vec<String> = Vec::new();

        parts.push(format!("## Prior Session: {}", self.title));
        parts.push(format!("Workspace: {}", self.workspace));

        if !self.tasks.is_empty() {
            let task_lines: Vec<String> = self
                .tasks
                .iter()
                .enumerate()
                .map(|(i, t)| format!("  {}. [{}] {}", i + 1, t.status.as_str(), t.description))
                .collect();
            parts.push(format!("Tasks completed:\n{}", task_lines.join("\n")));
        }

        if !self.files_created.is_empty() {
            let file_lines: Vec<String> = self
                .files_created
                .iter()
                .take(30)
                .map(|f| format!("  - {} ({})", f.path, f.action))
                .collect();
            let suffix = if self.files_created.len() > 30 {
                format!("\n  ... and {} more", self.files_created.len() - 30)
            } else {
                String::new()
            };
            parts.push(format!(
                "Files created:\n{}{}",
                file_lines.join("\n"),
                suffix
            ));
        }

        if let Some(ref plan) = self.plan {
            let excerpt: String = plan.lines().take(20).collect::<Vec<_>>().join("\n");
            parts.push(format!("Implementation plan (excerpt):\n{excerpt}"));
        }

        let last_messages: Vec<String> = self
            .messages
            .iter()
            .rev()
            .take(6)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .map(|m| {
                let snippet: String = m.content.chars().take(200).collect();
                format!("[{}]: {}", m.role, snippet)
            })
            .collect();
        if !last_messages.is_empty() {
            parts.push(format!(
                "Recent conversation:\n{}",
                last_messages.join("\n")
            ));
        }

        parts.join("\n\n")
    }
}

/// A lightweight summary entry for listing available sessions.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub id: String,
    pub title: String,
    pub prompt: String,
    pub model: String,
    pub provider: String,
    pub updated_at: DateTime<Utc>,
    pub task_count: usize,
    pub completed_count: usize,
}

/// Manages session persistence under the autogpt home directory.
#[cfg(feature = "cli")]
pub struct SessionManager {
    pub base_dir: PathBuf,
}

#[cfg(feature = "cli")]
impl SessionManager {
    pub fn new(base_dir: Option<&str>) -> Self {
        let base = match base_dir {
            Some(dir) => PathBuf::from(dir),
            None => home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".autogpt"),
        };
        Self { base_dir: base }
    }

    pub fn sessions_dir(&self) -> PathBuf {
        self.base_dir.join("sessions")
    }

    pub fn session_dir(&self, session_id: &str) -> PathBuf {
        self.sessions_dir().join(session_id)
    }

    pub fn ensure_dirs(&self) -> anyhow::Result<()> {
        fs::create_dir_all(self.sessions_dir())?;
        Ok(())
    }

    pub fn save(&self, session: &Session) -> anyhow::Result<()> {
        let dir = self.session_dir(&session.id);
        fs::create_dir_all(&dir)?;

        let json = to_string_pretty(session)?;
        fs::write(dir.join("session.json"), json)?;

        if let Some(ref plan) = session.plan {
            fs::write(dir.join("implementation_plan.md"), plan)?;
        }

        if let Some(ref walkthrough) = session.walkthrough {
            fs::write(dir.join("walkthrough.md"), walkthrough)?;
        }

        if !session.tasks.is_empty() {
            fs::write(dir.join("tasks.md"), Self::render_tasks_md(&session.tasks))?;
        }

        if !session.reasoning_log.is_empty() {
            let reasoning_md = session
                .reasoning_log
                .iter()
                .enumerate()
                .map(|(i, t)| format!("## Task {} Reasoning\n\n{}\n", i + 1, t))
                .collect::<Vec<_>>()
                .join("\n");
            fs::write(dir.join("reasoning_log.md"), reasoning_md)?;
        }

        let stats_json = to_string_pretty(&session.stats)?;
        fs::write(dir.join("stats.json"), stats_json)?;

        Ok(())
    }

    pub fn load(&self, session_id: &str) -> anyhow::Result<Session> {
        let path = self.session_dir(session_id).join("session.json");
        let content = fs::read_to_string(path)?;
        Ok(from_str(&content)?)
    }

    pub fn list(&self) -> anyhow::Result<Vec<SessionEntry>> {
        let sessions_dir = self.sessions_dir();
        if !sessions_dir.exists() {
            return Ok(Vec::new());
        }

        let mut entries = Vec::new();
        for entry in fs::read_dir(&sessions_dir)? {
            let entry = entry?;
            if !entry.file_type()?.is_dir() {
                continue;
            }
            let session_file = entry.path().join("session.json");
            if !session_file.exists() {
                continue;
            }
            if let Ok(content) = fs::read_to_string(&session_file)
                && let Ok(session) = from_str::<Session>(&content)
            {
                let completed_count = session
                    .tasks
                    .iter()
                    .filter(|t| t.status == TaskStatus::Completed)
                    .count();
                entries.push(SessionEntry {
                    id: session.id,
                    title: session.title,
                    prompt: session.prompt,
                    model: session.model,
                    provider: session.provider,
                    updated_at: session.updated_at,
                    task_count: session.tasks.len(),
                    completed_count,
                });
            }
        }

        entries.sort_by_key(|b| cmp::Reverse(b.updated_at));
        Ok(entries)
    }

    fn render_tasks_md(tasks: &[SessionTask]) -> String {
        let mut md = String::from("# Tasks\n\n");
        for task in tasks {
            let checkbox = match task.status {
                TaskStatus::Completed => "[x]",
                TaskStatus::InProgress => "[/]",
                TaskStatus::Failed => "[-]",
                TaskStatus::Skipped => "[~]",
                TaskStatus::Pending => "[ ]",
            };
            md.push_str(&format!("- {} {}\n", checkbox, task.description));
        }
        md
    }

    pub fn generate_walkthrough(session: &Session) -> String {
        let mut md = String::from("# AutoGPT Session Walkthrough\n\n");
        md.push_str(&format!("**Session:** {}\n", session.title));
        md.push_str(&format!("**ID:** {}\n", session.id));
        md.push_str(&format!(
            "**Created:** {}\n",
            session.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!(
            "**Model:** {} ({})\n",
            session.model, session.provider
        ));
        md.push_str(&format!("**Workspace:** {}\n\n", session.workspace));

        if let Some(ref plan) = session.plan {
            md.push_str("## Implementation Plan\n\n");
            md.push_str(plan);
            md.push_str("\n\n");
        }

        if !session.tasks.is_empty() {
            md.push_str("## Tasks\n\n");
            for task in &session.tasks {
                md.push_str(&format!("- {} {}\n", task.status.icon(), task.description));
            }
            md.push('\n');
        }

        if !session.files_created.is_empty() {
            md.push_str("## Files Created\n\n");
            for file in &session.files_created {
                md.push_str(&format!("- `{}` ({})\n", file.path, file.action));
            }
            md.push('\n');
        }

        md.push_str("## Conversation\n\n");
        for msg in &session.messages {
            md.push_str(&format!(
                "**{}** *({})*:\n{}\n\n",
                msg.role,
                msg.timestamp.format("%H:%M:%S"),
                msg.content
            ));
        }

        md
    }
}

#[cfg(feature = "cli")]
impl Default for SessionManager {
    fn default() -> Self {
        Self::new(None)
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
