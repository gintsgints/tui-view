//! Bundled [`FormatView`](crate::view::FormatView) plugins.
//!
//! Each plugin lives behind a cargo feature so downstream crates only compile
//! the formats they use. The [`markdown`], [`json`], [`plaintext`], and
//! [`hex`] views are enabled by default; the `treesitter` source-code view is
//! opt-in, one cargo feature per language grammar.

#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "plaintext")]
pub mod plaintext;

#[cfg(feature = "hex")]
pub mod hex;

#[cfg(feature = "treesitter")]
pub mod treesitter;
