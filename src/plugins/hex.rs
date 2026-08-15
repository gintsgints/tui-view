//! A hex-dump [`FormatView`].
//!
//! Renders raw bytes in the classic three-column layout — offset, hex pairs,
//! printable ASCII — and is the fallback for binary files: it claims any
//! content that [`is_binary`] reports on, whatever the file is named, so a
//! `.png` (or a `.json` that turns out to be a JPEG) still opens instead of
//! showing replacement characters.
//!
//! ```text
//! 00000000  89 50 4e 47 0d 0a 1a 0a  00 00 00 0d 49 48 44 52  |.PNG........IHDR|
//! ```

use std::fmt::Write as _;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use crate::view::{is_binary, FormatView};

/// Row widths the auto layout will choose between, widest first.
const AUTO_WIDTHS: [usize; 4] = [32, 16, 8, 4];

/// Renders raw bytes as a hex dump.
#[derive(Debug, Clone, Default)]
pub struct HexView {
    /// Bytes per row, or `None` to fit the viewport width.
    bytes_per_line: Option<usize>,
    theme: HexTheme,
}

impl HexView {
    /// A hex view that fits its row width to the viewport.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A hex view with a fixed number of bytes per row.
    ///
    /// `bytes_per_line` is clamped to at least 1. Rows wider than the viewport
    /// are truncated by the widget rather than wrapped.
    #[must_use]
    pub fn with_bytes_per_line(mut self, bytes_per_line: usize) -> Self {
        self.bytes_per_line = Some(bytes_per_line.max(1));
        self
    }

    /// A hex view with a custom [`HexTheme`].
    #[must_use]
    pub fn with_theme(mut self, theme: HexTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Bytes per row for a viewport of `width` columns.
    fn row_len(&self, width: u16) -> usize {
        if let Some(n) = self.bytes_per_line {
            return n;
        }
        let width = width as usize;
        AUTO_WIDTHS
            .into_iter()
            .find(|&n| row_width(n) <= width)
            .unwrap_or(*AUTO_WIDTHS.last().unwrap())
    }

    /// Dump `bytes` as one [`Line`] per row of `row_len` bytes.
    fn dump(&self, bytes: &[u8], row_len: usize) -> Text<'static> {
        if bytes.is_empty() {
            return Text::from(Line::from(Span::styled("(empty)", self.theme.offset)));
        }
        let lines = bytes
            .chunks(row_len)
            .enumerate()
            .map(|(i, chunk)| self.row(i * row_len, chunk, row_len))
            .collect::<Vec<_>>();
        Text::from(lines)
    }

    /// One dump row: offset, hex pairs padded to `row_len` columns, ASCII.
    fn row(&self, offset: usize, chunk: &[u8], row_len: usize) -> Line<'static> {
        let mut spans = vec![
            Span::styled(format!("{offset:08x}"), self.theme.offset),
            Span::raw("  "),
        ];

        // Hex column. Each byte keeps its own style so zero bytes can recede;
        // separators are padding, and short final chunks are padded to keep
        // the ASCII column aligned across every row.
        let mut hex = String::new();
        for i in 0..row_len {
            match chunk.get(i) {
                Some(&b) => {
                    let style = if b == 0 {
                        self.theme.zero
                    } else {
                        self.theme.byte
                    };
                    if !hex.is_empty() {
                        spans.push(Span::raw(std::mem::take(&mut hex)));
                    }
                    spans.push(Span::styled(format!("{b:02x}"), style));
                }
                None => hex.push_str("  "),
            }
            if i + 1 < row_len {
                hex.push(' ');
                if (i + 1) % 8 == 0 {
                    hex.push(' ');
                }
            }
        }
        spans.push(Span::raw(hex + "  "));

        // ASCII column, printable bytes only.
        spans.push(Span::styled("|", self.theme.gutter));
        let mut run = String::new();
        let mut run_printable = None;
        for &b in chunk {
            let printable = b.is_ascii_graphic() || b == b' ';
            if run_printable != Some(printable) && !run.is_empty() {
                let style = self.ascii_style(run_printable == Some(true));
                spans.push(Span::styled(std::mem::take(&mut run), style));
            }
            run_printable = Some(printable);
            run.push(if printable { char::from(b) } else { '.' });
        }
        if !run.is_empty() {
            let style = self.ascii_style(run_printable == Some(true));
            spans.push(Span::styled(run, style));
        }
        spans.push(Span::styled("|", self.theme.gutter));

        Line::from(spans)
    }

    fn ascii_style(&self, printable: bool) -> Style {
        if printable {
            self.theme.ascii
        } else {
            self.theme.nonprintable
        }
    }
}

