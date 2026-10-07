//! Golden tests for `format_document`.
#![expect(
    clippy::expect_used,
    reason = "test helpers outside #[test] fns panic on failure by design"
)]

mod common;

/// Formats a whole document, failing the test on a reflow error.
fn format(input: &str) -> String {
    markdown_formatter::format_document(input)
        .expect("format_document returned an error")
}

golden!(
    "document",
    "md",
    format,
    [
        crlf,
        empty,
        no_frontmatter,
        no_trailing_newline,
        unclosed_frontmatter,
        with_frontmatter,
    ]
);

#[test]
fn should_return_error_when_line_endings_are_mixed() {
    let result = markdown_formatter::format_document("a\r\nb\n");

    assert!(result.is_err(), "got {result:?}");
}
