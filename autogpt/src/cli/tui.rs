// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use colored::Colorize;
#[cfg(feature = "cli")]
use indicatif::{ProgressBar, ProgressStyle};
#[cfg(feature = "cli")]
use std::time::Duration;
#[cfg(feature = "cli")]
use termimad::MadSkin;
#[cfg(feature = "cli")]
use tracing::{error, info, warn};

/// Terminal width used for box drawings.
#[cfg(feature = "cli")]
const BOX_WIDTH: usize = 80;

/// Available model entry for the model selector.
#[cfg(feature = "cli")]
#[derive(Debug, Clone)]
pub struct ModelEntry {
    pub id: String,
    pub display_name: String,
    pub description: String,
}

/// Status variants for task progress indicators.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Skipped,
}

/// Prints the AutoGPT gradient pixel-art ASCII banner.
///
/// Each column of the banner is coloured across a hot-pink → lavender → cyan gradient
/// using `colored`'s true-colour support, giving visual depth similar to the Gemini CLI.
#[cfg(feature = "cli")]
pub fn print_banner() {
    let logo_lines = [
        " █████╗ ██╗   ██╗████████╗ ██████╗  ██████╗ ██████╗ ████████╗",
        "██╔══██╗██║   ██║╚══██╔══╝██╔═══██╗██╔════╝ ██╔══██╗╚══██╔══╝",
        "███████║██║   ██║   ██║   ██║   ██║██║  ███╗██████╔╝   ██║   ",
        "██╔══██║██║   ██║   ██║   ██║   ██║██║   ██║██╔═══╝    ██║   ",
        "██║  ██║╚██████╔╝   ██║   ╚██████╔╝╚██████╔╝██║        ██║   ",
        "╚═╝  ╚═╝ ╚═════╝    ╚═╝    ╚═════╝  ╚═════╝ ╚═╝        ╚═╝   ",
    ];

    let gradient_stops: &[(u8, u8, u8)] = &[
        (255, 80, 180),
        (220, 110, 200),
        (180, 140, 230),
        (140, 170, 245),
        (100, 210, 250),
        (60, 230, 240),
    ];

    info!("");
    for (line_idx, line) in logo_lines.iter().enumerate() {
        let (r, g, b) = gradient_stops[line_idx % gradient_stops.len()];
        info!("{}", line.truecolor(r, g, b).bold());
    }
    info!("");
}

/// Prints the startup tips greeting block.
#[cfg(feature = "cli")]
pub fn print_greeting() {
    info!("{}", "Tips for getting started:".bold());
    info!(
        "  {}. Describe your project; AutoGPT will decompose and execute it autonomously.",
        "1".bright_cyan()
    );
    info!(
        "  {}. Review and approve the generated plan before execution begins.",
        "2".bright_cyan()
    );
    info!(
        "  {}. Use {} to list all available commands.",
        "3".bright_cyan(),
        "/help".bright_magenta().bold()
    );
    info!(
        "  {}. Use {} for unattended, continuous execution.",
        "4".bright_cyan(),
        "-y / --yolo".bright_yellow()
    );
    info!(
        "  {}. Press {} during execution to interrupt!",
        "5".bright_cyan(),
        "ESC".yellow().bold()
    );
    info!("");
}

/// Renders a yellow warning box to the terminal.
///
/// Used for home-directory warnings, update notifications, and other advisory messages.
/// All output is routed through `tracing::warn!`.
#[cfg(feature = "cli")]
pub fn render_warning_box(message: &str) {
    let inner_width = BOX_WIDTH - 2;
    let top = format!("╭{}╮", "─".repeat(inner_width));
    let bot = format!("╰{}╯", "─".repeat(inner_width));

    warn!("{}", top.bright_yellow());
    for line in message.lines() {
        let padded = format!("│ {:<width$} │", line, width = inner_width - 2);
        warn!("{}", padded.bright_yellow());
    }
    warn!("{}", bot.bright_yellow());
    warn!("");
}

/// Renders a version-update banner in yellow.
#[cfg(feature = "cli")]
pub fn render_update_banner(current: &str, latest: &str) {
    let msg = format!(
        "AutoGPT update available! {} → {}\nRun `cargo install autogpt` to update.",
        current, latest
    );
    render_warning_box(&msg);
}

/// Renders the bordered input prompt box hint.
///
/// This is printed above the `>` prompt to visually delimit the user input area.
#[cfg(feature = "cli")]
pub fn render_input_box_hint() {
    let inner_width = BOX_WIDTH - 2;
    let top = format!("╭{}╮", "─".repeat(inner_width));
    info!("{}", top.bright_blue());
}

