//! # tui-view
//!
//! A [ratatui](https://ratatui.rs) widget library for viewing text files
//! through **pluggable, per-format views**.
//!
//! The crate splits into three pieces:
//!
//! - [`FormatView`] — the plugin trait. Implement it to teach the viewer a new
//!   file format. It turns raw text into styled, width-wrapped [`Text`].
//! - [`ViewRegistry`] — an ordered set of views; picks one for a file by
//!   extension.
//! - [`TuiView`] + [`ViewState`] — a scrollable widget that renders whatever a
//!   view produced and remembers scroll position and a render cache.
//!
//! A [Markdown view](plugins::markdown::MarkdownView) ships behind the default
//! `markdown` feature and is the reference plugin.
//!
//! ## Quick start
//!
//! ```no_run
//! use std::path::Path;
//! use tui_view::{TuiView, ViewState, ViewRegistry};
//! use ratatui::widgets::Block;
//!
//! let registry = ViewRegistry::with_defaults();
//! let content = std::fs::read_to_string("README.md").unwrap();
//! let mut state =
//!     ViewState::from_path(Path::new("README.md"), content, &registry).unwrap();
//!
//! // In your draw loop:
//! # let mut terminal: ratatui::Terminal<ratatui::backend::TestBackend> = todo!();
//! terminal.draw(|f| {
//!     let widget = TuiView::new().block(Block::bordered().title("README.md"));
//!     f.render_stateful_widget(widget, f.area(), &mut state);
//! }).unwrap();
//!
//! // On key events:
//! state.scroll_down(1);
//! state.page_up();
//! ```
//!
//! ## Adding a format
//!
//! ```
//! use std::sync::Arc;
//! use ratatui::text::Text;
//! use tui_view::{FormatView, ViewRegistry};
//!
//! struct PlainText;
//! impl FormatView for PlainText {
//!     fn name(&self) -> &'static str { "Plain text" }
//!     fn extensions(&self) -> &[&str] { &["txt", "log"] }
//!     fn render(&self, content: &str, _width: u16) -> Text<'static> {
//!         Text::from(content.to_owned())
//!     }
//! }
//!
//! let mut registry = ViewRegistry::with_defaults();
//! registry.register(Arc::new(PlainText));
//! ```

mod view;
mod widget;

pub mod plugins;

pub use view::{FormatView, ViewRegistry};
pub use widget::{TuiView, ViewState};

// Re-export for convenience so downstream code needn't depend on ratatui's
// exact version to name the render output type.
pub use ratatui::text::Text;
