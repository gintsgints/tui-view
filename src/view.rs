//! The pluggable-view abstraction.
//!
//! A [`FormatView`] turns the raw contents of a file into styled, pre-wrapped
//! [`Text`]. Views are registered in a [`ViewRegistry`] and selected for a
//! file by extension, or — for views that opt in via
//! [`matches_content`](FormatView::matches_content) — by inspecting the bytes.
//! To add support for a new format, implement [`FormatView`] and register it.

use std::path::Path;
use std::sync::Arc;

use ratatui::text::Text;

/// Bytes inspected when sniffing whether content is binary.
const SNIFF_LEN: usize = 8192;

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

    /// Render raw `bytes` into styled lines wrapped to `width` columns.
    ///
    /// This is what the widget actually calls, so a view that cares about
    /// bytes the file's encoding cannot represent — a hex dump, say — should
    /// override it. The default implementation decodes lossily and defers to
    /// [`render`](FormatView::render), which is exact for UTF-8 input.
    fn render_bytes(&self, bytes: &[u8], width: u16) -> Text<'static> {
        self.render(&String::from_utf8_lossy(bytes), width)
    }

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

    /// Whether this view claims `bytes` on their content alone, whatever the
    /// file is named.
    ///
    /// A view that returns `true` here outranks extension matching in
    /// [`ViewRegistry::find_for`], so reserve it for content no text view can
    /// display — the hex view uses it to claim every binary file. The default
    /// implementation claims nothing.
    fn matches_content(&self, bytes: &[u8]) -> bool {
        let _ = bytes;
        false
    }
}

/// Whether `bytes` look like binary rather than text.
///
/// Heuristic, over the first [`SNIFF_LEN`] bytes: a NUL byte, or a sequence
/// that is not valid UTF-8, means binary. A cut-off multi-byte character at
/// the sniff boundary does not count, so a large UTF-8 file is never
/// misjudged. Empty content is text.
#[must_use]
pub fn is_binary(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(SNIFF_LEN)];
    if head.contains(&0) {
        return true;
    }
    match std::str::from_utf8(head) {
        Ok(_) => false,
        // `error_len() == None` means the input simply ended mid-character,
        // which is expected when the sniff window truncates the file.
        Err(e) => !(e.error_len().is_none() && head.len() < bytes.len()),
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
    /// With the default features this contains the Markdown, JSON, plain-text,
    /// and hex views, in that lookup order.
    #[must_use]
    pub fn with_defaults() -> Self {
        let mut r = Self::new();
        #[cfg(feature = "markdown")]
        r.register(Arc::new(crate::plugins::markdown::MarkdownView::new()));
        #[cfg(feature = "json")]
        r.register(Arc::new(crate::plugins::json::JsonView::new()));
        #[cfg(feature = "plaintext")]
        r.register(Arc::new(crate::plugins::plaintext::PlainTextView::new()));
        #[cfg(feature = "hex")]
        r.register(Arc::new(crate::plugins::hex::HexView::new()));
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

    /// The view for `path` given its `bytes`.
    ///
    /// Content wins over the file name: the first view whose
    /// [`matches_content`](FormatView::matches_content) accepts `bytes` is
    /// returned, so a binary file lands in the hex view even when its
    /// extension says otherwise. Failing that this is [`find`](Self::find).
    #[must_use]
    pub fn find_for(&self, path: &Path, bytes: &[u8]) -> Option<Arc<dyn FormatView>> {
        self.views
            .iter()
            .find(|v| v.matches_content(bytes))
            .or_else(|| self.views.iter().find(|v| v.matches(path)))
            .cloned()
    }

    /// Every registered view, in lookup order.
    #[must_use]
    pub fn views(&self) -> &[Arc<dyn FormatView>] {
        &self.views
    }
}
