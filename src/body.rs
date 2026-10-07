//! Reflows Markdown paragraph text at [`crate::WIDTH`] columns.

use std::{collections::HashSet, ops::Range};

use pulldown_cmark::{Event, LinkType, Options, Parser, Tag, TagEnd};
use unicode_width::UnicodeWidthStr;

use crate::WIDTH;

/// The parser options used for every parse in this file.
const OPTS: Options = Options::ENABLE_TABLES
    .union(Options::ENABLE_FOOTNOTES)
    .union(Options::ENABLE_STRIKETHROUGH)
    .union(Options::ENABLE_TASKLISTS)
    .union(Options::ENABLE_MATH)
    .union(Options::ENABLE_WIKILINKS);

/// Failure to reflow a Markdown body safely.
#[derive(Debug, thiserror::Error)]
pub enum BodyError {
    /// The reflowed text parses to a different document than the input.
    #[error("reflow would change the document structure; file left as is")]
    StructureChanged,
}

/// Reflows the paragraph text of a Markdown body.
///
/// Paragraph and tight-list-item prose is refilled at [`WIDTH`] columns;
/// every other line is copied byte for byte.
///
/// # Errors
///
/// Returns [`BodyError::StructureChanged`] when the reflowed text would
/// parse differently from `body`.
pub fn format(body: &str) -> Result<String, BodyError> {
    let output = reflow(body);

    if same_structure(body, &output) {
        Ok(output)
    } else {
        Err(BodyError::StructureChanged)
    }
}

// ── Parsing and line ownership ──────────────────────────────────────────────

/// Who owns a source line.
#[derive(Clone, Copy, PartialEq)]
enum Owner {
    /// No event touched the line; it is copied as is.
    Unowned,
    /// The line is prose of the block that starts at this offset.
    Prose(usize),
    /// The line is leaf content, or contested; it is copied as is.
    Verbatim,
}

/// What an inline event means for the reflow of its unit.
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    /// Ordinary inline content.
    Plain,
    /// Code, inline math, inline HTML, or a wikilink: never split.
    Atomic,
    /// Display math: the whole unit is kept verbatim.
    DisplayMath,
    /// A hard line break.
    HardBreak,
}

/// One inline event of a prose block.
struct Inline {
    /// Start offset of the owning prose block.
    block: usize,
    /// Source range of the event.
    range: Range<usize>,
    /// What the event is.
    kind: Kind,
}

/// An open block while scanning the event stream.
enum Frame {
    /// A list, item-less container, quote, or footnote definition.
    Container,
    /// A paragraph, identified by its start offset.
    Paragraph(usize),
    /// A list item (prose when it holds inline events directly).
    Item(usize),
    /// Any other block: its content is verbatim.
    Leaf,
}

/// The result of parsing a body: who owns each line, and the inlines.
struct Scan {
    /// Byte offset of the start of each line.
    line_starts: Vec<usize>,
    /// Owner of each line.
    owners: Vec<Owner>,
    /// Inline events of prose blocks, in document order.
    inlines: Vec<Inline>,
    /// Start offsets of the prose blocks that are paragraphs.
    paragraphs: HashSet<usize>,
}

impl Scan {
    /// Returns the index of the line holding byte `offset`.
    fn line_of(&self, offset: usize) -> usize {
        self.line_starts.partition_point(|&s| s <= offset) - 1
    }

    /// Returns the lines an event range touches.
    fn lines_of(&self, range: &Range<usize>) -> Range<usize> {
        let last = range.end.saturating_sub(1).max(range.start);

        self.line_of(range.start)..self.line_of(last) + 1
    }

    /// Marks every line of `range` verbatim.
    fn mark_verbatim(&mut self, range: &Range<usize>) {
        for line in self.lines_of(range) {
            self.owners[line] = Owner::Verbatim;
        }
    }

    /// Marks the lines of `range` as owned by prose block `id`.
    fn mark_prose(&mut self, range: &Range<usize>, id: usize) {
        for line in self.lines_of(range) {
            self.owners[line] = match self.owners[line] {
                Owner::Unowned => Owner::Prose(id),
                Owner::Prose(other) if other == id => Owner::Prose(id),
                _ => Owner::Verbatim,
            };
        }
    }

