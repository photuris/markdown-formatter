//! Re-wraps Markdown prose and YAML front matter at [`WIDTH`] columns.
//!
//! The body is parsed with `pulldown-cmark`; only paragraph text is
//! reflowed, and everything else (code, tables, headings, HTML) is kept
//! byte for byte. Front matter long scalars are folded with `>-`.

pub mod body;
pub mod frontmatter;

/// Maximum display width of a formatted line, in columns.
pub const WIDTH: usize = 79;

/// Formats a whole Markdown document: front matter, then body.
///
/// Unfinished: returns the input unchanged.
///
/// # Errors
///
/// Returns [`body::BodyError`] when the body cannot be reflowed safely.
pub fn format_document(input: &str) -> Result<String, body::BodyError> {
    Ok(input.to_owned())
}
