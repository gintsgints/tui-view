//! The bundled Tree-sitter grammars.
//!
//! Each language is a function returning a [`SyntaxLanguage`], compiled only
//! when its `lang-*` cargo feature is on — a grammar is a C parser, so paying
//! for the ones you do not open is a real cost. [`all`] collects whatever is
//! enabled.
//!
//! ```
//! # #[cfg(feature = "lang-rust")] {
//! use tui_view::plugins::treesitter::{languages, TreeSitterView};
//!
//! let view = TreeSitterView::new(languages::rust());
//! # }
//! ```

use super::SyntaxLanguage;

/// Rust — `.rs`.
#[cfg(feature = "lang-rust")]
#[must_use]
pub fn rust() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "Rust",
        &["rs"],
        || tree_sitter_rust::LANGUAGE.into(),
        tree_sitter_rust::HIGHLIGHTS_QUERY,
    )
    .with_injections(tree_sitter_rust::INJECTIONS_QUERY)
}

/// Python — `.py`, `.pyi`.
#[cfg(feature = "lang-python")]
#[must_use]
pub fn python() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "Python",
        &["py", "pyi"],
        || tree_sitter_python::LANGUAGE.into(),
        tree_sitter_python::HIGHLIGHTS_QUERY,
    )
}

/// JavaScript — `.js`, `.mjs`, `.cjs`.
#[cfg(feature = "lang-javascript")]
#[must_use]
pub fn javascript() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "JavaScript",
        &["js", "mjs", "cjs"],
        || tree_sitter_javascript::LANGUAGE.into(),
        tree_sitter_javascript::HIGHLIGHT_QUERY,
    )
    .with_injections(tree_sitter_javascript::INJECTIONS_QUERY)
    .with_locals(tree_sitter_javascript::LOCALS_QUERY)
}

/// JSX — `.jsx`. The JavaScript grammar plus the JSX-specific captures.
#[cfg(feature = "lang-javascript")]
#[must_use]
pub fn jsx() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "JSX",
        &["jsx"],
        || tree_sitter_javascript::LANGUAGE.into(),
        format!(
            "{}\n{}",
            tree_sitter_javascript::HIGHLIGHT_QUERY,
            tree_sitter_javascript::JSX_HIGHLIGHT_QUERY
        ),
    )
    .with_injections(tree_sitter_javascript::INJECTIONS_QUERY)
    .with_locals(tree_sitter_javascript::LOCALS_QUERY)
}

/// TypeScript — `.ts`, `.mts`, `.cts`.
///
/// The TypeScript query only covers what TypeScript adds, so the JavaScript
/// query goes in front of it; the TypeScript grammar is a superset and accepts
/// both.
#[cfg(feature = "lang-typescript")]
#[must_use]
pub fn typescript() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "TypeScript",
        &["ts", "mts", "cts"],
        || tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
        typescript_highlights(),
    )
    .with_locals(tree_sitter_typescript::LOCALS_QUERY)
}

/// TSX — `.tsx`.
#[cfg(feature = "lang-typescript")]
#[must_use]
pub fn tsx() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "TSX",
        &["tsx"],
        || tree_sitter_typescript::LANGUAGE_TSX.into(),
        format!(
            "{}\n{}",
            typescript_highlights(),
            tree_sitter_javascript::JSX_HIGHLIGHT_QUERY
        ),
    )
    .with_locals(tree_sitter_typescript::LOCALS_QUERY)
}

#[cfg(feature = "lang-typescript")]
fn typescript_highlights() -> String {
    format!(
        "{}\n{}",
        tree_sitter_javascript::HIGHLIGHT_QUERY,
        tree_sitter_typescript::HIGHLIGHTS_QUERY
    )
}

/// Go — `.go`.
#[cfg(feature = "lang-go")]
#[must_use]
pub fn go() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "Go",
        &["go"],
        || tree_sitter_go::LANGUAGE.into(),
        tree_sitter_go::HIGHLIGHTS_QUERY,
    )
}

/// C — `.c`, `.h`.
#[cfg(feature = "lang-c")]
#[must_use]
pub fn c() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "C",
        &["c", "h"],
        || tree_sitter_c::LANGUAGE.into(),
        tree_sitter_c::HIGHLIGHT_QUERY,
    )
}

/// Shell — `.sh`, `.bash`, `.zsh`.
#[cfg(feature = "lang-bash")]
#[must_use]
pub fn bash() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "Shell",
        &["sh", "bash", "zsh"],
        || tree_sitter_bash::LANGUAGE.into(),
        tree_sitter_bash::HIGHLIGHT_QUERY,
    )
}

/// TOML — `.toml`.
#[cfg(feature = "lang-toml")]
#[must_use]
pub fn toml() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "TOML",
        &["toml"],
        || tree_sitter_toml_ng::LANGUAGE.into(),
        tree_sitter_toml_ng::HIGHLIGHTS_QUERY,
    )
}

/// YAML — `.yaml`, `.yml`.
#[cfg(feature = "lang-yaml")]
#[must_use]
pub fn yaml() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "YAML",
        &["yaml", "yml"],
        || tree_sitter_yaml::LANGUAGE.into(),
        tree_sitter_yaml::HIGHLIGHTS_QUERY,
    )
}

/// HTML — `.html`, `.htm`.
#[cfg(feature = "lang-html")]
#[must_use]
pub fn html() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "HTML",
        &["html", "htm"],
        || tree_sitter_html::LANGUAGE.into(),
        tree_sitter_html::HIGHLIGHTS_QUERY,
    )
    .with_injections(tree_sitter_html::INJECTIONS_QUERY)
}

/// CSS — `.css`.
#[cfg(feature = "lang-css")]
#[must_use]
pub fn css() -> SyntaxLanguage {
    SyntaxLanguage::new(
        "CSS",
        &["css"],
        || tree_sitter_css::LANGUAGE.into(),
        tree_sitter_css::HIGHLIGHTS_QUERY,
    )
}

/// Every language enabled by the current `lang-*` features.
///
/// The order is the lookup order used by
/// [`views`](super::views); extensions do not overlap, so it only decides
/// which view a registry reports first.
#[must_use]
pub fn all() -> Vec<SyntaxLanguage> {
    vec![
        #[cfg(feature = "lang-rust")]
        rust(),
        #[cfg(feature = "lang-python")]
        python(),
        #[cfg(feature = "lang-javascript")]
        javascript(),
        #[cfg(feature = "lang-javascript")]
        jsx(),
        #[cfg(feature = "lang-typescript")]
        typescript(),
        #[cfg(feature = "lang-typescript")]
        tsx(),
        #[cfg(feature = "lang-go")]
        go(),
        #[cfg(feature = "lang-c")]
        c(),
        #[cfg(feature = "lang-bash")]
        bash(),
        #[cfg(feature = "lang-toml")]
        toml(),
        #[cfg(feature = "lang-yaml")]
        yaml(),
        #[cfg(feature = "lang-html")]
        html(),
        #[cfg(feature = "lang-css")]
        css(),
    ]
}
