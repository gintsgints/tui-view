//! A minimal terminal file viewer built on `tui-view`.
//!
//! Usage: `cargo run --example viewer -- path/to/file.md`
//! Binary files open in the hex view; unknown or extensionless text files
//! open in the plain-text view. Source files are highlighted when the matching
//! grammar is compiled in:
//! `cargo run --features languages --example viewer -- src/lib.rs`
//! Keys: ↑/↓ or j/k scroll · PgUp/PgDn or Space · g/G top/bottom · q quit.

use std::io;
use std::path::PathBuf;

use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::style::{Style, Stylize};
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding};
use ratatui::DefaultTerminal;
use std::sync::Arc;
use tui_view::plugins::plaintext::PlainTextView;
use tui_view::{TuiView, ViewRegistry, ViewState};

fn main() -> io::Result<()> {
    let Some(path) = std::env::args_os().nth(1).map(PathBuf::from) else {
        eprintln!("usage: viewer <file>");
        std::process::exit(2);
    };
    let content = std::fs::read(&path)?;

    let registry = ViewRegistry::with_defaults();
    // `find_for` sends binary content to the hex view; fall back to the
    // plain-text view for unknown or extensionless text files so any file
    // still opens.
    let view = registry
        .find_for(&path, &content)
        .unwrap_or_else(|| Arc::new(PlainTextView::new()));
    let mut state = ViewState::from_bytes(content, view);
    let title = format!(
        " {} — {} ",
        path.file_name().unwrap_or_default().to_string_lossy(),
        state.view().name()
    );

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, &mut state, &title);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, state: &mut ViewState, title: &str) -> io::Result<()> {
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
