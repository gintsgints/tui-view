//! A Tree-sitter [`FormatView`] for source code.
//!
//! One view handles one language: it parses the file with that language's
//! [Tree-sitter](https://tree-sitter.github.io) grammar, runs the grammar's
//! `highlights.scm` query over the tree, and paints each captured range with
//! the matching [`SyntaxTheme`] style. Anything the query does not claim keeps
//! the theme's default text style, so the file always renders in full.
//!
//! The bundled grammars live in [`languages`], each behind its own `lang-*`
//! cargo feature. [`views`] builds a view per enabled language:
//!
//! ```no_run
//! use tui_view::{plugins::treesitter, ViewRegistry};
//!
//! let mut registry = ViewRegistry::new();
//! for view in treesitter::views() {
//!     registry.register(view);
//! }
//! ```
//!
//! To highlight a language that is not bundled, describe it with
//! [`SyntaxLanguage`] and hand it to [`TreeSitterView::new`] — no change to
//! this crate is needed:
//!
//! ```ignore
//! let lang = SyntaxLanguage::new(
//!     "Zig",
//!     &["zig"],
//!     || tree_sitter_zig::LANGUAGE.into(),
//!     tree_sitter_zig::HIGHLIGHTS_QUERY,
//! );
//! registry.register(Arc::new(TreeSitterView::new(lang)));
//! ```
//!
//! Parsing failures are not fatal: a grammar whose queries do not compile, or
//! a file the highlighter gives up on, falls back to unstyled text.

pub mod languages;

use std::borrow::Cow;
use std::fmt;
use std::sync::{Arc, OnceLock};

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use tree_sitter::Language;
use tree_sitter_highlight::{HighlightConfiguration, HighlightEvent, Highlighter};

use crate::view::FormatView;
use crate::wrap::wrap_spans;

/// Capture names recognised by the highlighter, in index order.
///
/// A grammar query capture matches the most specific entry that is a prefix of
/// it, so `@function.method.builtin` resolves to `function.builtin` here. A
/// capture matching nothing in this list is drawn with
/// [`SyntaxTheme::text`].
pub const HIGHLIGHT_NAMES: &[&str] = &[
    "attribute",
    "comment",
    "constant",
    "constant.builtin",
    "constructor",
    "embedded",
    "escape",
    "function",
    "function.builtin",
    "function.method",
    "keyword",
    "label",
    "module",
    "number",
    "operator",
    "property",
    "punctuation",
    "punctuation.bracket",
    "punctuation.delimiter",
    "punctuation.special",
    "string",
    "string.escape",
    "string.special",
    "tag",
    "type",
    "type.builtin",
    "variable",
    "variable.builtin",
    "variable.member",
    "variable.parameter",
];

/// A language the [`TreeSitterView`] can highlight: a grammar plus its
/// queries.
///
/// Build one with [`new`](Self::new) and add the optional queries with
/// [`with_injections`](Self::with_injections) and
/// [`with_locals`](Self::with_locals). The bundled languages in [`languages`]
/// are built exactly this way.
#[derive(Debug, Clone)]
pub struct SyntaxLanguage {
    /// Human-readable name, e.g. `"Rust"`. Doubles as the view's
    /// [`name`](FormatView::name).
    pub name: &'static str,
    /// Lower-case extensions handled, without the leading dot.
    pub extensions: &'static [&'static str],
    /// Loads the grammar, e.g. `|| tree_sitter_rust::LANGUAGE.into()`.
    pub grammar: fn() -> Language,
    /// The grammar's `highlights.scm`. Without it nothing is styled.
    pub highlights: Cow<'static, str>,
    /// The grammar's `injections.scm`, for languages embedded in this one.
    /// Empty when there are none.
    pub injections: Cow<'static, str>,
    /// The grammar's `locals.scm`, which tracks local definitions so a
    /// variable is not mistaken for a function. Empty when unused.
    pub locals: Cow<'static, str>,
}

