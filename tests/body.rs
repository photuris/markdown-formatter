//! Golden tests for `body::format`.
#![expect(
    clippy::expect_used,
    reason = "test helpers outside #[test] fns panic on failure by design"
)]

mod common;

use markdown_formatter::body;

/// Formats a body, failing the test on a reflow error.
fn format(input: &str) -> String {
    body::format(input).expect("body::format returned an error")
}

golden!(
    "body",
    "md",
    format,
    [
        atomic,
        block_start_guard,
        blockquotes,
        footnote,
        hard_breaks,
        lists,
        paragraphs,
        verbatim,
    ]
);