impl FormatView for HexView {
    fn name(&self) -> &'static str {
        "Hex"
    }

    fn extensions(&self) -> &[&str] {
        &[
            "bin", "dat", "hex", "dump", "o", "obj", "exe", "dll", "so", "dylib", "wasm", "class",
        ]
    }

    fn render(&self, content: &str, width: u16) -> Text<'static> {
        self.dump(content.as_bytes(), self.row_len(width))
    }

    fn render_bytes(&self, bytes: &[u8], width: u16) -> Text<'static> {
        self.dump(bytes, self.row_len(width))
    }

    /// Claims every binary file, so unreadable content never reaches a text
    /// view. See [`is_binary`].
    fn matches_content(&self, bytes: &[u8]) -> bool {
        is_binary(bytes)
    }
}

/// Columns a row of `row_len` bytes occupies: offset, hex pairs with a wider
/// gap every eight bytes, then the ASCII column between its bars.
#[must_use]
fn row_width(row_len: usize) -> usize {
    let hex = 3 * row_len - 1 + (row_len - 1) / 8;
    8 + 2 + hex + 2 + row_len + 2
}

/// Styling for the [`HexView`]. Adjust individual fields to reskin.
#[derive(Debug, Clone)]
pub struct HexTheme {
    /// The leading byte offset.
    pub offset: Style,
    /// Hex pairs.
    pub byte: Style,
    /// Hex pairs for `00`, dimmed so padding regions read as empty.
    pub zero: Style,
    /// Printable characters in the ASCII column.
    pub ascii: Style,
    /// The `.` standing in for non-printable bytes.
    pub nonprintable: Style,
    /// The `|` bars around the ASCII column.
    pub gutter: Style,
}

impl Default for HexTheme {
    fn default() -> Self {
        Self {
            offset: Style::new().fg(Color::DarkGray),
            byte: Style::new().fg(Color::Cyan),
            zero: Style::new().fg(Color::DarkGray),
            ascii: Style::new().fg(Color::Green),
            nonprintable: Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM),
            gutter: Style::new().fg(Color::Gray),
        }
    }
}

/// A hex dump of `bytes` as plain text, one row per `row_len` bytes.
///
/// Handy for tests and logging; the view itself renders styled spans.
#[must_use]
pub fn dump_to_string(bytes: &[u8], row_len: usize) -> String {
    let row_len = row_len.max(1);
    let view = HexView::new().with_bytes_per_line(row_len);
    let mut out = String::new();
    for line in view.dump(bytes, row_len).lines {
        for span in &line.spans {
            out.push_str(span.content.as_ref());
        }
        let _ = writeln!(out);
    }
    out
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
    fn dumps_classic_layout() {
        let lines = plain(
            &HexView::new()
                .with_bytes_per_line(16)
                .render_bytes(b"Hello, hex view!\x00\xff", 120),
        );
        assert_eq!(
            lines[0],
            "00000000  48 65 6c 6c 6f 2c 20 68  65 78 20 76 69 65 77 21  |Hello, hex view!|"
        );
        // Short final row: hex column padded so the ASCII bars stay aligned.
        assert_eq!(
            lines[1],
            "00000010  00 ff                                             |..|"
        );
    }

    #[test]
    fn claims_binary_content_only() {
        let view = HexView::new();
        assert!(view.matches_content(b"\x89PNG\r\n\x1a\n\x00\x00"));
        assert!(view.matches_content(&[0xff, 0xfe, 0x00, 0x01]));
        assert!(!view.matches_content(b"plain text\n"));
        assert!(!view.matches_content("hyv\u{e4}\u{e4} p\u{e4}iv\u{e4}\u{e4}".as_bytes()));
        assert!(!view.matches_content(b""));
    }

    #[test]
    fn matches_binary_extensions() {
        assert!(HexView::new().matches(std::path::Path::new("a.BIN")));
        assert!(!HexView::new().matches(std::path::Path::new("a.md")));
    }

    #[test]
    fn auto_width_fits_viewport() {
        let view = HexView::new();
        assert_eq!(view.row_len(200), 32);
        assert_eq!(view.row_len(78), 16);
        assert_eq!(view.row_len(60), 8);
        assert_eq!(view.row_len(30), 4);
        // Narrower than any layout: falls back to the narrowest row.
        assert_eq!(view.row_len(5), 4);
    }

    #[test]
    fn rows_fit_their_advertised_width() {
        for n in AUTO_WIDTHS {
            let bytes: Vec<u8> = (0..u8::try_from(n).unwrap()).collect();
            for line in plain(
                &HexView::new()
                    .with_bytes_per_line(n)
                    .render_bytes(&bytes, 200),
            ) {
                assert_eq!(line.chars().count(), row_width(n), "row of {n} bytes");
            }
        }
    }

    #[test]
    fn renders_str_content_as_bytes() {
        let text = HexView::new().with_bytes_per_line(4).render("abc", 80);
        assert_eq!(plain(&text), vec!["00000000  61 62 63     |abc|"]);
    }

    #[test]
    fn empty_input_renders_a_placeholder() {
        assert_eq!(
            plain(&HexView::new().render_bytes(b"", 80)),
            vec!["(empty)"]
        );
    }

    #[test]
    fn dump_to_string_round_trips() {
        assert_eq!(dump_to_string(b"ab", 2), "00000000  61 62  |ab|\n");
    }
}
