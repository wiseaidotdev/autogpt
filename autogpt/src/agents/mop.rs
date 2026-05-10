// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! # Mixture of Providers (MoP)
//!
//! Implements the **Mixture of Providers** architecture, a strategy inspired by
//! Mixture of Experts in deep learning, applied to AI API providers.
//!
//! Each user prompt is fanned out in parallel to every configured provider. A
//! judge function scores all responses and selects the highest-quality one,
//! maximising accuracy, reliability, and cross-model diversity.
//!
//! ## Feature Gate
//!
//! Compiled only when the `mop` cargo feature flag is enabled:
//!
//! ```sh
//! cargo build --features "cli,gem,oai,mop"
//! ```
//!
//! ## Entry Point
//!
//! Use [`run_generic_agent_loop_mop`] as a drop-in replacement for the standard
//! interactive loop when `--mixture` is passed on the command line.

#![cfg(feature = "mop")]

use crate::agents::agent::AgentGPT;
use crate::common::utils::{
    Capability, ClientType, ContextManager, Knowledge, Persona, Planner, Reflection, Status, Task,
    TaskScheduler, Tool,
};
#[allow(unused_imports)]
#[cfg(feature = "hf")]
use crate::prelude::hf_model_from_str;
use crate::prelude::*;
use crate::traits::agent::Agent;
use anyhow::Result;
use async_trait::async_trait;
use auto_derive::Auto;
use colored::Colorize;
use futures::future::join_all;
use std::borrow::Cow;
use std::env::var;
use tracing::{info, warn};

#[cfg(feature = "net")]
use crate::collaboration::Collaborator;

use crate::traits::functions::{AsyncFunctions, Functions};

/// A thin agent wrapper used to fan a single prompt to one specific provider.
///
/// The `Auto` derive macro provides the full `Agent`, `Functions`, and
/// `AsyncFunctions` implementation including `generate()`, so no provider-
/// specific streaming code needs to be duplicated here.
#[derive(Debug, Default, Clone, Auto)]
pub struct MopAgent {
    pub agent: AgentGPT,
    pub client: ClientType,
}

#[async_trait]
impl Executor for MopAgent {
    /// No-op execution path, `MopAgent` is only used for `generate()` calls,
    /// not for the full agentic task pipeline.
    async fn execute<'a>(
        &'a mut self,
        _task: &'a mut Task,
        _execute: bool,
        _browse: bool,
        _max_tries: u64,
    ) -> Result<()> {
        Ok(())
    }
}

/// Detects whether a response is a refusal or error message.
///
/// Returns `true` for common refusal patterns ("As an AI...", "I cannot..."),
/// API error strings, and empty / whitespace-only content.
fn is_refusal_or_error(text: &str) -> bool {
    let t = text.trim().to_lowercase();
    if t.is_empty() {
        return true;
    }
    let refusal_prefixes = [
        "as an ai",
        "as a language model",
        "i cannot",
        "i can't",
        "i'm unable",
        "i am unable",
        "i won't",
        "i will not",
        "sorry, i",
        "i'm sorry",
        "i apologize",
        "error:",
        "error occurred",
        "an error",
        "unfortunately, i",
        "i don't have the ability",
    ];

    let is_refusal = refusal_prefixes
        .iter()
        .any(|p| t.starts_with(p) || t.contains(p));

    if is_refusal {
        let has_code = t.contains("```");
        let has_structure = t.contains('#') || t.contains("- ") || t.contains("* ");
        if !has_code && !has_structure && t.len() < 200 {
            return true;
        }
    }

    false
}

/// Scores the length of a response using a calibrated bell-curve-like reward.
///
/// Very short responses (< 50 chars) score near zero. Responses in the
/// 300-1500 char sweet-spot score at their full weight. Responses beyond
/// 3000 chars are capped to prevent verbosity from dominating.
fn score_length(text: &str) -> i64 {
    let len = text.trim().len();
    match len {
        0 => 0,
        1..=299 => 100 + (len as i64) * 2,
        300..=1499 => 700 + (len as i64 - 300),
        1500..=3000 => 1900 + (len as i64 - 1500) / 2,
        _ => 2650,
    }
}

