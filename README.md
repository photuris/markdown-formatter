# markdown-formatter

`mdfmt` re-wraps Markdown files in place at 79 columns, following the
`markdown-style` skill.

```sh
cargo install --path .
mdfmt README.md docs/*.md
```

## What it changes

- **Paragraph prose** is reflowed with greedy fill to 79 display
  columns, including prose inside lists, block quotes, and footnotes.
  Short lines are joined, and list and quote markers are kept, with
  continuation lines indented to match.
- **Front matter**: a top-level plain string longer than 79 columns
  becomes a `>-` block with a 2-space indent. Existing `>-` and `>`
  blocks are re-wrapped to 79, and a plain value that spans lines is
  joined (or folded if it is still too long).

## What it never changes

- Headings, code blocks (fenced and indented), tables, HTML blocks,
  thematic breaks, link reference definitions, and display math are
  copied byte for byte.
- Inline code, inline HTML, wikilinks, and URLs are never split.
  Non-breaking spaces are kept.
- Hard line breaks (two trailing spaces or `\`) and callout title
  lines (`> [!note] Title`) are kept.
- YAML that is quoted, a `|` block, nested, a list, a flow
  collection, anchored, commented, or not clearly a string (`true`,
  `1234`, dates) is kept.
- Files inside an Obsidian vault (any ancestor directory holds
  `.obsidian/`) are skipped with a warning, since vault notes are not
  hard-wrapped.

## Safety

After reflowing, `mdfmt` parses the result again and compares it with
the original. If the document structure differs (for example, a wrap
would turn `_ _ _` into a horizontal rule), it reports an error and
leaves the file untouched. Files with mixed CRLF and LF line endings
are refused the same way. A file is written only when its content
changes.

## Exit codes

| Code | Meaning |
|------|---------|
| 0 | Every file formatted, unchanged, or skipped |
| 1 | At least one file failed (I/O error or unsafe reflow); the other files are still processed |
| 2 | Usage error, such as no paths given |

Diagnostics go to stderr, at `warn` level by default. Set `RUST_LOG`
to change it. On success, nothing is printed.

## Limitations

- There is no `--check` mode, no stdin input, and no width option.
- A line that starts with an inline HTML tag followed by a word too
  long to share its line is refused, because the tag would end up
  alone on a line and parse as an HTML block.
- A failure partway through writing a file can leave it truncated.
  `mdfmt` writes in place rather than through a temporary file, so
  that symlinks and permissions are preserved.

## Development

```sh
cargo fmt
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Golden fixtures live in `tests/fixtures/<group>/<name>.{in,out}.<ext>`.
Every `.in` must format to its `.out`, and every `.out` to itself.

## License

MIT. See [LICENSE](LICENSE).
