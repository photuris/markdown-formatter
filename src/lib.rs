//! Re-wraps Markdown prose and YAML front matter at [`WIDTH`] columns.
//!
//! The body is parsed with `pulldown-cmark`; only paragraph text is
//! reflowed, and everything else (code, tables, headings, HTML) is kept
//! byte for byte. Front matter long scalars are folded with `>-`.

use std::{
    fs,
    path::{Path, PathBuf},
};

pub mod body;
pub mod frontmatter;

/// Maximum display width of a formatted line, in columns.
pub const WIDTH: usize = 79;

/// Why a document could not be formatted.
#[derive(Debug, thiserror::Error)]
pub enum DocumentError {
    /// Some lines end in CRLF and others in a bare LF.
    #[error("mixed CRLF and LF line endings; file left as is")]
    MixedLineEndings,
    /// The body could not be reflowed safely.
    #[error(transparent)]
    Body(#[from] body::BodyError),
}

/// What [`format_file`] did with a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The file was already formatted and was not written.
    Unchanged,
    /// The file was reformatted and written.
    Rewritten,
    /// The file is inside an Obsidian vault and was not read.
    SkippedVault,
    /// The file would change but was not written (check mode).
    NeedsFormatting,
}

/// What [`format_file`] does with a file whose formatted text differs
/// from its contents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Write the formatted text back to the file.
    Write,
    /// Leave the file alone and report that it needs formatting.
    Check,
}

/// Why a file could not be formatted.
#[derive(Debug, thiserror::Error)]
pub enum FileError {
    /// Reading or writing the file failed.
    #[error("{}: {source}", path.display())]
    Io {
        /// The file that failed.
        path: PathBuf,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// The file contents could not be formatted.
    #[error("{}: {source}", path.display())]
    Format {
        /// The file that failed.
        path: PathBuf,
        /// The underlying formatting error.
        source: DocumentError,
    },
}

/// Formats a whole Markdown document: front matter, then body.
///
/// Line endings are normalised to LF for formatting and restored to CRLF
/// afterwards when the input used CRLF.
///
/// # Errors
///
/// Returns [`DocumentError::MixedLineEndings`] when CRLF and bare LF are
/// mixed, and [`DocumentError::Body`] when the body cannot be reflowed
/// safely.
pub fn format_document(input: &str) -> Result<String, DocumentError> {
    let crlf = input.contains("\r\n");

    if crlf && has_bare_lf(input) {
        return Err(DocumentError::MixedLineEndings);
    }

    let text = input.replace("\r\n", "\n");

    let output = match split_front_matter(&text) {
        Some((yaml, closing, rest)) => format!(
            "---\n{}{closing}{}",
            frontmatter::format(yaml),
            body::format(rest)?
        ),
        None => body::format(&text)?,
    };

    if crlf {
        return Ok(output.replace('\n', "\r\n"));
    }

    Ok(output)
}

/// Whether `text` has a `\n` that is not preceded by `\r`.
fn has_bare_lf(text: &str) -> bool {
    let bytes = text.as_bytes();

    text.match_indices('\n')
        .any(|(i, _)| i == 0 || bytes[i - 1] != b'\r')
}

/// Splits LF-only `text` into `(yaml, closing fence line, body)`.
///
/// Returns `None` when `text` has no front matter: it must start with a
/// `---` line and a later line must be exactly `---`.
fn split_front_matter(text: &str) -> Option<(&str, &str, &str)> {
    let after_open = text.strip_prefix("---\n")?;
    let mut offset = 0;

    for line in after_open.split_inclusive('\n') {
        if line == "---\n" || line == "---" {
            let end = offset + line.len();

            return Some((&after_open[..offset], line, &after_open[end..]));
        }

        offset += line.len();
    }

    None
}

/// Whether an ancestor directory of `path` contains a `.obsidian`
/// directory, which marks an Obsidian vault.
///
/// Returns `false` when `path` cannot be canonicalized.
pub fn in_obsidian_vault(path: &Path) -> bool {
    let Ok(path) = fs::canonicalize(path) else {
        return false;
    };

    path.ancestors()
        .skip(1)
        .any(|dir| dir.join(".obsidian").is_dir())
}

/// Formats the file at `path`, in place or as a dry run per `mode`.
///
/// Files inside an Obsidian vault are skipped without being read. When
/// the formatted text equals the contents, the result is
/// [`Outcome::Unchanged`] in both modes. Otherwise [`Mode::Write`] writes
/// the file and returns [`Outcome::Rewritten`], while [`Mode::Check`]
/// leaves it untouched and returns [`Outcome::NeedsFormatting`]. A
/// failure during the write itself may leave the file partly written.
///
/// # Errors
///
/// Returns [`FileError::Io`] when the file cannot be read or written, and
/// [`FileError::Format`] when its contents cannot be formatted. Neither
/// read nor format errors modify the file.
pub fn format_file(path: &Path, mode: Mode) -> Result<Outcome, FileError> {
    if in_obsidian_vault(path) {
        return Ok(Outcome::SkippedVault);
    }

    let io_err = |source| FileError::Io {
        path: path.to_owned(),
        source,
    };
    let input = fs::read_to_string(path).map_err(io_err)?;
    let output =
        format_document(&input).map_err(|source| FileError::Format {
            path: path.to_owned(),
            source,
        })?;

    if output == input {
        return Ok(Outcome::Unchanged);
    }

    if mode == Mode::Check {
        return Ok(Outcome::NeedsFormatting);
    }

    fs::write(path, output).map_err(io_err)?;

    Ok(Outcome::Rewritten)
}
