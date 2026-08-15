//! Shared soft-wrapping for views that render source lines verbatim.
//!
//! Views must hand the widget text already wrapped to the viewport width, so
//! the scroll offset is an exact row count. [`wrap_spans`] is the one place
//! that does the splitting; plugins feed it one logical source line at a time
//! instead of rolling their own.

use ratatui::text::{Line, Span};

/// Soft-wrap one logical line, given as its `spans`, to `width` columns.
///
/// Content is preserved verbatim — indentation and runs of spaces are kept —
/// so an over-long line is split at the width boundary rather than at a word
/// boundary. Spans are split too, keeping each fragment's style, so a
/// highlighted token that straddles the boundary stays highlighted on both
/// rows. Every call pushes at least one row: an empty line stays an empty row.
pub(crate) fn wrap_spans(spans: Vec<Span<'static>>, width: usize, out: &mut Vec<Line<'static>>) {
    let width = width.max(1);
    let mut row: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;
    for span in spans {
        if span.content.is_empty() {
            continue;
        }
        let chars: Vec<char> = span.content.chars().collect();
        let mut start = 0;
        while start < chars.len() {
            if used == width {
                out.push(Line::from(std::mem::take(&mut row)));
                used = 0;
            }
            let take = (width - used).min(chars.len() - start);
            let chunk: String = chars[start..start + take].iter().collect();
            row.push(Span::styled(chunk, span.style));
            used += take;
            start += take;
        }
    }
    out.push(Line::from(row));
}

#[cfg(test)]
mod tests {
    use ratatui::style::{Color, Style};

    use super::*;

    fn rows(out: &[Line<'static>]) -> Vec<String> {
        out.iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    #[test]
    fn empty_line_stays_one_row() {
        let mut out = Vec::new();
        wrap_spans(Vec::new(), 10, &mut out);
        assert_eq!(rows(&out), vec![""]);
    }

    #[test]
    fn exact_width_does_not_emit_a_blank_row() {
        let mut out = Vec::new();
        wrap_spans(vec![Span::raw("12345")], 5, &mut out);
        assert_eq!(rows(&out), vec!["12345"]);
    }

    #[test]
    fn splits_across_spans_at_the_boundary() {
        let mut out = Vec::new();
        let red = Style::new().fg(Color::Red);
        wrap_spans(
            vec![Span::raw("abc"), Span::styled("defgh", red)],
            4,
            &mut out,
        );
        assert_eq!(rows(&out), vec!["abcd", "efgh"]);
        // The style survives the split.
        assert_eq!(out[0].spans[1].style, red);
        assert_eq!(out[1].spans[0].style, red);
    }

    #[test]
    fn concatenating_rows_reproduces_the_source() {
        let mut out = Vec::new();
        wrap_spans(vec![Span::raw("aa bb cc dd ee")], 5, &mut out);
        assert_eq!(rows(&out).concat(), "aa bb cc dd ee");
    }
}
