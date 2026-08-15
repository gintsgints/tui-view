# tui-view

A [ratatui](https://ratatui.rs) widget library for viewing text files through
**pluggable, per-format views**. Ships with Markdown, JSON, and plain-text
views; add your own by implementing one trait.

## Architecture

| Piece | Role |
|-------|------|
| `FormatView` | The plugin trait. Turns raw text into styled, width-wrapped `Text`. |
| `ViewRegistry` | Ordered set of views; picks one for a file by extension. |
| `ViewState` | Holds content, chosen view, render cache, scroll position. |
| `TuiView` | Scrollable `StatefulWidget` that draws a `ViewState`. |

A view renders once per width (first draw / resize) and is cached, so scrolling
is free. Views pre-wrap to the target width, so the widget scrolls by exact
line count.

## Usage

```rust
use std::path::Path;
use ratatui::widgets::Block;
use tui_view::{TuiView, ViewState, ViewRegistry};

let registry = ViewRegistry::with_defaults();
let content = std::fs::read_to_string("README.md")?;
let mut state = ViewState::from_path(Path::new("README.md"), content, &registry).unwrap();

// draw loop:
terminal.draw(|f| {
    let w = TuiView::new().block(Block::bordered().title("README.md"));
    f.render_stateful_widget(w, f.area(), &mut state);
})?;

// key handling:
state.scroll_down(1);
state.page_up();
state.scroll_to_bottom();
```

## Adding a format

```rust
use std::sync::Arc;
use ratatui::text::Text;
use tui_view::{FormatView, ViewRegistry};

struct PlainText;
impl FormatView for PlainText {
    fn name(&self) -> &'static str { "Plain text" }
    fn extensions(&self) -> &[&str] { &["txt", "log"] }
    fn render(&self, content: &str, _width: u16) -> Text<'static> {
        Text::from(content.to_owned())
    }
}

let mut registry = ViewRegistry::with_defaults();
registry.register(Arc::new(PlainText));
```

Register more specific views first — the first matching view wins.

## Bundled views

- **Markdown** (`markdown` feature) — headings, emphasis / strong /
  strikethrough, inline code and math, links and images, bullet + ordered
  lists (nested), task-list checkboxes, block quotes, fenced/indented code
  blocks, thematic breaks. Reskin via `MarkdownView::with_theme(Theme { .. })`.
- **JSON** (`json` feature) — parses and pretty-prints with syntax
  highlighting (keys, strings, numbers, booleans, `null`, punctuation). Invalid
  JSON falls back to raw text under an error banner. Reskin via
  `JsonView::with_theme(JsonTheme { .. })`.
- **Plain text** (`plaintext` feature) — verbatim content, source line breaks
  preserved, long lines soft-wrapped to width.

All three are on by default. Use `default-features = false` (plus the features
you want) to trim what compiles.

## Example viewer

```
cargo run --example viewer -- sample.md
```

Keys: `↑`/`↓` or `j`/`k` scroll · `PgUp`/`PgDn` or `Space` page · `g`/`G`
top/bottom · `q` quit.

## Feature flags

- `markdown` (default) — the Markdown `FormatView`.
- `json` (default) — the JSON `FormatView`.
- `plaintext` (default) — the plain-text `FormatView`.

## License

MIT OR Apache-2.0
