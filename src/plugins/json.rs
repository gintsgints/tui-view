//! A JSON [`FormatView`].
//!
//! Parses the document (via `serde_json`) and re-renders it pretty-printed with
//! syntax highlighting: object keys, strings, numbers, booleans, `null`, and
//! punctuation each get their own style. Invalid JSON falls back to the raw
//! text under an error banner so the content is never hidden.

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span, Text};
use serde_json::Value;

use crate::view::FormatView;

/// Renders JSON documents with syntax highlighting.
#[derive(Debug, Clone, Default)]
pub struct JsonView {
    theme: JsonTheme,
}

impl JsonView {
    /// A JSON view with the default [`JsonTheme`].
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A JSON view with a custom [`JsonTheme`].
    #[must_use]
    pub fn with_theme(theme: JsonTheme) -> Self {
        Self { theme }
    }
}

impl FormatView for JsonView {
    fn name(&self) -> &'static str {
        "JSON"
    }

    fn extensions(&self) -> &[&str] {
        &["json"]
    }

    fn render(&self, content: &str, _width: u16) -> Text<'static> {
        match serde_json::from_str::<Value>(content) {
            Ok(value) => {
                let mut out = Vec::new();
                emit(&mut out, 0, Vec::new(), &value, false, &self.theme);
                Text::from(out)
            }
            Err(err) => fallback(content, &err, &self.theme),
        }
    }
}

/// Styling for the [`JsonView`]. Adjust individual fields to reskin.
#[derive(Debug, Clone)]
pub struct JsonTheme {
    /// Object keys.
    pub key: Style,
    /// String values.
    pub string: Style,
    /// Numeric values.
    pub number: Style,
    /// `true` / `false`.
    pub boolean: Style,
    /// `null`.
    pub null: Style,
    /// Structural punctuation: braces, brackets, colons, commas.
    pub punct: Style,
    /// Banner shown when the document does not parse.
    pub error: Style,
}

impl Default for JsonTheme {
    fn default() -> Self {
        Self {
            key: Style::new().fg(Color::Cyan),
            string: Style::new().fg(Color::Green),
            number: Style::new().fg(Color::Yellow),
            boolean: Style::new().fg(Color::Magenta),
            null: Style::new()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
            punct: Style::new().fg(Color::Gray),
            error: Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
        }
    }
}

fn indent(level: usize) -> String {
    "  ".repeat(level)
}

fn punct(text: &'static str, theme: &JsonTheme) -> Span<'static> {
    Span::styled(text, theme.punct)
}

/// Emit `value` as pretty-printed lines. `prefix` holds spans already placed on
/// the opening line (indent, and for object members the key and `: `). `comma`
/// requests a trailing comma after the value's closing token.
fn emit(
    out: &mut Vec<Line<'static>>,
    level: usize,
    mut prefix: Vec<Span<'static>>,
    value: &Value,
    comma: bool,
    theme: &JsonTheme,
) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            prefix.push(punct("{", theme));
            out.push(Line::from(prefix));
            let last = map.len() - 1;
            for (i, (k, v)) in map.iter().enumerate() {
                let mut child = vec![Span::raw(indent(level + 1))];
                child.push(Span::styled(
                    Value::String(k.clone()).to_string(),
                    theme.key,
                ));
                child.push(punct(": ", theme));
                emit(out, level + 1, child, v, i != last, theme);
            }
            close(out, level, "}", comma, theme);
        }
        Value::Array(arr) if !arr.is_empty() => {
            prefix.push(punct("[", theme));
            out.push(Line::from(prefix));
            let last = arr.len() - 1;
            for (i, v) in arr.iter().enumerate() {
                let child = vec![Span::raw(indent(level + 1))];
                emit(out, level + 1, child, v, i != last, theme);
            }
            close(out, level, "]", comma, theme);
        }
        _ => {
            let (text, style) = scalar(value, theme);
            prefix.push(Span::styled(text, style));
            if comma {
                prefix.push(punct(",", theme));
            }
            out.push(Line::from(prefix));
        }
    }
}

/// Push the closing token for a container at `level`.
fn close(
    out: &mut Vec<Line<'static>>,
    level: usize,
    bracket: &'static str,
    comma: bool,
    theme: &JsonTheme,
) {
    let mut spans = vec![Span::raw(indent(level)), punct(bracket, theme)];
    if comma {
        spans.push(punct(",", theme));
    }
    out.push(Line::from(spans));
}

/// The literal text and style for a scalar (or empty container) value.
fn scalar(value: &Value, theme: &JsonTheme) -> (String, Style) {
    match value {
        Value::Null => ("null".to_string(), theme.null),
        Value::Bool(_) => (value.to_string(), theme.boolean),
        Value::Number(_) => (value.to_string(), theme.number),
        Value::String(_) => (value.to_string(), theme.string),
        Value::Object(_) => ("{}".to_string(), theme.punct),
        Value::Array(_) => ("[]".to_string(), theme.punct),
    }
}

/// Render invalid JSON as raw text under an error banner.
fn fallback(content: &str, err: &serde_json::Error, theme: &JsonTheme) -> Text<'static> {
    let mut out = vec![
        Line::from(Span::styled(format!("invalid JSON: {err}"), theme.error)),
        Line::default(),
    ];
    for line in content.split('\n') {
        let line = line.strip_suffix('\r').unwrap_or(line);
        out.push(Line::from(line.to_string()));
    }
    Text::from(out)
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
        assert!(JsonView::new().matches(std::path::Path::new("config.JSON")));
        assert!(!JsonView::new().matches(std::path::Path::new("a.md")));
    }

    #[test]
    fn pretty_prints_nested() {
        // serde_json orders object keys alphabetically: name, nil, nums, ok.
        let src = r#"{"name":"x","nums":[1,2],"ok":true,"nil":null}"#;
        let lines = plain(&JsonView::new().render(src, 80));
        assert_eq!(lines[0], "{");
        assert!(lines.iter().any(|l| l == r#"  "name": "x","#));
        assert!(lines.iter().any(|l| l == r#"  "nil": null,"#));
        assert!(lines.iter().any(|l| l == "  \"nums\": ["));
        assert!(lines.iter().any(|l| l.trim() == "1,"));
        assert!(lines.iter().any(|l| l.trim() == "2"));
        assert!(lines.iter().any(|l| l == r#"  "ok": true"#));
        assert_eq!(lines.last().unwrap(), "}");
    }

    #[test]
    fn empty_containers_inline() {
        let lines = plain(&JsonView::new().render(r#"{"a":{},"b":[]}"#, 80));
        assert!(lines.iter().any(|l| l == r#"  "a": {},"#));
        assert!(lines.iter().any(|l| l == r#"  "b": []"#));
    }

    #[test]
    fn invalid_json_falls_back_to_raw() {
        let lines = plain(&JsonView::new().render("{not json", 80));
        assert!(lines[0].starts_with("invalid JSON:"));
        assert!(lines.iter().any(|l| l == "{not json"));
    }
}
