// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    super::main::render_logo,
    crate::tui::state::TuiState,
    crate::tui::theme::{ALL_THEME_VARIANTS, ThemePalette},
    ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Modifier, Style},
        symbols,
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    },
};

/// Renders the theme selection tab.
pub fn render_theme_tab(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let split = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    render_theme_selector(frame, split[0], state, palette);
    render_theme_preview(frame, split[1], state, palette);
}

/// Renders the left-side theme preset list with keyboard hints.
fn render_theme_selector(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Theme Presets",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let hint_area_h = 5;
    let list_area = Rect {
        height: inner.height.saturating_sub(hint_area_h),
        ..inner
    };
    let hint_area = Rect {
        y: inner.y + inner.height.saturating_sub(hint_area_h),
        height: hint_area_h,
        ..inner
    };

    let items: Vec<ListItem> = ALL_THEME_VARIANTS
        .iter()
        .enumerate()
        .map(|(i, variant)| {
            let is_selected = i == state.selected_theme_idx;
            let is_active = *variant == state.theme_config.variant;
            let name = variant.to_string();
            let preview_palette = ThemePalette::from_variant(*variant);
            let (pr, pg, pb) = preview_palette.logo_gradient[0];

            let style = if is_selected {
                Style::default()
                    .fg(palette.tab_active_fg)
                    .bg(palette.tab_active_bg)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(palette.fg)
            };

            let active_mark = if is_active { " ✓" } else { "  " };

            ListItem::new(Line::from(vec![
                Span::styled(
                    "  ● ",
                    Style::default().fg(ratatui::style::Color::Rgb(pr, pg, pb)),
                ),
                Span::styled(name, style),
                Span::styled(
                    active_mark,
                    Style::default().fg(palette.ok).add_modifier(Modifier::BOLD),
                ),
            ]))
        })
        .collect();

    frame.render_widget(List::new(items), list_area);

    let hints = Paragraph::new(vec![
        Line::from(Span::styled("", Style::default())),
        Line::from(Span::styled(
            "  ↑ ↓  navigate presets",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  Enter  apply & save",
            Style::default().fg(palette.muted),
        )),
        Line::from(Span::styled(
            "  r      reset to Dark",
            Style::default().fg(palette.muted),
        )),
    ])
    .wrap(Wrap { trim: false });

    frame.render_widget(hints, hint_area);
}

/// Renders the right-side theme preview panel.
///
/// Shows the logo rendered in the preview palette colors and a set of named
/// color swatches for immediate visual comparison.
fn render_theme_preview(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let preview_variant = ALL_THEME_VARIANTS[state.selected_theme_idx];
    let preview_palette = ThemePalette::from_variant(preview_variant);

    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                format!("Preview: {}", preview_variant),
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(8), Constraint::Fill(1)])
        .split(inner);

    let logo_lines = render_logo(&preview_palette);
    frame.render_widget(
        Paragraph::new(logo_lines).alignment(ratatui::layout::Alignment::Center),
        vertical[0],
    );

    let swatch_area = vertical[1];
    let swatches = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ])
        .split(swatch_area);

    let color_roles = [
        ("Accent", preview_palette.accent),
        ("OK", preview_palette.ok),
        ("Warn", preview_palette.warn),
        ("Err", preview_palette.err),
        ("Chart 1", preview_palette.chart_1),
        ("Chart 2", preview_palette.chart_2),
    ];

    for (i, (label, color)) in color_roles.iter().enumerate() {
        if i >= swatches.len() {
            break;
        }
        let swatch_block = Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(*color))
            .style(Style::default().bg(preview_palette.bg));
        let swatch_inner = swatch_block.inner(swatches[i]);
        frame.render_widget(swatch_block, swatches[i]);
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                label.to_string(),
                Style::default().fg(*color).add_modifier(Modifier::BOLD),
            )))
            .alignment(ratatui::layout::Alignment::Center),
            swatch_inner,
        );
    }
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