/// Scores code quality within a response.
///
/// Awards points per fenced code block, with an additional bonus for blocks
/// that include a language specifier (e.g. ` ```rust `) indicating the model
/// understood the domain. Multiple relevant blocks compound the score.
fn score_code_quality(text: &str) -> i64 {
    let mut score: i64 = 0;
    let mut in_block = false;

    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            if !in_block {
                in_block = true;
                score += 120;
                let lang = trimmed.trim_start_matches('`').trim();
                if !lang.is_empty() {
                    score += 80;
                }
            } else {
                in_block = false;
            }
        }
    }
    score.min(800)
}

/// Scores structural richness: headings, lists, emphasis, and paragraph depth.
///
/// Well-structured responses that segment ideas with markdown formatting
/// demonstrate better reasoning and readability, and score higher.
fn score_structure(text: &str) -> i64 {
    let mut score: i64 = 0;
    let mut paragraph_count: usize = 0;
    let mut prev_blank = true;

    for line in text.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            prev_blank = true;
            continue;
        }

        if prev_blank {
            paragraph_count += 1;
        }
        prev_blank = false;

        if trimmed.starts_with('#') {
            score += 60;
        }

        if trimmed.starts_with("- ") || trimmed.starts_with("* ") || trimmed.starts_with("+ ") {
            score += 20;
        }

        if trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && trimmed.contains(". ")
        {
            score += 20;
        }

        if trimmed.contains("**") || trimmed.contains("__") {
            score += 10;
        }

        if trimmed.contains('`') && !trimmed.starts_with("```") {
            score += 10;
        }
    }

    if text.trim().len() > 20 {
        score += (paragraph_count.min(8) as i64) * 30;
    }
    score.min(600)
}

/// Scores response completeness and linguistic hygiene.
///
/// Rewards responses that end with proper punctuation or a closing code block,
/// and penalises common AI filler phrases ("Certainly!", "Of course!",
/// "Great question!") that add no informational value.
fn score_completeness(text: &str) -> i64 {
    let trimmed = text.trim();
    let mut score: i64 = 0;

    let last_char = trimmed.chars().last().unwrap_or(' ');
    if (matches!(last_char, '.' | '!' | '?' | ':' | '`') || trimmed.ends_with("```"))
        && trimmed.len() > 5
    {
        score += 100;
    }

    let lower = trimmed.to_lowercase();
    let filler_phrases = [
        "certainly!",
        "of course!",
        "great question",
        "sure!",
        "absolutely!",
        "i'd be happy to",
        "i'd be glad to",
        "i hope this helps",
        "let me know if you",
        "feel free to ask",
    ];
    for phrase in filler_phrases {
        if lower.contains(phrase) {
            score -= 50;
        }
    }

    if trimmed.ends_with("...") || trimmed.ends_with("[...]") {
        score -= 150;
    }

    score
}

/// Scores evidence of logical reasoning and explanatory depth.
///
/// Responses that use connective language ("because", "therefore", "which
/// means") or explicitly enumerate steps demonstrate structured thinking and
/// are scored higher.
fn score_reasoning_depth(text: &str) -> i64 {
    let lower = text.to_lowercase();
    let mut score: i64 = 0;

    let reasoning_signals = [
        "because",
        "therefore",
        "thus",
        "hence",
        "as a result",
        "which means",
        "this ensures",
        "this allows",
        "note that",
        "importantly",
        "however",
        "alternatively",
        "instead",
        "step 1",
        "step 2",
        "first,",
        "second,",
        "finally,",
        "in summary",
        "to summarize",
        "in conclusion",
    ];

    for signal in reasoning_signals {
        if lower.contains(signal) {
            score += 25;
        }
    }

    score.min(400)
}

/// Scores repetitive text to penalise loops.
fn score_repetition(text: &str) -> i64 {
    let lines: Vec<&str> = text
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();
    if lines.len() < 3 {
        return 0;
    }
    let mut duplicates = 0;
    let mut seen = std::collections::HashSet::new();
    for line in &lines {
        if !seen.insert(line) {
            duplicates += 1;
        }
    }
    let ratio = (duplicates as f64) / (lines.len() as f64);
    if ratio > 0.3 {
        -((ratio * 1000.0) as i64)
    } else {
        0
    }
}

/// Scores valid JSON to reward structured outputs.
fn score_json(text: &str) -> i64 {
    let trimmed = text.trim();
    if ((trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']')))
        && serde_json::from_str::<serde_json::Value>(trimmed).is_ok()
    {
        return 200;
    }
    0
}

