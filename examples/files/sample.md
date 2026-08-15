# tui-view sample

A tour of what the **Markdown** view renders — long enough to scroll. Try it:

```
cargo run --example viewer -- examples/files/sample.md
```

Scroll with `j`/`k`, page with `Space`/`PgUp`, jump with `g`/`G`, quit `q`.
Each numbered section below lets you gauge how far you have scrolled.

## Inline styles

You get **bold**, *italic*, ~~strikethrough~~, `inline code`, and
[links](https://ratatui.rs) — and they wrap cleanly at the terminal width no
matter how long the paragraph runs on and on and on past the right edge of the
screen without any manual line breaks in the source at all.

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
> quoted passage reads as clearly set apart from the body even when it wraps
> across several rendered rows like this one does.

## Code block

```rust
fn main() {
    println!("kept verbatim, not wrapped");
}
```

---

# Part one: prose sections

## Section 1

Paragraph for section 1. The body text wraps to the terminal width, and the
viewer scrolls one rendered row at a time, so a wrapped paragraph behaves the
same as a run of short lines. Resize the window to confirm the wrap and scroll
math stay in step.

## Section 2

Paragraph for section 2. Markdown collapses soft line breaks in the source into
spaces, then the view re-wraps everything to the current width. That means the
same document reflows to fit an 80-column terminal or a 200-column one without
changing the file.

## Section 3

Paragraph for section 3. Inline `code spans`, **emphasis**, and *stress* all
survive the wrap because styling is tracked per glyph rather than per line, so
a bold phrase that **straddles a wrap boundary** stays bold on both rows.

## Section 4

Paragraph for section 4. Nothing here is special; it is filler to give the
scrollbar somewhere to travel. Keep pressing `Space` and watch the section
numbers climb toward the end marker.

## Section 5

Paragraph for section 5. A [link](https://example.com) mid-sentence is
underlined and colored, and the surrounding words wrap around it normally as
the paragraph grows past the right edge of the viewport.

## Section 6

Paragraph for section 6. Block quotes, lists, and code blocks each reset the
wrapping context, so the section headers stay flush left while their bodies
carry whatever indentation their block implies.

## Section 7

Paragraph for section 7. The render is cached per width, so scrolling through
these sections never re-parses the Markdown — only a resize does. That keeps
paging smooth even for documents far longer than this one.

## Section 8

Paragraph for section 8. Half way now. If `G` still jumps cleanly to the very
bottom and `g` snaps back to the title, the clamp logic is behaving on a
document taller than the screen.

# Part two: mixed blocks

## Section 9

> A quote to open section 9, wrapping across the gutter so the left border
> stays attached row after row after row as the sentence keeps going well past
> where a single line would end.

- list item 9.a with enough text to wrap under its hanging indent and prove the
  continuation lines stay aligned
- list item 9.b
- list item 9.c

## Section 10

```text
code block for section 10
lines here are kept verbatim
    including this indented one
and are never soft-wrapped
```

Follow-up paragraph after the code block in section 10, back to normal wrapped
body text so you can see the transition between block styles while scrolling.

## Section 11

1. numbered step one for section 11 with a long tail that wraps to show ordered
   markers behave like bullets under wrapping
2. numbered step two
3. numbered step three

## Section 12

Paragraph for section 12. Mixing **bold**, `code`, and ~~strikethrough~~ in one
sentence to confirm adjacent style runs merge into single spans without leaking
stray spaces between them.

## Section 13

> Nested structure check for section 13:
>
> - a bullet inside a quote
> - another bullet inside the same quote
>
> and a closing quoted line after the list.

## Section 14

Paragraph for section 14. Getting close to the end. The point of all this text
is simply volume: enough rendered rows that every scroll key has meaningful
distance to cover.

## Section 15

Paragraph for section 15. One more block of filler prose that wraps across the
width so the final stretch before the end marker is as tall as the sections
that came before it.

## Section 16

The last section. Below is a horizontal rule, then the end marker. Press `G`
now to confirm the bottom is reachable, then `g` to fly back to the top.

---

**End of document.** Press `g` to return to the top, `G` to come back here.
