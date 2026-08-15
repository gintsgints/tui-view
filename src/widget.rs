//! The scrollable viewer widget and its state.

use std::sync::Arc;

use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, StatefulWidget, Widget};

use crate::view::{FormatView, ViewRegistry};

/// Retained state for a [`TuiView`]: the source text, its chosen view, the
/// cached render, and the scroll position.
///
/// The state re-renders lazily: the [`FormatView`] is only invoked when the
/// available width changes (first draw or terminal resize), so scrolling is
/// cheap. Scroll offsets are clamped against the viewport height recorded on
/// the previous draw.
pub struct ViewState {
    content: String,
    view: Arc<dyn FormatView>,
    cache: Option<Cached>,
    scroll: usize,
    /// Text rows visible on the last draw, used to clamp paging/scrolling.
    viewport: u16,
}

struct Cached {
    width: u16,
    text: Text<'static>,
}

impl ViewState {
    /// Build state from raw `content` rendered by an explicit `view`.
    pub fn new(content: impl Into<String>, view: Arc<dyn FormatView>) -> Self {
        Self {
            content: content.into(),
            view,
            cache: None,
            scroll: 0,
            viewport: 0,
        }
    }

    /// Build state by looking `path` up in `registry` to pick a view.
    ///
    /// Returns `None` if no registered view matches the path's extension.
    pub fn from_path(
        path: &std::path::Path,
        content: impl Into<String>,
        registry: &ViewRegistry,
    ) -> Option<Self> {
        let view = registry.find(path)?;
        Some(Self::new(content, view))
    }

    /// The view rendering this content.
    #[must_use]
    pub fn view(&self) -> &Arc<dyn FormatView> {
        &self.view
    }

    /// Replace the content, keeping the same view. Invalidates the cache and
    /// resets scroll to the top.
    pub fn set_content(&mut self, content: impl Into<String>) {
        self.content = content.into();
        self.cache = None;
        self.scroll = 0;
    }

    /// Swap the view used to render the current content. Invalidates the cache.
    pub fn set_view(&mut self, view: Arc<dyn FormatView>) {
        self.view = view;
        self.cache = None;
    }

    /// Current scroll offset, in rendered rows from the top.
    #[must_use]
    pub fn scroll(&self) -> usize {
        self.scroll
    }

    /// Total rendered rows, or `0` before the first draw.
    #[must_use]
    pub fn line_count(&self) -> usize {
        self.cache.as_ref().map_or(0, |c| c.text.height())
    }

    /// Largest valid scroll offset given the last viewport height.
    #[must_use]
    pub fn max_scroll(&self) -> usize {
        self.line_count().saturating_sub(self.viewport as usize)
    }

    /// Scroll up by `n` rows.
    pub fn scroll_up(&mut self, n: usize) {
        self.scroll = self.scroll.saturating_sub(n);
    }

    /// Scroll down by `n` rows, clamped to [`max_scroll`](Self::max_scroll).
    pub fn scroll_down(&mut self, n: usize) {
        self.scroll = (self.scroll + n).min(self.max_scroll());
    }

    /// Scroll up by one viewport height.
    pub fn page_up(&mut self) {
        self.scroll_up(self.viewport as usize);
    }

    /// Scroll down by one viewport height.
    pub fn page_down(&mut self) {
        self.scroll_down(self.viewport as usize);
    }

    /// Jump to the top.
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    /// Jump to the bottom.
    pub fn scroll_to_bottom(&mut self) {
        self.scroll = self.max_scroll();
    }

    /// Ensure the cache matches `width`, rendering via the view if needed.
    fn ensure(&mut self, width: u16) -> &Text<'static> {
        let stale = self.cache.as_ref().is_none_or(|c| c.width != width);
        if stale {
            let text = self.view.render(&self.content, width);
            self.cache = Some(Cached { width, text });
        }
        &self.cache.as_ref().unwrap().text
    }
}

/// A scrollable viewer widget.
///
/// The widget is stateless configuration; content, render cache and scroll
/// live in [`ViewState`]. Draw it with
/// [`StatefulWidget`](ratatui::widgets::StatefulWidget):
///
/// ```no_run
/// # use ratatui::{widgets::{Block, StatefulWidget}, layout::Rect, buffer::Buffer};
/// # use tui_view::{TuiView, ViewState, ViewRegistry};
/// # use std::sync::Arc;
/// # let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
/// # let reg = ViewRegistry::with_defaults();
/// # let view = reg.views()[0].clone();
/// let mut state = ViewState::new("# Hello", view);
/// TuiView::new()
///     .block(Block::bordered().title("README.md"))
///     .render(Rect::new(0, 0, 80, 24), &mut buf, &mut state);
/// ```
#[derive(Default)]
pub struct TuiView<'a> {
    block: Option<Block<'a>>,
}

impl<'a> TuiView<'a> {
    /// A viewer with no surrounding block.
    #[must_use]
    pub fn new() -> Self {
        Self { block: None }
    }

    /// Wrap the content in a [`Block`] (borders, title, padding).
    #[must_use]
    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }
}

impl StatefulWidget for TuiView<'_> {
    type State = ViewState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut ViewState) {
        // Resolve the drawing area inside the optional block.
        let inner = match &self.block {
            Some(b) => b.inner(area),
            None => area,
        };
        if let Some(block) = self.block {
            block.render(area, buf);
        }
        if inner.width == 0 || inner.height == 0 {
            return;
        }

        state.viewport = inner.height;
        // Render (or reuse) the wrapped text for this width, then clamp scroll.
        let total = state.ensure(inner.width).height();
        let max = total.saturating_sub(inner.height as usize);
        if state.scroll > max {
            state.scroll = max;
        }

        let text = &state.cache.as_ref().unwrap().text;
        let start = state.scroll;
        let end = (start + inner.height as usize).min(text.lines.len());
        for (line, y) in text.lines[start..end].iter().zip(inner.y..inner.bottom()) {
            buf.set_line(inner.x, y, line, inner.width);
        }
    }
}