impl SyntaxLanguage {
    /// A language with only a highlights query.
    #[must_use]
    pub fn new(
        name: &'static str,
        extensions: &'static [&'static str],
        grammar: fn() -> Language,
        highlights: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            name,
            extensions,
            grammar,
            highlights: highlights.into(),
            injections: Cow::Borrowed(""),
            locals: Cow::Borrowed(""),
        }
    }

    /// Add the injections query.
    #[must_use]
    pub fn with_injections(mut self, query: impl Into<Cow<'static, str>>) -> Self {
        self.injections = query.into();
        self
    }

    /// Add the locals query.
    #[must_use]
    pub fn with_locals(mut self, query: impl Into<Cow<'static, str>>) -> Self {
        self.locals = query.into();
        self
    }
}

/// Renders source code with Tree-sitter syntax highlighting.
///
/// One view per language; see [`views`] for the bundled set.
pub struct TreeSitterView {
    language: SyntaxLanguage,
    theme: SyntaxTheme,
    /// Compiled lazily on first render — building the queries is the
    /// expensive part — and `None` when the grammar's queries do not compile.
    configured: OnceLock<Option<Configured>>,
}

/// A compiled grammar plus the styles for its recognised captures.
struct Configured {
    config: HighlightConfiguration,
    /// One style per entry of [`HIGHLIGHT_NAMES`], indexed by the highlight
    /// index the highlighter reports.
    styles: Vec<Style>,
}

impl TreeSitterView {
    /// A view for `language` with the default [`SyntaxTheme`].
    #[must_use]
    pub fn new(language: SyntaxLanguage) -> Self {
        Self::with_theme(language, SyntaxTheme::default())
    }

    /// A view for `language` with a custom [`SyntaxTheme`].
    #[must_use]
    pub fn with_theme(language: SyntaxLanguage, theme: SyntaxTheme) -> Self {
        Self {
            language,
            theme,
            configured: OnceLock::new(),
        }
    }

    /// The language this view highlights.
    #[must_use]
    pub fn language(&self) -> &SyntaxLanguage {
        &self.language
    }

    /// The compiled grammar, or `None` if its queries do not compile.
    fn configured(&self) -> Option<&Configured> {
        self.configured
            .get_or_init(|| {
                let mut config = HighlightConfiguration::new(
                    (self.language.grammar)(),
                    self.language.name,
                    &self.language.highlights,
                    &self.language.injections,
                    &self.language.locals,
                )
                .ok()?;
                config.configure(HIGHLIGHT_NAMES);
                let styles = HIGHLIGHT_NAMES
                    .iter()
                    .map(|name| self.theme.style_for(name))
                    .collect();
                Some(Configured { config, styles })
            })
            .as_ref()
    }

    /// Highlighted, width-wrapped lines, or `None` if the file cannot be
    /// highlighted and the caller should fall back to plain text.
    fn highlighted(&self, content: &str, width: usize) -> Option<Vec<Line<'static>>> {
        let configured = self.configured()?;
        let mut highlighter = Highlighter::new();
        let events = highlighter
            .highlight(&configured.config, content.as_bytes(), None, |_| None)
            .ok()?;

        let mut out: Vec<Line<'static>> = Vec::new();
        let mut row: Vec<Span<'static>> = Vec::new();
        // Nested captures: the innermost one wins, so the top of the stack is
        // the style in force.
        let mut stack: Vec<usize> = Vec::new();
        for event in events {
            match event.ok()? {
                HighlightEvent::HighlightStart(highlight) => stack.push(highlight.0),
                HighlightEvent::HighlightEnd => {
                    stack.pop();
                }
                HighlightEvent::Source { start, end } => {
                    let style = stack
                        .last()
                        .and_then(|i| configured.styles.get(*i))
                        .copied()
                        .unwrap_or(self.theme.text);
                    push_source(content.get(start..end)?, style, width, &mut row, &mut out);
                }
            }
        }
        flush_row(&mut row, width, &mut out);
        drop_trailing_blank(content, &mut out);
        Some(out)
    }
}

