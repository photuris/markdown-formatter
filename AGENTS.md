# markdown-formatter

`mdfmt` re-wraps Markdown files at 79 columns: paragraph prose is
reflowed, everything else (code, tables, headings, HTML, math) is kept
byte for byte, and long top-level YAML front-matter scalars are folded
with `>-`. Files inside an Obsidian vault (an ancestor holds
`.obsidian/`) are skipped.

## Layout

- `src/main.rs`: thin entry point (args, tracing, loop over files).
- `src/lib.rs`: `format_document`, `format_file`, vault detection.
- `src/frontmatter.rs`: YAML front-matter folding.
- `src/body.rs`: Markdown reflow on top of `pulldown-cmark`, plus the
  structure check that refuses any reflow that changes the parse.
- `tests/fixtures/<group>/<name>.{in,out}.<ext>`: golden files. Every
  `.in` must format to its `.out`, and every `.out` to itself.

## Commands

```sh
cargo fmt
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Follow the user's `rust-style` skill: 79-column `rustfmt`, doc comment
on every item, no `unwrap`/`expect` outside tests, `thiserror` in the
library and `anyhow` only in `main`.
