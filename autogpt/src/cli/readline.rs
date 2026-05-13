// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#[cfg(feature = "cli")]
use std::io::{self, Write};

#[cfg(feature = "cli")]
use termimad::crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType, disable_raw_mode, enable_raw_mode},
};

/// The canonical slash commands available in the interactive loop.
#[cfg(feature = "cli")]
pub const SLASH_COMMANDS: &[&str] = &[
    "/clear",
    "/help",
    "/mcp",
    "/mcp inspect",
    "/mcp list",
    "/mcp remove",
    "/models",
    "/provider",
    "/sessions",
    "/status",
    "/workspace",
];

/// The result returned by the `read_line` function.
#[cfg(feature = "cli")]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReadlineResult {
    /// The user submitted a line of text.
    Submit(String),
    /// The user interrupted the input (e.g., Ctrl+C).
    Interrupted,
    /// An error occurred during input.
    Error(String),
}

#[cfg(feature = "cli")]
const INPUT_LEFT_MARGIN: u16 = 4;
#[cfg(feature = "cli")]
const HEADER_ROWS: u16 = 3;
#[cfg(feature = "cli")]
const COMPLETION_COLUMN_WIDTH: usize = 22;

/// Returns every command that starts with `prefix`, sorted alphabetically.
#[cfg(feature = "cli")]
fn matching_commands<'a>(prefix: &str, commands: &[&'a str]) -> Vec<&'a str> {
    let mut m: Vec<&str> = commands
        .iter()
        .copied()
        .filter(|c| c.starts_with(prefix))
        .collect();
    m.sort_unstable();
    m
}

/// Truncates `s` to at most `max_visible` printable characters, preserving ANSI escapes.
/// This guarantees the status line occupies exactly one terminal row.
#[cfg(feature = "cli")]
fn visible_truncate(s: &str, max_visible: usize) -> String {
    let mut out = String::with_capacity(s.len());
    let mut visible = 0usize;
    let mut in_esc = false;
    for ch in s.chars() {
        if ch == '\x1b' {
            in_esc = true;
            out.push(ch);
        } else if in_esc {
            out.push(ch);
            if ch.is_ascii_alphabetic() {
                in_esc = false;
            }
        } else {
            if visible >= max_visible {
                break;
            }
            out.push(ch);
            visible += 1;
        }
    }
    out
}

/// Returns the number of rows needed to display `n` candidates in the terminal.
#[cfg(feature = "cli")]
fn grid_rows(n: usize, w: u16) -> u16 {
    let cols = (w as usize).saturating_div(COMPLETION_COLUMN_WIDTH).max(1);
    n.div_ceil(cols) as u16
}

/// Renders a completion candidate grid starting on the current line.
#[cfg(feature = "cli")]
fn render_grid(candidates: &[&str], sel: usize, w: u16, out: &mut impl Write) -> io::Result<()> {
    let cols = (w as usize).saturating_div(COMPLETION_COLUMN_WIDTH).max(1);
    let mut col = 0;
    for (i, cmd) in candidates.iter().enumerate() {
        execute!(
            out,
            SetForegroundColor(if i == sel {
                Color::White
            } else {
                Color::DarkGrey
            }),
            Print(format!("{:<COMPLETION_COLUMN_WIDTH$}", cmd)),
            ResetColor
        )?;
        col += 1;
        if col >= cols {
            execute!(out, Print("\r\n"), terminal::Clear(ClearType::UntilNewLine))?;
            col = 0;
        }
    }
    if col != 0 {
        execute!(out, Print("\r\n"))?;
    }
    Ok(())
}

/// Erases `n` rows below the current cursor position, then moves the cursor back up.
#[cfg(feature = "cli")]
fn erase_below(n: u16, out: &mut impl Write) -> io::Result<()> {
    for _ in 0..n {
        execute!(
            out,
            cursor::MoveDown(1),
            terminal::Clear(ClearType::CurrentLine)
        )?;
    }
    if n > 0 {
        execute!(out, cursor::MoveUp(n))?;
    }
    Ok(())
}

