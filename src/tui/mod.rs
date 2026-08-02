pub mod app;
pub mod screens;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::Line,
    widgets::{Block, Borders, Paragraph, Tabs},
    Terminal,
};
use std::io;

use app::{App, InputMode, Screen};

pub fn run_tui(
    issues: Vec<crate::github::issues::ScoredIssue>,
    stale_items: Vec<crate::analysis::stale::StaleItem>,
    repo_scores: Vec<crate::analysis::scoring::RepoScore>,
) -> anyhow::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    app.set_issues(issues);
    app.set_stale(stale_items);
    app.set_repo_scores(repo_scores);

    let result = run_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut App,
) -> anyhow::Result<()> {
    loop {
        terminal.draw(|f| draw(f, app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }

            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                    KeyCode::Tab => app.next_screen(),
                    KeyCode::BackTab => app.prev_screen(),
                    KeyCode::Down | KeyCode::Char('j') => app.next_item(),
                    KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
                    KeyCode::Enter => {
                        if app.screen == Screen::Issues {
                            app.screen = Screen::Detail;
                        }
                    }
                    KeyCode::Char('/') => app.start_filter(),
                    KeyCode::Char('c') => app.clear_filter(),
                    KeyCode::Char('d') => app.screen = Screen::Dashboard,
                    KeyCode::Char('i') => app.screen = Screen::Issues,
                    KeyCode::Char('r') => app.screen = Screen::Repos,
                    _ => {}
                },
                InputMode::Filtering => match key.code {
                    KeyCode::Enter => app.apply_filter(),
                    KeyCode::Esc => app.cancel_filter(),
                    KeyCode::Char(c) => app.input.push(c),
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    _ => {}
                },
            }
        }
    }
}

fn draw(f: &mut ratatui::Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(1),
        ])
        .split(f.area());

    // tab bar
    let titles = vec![
        Line::from("Dashboard [d]"),
        Line::from("Issues [i]"),
        Line::from("Repos [r]"),
        Line::from("Detail"),
    ];
    let selected_tab = match app.screen {
        Screen::Dashboard => 0,
        Screen::Issues => 1,
        Screen::Repos => 2,
        Screen::Detail => 3,
    };
    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title("gh-opp"))
        .select(selected_tab)
        .style(Style::default().fg(Color::White))
        .highlight_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        );
    f.render_widget(tabs, chunks[0]);

    // main content
    match app.screen {
        Screen::Dashboard => screens::dashboard::draw(f, app, chunks[1]),
        Screen::Issues => screens::issues::draw(f, app, chunks[1]),
        Screen::Repos => screens::repos::draw(f, app, chunks[1]),
        Screen::Detail => screens::detail::draw(f, app, chunks[1]),
    }

    // status bar
    let status_text = if app.input_mode == InputMode::Filtering {
        format!("Filter: {}_", app.input)
    } else if app.filter.is_empty() {
        format!(
            "{} | j/k: navigate | Tab: switch | /: filter | q: quit",
            app.status
        )
    } else {
        format!(
            "Filter: '{}' | c: clear | j/k: navigate | q: quit",
            app.filter
        )
    };
    let status =
        Paragraph::new(status_text).style(Style::default().fg(Color::DarkGray).bg(Color::Black));
    f.render_widget(status, chunks[2]);
}
