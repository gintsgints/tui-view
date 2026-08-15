//! Bundled [`FormatView`](crate::view::FormatView) plugins.
//!
//! Each plugin lives behind a cargo feature so downstream crates only compile
//! the formats they use. The [`markdown`] view is enabled by default.

#[cfg(feature = "markdown")]
pub mod markdown;