impl FormatView for TreeSitterView {
    fn name(&self) -> &'static str {
        self.language.name
    }

    fn extensions(&self) -> &[&str] {
        self.language.extensions
    }

    fn render(&self, content: &str, width: u16) -> Text<'static> {
        let width = width.max(1) as usize;
        let lines = self
            .highlighted(content, width)
            .unwrap_or_else(|| unstyled(content, width, self.theme.text));
        Text::from(lines)
    }
}

impl fmt::Debug for TreeSitterView {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The compiled grammar is a pile of query internals with nothing
        // useful to print, so it stays out.
        f.debug_struct("TreeSitterView")
            .field("language", &self.language.name)
            .field("theme", &self.theme)
            .finish_non_exhaustive()
    }
}

impl Clone for TreeSitterView {
    /// Clones the language and theme. The compiled grammar is not shared —
    /// the clone compiles its own on first render.
    fn clone(&self) -> Self {
        Self::with_theme(self.language.clone(), self.theme.clone())
    }
}

/// A [`TreeSitterView`] for every language bundled by the enabled `lang-*`
/// features, ready to hand to [`ViewRegistry::register`].
///
/// [`ViewRegistry::register`]: crate::ViewRegistry::register
#[must_use]
pub fn views() -> Vec<Arc<dyn FormatView>> {
    languages::all()
        .into_iter()
        .map(|lang| Arc::new(TreeSitterView::new(lang)) as Arc<dyn FormatView>)
        .collect()
}

/// Styling for the [`TreeSitterView`], one style per broad token category.
///
/// A capture name is matched on its first dot-separated component, so
/// `string.special.path` and `string` share [`string`](Self::string). A
/// category no grammar in use emits simply never shows up.
#[derive(Debug, Clone)]
pub struct SyntaxTheme {
    /// Anything the highlights query does not capture.
    pub text: Style,
    /// Comments and documentation comments.
    pub comment: Style,
    /// Keywords, and the `include`/`repeat`/`conditional` families.
    pub keyword: Style,
    /// String and character literals.
    pub string: Style,
    /// Escape sequences inside strings.
    pub escape: Style,
    /// Numeric literals.
    pub number: Style,
    /// Constants, including `true`/`false`/`null` in most grammars.
    pub constant: Style,
    /// Function and method names, at definition and call sites.
    pub function: Style,
    /// Types, classes, and constructors.
    pub type_name: Style,
    /// Variables and parameters.
    pub variable: Style,
    /// Object fields and struct members.
    pub property: Style,
    /// Operators.
    pub operator: Style,
    /// Brackets, delimiters, and other punctuation.
    pub punctuation: Style,
    /// Attributes, annotations, and decorators.
    pub attribute: Style,
    /// Markup tags — HTML elements, for instance.
    pub tag: Style,
}

impl Default for SyntaxTheme {
    fn default() -> Self {
        Self {
            text: Style::new(),
            comment: Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            keyword: Style::new().fg(Color::Magenta),
            string: Style::new().fg(Color::Green),
            escape: Style::new().fg(Color::LightGreen),
            number: Style::new().fg(Color::Yellow),
            constant: Style::new().fg(Color::LightYellow),
            function: Style::new().fg(Color::Blue),
            type_name: Style::new().fg(Color::Cyan),
            variable: Style::new(),
            property: Style::new().fg(Color::LightCyan),
            operator: Style::new().fg(Color::Gray),
            punctuation: Style::new().fg(Color::Gray),
            attribute: Style::new().fg(Color::LightMagenta),
            tag: Style::new().fg(Color::Red),
        }
    }
}

