# tui-view

A [ratatui](https://ratatui.rs) widget library for viewing text files through
**pluggable, per-format views**. Ships with Markdown, JSON, plain-text, hex,
and Tree-sitter source-code views; add your own by implementing one trait.

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
- **Hex** (`hex` feature) — offset / hex / ASCII dump. Claims any binary
  content through `ViewRegistry::find_for`, whatever the extension says, so no
  file is unopenable.
- **Source code** (one `lang-*` feature per language) — parses with the
  language's [Tree-sitter](https://tree-sitter.github.io) grammar and paints
  from its `highlights.scm`. Reskin via
  `TreeSitterView::with_theme(language, SyntaxTheme { .. })`.

Everything but the source-code view is on by default. Use
`default-features = false` (plus the features you want) to trim what compiles.

## Source code highlighting

Grammars are C parsers, so each language is its own opt-in feature:

```toml
tui-view = { version = "0.1", features = ["lang-rust", "lang-python"] }
# ...or every bundled grammar:
tui-view = { version = "0.1", features = ["languages"] }
```

Each feature registers one view per language:

| Feature | Views (extensions) |
|---------|--------------------|
| `lang-rust` | Rust (`rs`) |
| `lang-python` | Python (`py`, `pyi`) |
| `lang-javascript` | JavaScript (`js`, `mjs`, `cjs`), JSX (`jsx`) |
| `lang-typescript` | TypeScript (`ts`, `mts`, `cts`), TSX (`tsx`) |
| `lang-go` | Go (`go`) |
| `lang-c` | C (`c`, `h`) |
| `lang-bash` | Shell (`sh`, `bash`, `zsh`) |
| `lang-toml` | TOML (`toml`) |
| `lang-yaml` | YAML (`yaml`, `yml`) |
| `lang-html` | HTML (`html`, `htm`) |
| `lang-css` | CSS (`css`) |

`lang-typescript` implies `lang-javascript`: the TypeScript queries only cover
what TypeScript adds, so the JavaScript ones go in front of them.

Enabled languages are registered by `ViewRegistry::with_defaults`, after
Markdown and JSON so those keep their extensions.

Any other grammar crate works without a change here — add the grammar to your
own `Cargo.toml`, then describe it and register the view:

```rust
use std::sync::Arc;
use tui_view::plugins::treesitter::{SyntaxLanguage, TreeSitterView};
use tui_view::ViewRegistry;

let zig = SyntaxLanguage::new(
    "Zig",
    &["zig"],
    || tree_sitter_zig::LANGUAGE.into(),
    tree_sitter_zig::HIGHLIGHTS_QUERY,
);

let mut registry = ViewRegistry::with_defaults();
registry.register(Arc::new(TreeSitterView::new(zig)));
```

Optional queries are added with `.with_injections(..)` (languages embedded in
this one) and `.with_locals(..)` (local definitions, so a variable is not
mistaken for a function).

Highlighting never hides content: a grammar whose queries fail to compile, or
a file the highlighter cannot parse, falls back to unstyled text, and broken
syntax still renders every line — Tree-sitter recovers from parse errors.

## Example viewer

```
cargo run --example viewer -- examples/files/sample.md
cargo run --example viewer -- examples/files/sample.json
cargo run --example viewer -- examples/files/sample.txt
cargo run --features languages --example viewer -- examples/files/sample.rs
```

Keys: `↑`/`↓` or `j`/`k` scroll · `PgUp`/`PgDn` or `Space` page · `g`/`G`
top/bottom · `q` quit.

## Feature flags

- `markdown` (default) — the Markdown `FormatView`.
- `json` (default) — the JSON `FormatView`.
- `plaintext` (default) — the plain-text `FormatView`.
- `hex` (default) — the hex-dump `FormatView`.
- `treesitter` — the source-code `FormatView` with no grammars; useful only
  when you bring your own.
- `lang-*` — one Tree-sitter grammar each; each implies `treesitter`.
- `languages` — every bundled grammar.

## License

MIT OR Apache-2.0
