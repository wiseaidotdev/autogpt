// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use {
    crate::tui::state::TuiState,
    crate::tui::theme::ThemePalette,
    ratatui::{
        Frame,
        layout::{Constraint, Direction, Layout, Rect},
        style::{Modifier, Style},
        symbols,
        text::{Line, Span},
        widgets::{
            BarChart, Block, Borders, Chart, Dataset, GraphType, Paragraph, Sparkline, Wrap,
        },
    },
};

/// Renders the analytics tab with performance charts and stats.
pub fn render_analytics_tab(
    frame: &mut Frame,
    area: Rect,
    state: &TuiState,
    palette: &ThemePalette,
) {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(45),
            Constraint::Percentage(35),
            Constraint::Percentage(20),
        ])
        .split(area);

    render_token_chart(frame, vertical[0], state, palette);

    let bottom_row = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(vertical[1]);

    render_requests_bar_chart(frame, bottom_row[0], state, palette);
    render_kpi_panel(frame, bottom_row[1], state, palette);
    render_sparkline_panel(frame, vertical[2], state, palette);
}

/// Renders the token-usage line chart (sent vs. received over session time).
fn render_token_chart(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Tokens Over Time",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if state.chart_data.tokens_sent_series.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No data yet. Run a prompt to see token usage over time.",
                Style::default().fg(palette.muted),
            ))),
            inner,
        );
        return;
    }

    let sent_data: Vec<(f64, f64)> = state.chart_data.tokens_sent_series.clone();
    let recv_data: Vec<(f64, f64)> = state.chart_data.tokens_recv_series.clone();

    let max_x = sent_data
        .iter()
        .map(|(x, _)| *x)
        .fold(0.0f64, f64::max)
        .max(1.0);
    let max_y = sent_data
        .iter()
        .chain(recv_data.iter())
        .map(|(_, y)| *y)
        .fold(0.0f64, f64::max)
        .max(1.0);

    let datasets = vec![
        Dataset::default()
            .name("Tokens Sent")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(palette.chart_1))
            .data(&sent_data),
        Dataset::default()
            .name("Tokens Recv")
            .marker(symbols::Marker::Braille)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(palette.chart_2))
            .data(&recv_data),
    ];

    let x_axis = ratatui::widgets::Axis::default()
        .title(Span::styled(
            "Elapsed (s)",
            Style::default().fg(palette.muted),
        ))
        .style(Style::default().fg(palette.border))
        .bounds([0.0, max_x])
        .labels(vec![
            Span::styled("0", Style::default().fg(palette.muted)),
            Span::styled(
                format!("{:.0}", max_x / 2.0),
                Style::default().fg(palette.muted),
            ),
            Span::styled(format!("{:.0}", max_x), Style::default().fg(palette.muted)),
        ]);

    let y_axis = ratatui::widgets::Axis::default()
        .title(Span::styled("Tokens", Style::default().fg(palette.muted)))
        .style(Style::default().fg(palette.border))
        .bounds([0.0, max_y * 1.1])
        .labels(vec![
            Span::styled("0", Style::default().fg(palette.muted)),
            Span::styled(
                format!("{:.0}", max_y / 2.0),
                Style::default().fg(palette.muted),
            ),
            Span::styled(format!("{:.0}", max_y), Style::default().fg(palette.muted)),
        ]);

    let chart = Chart::new(datasets).x_axis(x_axis).y_axis(y_axis);
    frame.render_widget(chart, inner);
}