impl SyntaxTheme {
    /// The style for a capture name such as `function.builtin`.
    ///
    /// Unknown categories get [`text`](Self::text), so a grammar with its own
    /// capture vocabulary still renders.
    #[must_use]
    pub fn style_for(&self, capture: &str) -> Style {
        match capture.split('.').next().unwrap_or(capture) {
            "comment" => self.comment,
            "keyword" | "conditional" | "repeat" | "include" | "exception" | "storageclass" => {
                self.keyword
            }
            "string" | "character" => self.string,
            "escape" => self.escape,
            "number" | "float" | "boolean" => self.number,
            "constant" => self.constant,
            "function" | "method" => self.function,
            "type" | "constructor" | "namespace" | "module" => self.type_name,
            "variable" | "parameter" | "embedded" => self.variable,
            "property" | "field" => self.property,
            "operator" => self.operator,
            "punctuation" | "delimiter" | "bracket" => self.punctuation,
            "attribute" | "annotation" | "decorator" | "label" => self.attribute,
            "tag" => self.tag,
            _ => self.text,
        }
    }
}

/// Append `text` — one highlighted run, which may span line breaks — to the
/// row being built, flushing a wrapped row at every newline.
fn push_source(
    text: &str,
    style: Style,
    width: usize,
    row: &mut Vec<Span<'static>>,
    out: &mut Vec<Line<'static>>,
) {
    let mut rest = text;
    while let Some(nl) = rest.find('\n') {
        push_span(&rest[..nl], style, row);
        flush_row(row, width, out);
        rest = &rest[nl + 1..];
    }
    push_span(rest, style, row);
}

fn push_span(text: &str, style: Style, row: &mut Vec<Span<'static>>) {
    if !text.is_empty() {
        row.push(Span::styled(text.to_owned(), style));
    }
}

/// Wrap the finished row to `width` and push it, leaving `row` empty.
///
/// A CRLF file leaves the `\r` at the end of the row; drop it here rather than
/// rendering a stray control character.
fn flush_row(row: &mut Vec<Span<'static>>, width: usize, out: &mut Vec<Line<'static>>) {
    if let Some(last) = row.last_mut() {
        if let Some(trimmed) = last.content.strip_suffix('\r') {
            last.content = Cow::Owned(trimmed.to_owned());
        }
    }
    wrap_spans(std::mem::take(row), width, out);
}

/// Unstyled, width-wrapped fallback for content that cannot be highlighted.
fn unstyled(content: &str, width: usize, style: Style) -> Vec<Line<'static>> {
    let mut out = Vec::new();
    for source_line in content.split('\n') {
        let source_line = source_line.strip_suffix('\r').unwrap_or(source_line);
        wrap_spans(
            vec![Span::styled(source_line.to_owned(), style)],
            width,
            &mut out,
        );
    }
    drop_trailing_blank(content, &mut out);
    out
}

