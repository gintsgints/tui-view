//! A plain-text [`FormatView`].
//!
//! Renders content verbatim, preserving source line breaks and soft-wrapping
//! over-long lines to the available width. No styling is applied.

use ratatui::style::Style;
use ratatui::text::{Line, Span, Text};

use crate::view::FormatView;

/// Renders unstyled plain text, wrapped to the viewport width.
#[derive(Debug, Clone, Default)]
pub struct PlainTextView {
    /// Style applied to every line.
    style: Style,
}

impl PlainTextView {
    /// A plain-text view using the terminal's default style.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A plain-text view that renders every line in `style`.
    #[must_use]
    pub fn styled(style: Style) -> Self {
        Self { style }
    }
}

impl FormatView for PlainTextView {
    fn name(&self) -> &'static str {
        "Plain text"
    }

    fn extensions(&self) -> &[&str] {
        &["txt", "text", "log"]
    }

    fn render(&self, content: &str, width: u16) -> Text<'static> {
        let width = width.max(1) as usize;
        let mut out: Vec<Line<'static>> = Vec::new();
        for source_line in content.split('\n') {
            let source_line = source_line.strip_suffix('\r').unwrap_or(source_line);
            wrap_line(source_line, width, self.style, &mut out);
        }
        // A trailing newline yields one empty tail line; drop it so the buffer
        // does not gain a phantom blank row.
        if content.ends_with('\n') {
            out.pop();
        }
        Text::from(out)
    }
}

/// Word-wrap `text` to `width` columns, pushing one [`Line`] per visual row.
/// A blank source line still emits one (empty) row.
fn wrap_line(text: &str, width: usize, style: Style, out: &mut Vec<Line<'static>>) {
    if text.is_empty() {
        out.push(Line::default());
        return;
    }
    let mut line = String::new();
    let mut line_w = 0usize;
    for word in text.split(' ') {
        let ww = word.chars().count();
        let need = if line_w == 0 { ww } else { ww + 1 };
        if line_w > 0 && line_w + need > width {
            out.push(Line::from(Span::styled(std::mem::take(&mut line), style)));
            line_w = 0;
        }
        if line_w > 0 {
            line.push(' ');
            line_w += 1;
        }
        line.push_str(word);
        line_w += ww;
    }
    out.push(Line::from(Span::styled(line, style)));
}

#[cfg(test)]
mod tests {
    use super::*;

    fn plain(text: &Text) -> Vec<String> {
        text.lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn matches_extension() {
        assert!(PlainTextView::new().matches(std::path::Path::new("notes.txt")));
        assert!(PlainTextView::new().matches(std::path::Path::new("app.log")));
        assert!(!PlainTextView::new().matches(std::path::Path::new("a.json")));
    }

    #[test]
    fn preserves_lines_and_blanks() {
        let lines = plain(&PlainTextView::new().render("one\n\ntwo\n", 80));
        assert_eq!(lines, vec!["one", "", "two"]);
    }

    #[test]
    fn wraps_long_lines() {
        let lines = plain(&PlainTextView::new().render("aa bb cc dd ee", 5));
        for l in &lines {
            assert!(l.chars().count() <= 5, "overflow: {l:?}");
        }
        assert_eq!(lines.join(" "), "aa bb cc dd ee");
    }

    #[test]
    fn strips_carriage_returns() {
        let lines = plain(&PlainTextView::new().render("a\r\nb", 80));
        assert_eq!(lines, vec!["a", "b"]);
    }
}
