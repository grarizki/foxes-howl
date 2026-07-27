use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let issue = match app.selected_issue() {
        Some(i) => i,
        None => {
            let msg = Paragraph::new("No issue selected. Go to Issues tab [i] first.")
                .block(Block::default().borders(Borders::ALL).title("Detail"));
            f.render_widget(msg, area);
            return;
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Min(0),
        ])
        .split(area);

    // header
    let header = Paragraph::new(vec![Line::from(vec![
        Span::styled(
            format!("#{} ", issue.number),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(&issue.title),
    ])])
    .block(Block::default().borders(Borders::ALL).title("Issue"));
    f.render_widget(header, chunks[0]);

    // metadata
    let days_ago = (chrono::Utc::now() - issue.updated_at).num_days();
    let meta = vec![
        Line::from(vec![
            Span::styled("URL:      ", Style::default().fg(Color::Yellow)),
            Span::raw(&issue.url),
        ]),
        Line::from(vec![
            Span::styled("Score:    ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{:.2}", issue.score)),
        ]),
        Line::from(vec![
            Span::styled("Updated:  ", Style::default().fg(Color::Yellow)),
            Span::raw(format!("{} days ago", days_ago)),
        ]),
        Line::from(vec![
            Span::styled("Assigned: ", Style::default().fg(Color::Yellow)),
            Span::raw(issue.assignee.as_deref().unwrap_or("No")),
        ]),
    ];
    let meta_block = Paragraph::new(meta)
        .block(Block::default().borders(Borders::ALL).title("Metadata"))
        .wrap(Wrap { trim: true });
    f.render_widget(meta_block, chunks[1]);

    // labels
    let label_text = if issue.labels.is_empty() {
        vec![Line::from("No labels")]
    } else {
        issue
            .labels
            .iter()
            .map(|l| {
                let color =
                    if l.to_lowercase().contains("good") || l.to_lowercase().contains("help") {
                        Color::Green
                    } else if l.to_lowercase().contains("bug") {
                        Color::Red
                    } else {
                        Color::White
                    };
                Line::from(Span::styled(format!("  {}", l), Style::default().fg(color)))
            })
            .collect()
    };
    let labels_block = Paragraph::new(label_text)
        .block(Block::default().borders(Borders::ALL).title("Labels"))
        .wrap(Wrap { trim: true });
    f.render_widget(labels_block, chunks[2]);

    // body preview
    let body = issue
        .body_preview
        .as_deref()
        .unwrap_or("No description available.");
    let body_block = Paragraph::new(body)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Description (preview)"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(body_block, chunks[3]);
}
