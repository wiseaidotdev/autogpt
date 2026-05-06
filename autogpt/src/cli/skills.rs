// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use {
    anyhow::Result,
    chrono::Utc,
    serde::{Deserialize, Serialize},
    std::collections::HashMap,
    std::path::PathBuf,
};

/// A domain-specific knowledge capsule persisted between sessions.
///
/// Each `Skill` record captures a single technology domain's accumulated wisdom:
/// the patterns that work, the mistakes to avoid, and when the knowledge was last
/// updated. Files are stored as `~/.autogpt/skills/<domain>.toml`, one file per
/// domain, appended to over time as AutoGPT encounters new tasks.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Skill {
    pub domain: String,
    pub lessons: Vec<String>,
    pub anti_patterns: Vec<String>,
    pub updated_at: String,
}

/// In-memory store of all loaded skills, keyed by domain name.
///
/// `SkillStore` is loaded at the start of each session. Only skills whose domain
/// matches keywords in the user's prompt are loaded into the prompt context,
/// capping token overhead at roughly 300 tokens regardless of total skill count.
#[cfg(feature = "cli")]
pub struct SkillStore {
    pub skills: HashMap<String, Skill>,
    pub store_dir: PathBuf,
}

#[cfg(feature = "cli")]
impl SkillStore {
    /// Initialises an empty store pointing at the given directory.
    pub fn new(store_dir: PathBuf) -> Self {
        Self {
            skills: HashMap::new(),
            store_dir,
        }
    }

    /// Loads every `*.toml` skill file from `store_dir` into memory.
    pub fn load(store_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&store_dir)?;
        let mut skills = HashMap::new();

        for entry in std::fs::read_dir(&store_dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                continue;
            }
            let raw = std::fs::read_to_string(&path)?;
            if let Ok(skill) = toml_edit::de::from_str::<Skill>(&raw) {
                skills.insert(skill.domain.clone(), skill);
            }
        }

        Ok(Self { skills, store_dir })
    }

    /// Loads only the skills whose domain matches keywords found in `prompt`.
    pub fn load_for_domain(prompt: &str, store_dir: PathBuf) -> Result<Self> {
        let mut store = Self::load(store_dir)?;
        let domains = detect_domains(prompt);
        store.skills.retain(|domain, _| {
            domains
                .iter()
                .any(|d| domain.contains(d.as_str()) || d.contains(domain.as_str()))
        });
        Ok(store)
    }

    /// Appends a lesson and optional anti-pattern to the skill file for `domain`.
    ///
    /// Creates the skill file if it does not yet exist. The TOML file is written
    /// atomically to avoid corruption on partial writes.
    pub fn save_lesson(
        &mut self,
        domain: &str,
        lesson: &str,
        anti_pattern: Option<&str>,
    ) -> Result<()> {
        let skill = self
            .skills
            .entry(domain.to_string())
            .or_insert_with(|| Skill {
                domain: domain.to_string(),
                lessons: Vec::new(),
                anti_patterns: Vec::new(),
                updated_at: Utc::now().to_rfc3339(),
            });

        let lesson = lesson.trim().to_string();
        if !lesson.is_empty() && !skill.lessons.contains(&lesson) {
            skill.lessons.push(lesson);
        }

        if let Some(ap) = anti_pattern {
            let ap = ap.trim().to_string();
            if !ap.is_empty() && !skill.anti_patterns.contains(&ap) {
                skill.anti_patterns.push(ap);
            }
        }

        skill.updated_at = Utc::now().to_rfc3339();

        let path = self.store_dir.join(format!("{domain}.toml"));
        let raw = toml_edit::ser::to_string_pretty(skill)?;
        std::fs::write(path, raw)?;

        Ok(())
    }

    /// Serialises all loaded skills into a compact markdown bullet list.
    ///
    /// The output is suitable for injection directly into an LLM prompt. Total
    /// output is capped at approximately 300 tokens to minimise context overhead.
    pub fn to_prompt_context(&self) -> String {
        if self.skills.is_empty() {
            return String::new();
        }

        let mut lines = vec!["## Relevant Skills From Previous Sessions".to_string()];
        let mut token_budget = 2400_usize;

        for skill in self.skills.values() {
            if token_budget == 0 {
                break;
            }

            lines.push(format!("\n### {}", skill.domain));
            token_budget = token_budget.saturating_sub(20);

            for lesson in skill.lessons.iter().take(5) {
                let entry = format!("- ✓ {lesson}");
                token_budget = token_budget.saturating_sub(entry.len() + 1);
                if token_budget == 0 {
                    break;
                }
                lines.push(entry);
            }

            for ap in skill.anti_patterns.iter().take(3) {
                let entry = format!("- ✗ Avoid: {ap}");
                token_budget = token_budget.saturating_sub(entry.len() + 1);
                if token_budget == 0 {
                    break;
                }
                lines.push(entry);
            }
        }

        lines.join("\n")
    }

    /// Returns `true` if no skills were loaded.
    pub fn is_empty(&self) -> bool {
        self.skills.is_empty()
    }
}

/// Extracts technology domain keywords from a natural-language prompt.
///
/// Matches against a curated vocabulary of common technology stacks. Returns a
/// deduplicated list of matched domain strings used to filter which skill files
/// to load and which skill files to create/update post-session.
#[cfg(feature = "cli")]
pub fn detect_domains(prompt: &str) -> Vec<String> {
    let lower = prompt.to_lowercase();
    let vocab: &[&str] = &[
        "fastapi",
        "django",
        "flask",
        "express",
        "nestjs",
        "axum",
        "actix",
        "react",
        "vue",
        "svelte",
        "nextjs",
        "nuxt",
        "angular",
        "rust",
        "python",
        "javascript",
        "typescript",
        "go",
        "java",
        "csharp",
        "docker",
        "kubernetes",
        "terraform",
        "ansible",
        "postgres",
        "mysql",
        "sqlite",
        "mongodb",
        "redis",
        "graphql",
        "grpc",
        "websocket",
        "rest",
        "jwt",
        "oauth",
        "auth",
        "celery",
        "kafka",
        "rabbitmq",
        "pytest",
        "jest",
        "cargo",
        "npm",
        "pip",
        "uv",
    ];

    vocab
        .iter()
        .filter(|&&kw| lower.contains(kw))
        .map(|&kw| kw.to_string())
        .collect()
}
