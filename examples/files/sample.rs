//! A tui-view sample: Rust source exercising the Tree-sitter view.
//!
//! Every token family the default `SyntaxTheme` styles shows up here —
//! comments, keywords, strings, escapes, numbers, types, functions,
//! attributes, and punctuation.

use std::collections::BTreeMap;
use std::fmt;

/// How loud a message is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// Background noise.
    Debug,
    /// Worth reading.
    Info,
    /// Something is wrong.
    Error,
}

impl fmt::Display for Level {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Error => "error",
        };
        write!(f, "{name}")
    }
}

/// A counted log of messages, newest last.
#[derive(Debug, Default)]
pub struct Log {
    entries: Vec<(Level, String)>,
    counts: BTreeMap<&'static str, usize>,
}

impl Log {
    const CAPACITY: usize = 1_024;

    /// An empty log.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record `message` at `level`, dropping the oldest entry once the log is
    /// full.
    pub fn push(&mut self, level: Level, message: impl Into<String>) -> &mut Self {
        if self.entries.len() >= Self::CAPACITY {
            self.entries.remove(0);
        }
        self.entries.push((level, message.into()));
        let key = match level {
            Level::Debug => "debug",
            Level::Info => "info",
            Level::Error => "error",
        };
        *self.counts.entry(key).or_insert(0) += 1;
        self
    }

    /// Every message at `level`.
    pub fn at(&self, level: Level) -> impl Iterator<Item = &str> {
        self.entries
            .iter()
            .filter(move |(l, _)| *l == level)
            .map(|(_, m)| m.as_str())
    }

    /// The log rendered as text, one entry per line.
    #[must_use]
    pub fn render(&self) -> String {
        let mut out = String::with_capacity(self.entries.len() * 32);
        for (i, (level, message)) in self.entries.iter().enumerate() {
            // Tab and newline escapes, so the escape style has something to
            // paint.
            out.push_str(&format!("{i:>4}\t[{level}]\t{message}\n"));
        }
        out
    }
}

fn main() {
    let mut log = Log::new();
    log.push(Level::Info, "started")
        .push(Level::Debug, "cache warm: 0.75 hit rate")
        .push(Level::Error, "connection reset by peer");

    let errors: Vec<&str> = log.at(Level::Error).collect();
    assert_eq!(errors.len(), 1);

    print!("{}", log.render());
    println!("{} entries, {} kinds", log.entries.len(), log.counts.len());
}