    /// Records an inline event under the innermost open block.
    fn inline(&mut self, stack: &[Frame], range: Range<usize>, kind: Kind) {
        match stack.last() {
            Some(Frame::Paragraph(id) | Frame::Item(id)) => {
                self.mark_prose(&range, *id);
                self.inlines.push(Inline {
                    block: *id,
                    range,
                    kind,
                });
            }
            _ => self.mark_verbatim(&range),
        }
    }
}

/// Classifies a start tag: `None` for inline tags, else the block frame.
fn block_frame(tag: &Tag, start: usize) -> Option<Frame> {
    match tag {
        Tag::Emphasis
        | Tag::Strong
        | Tag::Strikethrough
        | Tag::Superscript
        | Tag::Subscript
        | Tag::Link { .. }
        | Tag::Image { .. } => None,
        Tag::Paragraph => Some(Frame::Paragraph(start)),
        Tag::Item => Some(Frame::Item(start)),
        Tag::List(_) | Tag::BlockQuote(_) | Tag::FootnoteDefinition(_) => {
            Some(Frame::Container)
        }
        _ => Some(Frame::Leaf),
    }
}

/// Returns whether an end tag closes an inline element.
fn is_inline_end(end: &TagEnd) -> bool {
    matches!(
        end,
        TagEnd::Emphasis
            | TagEnd::Strong
            | TagEnd::Strikethrough
            | TagEnd::Superscript
            | TagEnd::Subscript
            | TagEnd::Link
            | TagEnd::Image
    )
}

/// Returns the kind of an inline start tag.
fn start_kind(tag: &Tag) -> Kind {
    match tag {
        Tag::Link {
            link_type: LinkType::WikiLink { .. },
            ..
        } => Kind::Atomic,
        _ => Kind::Plain,
    }
}

/// Parses `body` and assigns every line an owner.
fn scan(body: &str) -> Scan {
    let mut line_starts = vec![0];

    line_starts.extend(body.match_indices('\n').map(|(i, _)| i + 1));

    let mut scan = Scan {
        owners: vec![Owner::Unowned; line_starts.len()],
        line_starts,
        inlines: Vec::new(),
        paragraphs: HashSet::new(),
    };
    let mut stack: Vec<Frame> = Vec::new();

    for (event, range) in Parser::new_ext(body, OPTS).into_offset_iter() {
        match event {
            Event::Start(tag) => match block_frame(&tag, range.start) {
                Some(frame) => {
                    match frame {
                        Frame::Leaf => scan.mark_verbatim(&range),
                        Frame::Paragraph(id) => {
                            scan.paragraphs.insert(id);
                        }
                        _ => {}
                    }
                    stack.push(frame);
                }
                None => scan.inline(&stack, range, start_kind(&tag)),
            },
            Event::End(end) if is_inline_end(&end) => {
                scan.inline(&stack, range, Kind::Plain);
            }
            Event::End(_) => {
                stack.pop();
            }
            Event::Text(_)
            | Event::FootnoteReference(_)
            | Event::SoftBreak => scan.inline(&stack, range, Kind::Plain),
            Event::Code(_) | Event::InlineMath(_) | Event::InlineHtml(_) => {
                scan.inline(&stack, range, Kind::Atomic);
            }
            Event::DisplayMath(_) => {
                scan.inline(&stack, range, Kind::DisplayMath);
            }
            Event::HardBreak => scan.inline(&stack, range, Kind::HardBreak),
            Event::TaskListMarker(_) => {}
            Event::Html(_) | Event::Rule => scan.mark_verbatim(&range),
        }
    }

    scan
}

// ── Reflowing units ─────────────────────────────────────────────────────────

/// The source text of one line of a unit, past its prefix.
struct Row<'a> {
    /// Byte offset of `text` in the body.
    start: usize,
    /// The line content.
    text: &'a str,
    /// Whether a hard break ends the segment on this line.
    break_after: bool,
}

/// Tokens between two hard breaks.
struct Segment<'a> {
    /// The tokens, in order.
    tokens: Vec<&'a str>,
    /// Whether the last output line ends with the two-space marker.
    marker: bool,
}

/// Reflows the whole body, line by line.
fn reflow(body: &str) -> String {
    let scan = scan(body);
    let lines: Vec<&str> = body.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;

    while i < lines.len() {
        let Owner::Prose(id) = scan.owners[i] else {
            out.push(lines[i].to_owned());
            i += 1;

            continue;
        };
        let mut last = i;

        while scan.owners.get(last + 1) == Some(&Owner::Prose(id)) {
            last += 1;
        }

        out.extend(reflow_unit(&scan, body, &lines, id, i..=last));
        i = last + 1;
    }

    out.join("\n")
}

