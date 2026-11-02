//! Black-box tests for unwrap mode (`Wrap::Unwrap`).
#![expect(
    clippy::expect_used,
    reason = "test helpers outside #[test] fns panic on failure by design"
)]

mod common;

use markdown_formatter::{WIDTH, Wrap, format_document, frontmatter};

/// Unwraps a whole document, failing the test on an error.
fn unwrap(input: &str) -> String {
    format_document(input, Wrap::Unwrap)
        .expect("format_document(Unwrap) returned an error")
}

/// Fills a whole document at the default width, failing on an error.
fn fill(input: &str) -> String {
    format_document(input, Wrap::Fill)
        .expect("format_document(Fill) returned an error")
}

golden!(
    "unwrap",
    "md",
    unwrap,
    [
        atomic,
        blockquotes,
        footnote,
        frontmatter,
        lists,
        paragraphs,
        verbatim,
    ]
);

/// Every wrapped golden output survives unwrap followed by fill.
#[test]
fn should_restore_wrapped_output_when_unwrapped_then_filled() {
    let fixtures = [
        ("body", "atomic"),
        ("body", "block_start_guard"),
        ("body", "blockquotes"),
        ("body", "footnote"),
        ("body", "hard_breaks"),
        ("body", "lists"),
        ("body", "paragraphs"),
        ("body", "verbatim"),
        ("document", "crlf"),
        ("document", "empty"),
        ("document", "no_frontmatter"),
        ("document", "no_trailing_newline"),
        ("document", "unclosed_frontmatter"),
        ("document", "with_frontmatter"),
    ];

    for (group, name) in fixtures {
        let wrapped = common::fixture(group, name, "out", "md");

        assert_eq!(fill(&unwrap(&wrapped)), wrapped, "{group}/{name}");
    }
}

/// Every folded front-matter golden output survives unwrap then fold.
#[test]
fn should_restore_folded_front_matter_when_unwrapped_then_folded() {
    let names = [
        "boundary",
        "folded_rewrap",
        "long_plain",
        "multiline_plain",
        "short_unchanged",
        "untouched",
    ];

    for name in names {
        let folded = common::fixture("frontmatter", name, "out", "yaml");
        let unwrapped = frontmatter::format(&folded, Wrap::Unwrap.width());

        assert_eq!(frontmatter::format(&unwrapped, WIDTH), folded, "{name}");
    }
}

#[test]
fn should_return_empty_when_unwrapping_empty_input() {
    assert_eq!(unwrap(""), "");
}

#[test]
fn should_keep_blank_lines_when_unwrapping_whitespace_only_input() {
    assert_eq!(unwrap("\n\n"), "\n\n");
    assert_eq!(unwrap("   \n"), "   \n");
}

#[test]
fn should_unwrap_each_of_two_identical_paragraphs() {
    assert_eq!(unwrap("a\nb\n\na\nb\n"), "a b\n\na b\n");
}

#[test]
fn should_keep_crlf_when_unwrapping() {
    assert_eq!(unwrap("a\r\nb\r\n"), "a b\r\n");
}

#[test]
fn should_keep_long_plain_front_matter_value_on_one_line() {
    let long = format!("---\nk: {}\n---\n", "word ".repeat(30).trim_end());

    assert_eq!(unwrap(&long), long);
}

/// Under plain CommonMark this is a link reference definition followed by a
/// paragraph; joining the lines would turn both into one paragraph.
#[test]
fn should_refuse_unwrap_that_changes_commonmark_structure() {
    let result =
        markdown_formatter::body::format("[^a]: http://x\nfoo\n", usize::MAX);

    assert!(
        matches!(
            result,
            Err(markdown_formatter::body::BodyError::StructureChanged)
        ),
        "got {result:?}"
    );
}

/// Under plain CommonMark the indented lines are a code block whose text
/// must not change, even though the extension set reads them as prose.
#[test]
fn should_refuse_unwrap_that_changes_commonmark_code_block() {
    let input = "[^a]: note\n\n    alpha\n    beta\n";

    let result = markdown_formatter::body::format(input, usize::MAX);

    assert!(
        matches!(
            result,
            Err(markdown_formatter::body::BodyError::StructureChanged)
        ),
        "got {result:?}"
    );
}

/// Under plain CommonMark this is a link reference definition whose
/// title holds a newline; joining the lines would change the title.
#[test]
fn should_refuse_unwrap_that_changes_reference_definition() {
    let input = "[^a]: http://example.com\n    \"alpha\n    beta\"\n";

    let result = markdown_formatter::body::format(input, usize::MAX);

    assert!(
        matches!(
            result,
            Err(markdown_formatter::body::BodyError::StructureChanged)
        ),
        "got {result:?}"
    );
}
