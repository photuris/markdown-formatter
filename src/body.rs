//! Reflows Markdown paragraph text at [`crate::WIDTH`] columns.

/// Failure to reflow a Markdown body safely.
#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    /// The reflowed text parses to a different document than the input.
    #[error("reflow would change the document structure; file left as is")]
    StructureChanged,
}

/// Reflows the paragraph text of a Markdown body.
///
/// Unfinished: returns the input unchanged.
///
/// # Errors
///
/// Returns [`BodyError::StructureChanged`] when the reflowed text would
/// parse differently from `body`.
pub fn format(body: &str) -> Result<String, BodyError> {
    Ok(body.to_owned())
}