/// A trailing newline ends the last line rather than starting a new one; drop
/// the empty row it would otherwise leave behind.
fn drop_trailing_blank(content: &str, out: &mut Vec<Line<'static>>) {
    if content.ends_with('\n') && out.last().is_some_and(|line| line.width() == 0) {
        out.pop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(feature = "lang-rust")]
    fn plain(text: &Text) -> Vec<String> {
        text.lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect())
            .collect()
    }

    /// Every style used across a rendered line, in order.
    #[cfg(feature = "lang-rust")]
    fn styles(text: &Text, line: usize) -> Vec<(String, Style)> {
        text.lines[line]
            .spans
            .iter()
            .map(|s| (s.content.to_string(), s.style))
            .collect()
    }

    #[cfg(feature = "lang-rust")]
    fn rust_view() -> TreeSitterView {
        TreeSitterView::new(languages::rust())
    }

    #[test]
    fn every_bundled_grammar_compiles() {
        for language in languages::all() {
            let name = language.name;
            let view = TreeSitterView::new(language);
            assert!(
                view.configured().is_some(),
                "{name}: highlight queries failed to compile"
            );
        }
    }

    #[test]
    fn bundled_extensions_are_lower_case_and_dotless() {
        for language in languages::all() {
            for ext in language.extensions {
                assert_eq!(*ext, ext.to_ascii_lowercase(), "{}", language.name);
                assert!(!ext.starts_with('.'), "{}", language.name);
            }
        }
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn matches_extension() {
        assert!(rust_view().matches(std::path::Path::new("main.RS")));
        assert!(!rust_view().matches(std::path::Path::new("main.py")));
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn preserves_source_verbatim() {
        let src = "fn main() {\n    // hi\n    let x = 1;\n}\n";
        let lines = plain(&rust_view().render(src, 80));
        assert_eq!(
            lines,
            vec!["fn main() {", "    // hi", "    let x = 1;", "}"]
        );
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn styles_keywords_comments_and_numbers() {
        let theme = SyntaxTheme::default();
        let view = rust_view();
        let text = view.render("// note\nlet x = 42;\n", 80);

        let comment = styles(&text, 0);
        assert_eq!(comment[0].0, "// note");
        assert_eq!(comment[0].1, theme.comment);

        let code = styles(&text, 1);
        assert_eq!(
            code.iter()
                .find(|(content, _)| content == "let")
                .map(|(_, style)| *style),
            Some(theme.keyword)
        );
        // Rust's query captures integer literals as `constant.builtin`;
        // grammars that use `number` land on `theme.number` instead.
        assert_eq!(
            code.iter()
                .find(|(content, _)| content == "42")
                .map(|(_, style)| *style),
            Some(theme.constant)
        );
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn wraps_long_lines_and_keeps_styles() {
        let view = rust_view();
        let text = view.render("let value = \"aaaaaaaaaaaaaaaaaaaa\";\n", 10);
        let lines = plain(&text);
        for line in &lines {
            assert!(line.chars().count() <= 10, "overflow: {line:?}");
        }
        assert_eq!(lines.concat(), "let value = \"aaaaaaaaaaaaaaaaaaaa\";");
        // The wrapped string literal keeps the string style on both rows.
        let string_style = SyntaxTheme::default().string;
        assert!(
            text.lines
                .iter()
                .filter(|l| l.spans.iter().any(|s| s.style == string_style))
                .count()
                >= 2
        );
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn strips_carriage_returns() {
        let lines = plain(&rust_view().render("let a = 1;\r\nlet b = 2;\r\n", 80));
        assert_eq!(lines, vec!["let a = 1;", "let b = 2;"]);
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn broken_source_still_renders_every_line() {
        // Tree-sitter recovers from errors; the text must survive intact.
        let src = "fn ((( {\nlet x = ;\n";
        let lines = plain(&rust_view().render(src, 80));
        assert_eq!(lines, vec!["fn ((( {", "let x = ;"]);
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn custom_theme_is_applied() {
        let theme = SyntaxTheme {
            keyword: Style::new().fg(Color::Red),
            ..SyntaxTheme::default()
        };
        let view = TreeSitterView::with_theme(languages::rust(), theme);
        let text = view.render("let x = 1;", 80);
        assert!(text.lines[0]
            .spans
            .iter()
            .any(|s| s.content == "let" && s.style == Style::new().fg(Color::Red)));
    }

    #[test]
    fn unknown_captures_fall_back_to_text_style() {
        let theme = SyntaxTheme::default();
        assert_eq!(theme.style_for("not.a.category"), theme.text);
        assert_eq!(theme.style_for("string.special.path"), theme.string);
        assert_eq!(theme.style_for("function.builtin"), theme.function);
    }

    #[test]
    #[cfg(feature = "lang-rust")]
    fn a_grammar_with_broken_queries_falls_back_to_plain_text() {
        // `(nonsense` is not a valid query, so no configuration is built.
        let language = SyntaxLanguage::new(
            "Broken",
            &["broken"],
            || tree_sitter_rust::LANGUAGE.into(),
            "(nonsense",
        );
        let view = TreeSitterView::new(language);
        assert!(view.configured().is_none());
        assert_eq!(plain(&view.render("let x = 1;\n", 80)), vec!["let x = 1;"]);
    }
}
