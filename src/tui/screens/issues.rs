use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Row, Table, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let filtered = app.filtered_issues();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(4)])
        .split(area);

    // issue table
    let header = Row::new(vec!["Score", "#", "Title", "Labels", "Updated"])
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .height(1);

    let rows: Vec<Row> = filtered
        .iter()
        .enumerate()
        .map(|(i, issue)| {
            let labels = if issue.labels.is_empty() {
                "-".to_string()
            } else {
                issue.labels.join(", ")
            };
            let days_ago = (chrono::Utc::now() - issue.updated_at).num_days();
            let style = if i == app.selected {
                Style::default().bg(Color::DarkGray).fg(Color::White)
            } else {
                Style::default()
            };
            Row::new(vec![
                format!("{:.1}", issue.score),
                issue.number.to_string(),
                truncate(&issue.title, 40),
                truncate(&labels, 25),
                format!("{}d", days_ago),
            ])
            .style(style)
        })
        .collect();

    let table = Table::new(
        rows,
        [
            Constraint::Length(6),
            Constraint::Length(7),
            Constraint::Min(20),
            Constraint::Length(20),
            Constraint::Length(6),
        ],
    )
    .header(header)
    .block(Block::default().borders(Borders::ALL).title(format!(
        "Issues ({}/{})",
        filtered.len(),
        app.issues.len()
    )))
    .row_highlight_style(
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
    );

    f.render_widget(table, chunks[0]);

    // preview pane
    if let Some(issue) = app.selected_issue() {
        let preview_text = vec![
            Line::from(vec![
                Span::styled("Title: ", Style::default().fg(Color::Yellow)),
                Span::raw(&issue.title),
            ]),
            Line::from(vec![
                Span::styled("URL: ", Style::default().fg(Color::Yellow)),
                Span::raw(&issue.url),
            ]),
            Line::from(vec![
                Span::styled("Labels: ", Style::default().fg(Color::Yellow)),
                Span::raw(issue.labels.join(", ")),
            ]),
            Line::from(vec![
                Span::styled("Body: ", Style::default().fg(Color::Yellow)),
                Span::raw(issue.body_preview.as_deref().unwrap_or("No description")),
            ]),
        ];
        let preview = ratatui::widgets::Paragraph::new(preview_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Preview (Enter: detail)"),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(preview, chunks[1]);
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}...", &s[..max.saturating_sub(3)])
    }
}
