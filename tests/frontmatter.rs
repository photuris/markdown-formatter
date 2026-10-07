//! Golden tests for `frontmatter::format`.

mod common;

use markdown_formatter::frontmatter;

golden!(
    "frontmatter",
    "yaml",
    frontmatter::format,
    [
        boundary,
        folded_rewrap,
        long_plain,
        multiline_plain,
        short_unchanged,
        untouched,
    ]
);
