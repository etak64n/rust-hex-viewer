use std::{
    io::{Stdout, stdout},
    time::{Duration, Instant},
};

use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    layout::{Constraint, Direction, Layout},
    prelude::CrosstermBackend,
    style::{Color, Style},
    widgets::{Block, Borders, Paragraph, Wrap},
};

use crate::app::App;

type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

/// Setup the TUI and drive the event loop.
pub fn run(app: &mut App) -> Result<()> {
    let mut terminal = setup_terminal()?;
    let result = run_loop(&mut terminal, app);
    restore_terminal(&mut terminal)?;
    result
}

fn setup_terminal() -> Result<CrosstermTerminal> {
    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut CrosstermTerminal) -> Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn run_loop(terminal: &mut CrosstermTerminal, app: &mut App) -> Result<()> {
    let tick_rate = Duration::from_millis(250);
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| draw_ui(f, app))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_millis(0));

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if !handle_key(app, key) {
                    break;
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            last_tick = Instant::now();
        }
    }
    Ok(())
}

fn draw_ui(frame: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(1)])
        .split(frame.area());

    let visible_rows = chunks[0].height.max(1) as usize;
    app.set_view_rows(visible_rows);

    let body = Paragraph::new(app.render_lines(visible_rows))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(format!(" {} ", app.file_name())),
        )
        .wrap(Wrap { trim: false });

    frame.render_widget(body, chunks[0]);

    let status = Paragraph::new(app.status_line()).style(Style::default().fg(Color::Gray));
    frame.render_widget(status, chunks[1]);
}

fn handle_key(app: &mut App, key: KeyEvent) -> bool {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => return false,
        KeyCode::Down | KeyCode::Char('j') => app.scroll_rows(1),
        KeyCode::Up | KeyCode::Char('k') => app.scroll_rows(-1),
        KeyCode::PageDown | KeyCode::Char(' ') => app.scroll_rows(app.view_rows() as isize),
        KeyCode::PageUp => app.scroll_rows(-(app.view_rows() as isize)),
        KeyCode::Home | KeyCode::Char('g') => app.scroll_to_start(),
        KeyCode::End | KeyCode::Char('G') => app.scroll_to_end(),
        _ => {}
    }
    true
}
