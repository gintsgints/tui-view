//! The pluggable-view abstraction.
//!
//! A [`FormatView`] turns the raw text of a file into styled, pre-wrapped
//! [`Text`]. Views are registered in a [`ViewRegistry`] and selected for a
//! file by extension. To add support for a new format, implement
//! [`FormatView`] and register it.

use std::path::Path;
use std::sync::Arc;

use ratatui::text::Text;

/// A plugin that renders one family of text file formats.
///
/// Implementations receive the raw file contents and the width (in columns)
/// available for rendering, and must return a [`Text`] that is already wrapped
/// to that width. The viewer widget scrolls this text line by line, so a view
/// that does its own wrapping keeps scroll math exact.
pub trait FormatView: Send + Sync {
    /// Human-readable name, e.g. `"Markdown"`.
    fn name(&self) -> &'static str;

    /// Lower-case file extensions handled, without the leading dot,
    /// e.g. `["md", "markdown"]`.
    fn extensions(&self) -> &[&str];

    /// Render `content` into styled lines wrapped to `width` columns.
    fn render(&self, content: &str, width: u16) -> Text<'static>;

    /// Whether this view handles `path`, matched on its extension.
    ///
    /// The default implementation compares the path's extension
    /// (case-insensitively) against [`extensions`](FormatView::extensions).
    fn matches(&self, path: &Path) -> bool {
        match path.extension().and_then(|e| e.to_str()) {
            Some(ext) => {
                let ext = ext.to_ascii_lowercase();
                self.extensions().iter().any(|e| *e == ext)
            }
            None => false,
        }
    }
}

/// An ordered collection of [`FormatView`] plugins.
///
/// Views are tried in registration order; the first whose
/// [`matches`](FormatView::matches) returns `true` wins. Register more
/// specific views before more general ones.
#[derive(Default, Clone)]
pub struct ViewRegistry {
    views: Vec<Arc<dyn FormatView>>,
}

impl ViewRegistry {
    /// An empty registry with no views.
    #[must_use]
    pub fn new() -> Self {
        Self { views: Vec::new() }
    }

    /// A registry pre-populated with every view enabled by crate features.
    ///
    /// With the default `markdown` feature this contains the Markdown view.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut r = Self::new();
        #[cfg(feature = "markdown")]
        r.register(Arc::new(crate::plugins::markdown::MarkdownView::new()));
        r
    }

    /// Add a view to the end of the lookup order.
    pub fn register(&mut self, view: Arc<dyn FormatView>) -> &mut Self {
        self.views.push(view);
        self
    }

    /// The first registered view that matches `path`, if any.
    #[must_use]
    pub fn find(&self, path: &Path) -> Option<Arc<dyn FormatView>> {
        self.views.iter().find(|v| v.matches(path)).cloned()
    }

    /// Every registered view, in lookup order.
    #[must_use]
    pub fn views(&self) -> &[Arc<dyn FormatView>] {
        &self.views
    }
}
