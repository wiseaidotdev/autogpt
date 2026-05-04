// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use chrono::{DateTime, Utc};
#[cfg(feature = "cli")]
use serde::{Deserialize, Serialize};
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
        }
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
            None => dirs::home_dir()
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

        let json = serde_json::to_string_pretty(session)?;
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

        Ok(())
    }

    pub fn load(&self, session_id: &str) -> anyhow::Result<Session> {
        let path = self.session_dir(session_id).join("session.json");
        let content = fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
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
                && let Ok(session) = serde_json::from_str::<Session>(&content)
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

        entries.sort_by_key(|b| std::cmp::Reverse(b.updated_at));
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
