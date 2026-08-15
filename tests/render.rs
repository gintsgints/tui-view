//! End-to-end: render through a real ratatui backend buffer and read it back.

use std::path::Path;

use ratatui::backend::TestBackend;
use ratatui::widgets::Block;
use ratatui::Terminal;
use tui_view::{TuiView, ViewRegistry, ViewState};

/// Flatten a rendered buffer into one string per row.
fn rows(term: &Terminal<TestBackend>) -> Vec<String> {
    let buf = term.backend().buffer();
    let area = *buf.area();
    (0..area.height)
        .map(|y| {
            (0..area.width)
                .map(|x| buf[(x, y)].symbol())
                .collect::<String>()
                .trim_end()
                .to_string()
        })
        .collect()
}

const DOC: &str = "# Heading\n\nBody paragraph with **bold** words that keep going long enough to wrap.\n\n- alpha\n- beta\n";

#[test]
fn renders_document_into_buffer() {
    let reg = ViewRegistry::with_defaults();
    let mut state = ViewState::from_path(Path::new("doc.md"), DOC, &reg).expect("markdown view");

    let mut term = Terminal::new(TestBackend::new(40, 12)).unwrap();
    term.draw(|f| {
        f.render_stateful_widget(
            TuiView::new().block(Block::bordered()),
            f.area(),
            &mut state,
        );
    })
    .unwrap();

    let text = rows(&term).join("\n");
    assert!(text.contains("# Heading"), "missing heading:\n{text}");
    assert!(text.contains("• alpha"), "missing bullet:\n{text}");
    // Inside a bordered 40-col area no content row exceeds the frame.
    for row in rows(&term) {
        assert!(row.chars().count() <= 40);
    }
    assert!(state.line_count() > 0);
}

#[test]
fn scroll_clamps_to_bottom() {
    let reg = ViewRegistry::with_defaults();
    let mut long = String::new();
    for i in 0..200 {
        use std::fmt::Write;
        writeln!(long, "line {i}\n").unwrap();
    }
    let mut state = ViewState::from_path(Path::new("d.md"), long, &reg).unwrap();

    let mut term = Terminal::new(TestBackend::new(30, 10)).unwrap();
    // Draw once so the view renders and viewport height is known.
    term.draw(|f| f.render_stateful_widget(TuiView::new(), f.area(), &mut state))
        .unwrap();

    state.scroll_down(100_000);
    term.draw(|f| f.render_stateful_widget(TuiView::new(), f.area(), &mut state))
        .unwrap();

    // Clamped: can never scroll past the last screenful.
    assert_eq!(state.scroll(), state.max_scroll());
    assert!(state.scroll() <= state.line_count());
}
