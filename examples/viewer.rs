//! A minimal terminal file viewer built on `tui-view`.
//!
//! Usage: `cargo run --example viewer -- path/to/file.md`
//! Unknown or extensionless files open in the plain-text view.
//! Keys: ↑/↓ or j/k scroll · PgUp/PgDn or Space · g/G top/bottom · q quit.

use std::io::{self, stdout};
use std::path::PathBuf;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use ratatui::backend::CrosstermBackend;
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding};
use ratatui::Terminal;
use std::sync::Arc;
use tui_view::plugins::plaintext::PlainTextView;
use tui_view::{TuiView, ViewRegistry, ViewState};

fn main() -> io::Result<()> {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: viewer <file>");
        std::process::exit(2);
    };
    let content = std::fs::read_to_string(&path)?;

    let registry = ViewRegistry::with_defaults();
    // Fall back to the plain-text view for unknown or extensionless files so
    // any file still opens.
    let mut state = match ViewState::from_path(&path, content.as_str(), &registry) {
        Some(state) => state,
        None => ViewState::new(content, Arc::new(PlainTextView::new())),
    };
    let title = format!(
        " {} — {} ",
        path.file_name().unwrap_or_default().to_string_lossy(),
        state.view().name()
    );

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let result = run(&mut terminal, &mut state, &title);

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    result
}

fn run<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    state: &mut ViewState,
    title: &str,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| {
            let block = Block::bordered()
                .title(Line::from(title.bold()))
                .title_bottom(
                    Line::from(format!(
                        " {}/{} ",
                        state.scroll() + 1,
                        state.line_count().max(1)
                    ))
                    .right_aligned(),
                )
                .border_style(Style::new().dim())
                .padding(Padding::horizontal(1));
            f.render_stateful_widget(TuiView::new().block(block), f.area(), state);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.kind != KeyEventKind::Press {
                continue;
            }
            match key.code {
                KeyCode::Char('q') | KeyCode::Esc => break,
                KeyCode::Down | KeyCode::Char('j') => state.scroll_down(1),
                KeyCode::Up | KeyCode::Char('k') => state.scroll_up(1),
                KeyCode::PageDown | KeyCode::Char(' ') => state.page_down(),
                KeyCode::PageUp => state.page_up(),
                KeyCode::Char('g') | KeyCode::Home => state.scroll_to_top(),
                KeyCode::Char('G') | KeyCode::End => state.scroll_to_bottom(),
                _ => {}
            }
        }
    }
    Ok(())
}
