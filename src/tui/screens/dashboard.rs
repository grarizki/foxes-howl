use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;

pub fn draw(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(5),
            Constraint::Min(0),
        ])
        .split(area);

    // stats overview
    let total_issues = app.issues.len();
    let total_stale = app.stale_items.len();
    let total_repos = app.repo_scores.len();

    let stats = vec![
        Line::from(vec![
            Span::styled(
                format!("{} ", total_issues),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("contribution opportunities"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", total_stale),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("stale items"),
        ]),
        Line::from(vec![
            Span::styled(
                format!("{} ", total_repos),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("repos analyzed"),
        ]),
    ];
    let stats_block = Paragraph::new(stats)
        .block(Block::default().borders(Borders::ALL).title("Overview"))
        .wrap(Wrap { trim: true });
    f.render_widget(stats_block, chunks[0]);

    // top opportunities
    let top_lines: Vec<Line> = app
        .issues
        .iter()
        .take(5)
        .map(|i| {
            Line::from(vec![
                Span::styled(
                    format!("{:.1} ", i.score),
                    Style::default().fg(Color::Yellow),
                ),
                Span::raw(format!("#{} ", i.number)),
                Span::raw(&i.title),
            ])
        })
        .collect();

    let top_block = Paragraph::new(top_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Top Opportunities"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(top_block, chunks[1]);

    // repo health
    let repo_lines: Vec<Line> = app
        .repo_scores
        .iter()
        .take(5)
        .map(|r| {
            let color = if r.composite_score > 0.6 {
                Color::Red
            } else if r.composite_score > 0.3 {
                Color::Yellow
            } else {
                Color::Green
            };
            Line::from(vec![
                Span::styled(
                    format!("{:.2} ", r.composite_score),
                    Style::default().fg(color),
                ),
                Span::raw(format!(
                    "{} ({} opps, {} stale)",
                    r.repo, r.opportunity_count, r.stale_count
                )),
            ])
        })
        .collect();

    let repo_block = Paragraph::new(repo_lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Repo Health (higher = more opportunity)"),
        )
        .wrap(Wrap { trim: true });
    f.render_widget(repo_block, chunks[2]);
}