/// Renders the help table for all available slash commands.
#[cfg(feature = "cli")]
pub fn render_help_table() {
    info!("");
    info!("{}", "Available Commands".bright_cyan().bold());
    info!("{}", "─".repeat(BOX_WIDTH).bright_black());

    let commands: &[(&str, &str)] = &[
        (
            "/help",
            "Display this help table with all available commands",
        ),
        (
            "/sessions",
            "List and interactively resume a previous session",
        ),
        (
            "/models",
            "List available models for the current provider and switch",
        ),
        ("/clear", "Clear the screen and reprint the AutoGPT banner"),
        (
            "/status",
            "Show current session info, task progress, and model",
        ),
        ("/workspace", "Display the current workspace directory path"),
        ("/provider", "Show and switch the active LLM provider"),
        ("exit / quit", "Save the current session and exit AutoGPT"),
    ];

    for (cmd, desc) in commands {
        info!("  {:<15}  {}", cmd.bright_magenta().bold(), desc.white());
    }

    info!("{}", "─".repeat(BOX_WIDTH).bright_black());
    info!("");
}

/// Renders an interactive model selector and returns the zero-based index of the chosen model.
///
/// Uses `dialoguer::Select` for a terminal-native selection UI with keyboard navigation.
#[cfg(feature = "cli")]
pub fn render_model_selector(models: &[ModelEntry], current_idx: usize) -> usize {
    use dialoguer::{Select, theme::ColorfulTheme};

    info!("");
    info!("{}", "Select Model".bright_cyan().bold());
    info!("{}", "─".repeat(BOX_WIDTH).bright_black());

    let labels: Vec<String> = models
        .iter()
        .enumerate()
        .map(|(i, m)| {
            if i == current_idx {
                format!("● {} - {}", m.display_name, m.description)
            } else {
                format!("  {} - {}", m.display_name, m.description)
            }
        })
        .collect();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .items(&labels)
        .default(current_idx)
        .interact()
        .unwrap_or(current_idx);

    info!("{}", "─".repeat(BOX_WIDTH).bright_black());
    selection
}

/// Renders markdown content using `termimad`, applying syntax colour to code blocks.
///
/// `termimad` writes directly to stdout - this is intentional for rich terminal output
/// that cannot be easily proxied through `tracing`.
#[cfg(feature = "cli")]
pub fn render_markdown(content: &str) {
    let mut skin = MadSkin::default();
    skin.code_block
        .set_fg(termimad::crossterm::style::Color::Cyan);
    skin.inline_code
        .set_fg(termimad::crossterm::style::Color::Cyan);
    skin.bold.set_fg(termimad::crossterm::style::Color::White);
    skin.print_text(content);
}

/// Creates and starts a themed spinner with autogpt-inspired tick animation.
#[cfg(feature = "cli")]
pub fn create_spinner(message: &str) -> ProgressBar {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::with_template("{spinner:.magenta} {msg:.cyan}")
            .unwrap()
            .tick_strings(&[
                "⟳ synthesizing ",
                "⟲ processing  ",
                "↻ computing   ",
                "↺ reasoning   ",
                "⟳ executing   ",
                "⟲ reflecting  ",
                "↻ verifying   ",
                "↺ finalizing  ",
            ]),
    );
    spinner.set_message(message.to_string());
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner
}

/// Logs a task item with a coloured status icon via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_task_item(description: &str, status: TaskStatus) {
    let (icon, coloured) = match status {
        TaskStatus::Pending => ("○", format!("○ {}", description).bright_black().to_string()),
        TaskStatus::InProgress => (
            "●",
            format!("● {}", description)
                .bright_yellow()
                .bold()
                .to_string(),
        ),
        TaskStatus::Completed => (
            "✓",
            format!("✓ {}", description)
                .bright_green()
                .bold()
                .to_string(),
        ),
        TaskStatus::Failed => (
            "✗",
            format!("✗ {}", description).bright_red().bold().to_string(),
        ),
        TaskStatus::Skipped => ("⊘", format!("⊘ {}", description).bright_black().to_string()),
    };
    let _ = icon;
    info!("  {}", coloured);
}

/// Logs a section header with a coloured divider via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_section(title: &str) {
    info!("");
    info!("{} {}", "▸".bright_cyan(), title.bold().white());
}

/// Logs an agent-tagged message via `tracing::info!`.
#[cfg(feature = "cli")]
pub fn print_agent_msg(tag: &str, message: &str) {
    info!(
        "{} {}",
        format!("[AutoGPT·{}]", tag).bright_magenta().bold(),
        message.white()
    );
}

/// Logs a warning message via `tracing::warn!` with yellow colouring.
#[cfg(feature = "cli")]
pub fn print_warning(message: &str) {
    warn!("{}  {}", "⚠".bright_yellow(), message.bright_yellow());
}

/// Logs an error message via `tracing::error!` with red colouring.
#[cfg(feature = "cli")]
pub fn print_error(message: &str) {
    error!("{}  {}", "✗".bright_red(), message.bright_red());
}

/// Logs a success message via `tracing::info!` with green colouring.
#[cfg(feature = "cli")]
pub fn print_success(message: &str) {
    info!("{}  {}", "✓".bright_green(), message.bright_green().bold());
}

/// Renders a single labelled status bar line showing working context.
#[cfg(feature = "cli")]
pub fn print_status_bar(cwd: &str, model: &str, provider: &str) {
    info!(
        "  {}  {}  {}",
        cwd.bright_blue(),
        model.bright_magenta(),
        provider.bright_black()
    );
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
