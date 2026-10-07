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

/// A paragraph whose greedy reflow leaves `_ _ _` alone on the last
/// line, which would parse as a thematic break.
const MAKES_RULE: &str = "Underscores y word word word word word word word \
word word word word word word _ _ _\n";

#[test]
fn should_return_structure_changed_when_reflow_creates_thematic_break() {
    let result = body::format(MAKES_RULE);

    assert!(
        matches!(result, Err(body::BodyError::StructureChanged)),
        "got {result:?}"
    );
}

#[test]
fn should_keep_nonbreaking_space_when_reflowing() {
    let input = format!("{}10\u{a0}kg together.\n", "word ".repeat(15));

    let output = body::format(&input).expect("format");

    assert!(output.contains("10\u{a0}kg"), "got {output:?}");
}

#[test]
fn should_keep_inline_html_attribute_spacing_when_reflowing() {
    let input = format!("{}<a href=\"a  b\">go</a>.\n", "word ".repeat(14));

    let output = body::format(&input).expect("format");

    assert!(output.contains("<a href=\"a  b\">"), "got {output:?}");
}
