//! A Markdown [`FormatView`].
//!
//! Parses `CommonMark` (via `pulldown-cmark`) and renders it to styled,
//! width-wrapped [`Text`]. Supported constructs: headings, paragraphs, inline
//! emphasis/strong/strikethrough, inline code, links and images, bullet and
//! ordered lists (nested), task-list checkboxes, block quotes, fenced/indented
//! code blocks, and thematic breaks.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};

use crate::view::FormatView;

/// Renders Markdown documents.
#[derive(Debug, Clone, Default)]
pub struct MarkdownView {
    theme: Theme,
}

impl MarkdownView {
    /// A Markdown view with the default [`Theme`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A Markdown view with a custom [`Theme`].
    #[must_use]
    pub fn with_theme(theme: Theme) -> Self {
        Self { theme }
    }
}

impl FormatView for MarkdownView {
    fn name(&self) -> &'static str {
        "Markdown"
    }

    fn extensions(&self) -> &[&str] {
        &["md", "markdown", "mdown", "mkd"]
    }

    fn render(&self, content: &str, width: u16) -> Text<'static> {
        Renderer::new(&self.theme, width.max(1) as usize).run(content)
    }
}

/// Styling for the [`MarkdownView`]. Adjust individual fields to reskin.
#[derive(Debug, Clone)]
pub struct Theme {
    /// Style per heading level (index 0 = H1 .. index 5 = H6).
    pub headings: [Style; 6],
    /// Base body-text style.
    pub text: Style,
    /// Inline `code` spans.
    pub inline_code: Style,
    /// Fenced/indented code block lines.
    pub code_block: Style,
    /// Link text.
    pub link: Style,
    /// Block-quote text and its `│` gutter.
    pub quote: Style,
    /// List bullet / number markers.
    pub marker: Style,
    /// Thematic break (horizontal rule).
    pub rule: Style,
}

impl Default for Theme {
    fn default() -> Self {
        let bold = Modifier::BOLD;
        Self {
            headings: [
                Style::new()
                    .fg(Color::Cyan)
                    .add_modifier(bold | Modifier::UNDERLINED),
                Style::new().fg(Color::Green).add_modifier(bold),
                Style::new().fg(Color::Yellow).add_modifier(bold),
                Style::new().fg(Color::Magenta).add_modifier(bold),
                Style::new().fg(Color::Blue).add_modifier(bold),
                Style::new().fg(Color::Gray).add_modifier(bold),
            ],
            text: Style::new(),
            inline_code: Style::new().fg(Color::Rgb(0xd1, 0x9a, 0x66)),
            code_block: Style::new().fg(Color::Rgb(0x98, 0xc3, 0x79)),
            link: Style::new()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED),
            quote: Style::new().fg(Color::Gray).add_modifier(Modifier::ITALIC),
            marker: Style::new().fg(Color::Yellow),
            rule: Style::new().fg(Color::DarkGray),
        }
    }
}

/// A single inline glyph tagged with its style. Newlines are kept as data so
/// hard breaks survive wrapping.
type Glyph = (char, Style);

/// Walks Markdown events and builds the output [`Text`].
struct Renderer<'t> {
    theme: &'t Theme,
    width: usize,
    out: Vec<Line<'static>>,
    /// Pending inline glyphs for the block currently being built.
    inline: Vec<Glyph>,
    /// Style stack for nested inline spans (emphasis, links, ...).
    styles: Vec<Style>,
    /// Block-quote nesting depth.
    quotes: usize,
    /// One entry per open list item; value is that item's marker column width.
    indents: Vec<usize>,
    /// Ordered-list counters (`None` = bullet list).
    lists: Vec<Option<u64>>,
    /// Marker for the current item's first line, consumed on first flush.
    pending_marker: Option<Span<'static>>,
    /// Accumulator for the current fenced/indented code block.
    code: Option<String>,
}

