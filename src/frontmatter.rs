//! Folds long top-level YAML scalars in front matter at [`crate::WIDTH`].

/// Formats the YAML text found between the `---` fences.
///
/// `yaml` is every line between the fences, each ending in `\n`; the
/// result has the same shape. Unfinished: returns the input unchanged.
pub fn format(yaml: &str) -> String {
    yaml.to_owned()
}