/// Copies the lines of a unit unchanged.
fn verbatim(lines: &[&str], span: &Range<usize>) -> Vec<String> {
    lines[span.clone()]
        .iter()
        .map(|l| (*l).to_owned())
        .collect()
}

/// Returns the offset where the unit's first content starts, if any.
fn first_content(
    scan: &Scan,
    id: usize,
    first: usize,
    inlines: &[&Inline],
) -> Option<usize> {
    if scan.paragraphs.contains(&id) && scan.line_of(id) == first {
        return Some(id);
    }

    inlines
        .iter()
        .map(|i| i.range.start)
        .filter(|&s| scan.line_of(s) == first)
        .min()
}

/// Replaces every char of a prefix that is not `>` or a space.
fn continuation_prefix(prefix: &str) -> String {
    prefix
        .chars()
        .map(|c| if c == '>' || c == ' ' { c } else { ' ' })
        .collect()
}

/// Reflows one unit (a run of lines of prose block `id`).
fn reflow_unit(
    scan: &Scan,
    body: &str,
    lines: &[&str],
    id: usize,
    unit: std::ops::RangeInclusive<usize>,
) -> Vec<String> {
    let (first, last) = (*unit.start(), *unit.end());
    let span = first..last + 1;
    let inlines: Vec<&Inline> = scan
        .inlines
        .iter()
        .filter(|i| {
            i.block == id && span.contains(&scan.line_of(i.range.start))
        })
        .collect();
    let unsafe_inline = |i: &&Inline| match i.kind {
        Kind::DisplayMath => true,
        Kind::Atomic => body[i.range.clone()].contains('\n'),
        _ => false,
    };

    if inlines.iter().any(unsafe_inline) {
        return verbatim(lines, &span);
    }

    let Some(c) = first_content(scan, id, first, &inlines) else {
        return verbatim(lines, &span);
    };
    let line_start = scan.line_starts[first];
    let p1 = &body[line_start..c];

    if p1.contains('\t') {
        return verbatim(lines, &span);
    }

    let pc = continuation_prefix(p1);
    let callout = p1.contains('>') && body[c..].starts_with("[!");
    let mut out = Vec::new();
    let (p1, from) = if callout {
        out.push(lines[first].to_owned());

        if first == last {
            return out;
        }

        (pc.as_str(), first + 1)
    } else {
        (p1, first)
    };
    let atomic: Vec<Range<usize>> = inlines
        .iter()
        .filter(|i| i.kind == Kind::Atomic)
        .map(|i| i.range.clone())
        .collect();
    let breaks: HashSet<usize> = inlines
        .iter()
        .filter(|i| i.kind == Kind::HardBreak)
        .map(|i| scan.line_of(i.range.start))
        .collect();
    let rows: Vec<Row> = (from..=last)
        .map(|l| row(scan, lines, l, (first, c), breaks.contains(&l)))
        .collect();

    out.extend(fill(&segments(&rows, &atomic), p1, &pc));

    out
}

/// Builds the row for line `l`; `(first, c)` is the unit's first line and
/// the offset where its content starts.
fn row<'a>(
    scan: &Scan,
    lines: &[&'a str],
    l: usize,
    (first, c): (usize, usize),
    break_after: bool,
) -> Row<'a> {
    let line = lines[l];
    let line_start = scan.line_starts[l];
    let skip = if l == first {
        c - line_start
    } else {
        line.len() - line.trim_start_matches([' ', '\t', '>']).len()
    };

    Row {
        start: line_start + skip,
        text: &line[skip..],
        break_after,
    }
}

/// Splits rows into segments of tokens at hard breaks.
fn segments<'a>(
    rows: &[Row<'a>],
    atomic: &[Range<usize>],
) -> Vec<Segment<'a>> {
    let mut out = Vec::new();
    let mut current = Vec::new();

    for row in rows {
        current.extend(tokens(row, atomic));

        if row.break_after {
            out.push(Segment {
                tokens: std::mem::take(&mut current),
                marker: row.text.ends_with("  "),
            });
        }
    }

    if !current.is_empty() || out.is_empty() {
        out.push(Segment {
            tokens: current,
            marker: false,
        });
    }

    out
}

