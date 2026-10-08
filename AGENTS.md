# markdown-formatter

`mdfmt` re-wraps Markdown files at 79 columns: paragraph prose is reflowed,
everything else (code, tables, headings, HTML, math) is kept byte for byte, and
long top-level YAML front-matter scalars are folded with `>-`. Files inside an
Obsidian vault (an ancestor holds `.obsidian/`) are skipped.

## Layout

- `src/main.rs`: thin entry point (args, tracing, loop over files).
- `src/lib.rs`: `format_document`, `format_file`, vault detection.
- `src/frontmatter.rs`: YAML front-matter folding.
- `src/body.rs`: Markdown reflow on top of `pulldown-cmark`, plus the structure
  check that refuses any reflow that changes the parse.
- `tests/fixtures/<group>/<name>.{in,out}.<ext>`: golden files. Every `.in`
  must format to its `.out`, and every `.out` to itself.

## Commands

```sh
cargo fmt
cargo clippy --all-targets --locked -- -D warnings
cargo test --locked
```

Follow the user's `rust-style` skill: 79-column `rustfmt`, doc comment on every
item, no `unwrap`/`expect` outside tests, `thiserror` in the library and
`anyhow` only in `main`.

## graphify

This project has a knowledge graph at graphify-out/ with god nodes, community
structure, and cross-file relationships.

When the user types `/graphify`, use the installed graphify skill or
instructions before doing anything else.

Rules:
- For codebase questions, first run `graphify query "<question>"` when
  graphify-out/graph.json exists. Use `graphify path "<A>" "<B>"` for
  relationships and `graphify explain "<concept>"` for focused concepts. These
  return a scoped subgraph, usually much smaller than GRAPH_REPORT.md or raw
  grep output.
- Dirty graphify-out/ files are expected after hooks or incremental updates;
  dirty graph files are not a reason to skip graphify. Only skip graphify if
  the task is about stale or incorrect graph output, or the user explicitly
  says not to use it.
- If graphify-out/wiki/index.md exists, use it for broad navigation instead of
  raw source browsing.
- Read graphify-out/GRAPH_REPORT.md only for broad architecture review or when
  query/path/explain do not surface enough context.
- After modifying code, run `graphify update .` to keep the graph current
  (AST-only, no API cost).
