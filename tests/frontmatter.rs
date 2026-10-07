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

/// A quoted scalar whose unindented continuation looks like a long key.
const QUOTED_TAIL: &str = "x: this is a long line inside a quoted string \
and should stay byte for byte even when it exceeds the width limit\nlast\"\n";

#[test]
fn should_keep_quoted_scalar_when_extra_space_precedes_quote() {
    let input = format!("key:  \"first\n{QUOTED_TAIL}");

    assert_eq!(frontmatter::format(&input), input);
}

#[test]
fn should_keep_quoted_scalar_when_anchor_precedes_quote() {
    let input = format!("key: &label \"first\n{QUOTED_TAIL}");

    assert_eq!(frontmatter::format(&input), input);
}

#[test]
fn should_keep_next_quoted_scalar_when_previous_ends_in_escaped_backslash() {
    let input =
        format!("key: \"ends in slash\\\\\"\nother: \"first\n{QUOTED_TAIL}");

    assert_eq!(frontmatter::format(&input), input);
}

#[test]
fn should_keep_input_unchanged_when_yaml_is_invalid() {
    let input = "description: This long plain value would fold if the \
document parsed, but the next line is broken YAML.\n: : [\n";

    assert_eq!(frontmatter::format(input), input);
}