impl<'t> Renderer<'t> {
    fn new(theme: &'t Theme, width: usize) -> Self {
        Self {
            theme,
            width,
            out: Vec::new(),
            inline: Vec::new(),
            styles: vec![theme.text],
            quotes: 0,
            indents: Vec::new(),
            lists: Vec::new(),
            pending_marker: None,
            code: None,
        }
    }

    fn run(mut self, content: &str) -> Text<'static> {
        let opts = Options::ENABLE_STRIKETHROUGH | Options::ENABLE_TASKLISTS;
        for event in Parser::new_ext(content, opts) {
            self.event(event);
        }
        self.flush_inline();
        Text::from(self.out)
    }

    fn cur_style(&self) -> Style {
        *self.styles.last().unwrap()
    }

    /// Push `patch` merged onto the current style.
    fn push_style(&mut self, patch: Style) {
        let merged = self.cur_style().patch(patch);
        self.styles.push(merged);
    }

    fn pop_style(&mut self) {
        if self.styles.len() > 1 {
            self.styles.pop();
        }
    }

    fn text(&mut self, s: &str) {
        let style = self.cur_style();
        self.inline.extend(s.chars().map(|c| (c, style)));
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(&tag),
            Event::End(tag) => self.end(tag),
            Event::Text(t) => {
                if let Some(code) = self.code.as_mut() {
                    code.push_str(&t);
                } else {
                    self.text(&t);
                }
            }
            Event::Code(t) => {
                self.push_style(self.theme.inline_code);
                self.text(&t);
                self.pop_style();
            }
            Event::SoftBreak => self.text(" "),
            Event::HardBreak => self.inline.push(('\n', self.cur_style())),
            Event::Rule => self.rule(),
            Event::TaskListMarker(done) => {
                let mark = if done { "[x] " } else { "[ ] " };
                let style = self.theme.marker;
                self.inline.extend(mark.chars().map(|c| (c, style)));
            }
            Event::Html(h) | Event::InlineHtml(h) => {
                // Render raw HTML dimmed rather than dropping it.
                let style = self.theme.text.add_modifier(Modifier::DIM);
                self.inline.extend(h.trim_end().chars().map(|c| (c, style)));
            }
            Event::FootnoteReference(name) => self.text(&format!("[^{name}]")),
            Event::InlineMath(m) | Event::DisplayMath(m) => {
                self.push_style(self.theme.inline_code);
                self.text(&m);
                self.pop_style();
            }
        }
    }

    fn start(&mut self, tag: &Tag<'_>) {
        match tag {
            Tag::Heading { level, .. } => {
                let idx = heading_index(*level);
                let style = self.theme.headings[idx];
                self.styles.push(style);
                let hashes = "#".repeat(idx + 1);
                self.inline
                    .extend(format!("{hashes} ").chars().map(|c| (c, style)));
            }
            Tag::BlockQuote(_) => {
                self.flush_inline();
                self.quotes += 1;
            }
            Tag::CodeBlock(_) => {
                self.flush_inline();
                self.code = Some(String::new());
            }
            Tag::List(start) => {
                // Flush any text that preceded a nested list in this item.
                self.flush_inline();
                self.lists.push(*start);
            }
            Tag::Item => {
                let marker = match self.lists.last_mut() {
                    Some(Some(n)) => {
                        let m = format!("{n}. ");
                        *n += 1;
                        m
                    }
                    _ => "• ".to_string(),
                };
                self.indents.push(marker.chars().count());
                self.pending_marker = Some(Span::styled(marker, self.theme.marker));
            }
            Tag::Emphasis => self.push_style(Style::new().add_modifier(Modifier::ITALIC)),
            Tag::Strong => self.push_style(Style::new().add_modifier(Modifier::BOLD)),
            Tag::Strikethrough => self.push_style(Style::new().add_modifier(Modifier::CROSSED_OUT)),
            Tag::Link { .. } | Tag::Image { .. } => self.push_style(self.theme.link),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Paragraph => {
                self.flush_inline();
                self.blank();
            }
            TagEnd::Heading(_) => {
                self.flush_inline();
                self.pop_style();
                self.blank();
            }
            TagEnd::BlockQuote(_) => {
                self.flush_inline();
                self.quotes = self.quotes.saturating_sub(1);
                self.blank();
            }
            TagEnd::CodeBlock => self.flush_code(),
            TagEnd::List(_) => {
                self.lists.pop();
                self.blank();
            }
            TagEnd::Item => {
                self.flush_inline();
                self.indents.pop();
                self.pending_marker = None;
            }
            TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Link
            | TagEnd::Image => self.pop_style(),
            _ => {}
        }
    }

    /// Total columns consumed by the quote gutter plus list indentation.
    fn prefix_width(&self) -> usize {
        self.quotes * 2 + self.indents.iter().sum::<usize>()
    }

    /// Build the leading spans for a wrapped line. On the first line of a list
    /// item this emits the pending marker; otherwise it is blank padding.
    fn prefix(&mut self, first: bool) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        for _ in 0..self.quotes {
            spans.push(Span::styled("│ ", self.theme.quote));
        }
        let indent: usize = self.indents.iter().sum();
        match (first, self.pending_marker.take()) {
            (true, Some(marker)) => {
                let pad = indent.saturating_sub(marker.width());
                if pad > 0 {
                    spans.push(Span::raw(" ".repeat(pad)));
                }
                spans.push(marker);
            }
            _ if indent > 0 => spans.push(Span::raw(" ".repeat(indent))),
            _ => {}
        }
        spans
    }

    /// Wrap and emit the accumulated inline glyphs as one block of lines.
    // The wrap macros reset `line_w`/`has_word` at boundaries where the reset
    // is sometimes the last write before the block ends; that is intentional.
    #[allow(unused_assignments)]
    fn flush_inline(&mut self) {
        if self.inline.is_empty() {
            return;
        }
        let glyphs = std::mem::take(&mut self.inline);
        // Inside a block quote, tint plain body text with the quote style.
        let quoted = self.quotes > 0;
        let theme = self.theme.clone();
        let pw = self.prefix_width();
        // Absolute column at which a line must wrap (leaves >= 1 text column).
        let limit = self.width.max(pw + 1);

        let mut line = self.prefix(true);
        let mut line_w = pw;
        let mut has_word = false;
        // The word currently being assembled, awaiting flush to the line.
        let mut word: Vec<Glyph> = Vec::new();

        // Macros (not closures) so they can borrow `self` freely per use.
        macro_rules! flush_word {
            () => {{
                if !word.is_empty() {
                    if has_word {
                        line.push(Span::raw(" "));
                        line_w += 1;
                    }
                    for span in group_spans(&word, quoted, &theme) {
                        line_w += span.content.chars().count();
                        line.push(span);
                    }
                    word.clear();
                    has_word = true;
                }
            }};
        }
        macro_rules! start_line {
            () => {{
                self.out.push(Line::from(std::mem::take(&mut line)));
                line = self.prefix(false);
                line_w = pw;
                has_word = false;
            }};
        }

        for (ch, st) in glyphs {
            match ch {
                '\n' => {
                    flush_word!();
                    start_line!();
                }
                ' ' => {
                    // `word` holds the just-completed word; wrap if it overflows.
                    if has_word && line_w + 1 + word.len() > limit {
                        let saved = std::mem::take(&mut word);
                        start_line!();
                        word = saved;
                    }
                    flush_word!();
                }
                _ => word.push((ch, st)),
            }
        }
        if has_word && line_w + 1 + word.len() > limit {
            let saved = std::mem::take(&mut word);
            start_line!();
            word = saved;
        }
        flush_word!();
        self.out.push(Line::from(line));
    }

    fn flush_code(&mut self) {
        let Some(code) = self.code.take() else { return };
        let style = self.theme.code_block;
        let src = code.strip_suffix('\n').unwrap_or(&code).to_string();
        for raw in src.split('\n') {
            let mut spans = self.prefix(false);
            spans.push(Span::styled(raw.to_string(), style));
            self.out.push(Line::from(spans));
        }
        self.blank();
    }

    fn rule(&mut self) {
        self.flush_inline();
        let dashes = "─".repeat(self.width.max(1));
        self.out
            .push(Line::from(Span::styled(dashes, self.theme.rule)));
        self.blank();
    }

    /// Append a blank separator line, coalescing consecutive blanks and never
    /// leading with one.
    fn blank(&mut self) {
        let last_blank = self
            .out
            .last()
            .is_none_or(|l| l.spans.iter().all(|s| s.content.trim().is_empty()));
        if !last_blank {
            self.out.push(Line::default());
        }
    }
}

