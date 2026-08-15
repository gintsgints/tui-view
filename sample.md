# tui-view sample

A quick tour of what the **Markdown** view renders. Try it with:

```
cargo run --example viewer -- sample.md
```

## Inline styles

You get **bold**, *italic*, ~~strikethrough~~, `inline code`, and
[links](https://ratatui.rs) — and they wrap cleanly at the terminal width no
matter how long the paragraph runs on and on and on past the right edge.

## Lists

- first bullet
- second bullet with a longer body that wraps under its own hanging indent so
  continuation lines line up past the marker
  - nested bullet
  - another nested one
- back to the top level

1. ordered one
2. ordered two
3. ordered three

- [x] finished task
- [ ] pending task

## Block quote

> Quotes get a gutter down the left side, and the text inside is tinted so the
> quoted passage reads as clearly set apart from the body.

## Code block

```rust
fn main() {
    println!("kept verbatim, not wrapped");
}
```

---

That's the whole feature set. Scroll with `j`/`k`, page with `Space`, quit `q`.
