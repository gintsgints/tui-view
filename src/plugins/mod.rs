//! Bundled [`FormatView`](crate::view::FormatView) plugins.
//!
//! Each plugin lives behind a cargo feature so downstream crates only compile
//! the formats they use. The [`markdown`], [`json`], [`plaintext`], and
//! [`hex`] views are enabled by default.

#[cfg(feature = "markdown")]
pub mod markdown;

#[cfg(feature = "json")]
pub mod json;

#[cfg(feature = "plaintext")]
pub mod plaintext;

#[cfg(feature = "hex")]
pub mod hex;