/// Group a word's glyphs into styled spans, merging runs of equal style.
fn group_spans(word: &[Glyph], quoted: bool, theme: &Theme) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut buf = String::new();
    let mut cur: Option<Style> = None;
    for &(ch, mut st) in word {
        // In a block quote, default body text picks up the quote tint.
        if quoted && st == theme.text {
            st = theme.quote;
        }
        match cur {
            Some(s) if s == st => buf.push(ch),
            Some(s) => {
                spans.push(Span::styled(std::mem::take(&mut buf), s));
                buf.push(ch);
                cur = Some(st);
            }
            None => {
                buf.push(ch);
                cur = Some(st);
            }
        }
    }
    if let Some(s) = cur {
        spans.push(Span::styled(buf, s));
    }
    spans
}

fn heading_index(level: HeadingLevel) -> usize {
    match level {
        HeadingLevel::H1 => 0,
        HeadingLevel::H2 => 1,
        HeadingLevel::H3 => 2,
        HeadingLevel::H4 => 3,
        HeadingLevel::H5 => 4,
        HeadingLevel::H6 => 5,
    }
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
    fn matches_by_extension() {
        let v = MarkdownView::new();
        assert!(v.matches(std::path::Path::new("README.MD")));
        assert!(v.matches(std::path::Path::new("a/b/notes.markdown")));
        assert!(!v.matches(std::path::Path::new("main.rs")));
    }

    #[test]
    fn heading_gets_hash_prefix_and_style() {
        let t = MarkdownView::new().render("# Title", 80);
        assert_eq!(plain(&t)[0], "# Title");
        let first = &t.lines[0];
        assert!(first.spans[0].style.add_modifier.contains(Modifier::BOLD));
    }

    #[test]
    fn wraps_to_width() {
        // 12 one-char words; at width 10 no line may exceed 10 columns.
        let src = "aa bb cc dd ee ff gg hh ii jj kk ll";
        let t = MarkdownView::new().render(src, 10);
        for line in plain(&t) {
            assert!(line.chars().count() <= 10, "overflow: {line:?}");
        }
        // All words survive the wrap.
        let joined: String = plain(&t).join(" ");
        for w in src.split(' ') {
            assert!(joined.contains(w));
        }
    }

    #[test]
    fn adjacent_styles_join_without_space() {
        // "**foo**bar" must render as one word "foobar", not "foo bar".
        let t = MarkdownView::new().render("**foo**bar", 80);
        assert!(plain(&t).iter().any(|l| l.contains("foobar")));
    }

    #[test]
    fn bullet_list_marked_and_indented() {
        let t = MarkdownView::new().render("- one\n- two", 80);
        let lines = plain(&t);
        assert!(lines.iter().any(|l| l.starts_with("• one")));
        assert!(lines.iter().any(|l| l.starts_with("• two")));
    }

    #[test]
    fn block_quote_gets_gutter() {
        let t = MarkdownView::new().render("> quoted", 80);
        assert!(plain(&t).iter().any(|l| l.starts_with("│ ")));
    }

    #[test]
    fn code_block_kept_verbatim() {
        let t = MarkdownView::new().render("```\nfn main() {}\n```", 80);
        assert!(plain(&t).iter().any(|l| l == "fn main() {}"));
    }
}