/// Splits a row on spaces and tabs outside atomic ranges.
fn tokens<'a>(row: &Row<'a>, atomic: &[Range<usize>]) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;

    for (i, ch) in row.text.char_indices() {
        let splits = matches!(ch, ' ' | '\t')
            && !atomic.iter().any(|r| r.contains(&(row.start + i)));

        match (splits, start) {
            (true, Some(s)) => {
                out.push(&row.text[s..i]);
                start = None;
            }
            (false, None) => start = Some(i),
            _ => {}
        }
    }

    if let Some(s) = start {
        out.push(&row.text[s..]);
    }

    out
}

/// Fills segments greedily into lines starting with `p1` / `pc`.
fn fill(segments: &[Segment], p1: &str, pc: &str) -> Vec<String> {
    let mut out = Vec::new();

    for (n, segment) in segments.iter().enumerate() {
        let mut line = if n == 0 { p1 } else { pc }.to_owned();
        let mut placed = false;

        for token in &segment.tokens {
            if !placed {
                line.push_str(token);
                placed = true;
            } else if line.width() + 1 + token.width() <= WIDTH
                || block_starting(token)
            {
                line.push(' ');
                line.push_str(token);
            } else {
                out.push(line.trim_end_matches(' ').to_owned());
                line = format!("{pc}{token}");
            }
        }

        let mut last = line.trim_end_matches(' ').to_owned();

        if segment.marker {
            last.push_str("  ");
        }

        out.push(last);
    }

    out
}

/// Returns whether `token` would start a new block at a line start.
fn block_starting(token: &str) -> bool {
    let all = |c: char| token.chars().all(|x| x == c);
    let digits = token.len() - 1;

    (matches!(token.len(), 1..=6) && all('#'))
        || matches!(token, "-" | "+" | "*")
        || all('=')
        || all('-')
        || (token.len() >= 3 && (all('*') || all('_')))
        || ((1..=9).contains(&digits)
            && (token.ends_with('.') || token.ends_with(')'))
            && token[..digits].bytes().all(|b| b.is_ascii_digit()))
        || token.starts_with(['>', '<', '|'])
        || token.starts_with("$$")
        || token.starts_with("```")
        || token.starts_with("~~~")
        || is_footnote_label(token)
}

/// Returns whether `token` has the form `[^label]:`.
fn is_footnote_label(token: &str) -> bool {
    token
        .strip_prefix("[^")
        .and_then(|rest| rest.strip_suffix("]:"))
        .is_some_and(|label| !label.is_empty() && !label.contains(']'))
}

// ── Structure check ─────────────────────────────────────────────────────────

/// Returns whether `a` and `b` parse to the same document, ignoring where
/// text wraps.
fn same_structure(a: &str, b: &str) -> bool {
    normalized(a) == normalized(b)
}

/// Parses `src`, turning soft breaks into spaces and merging adjacent
/// text events with whitespace runs collapsed.
fn normalized(src: &str) -> Vec<Event<'_>> {
    let mut out = Vec::new();
    let mut text = String::new();

    for event in Parser::new_ext(src, OPTS) {
        match event {
            Event::Text(t) => text.push_str(&t),
            Event::SoftBreak => text.push(' '),
            other => {
                flush_text(&mut text, &mut out);
                out.push(other);
            }
        }
    }

    flush_text(&mut text, &mut out);

    out
}

/// Moves pending text into `out` as one event with collapsed whitespace.
fn flush_text(text: &mut String, out: &mut Vec<Event<'_>>) {
    if text.is_empty() {
        return;
    }

    let mut collapsed = String::new();
    let mut in_space = false;

    for ch in text.chars() {
        let space = matches!(ch, ' ' | '\t' | '\n');

        if !space || !in_space {
            collapsed.push(if space { ' ' } else { ch });
        }

        in_space = space;
    }

    out.push(Event::Text(collapsed.into()));
    text.clear();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_be_unchanged_when_callout_has_no_body() {
        assert_eq!(format("> [!note]\n").ok().as_deref(), Some("> [!note]\n"));
        assert_eq!(format("> [!note]").ok().as_deref(), Some("> [!note]"));
    }

    mod same_structure {
        use super::*;

        #[test]
        fn should_be_false_when_list_becomes_paragraph() {
            assert!(!same_structure("- a\n", "a\n"));
        }

        #[test]
        fn should_be_true_when_only_line_breaks_move() {
            assert!(same_structure("a b\nc\n", "a\nb c\n"));
        }
    }
}