/// Redraws only the input row content without touching any other row.
#[cfg(feature = "cli")]
fn draw_input_row(
    buf: &str,
    cursor_pos: usize,
    ghost: Option<&str>,
    w: u16,
    out: &mut impl Write,
) -> io::Result<()> {
    let inner = w.saturating_sub(2) as usize;
    let avail = inner.saturating_sub(4);
    let buf_vis: String = buf.chars().take(avail).collect();
    let ghost_vis: String = ghost
        .unwrap_or("")
        .chars()
        .take(avail.saturating_sub(buf_vis.chars().count()))
        .collect();
    let content_len = 3 + buf_vis.chars().count() + ghost_vis.chars().count();
    let row_pad = inner.saturating_sub(content_len + 1);

    write!(out, "\r")?;
    execute!(out, SetForegroundColor(Color::Blue), Print("│"), ResetColor)?;
    write!(out, " > {}", buf_vis)?;
    if !ghost_vis.is_empty() {
        execute!(
            out,
            SetForegroundColor(Color::DarkGrey),
            Print(&ghost_vis),
            ResetColor
        )?;
    }
    write!(out, "{}", " ".repeat(row_pad))?;
    execute!(
        out,
        SetForegroundColor(Color::Blue),
        Print(" │"),
        ResetColor
    )?;

    let col = (INPUT_LEFT_MARGIN + cursor_pos.min(avail) as u16).min(w.saturating_sub(1));
    execute!(out, cursor::MoveToColumn(col))?;
    out.flush()?;
    Ok(())
}

/// Draws the full bordered box and leaves the cursor on the input row.
#[cfg(feature = "cli")]
fn draw_box(
    status: &str,
    hint: &str,
    buf: &str,
    cursor_pos: usize,
    ghost: Option<&str>,
    w: u16,
    out: &mut impl Write,
) -> io::Result<()> {
    let inner = w.saturating_sub(2) as usize;
    write!(out, "\r{}\r\n", visible_truncate(status, w as usize))?;
    execute!(
        out,
        SetForegroundColor(Color::Blue),
        Print(format!("╭{}╮\r\n", "─".repeat(inner))),
        ResetColor
    )?;
    let hint_vis: String = hint.chars().take(inner.saturating_sub(3)).collect();
    let hint_pad = inner.saturating_sub(2 + hint_vis.chars().count() + 1);
    execute!(
        out,
        SetForegroundColor(Color::Blue),
        Print("│"),
        ResetColor,
        SetForegroundColor(Color::DarkGrey),
        Print(format!("  {}{}", hint_vis, " ".repeat(hint_pad))),
        ResetColor,
        SetForegroundColor(Color::Blue),
        Print(" │\r\n"),
        ResetColor
    )?;
    draw_input_row(buf, cursor_pos, ghost, w, out)?;
    write!(out, "\r\n")?;
    execute!(
        out,
        SetForegroundColor(Color::Blue),
        Print(format!("╰{}╯", "─".repeat(inner))),
        ResetColor
    )?;
    execute!(out, Print("\r"), cursor::MoveUp(1))?;
    let avail = inner.saturating_sub(4);
    let col = (INPUT_LEFT_MARGIN + cursor_pos.min(avail) as u16).min(w.saturating_sub(1));
    execute!(out, cursor::MoveToColumn(col))?;
    out.flush()?;
    Ok(())
}

/// Returns the cursor position moved one word to the left.
#[cfg(feature = "cli")]
fn word_left(buf: &str, pos: usize) -> usize {
    let chars: Vec<char> = buf.chars().collect();
    let mut i = pos;
    while i > 0 && chars[i - 1].is_whitespace() {
        i -= 1;
    }
    while i > 0 && !chars[i - 1].is_whitespace() {
        i -= 1;
    }
    i
}

/// Returns the cursor position moved one word to the right.
#[cfg(feature = "cli")]
fn word_right(buf: &str, pos: usize) -> usize {
    let chars: Vec<char> = buf.chars().collect();
    let len = chars.len();
    let mut i = pos;
    while i < len && chars[i].is_whitespace() {
        i += 1;
    }
    while i < len && !chars[i].is_whitespace() {
        i += 1;
    }
    i
}