/// Renders the per-tick request/response bar chart.
fn render_requests_bar_chart(
    frame: &mut Frame,
    area: Rect,
    state: &TuiState,
    palette: &ThemePalette,
) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Requests / Responses per Tick",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let req_data: Vec<(&str, u64)> = state
        .chart_data
        .requests_per_minute
        .iter()
        .map(|v| ("", *v))
        .collect();

    if req_data.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  No request data yet.",
                Style::default().fg(palette.muted),
            ))),
            inner,
        );
        return;
    }

    let bar_chart = BarChart::default()
        .bar_width(3)
        .bar_gap(1)
        .bar_style(Style::default().fg(palette.chart_1))
        .value_style(Style::default().fg(palette.fg).add_modifier(Modifier::BOLD))
        .label_style(Style::default().fg(palette.muted))
        .data(&req_data)
        .max(req_data.iter().map(|(_, v)| *v).max().unwrap_or(1));

    frame.render_widget(bar_chart, inner);
}

/// Renders the session-level KPI summary panel (totals, averages, success rate).
fn render_kpi_panel(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Session KPIs",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let total_tokens = state.stats.tokens_sent + state.stats.tokens_received;
    let avg_tokens = if state.stats.requests > 0 {
        total_tokens / state.stats.requests as u64
    } else {
        0
    };
    let completed = state
        .tasks
        .iter()
        .filter(|t| t.status == crate::cli::session::TaskStatus::Completed)
        .count();
    let success_rate = (completed * 100)
        .checked_div(state.total_tasks)
        .unwrap_or(0);

    let elapsed = state
        .chart_data
        .session_start
        .map(|s| s.elapsed().as_secs())
        .unwrap_or(0);
    let elapsed_str = format!("{}m {}s", elapsed / 60, elapsed % 60);

    let lines = vec![
        Line::from(vec![
            Span::styled("  Total Tokens   ", Style::default().fg(palette.muted)),
            Span::styled(
                total_tokens.to_string(),
                Style::default()
                    .fg(palette.accent)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Avg Tok/Req    ", Style::default().fg(palette.muted)),
            Span::styled(avg_tokens.to_string(), Style::default().fg(palette.fg)),
        ]),
        Line::from(vec![
            Span::styled("  Session Time   ", Style::default().fg(palette.muted)),
            Span::styled(elapsed_str, Style::default().fg(palette.chart_1)),
        ]),
        Line::from(vec![
            Span::styled("  Success Rate   ", Style::default().fg(palette.muted)),
            Span::styled(
                format!("{}%", success_rate),
                Style::default().fg(if success_rate >= 80 {
                    palette.ok
                } else {
                    palette.warn
                }),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tasks Done     ", Style::default().fg(palette.muted)),
            Span::styled(
                format!("{}/{}", completed, state.total_tasks),
                Style::default().fg(palette.ok),
            ),
        ]),
    ];

    frame.render_widget(Paragraph::new(lines).wrap(Wrap { trim: false }), inner);
}

/// Renders a rolling token-rate sparkline across the bottom of the analytics tab.
fn render_sparkline_panel(frame: &mut Frame, area: Rect, state: &TuiState, palette: &ThemePalette) {
    let block = Block::default()
        .title(Line::from(vec![
            Span::styled(" ◆ ", Style::default().fg(palette.accent)),
            Span::styled(
                "Token Rate (rolling)",
                Style::default().fg(palette.fg).add_modifier(Modifier::BOLD),
            ),
        ]))
        .borders(Borders::ALL)
        .border_set(symbols::border::ROUNDED)
        .border_style(Style::default().fg(palette.border))
        .style(Style::default().bg(palette.bg));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let spark_data: Vec<u64> = state
        .chart_data
        .token_rate_sparkline
        .iter()
        .cloned()
        .collect();
    if spark_data.is_empty() {
        frame.render_widget(
            Paragraph::new(Span::styled(
                "  Waiting for data...",
                Style::default().fg(palette.muted),
            )),
            inner,
        );
        return;
    }

    let sparkline = Sparkline::default()
        .style(Style::default().fg(palette.chart_1))
        .data(&spark_data)
        .max(*spark_data.iter().max().unwrap_or(&1));

    frame.render_widget(sparkline, inner);
}

// Copyright 2026 Mahmoud Harmouch.
//
// Licensed under the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
