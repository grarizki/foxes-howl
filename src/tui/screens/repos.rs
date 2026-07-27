use ratatui::{
    layout::{Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Span,
    widgets::{Block, Borders, Row, Table},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let header = Row::new(vec!["Score", "Repo", "Opps", "Stale", "README", "Quality"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = app
        .repo_scores
        .iter()
        .enumerate()
        .map(|(i, r)| {
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            let color = if r.composite_score > 0.6 {
                Color::Red
            } else if r.composite_score > 0.3 {
                Color::Yellow
            } else {
                Color::Green
            };
            Row::new(vec![
                Span::styled(
                    format!("{:.2}", r.composite_score),
                    Style::default().fg(color),
                ),
                Span::raw(truncate(&r.repo, 30)),
                Span::raw(r.opportunity_count.to_string()),
                Span::raw(r.stale_count.to_string()),
                Span::raw(format!("{:.0}%", r.readme_score * 100.0)),
                Span::raw(format!("{:.0}%", r.code_quality_score * 100.0)),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(7),
            Constraint::Min(20),
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Length(8),
            Constraint::Length(9),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Repos ({})", app.repo_scores.len())),
    );

    f.render_widget(table, area);
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