/// Moves the cursor past the bottom border of the input box and resets to column 0.
#[cfg(feature = "cli")]
fn exit_box(grid_lines: u16, out: &mut impl Write) -> io::Result<()> {
    erase_below(grid_lines, out)?;
    execute!(out, cursor::MoveDown(1))?;
    write!(out, "\r\n")?;
    execute!(out, cursor::MoveToColumn(0))?;
    out.flush()?;
    Ok(())
}

/// Reads one line of user input inside a fully-rendered, resize-aware bordered box.
#[cfg(feature = "cli")]
pub fn read_line(
    status_line: &str,
    hint: &str,
    commands: &[&str],
    history: &[String],
) -> ReadlineResult {
    let mut out = io::stdout();
    let (mut term_width, _) = terminal::size().unwrap_or((80, 24));
    if let Err(e) = writeln!(out) {
        return ReadlineResult::Error(e.to_string());
    }
    if let Err(e) = draw_box(status_line, hint, "", 0, None, term_width, &mut out) {
        return ReadlineResult::Error(e.to_string());
    }
    if let Err(e) = enable_raw_mode() {
        return ReadlineResult::Error(e.to_string());
    }
    let mut buf = String::new();
    let mut cursor_pos: usize = 0;
    let mut tab_idx: Option<usize> = None;
    let mut grid_lines: u16 = 0;
    let mut hist_idx: Option<usize> = None;
    let final_result = loop {
        let matches: Vec<&str> = if buf.starts_with('/') {
            matching_commands(&buf, commands)
        } else {
            vec![]
        };
        let ghost: Option<String> = {
            let idx = tab_idx.unwrap_or(0);
            if !matches.is_empty()
                && cursor_pos == buf.chars().count()
                && matches[idx].len() > buf.len()
            {
                Some(matches[idx][buf.len()..].to_string())
            } else {
                None
            }
        };
        if let Err(e) = draw_input_row(&buf, cursor_pos, ghost.as_deref(), term_width, &mut out) {
            break ReadlineResult::Error(e.to_string());
        }
        match event::read() {
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Enter,
                ..
            })) => {
                let val = if let Some(idx) = tab_idx {
                    matches
                        .get(idx)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| buf.clone())
                } else {
                    buf.clone()
                };
                if let Err(e) = exit_box(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                break ReadlineResult::Submit(val);
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                if let Err(e) = exit_box(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                break ReadlineResult::Interrupted;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Tab, ..
            })) => {
                if matches.is_empty() {
                    let bidx = buf
                        .char_indices()
                        .nth(cursor_pos)
                        .map(|(i, _)| i)
                        .unwrap_or(buf.len());
                    buf.insert_str(bidx, "  ");
                    cursor_pos += 2;
                    continue;
                }
                let new_idx = tab_idx.map(|i| (i + 1) % matches.len()).unwrap_or(0);
                tab_idx = Some(new_idx);
                buf = matches[new_idx].to_string();
                cursor_pos = buf.chars().count();
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                let rows = grid_rows(matches.len(), term_width);
                if let Err(e) = execute!(out, cursor::SavePosition, cursor::MoveDown(2)) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = write!(out, "\r\n") {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = render_grid(&matches, new_idx, term_width, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = execute!(out, cursor::RestorePosition) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = rows + 1;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::BackTab,
                ..
            })) => {
                if matches.is_empty() {
                    continue;
                }
                let len = matches.len();
                let new_idx = tab_idx
                    .map(|i| if i == 0 { len - 1 } else { i - 1 })
                    .unwrap_or(len - 1);
                tab_idx = Some(new_idx);
                buf = matches[new_idx].to_string();
                cursor_pos = buf.chars().count();
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                let rows = grid_rows(matches.len(), term_width);
                if let Err(e) = execute!(out, cursor::SavePosition, cursor::MoveDown(2)) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = write!(out, "\r\n") {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = render_grid(&matches, new_idx, term_width, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = execute!(out, cursor::RestorePosition) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = rows + 1;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::NONE,
                ..
            })) => {
                cursor_pos = cursor_pos.saturating_sub(1);
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::NONE,
                ..
            })) => {
                let len = buf.chars().count();
                if cursor_pos < len {
                    cursor_pos += 1;
                } else if let Some(ref sfx) = ghost {
                    buf.push_str(sfx);
                    cursor_pos = buf.chars().count();
                    tab_idx = None;
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Left,
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                cursor_pos = word_left(&buf, cursor_pos);
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Right,
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                cursor_pos = word_right(&buf, cursor_pos);
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Home,
                ..
            })) => {
                cursor_pos = 0;
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::End, ..
            })) => {
                cursor_pos = buf.chars().count();
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Backspace,
                ..
            })) => {
                if cursor_pos > 0 {
                    let bidx = buf
                        .char_indices()
                        .nth(cursor_pos - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    buf.remove(bidx);
                    cursor_pos -= 1;
                }
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Delete,
                ..
            })) => {
                if cursor_pos < buf.chars().count() {
                    let bidx = buf
                        .char_indices()
                        .nth(cursor_pos)
                        .map(|(i, _)| i)
                        .unwrap_or(buf.len());
                    buf.remove(bidx);
                }
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('w'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                let np = word_left(&buf, cursor_pos);
                let chars: Vec<char> = buf.chars().collect();
                buf = format!(
                    "{}{}",
                    chars[..np].iter().collect::<String>(),
                    chars[cursor_pos..].iter().collect::<String>()
                );
                cursor_pos = np;
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('k'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                let bidx = buf
                    .char_indices()
                    .nth(cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(buf.len());
                buf.truncate(bidx);
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char('u'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })) => {
                buf = buf.chars().skip(cursor_pos).collect();
                cursor_pos = 0;
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Esc, ..
            })) => {
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
                buf.clear();
                cursor_pos = 0;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Up, ..
            })) => {
                if history.is_empty() {
                    continue;
                }
                let ni = match hist_idx {
                    None => history.len() - 1,
                    Some(0) => 0,
                    Some(i) => i - 1,
                };
                hist_idx = Some(ni);
                buf = history[ni].clone();
                cursor_pos = buf.chars().count();
                tab_idx = None;
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Down,
                ..
            })) => {
                if let Some(idx) = hist_idx {
                    if idx + 1 < history.len() {
                        hist_idx = Some(idx + 1);
                        buf = history[idx + 1].clone();
                    } else {
                        hist_idx = None;
                        buf.clear();
                    }
                    cursor_pos = buf.chars().count();
                }
                tab_idx = None;
            }
            Ok(Event::Resize(new_w, _)) => {
                term_width = new_w;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
                if let Err(e) = execute!(out, Print("\r"), cursor::MoveUp(HEADER_ROWS)) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = execute!(out, terminal::Clear(ClearType::FromCursorDown)) {
                    break ReadlineResult::Error(e.to_string());
                }
                if let Err(e) = draw_box(
                    status_line,
                    hint,
                    &buf,
                    cursor_pos,
                    ghost.as_deref(),
                    new_w,
                    &mut out,
                ) {
                    break ReadlineResult::Error(e.to_string());
                }
            }
            Ok(Event::Key(KeyEvent {
                code: KeyCode::Char(c),
                modifiers,
                ..
            })) if modifiers == KeyModifiers::NONE || modifiers == KeyModifiers::SHIFT => {
                tab_idx = None;
                if let Err(e) = erase_below(grid_lines, &mut out) {
                    break ReadlineResult::Error(e.to_string());
                }
                grid_lines = 0;
                let bidx = buf
                    .char_indices()
                    .nth(cursor_pos)
                    .map(|(i, _)| i)
                    .unwrap_or(buf.len());
                buf.insert(bidx, c);
                cursor_pos += 1;
            }
            Err(e) => {
                break ReadlineResult::Error(e.to_string());
            }
            _ => {}
        }
    };
    let _ = disable_raw_mode();
    let _ = out.flush();
    final_result
}