/// Assigns an overall quality score to a single LLM response.
///
/// Combines five independent sub-scores with different weights reflecting
/// their relative importance in real-world agentic tasks:
///
/// | Signal | Max weight |
/// |---|---|
/// | Length calibration | 2 250 |
/// | Code quality | 800 |
/// | Structural richness | 600 |
/// | Reasoning depth | 400 |
/// | Completeness / hygiene | 100+ |
///
/// Returns `-1` immediately for refusals and error responses so they are
/// excluded by the `judge` filter before any sub-scoring occurs.
fn score_response(response: &str) -> i64 {
    if is_refusal_or_error(response) {
        return -1;
    }

    score_length(response)
        + score_code_quality(response)
        + score_structure(response)
        + score_reasoning_depth(response)
        + score_completeness(response)
        + score_repetition(response)
        + score_json(response)
}

/// Evaluates provider responses and returns the one with the highest score.
///
/// Returns a tuple of `(provider_name, response_text)`. If all responses are
/// unusable, returns `None`.
pub fn judge(responses: Vec<(String, String)>) -> Option<(String, String)> {
    responses
        .into_iter()
        .map(|(provider, resp)| (score_response(&resp), provider, resp))
        .filter(|(score, _, _)| *score >= 0)
        .max_by(|(s1, _, _), (s2, _, _)| {
            if s1 == s2 {
                std::cmp::Ordering::Less
            } else {
                s1.cmp(s2)
            }
        })
        .map(|(_, provider, resp)| (provider, resp))
}

/// Returns the names of all providers that have credentials configured in the
/// current environment and matching feature flags compiled in.
pub fn configured_providers() -> Vec<String> {
    let mut providers = Vec::new();

    #[cfg(feature = "gem")]
    if var("GEMINI_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        providers.push("gemini".to_string());
    }

    #[cfg(feature = "oai")]
    if var("OPENAI_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        providers.push("openai".to_string());
    }

    #[cfg(feature = "cld")]
    if var("ANTHROPIC_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        providers.push("anthropic".to_string());
    }

    #[cfg(feature = "xai")]
    if var("XAI_API_KEY").map(|k| !k.is_empty()).unwrap_or(false) {
        providers.push("xai".to_string());
    }

    #[cfg(feature = "co")]
    if var("COHERE_API_KEY")
        .map(|k| !k.is_empty())
        .unwrap_or(false)
    {
        providers.push("cohere".to_string());
    }

    providers
}

/// Fans `prompt` out to all configured providers concurrently via [`MopAgent`]
/// instances and returns the highest-scored `(provider_name, response)` pair.
///
/// Each `MopAgent` is constructed with its `AI_PROVIDER` env-var set
/// transiently so that `ClientType::from_env()` picks the right backend.
/// On error the provider is silently skipped. Returns `None` only when every
/// provider fails.
pub async fn run_mixture(prompt: &str) -> Option<(String, String)> {
    let providers = configured_providers();

    if providers.is_empty() {
        warn!(
            "{}",
            "MoP: no providers configured, falling back to default."
                .bright_yellow()
                .bold()
        );
        return None;
    }

    info!(
        "{}",
        format!("MoP: fanning out to {} provider(s)...", providers.len())
            .bright_cyan()
            .bold()
    );

    let futs: Vec<_> = providers
        .iter()
        .map(|provider_name| {
            let p = provider_name.clone();
            let prompt_owned = prompt.to_string();
            async move {
                let original = var("AI_PROVIDER").unwrap_or_default();
                unsafe { std::env::set_var("AI_PROVIDER", &p) };
                let mut agent = MopAgent {
                    client: ClientType::from_env(),
                    ..Default::default()
                };
                unsafe { std::env::set_var("AI_PROVIDER", &original) };
                let response = agent.generate(&prompt_owned).await.unwrap_or_default();
                (p, response)
            }
        })
        .collect();

    let results: Vec<(String, String)> = join_all(futs).await;

    for (provider, resp) in &results {
        let preview: String = resp.chars().take(80).collect();
        info!(
            "  {} score={} preview={}...",
            format!("[{}]", provider).bright_magenta(),
            score_response(resp),
            preview.bright_black()
        );
    }

    judge(results)
}
