//! Folds long top-level YAML scalars in front matter at [`crate::WIDTH`].

use unicode_width::UnicodeWidthStr;

use crate::WIDTH;

/// Characters that may not start a plain scalar.
const INDICATORS: &str = "\"'[{|>&*!%@`#,";

/// Formats the YAML text found between the `---` fences.
///
/// `yaml` is every line between the fences, each ending in `\n`; the
/// result has the same shape. Long top-level plain scalars, multi-line
/// plain scalars, and existing `>` / `>-` blocks are re-folded at
/// [`WIDTH`]; every other line is copied unchanged.
pub fn format(yaml: &str) -> String {
    let lines: Vec<&str> = yaml.split_inclusive('\n').collect();
    let mut out = String::with_capacity(yaml.len());
    let mut i = 0;

    while i < lines.len() {
        let Some((key, value)) = parse_key_line(lines[i]) else {
            out.push_str(lines[i]);
            i += 1;

            continue;
        };

        if let Some(end) = open_quote_end(value, &lines, i) {
            out.push_str(&lines[i..end].concat());
            i = end;

            continue;
        }

        let mut end = i + 1;

        while end < lines.len() && is_continuation(lines[end]) {
            end += 1;
        }

        while end > i + 1 && is_blank(lines[end - 1]) {
            end -= 1;
        }

        match format_entry(key, value, &lines[i + 1..end]) {
            Some(text) => out.push_str(&text),
            None => out.push_str(&lines[i..end].concat()),
        }

        i = end;
    }

    out
}

/// Splits a key line into its key and value, or returns `None`.
fn parse_key_line(line: &str) -> Option<(&str, &str)> {
    let line = line.strip_suffix('\n').unwrap_or(line);
    let first = line.chars().next()?;

    if !(first.is_ascii_alphanumeric() || first == '_') {
        return None;
    }

    let colon = line.find(|c: char| {
        !(c.is_ascii_alphanumeric() || c == '_' || c == '-')
    })?;
    let rest = line[colon..].strip_prefix(':')?;

    if rest.is_empty() {
        return Some((&line[..colon], ""));
    }

    Some((&line[..colon], rest.strip_prefix(' ')?))
}

/// Returns the exclusive end of an entry whose quoted value is still
/// open at the end of line `i`, or `None` when the quote closes there or
/// the value is not quoted. An unclosed quote runs to the end of input.
fn open_quote_end(value: &str, lines: &[&str], i: usize) -> Option<usize> {
    let quote = value.chars().next().filter(|c| matches!(c, '"' | '\''))?;

    if quote_closes(quote, &value[1..]) {
        return None;
    }

    let closing = lines[i + 1..].iter().position(|l| quote_closes(quote, l));

    Some(closing.map_or(lines.len(), |p| i + 2 + p))
}

/// Whether `text` contains the quote that closes a `quote` string: a
/// `"` not preceded by a backslash, or a `'` not followed by another.
fn quote_closes(quote: char, text: &str) -> bool {
    let mut chars = text.chars().peekable();
    let mut prev = ' ';

    while let Some(c) = chars.next() {
        if c == quote && quote == '"' && prev != '\\' {
            return true;
        }

        if c == quote && quote == '\'' {
            if chars.peek() != Some(&'\'') {
                return true;
            }

            chars.next();
        }

        prev = c;
    }

    false
}

/// Whether a line is empty or only whitespace.
fn is_blank(line: &str) -> bool {
    line.trim().is_empty()
}

/// Whether a line belongs to the entry above it.
fn is_continuation(line: &str) -> bool {
    line.starts_with(' ') || is_blank(line)
}

/// Whether `s` can be written as a plain scalar without quoting.
fn is_plain_safe(s: &str) -> bool {
    let Some(first) = s.chars().next() else {
        return false;
    };

    !INDICATORS.contains(first)
        && !["- ", "? ", ": "].iter().any(|p| s.starts_with(p))
        && !s.contains('\t')
        && !s.contains("  ")
        && !s.contains(" #")
        && !s.contains(": ")
        && !s.ends_with(':')
        && !s.ends_with(char::is_whitespace)
}

/// Whether the plain scalar `s` always resolves to a string.
fn is_string_safe(s: &str) -> bool {
    is_plain_safe(s)
        && s.contains(' ')
        && s.chars().next().is_some_and(char::is_alphabetic)
}

/// Rewrites one entry, or returns `None` to keep it verbatim.
fn format_entry(key: &str, value: &str, cont: &[&str]) -> Option<String> {
    if cont.is_empty() {
        let width = key.width() + 2 + value.width();

        if !is_string_safe(value) || width <= WIDTH {
            return None;
        }

        return Some(fold(key, ">-", value.split(' '), "  "));
    }

    if value == ">-" || value == ">" {
        return fold_block(key, value, cont);
    }

    join_plain(key, value, cont)
}

/// Case B: re-folds an existing `>` or `>-` block with even indent.
fn fold_block(key: &str, indicator: &str, cont: &[&str]) -> Option<String> {
    let lines: Vec<&str> = cont
        .iter()
        .map(|l| l.strip_suffix('\n').unwrap_or(l))
        .collect();
    let n = lines[0].len() - lines[0].trim_start_matches(' ').len();

    if n == 0 {
        return None;
    }

    let indent = " ".repeat(n);
    let even = lines.iter().all(|l| {
        l.strip_prefix(indent.as_str()).is_some_and(|body| {
            !body.is_empty()
                && !body.starts_with(' ')
                && !body.contains("  ")
                && !l.contains('\t')
                && !l.ends_with(char::is_whitespace)
        })
    });

    if !even {
        return None;
    }

    let words = lines
        .iter()
        .flat_map(|l| l.split(' '))
        .filter(|w| !w.is_empty());

    Some(fold(key, indicator, words, &indent))
}

/// Case C: joins a multi-line plain scalar onto one line or folds it.
fn join_plain(key: &str, value: &str, cont: &[&str]) -> Option<String> {
    if !is_plain_safe(value) {
        return None;
    }

    let mut parts = vec![value];

    for line in cont {
        let part = line
            .strip_suffix('\n')
            .unwrap_or(line)
            .trim_start_matches(' ');

        if is_blank(part) || !is_plain_safe(part) {
            return None;
        }

        parts.push(part);
    }

    let joined = parts.join(" ");

    if !is_string_safe(&joined) {
        return None;
    }

    if key.width() + 2 + joined.width() <= WIDTH {
        return Some(format!("{key}: {joined}\n"));
    }

    Some(fold(key, ">-", joined.split(' '), "  "))
}

/// Greedy-fills `words` under `key: indicator`, one `indent` per line.
fn fold<'a>(
    key: &str,
    indicator: &str,
    words: impl Iterator<Item = &'a str>,
    indent: &str,
) -> String {
    let mut out = format!("{key}: {indicator}\n");
    let mut line = String::new();

    for word in words {
        if line.is_empty() {
            line.push_str(indent);
            line.push_str(word);
        } else if line.width() + 1 + word.width() <= WIDTH {
            line.push(' ');
            line.push_str(word);
        } else {
            out.push_str(&line);
            out.push('\n');
            line = format!("{indent}{word}");
        }
    }

    if !line.is_empty() {
        out.push_str(&line);
        out.push('\n');
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    mod format {
        use super::*;

        #[test]
        fn should_keep_entry_verbatim_when_quoted_scalar_spans_lines() {
            let input = "key: \"first\nx: this is a long line inside a quoted \
string and should stay byte for byte even when it exceeds the width limit\n\
last\"\n";

            assert_eq!(format(input), input);
        }
    }
}
