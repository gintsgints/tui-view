# Agent instructions

## After every code change

Run both, in order, and fix all output before considering the change done:

```sh
cargo fmt --all
cargo clippy --all-targets --all-features -- -W clippy::pedantic -D warnings
```

- `cargo fmt --all` — format. Never hand-format; let rustfmt decide.
- `cargo clippy … -W clippy::pedantic -D warnings` — lint at pedantic level
  with warnings promoted to errors. The change is not finished while this
  prints anything.

Then run the tests:

```sh
cargo test --all-features
```

## Commits

- Commits read as if written by the human author.
- Conventional Commits format; subject in the imperative, explain *why* in the
  body when it is not obvious.

## Rules

- Fix the cause, not the symptom — do not silence a pedantic lint with
  `#[allow(...)]` unless the lint is genuinely wrong for that spot; if you
  allow one, add a one-line comment saying why.
- Keep public items documented; `missing_docs`-style gaps are treated as
  failures here.
- Prefer the dedicated wrapping/scroll paths already in `src/`; do not
  reimplement them per plugin.
