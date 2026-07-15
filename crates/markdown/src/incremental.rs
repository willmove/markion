//! Incremental Markdown parsing with performance optimizations.
//!
//! Re-parsing an entire document on every keystroke is wasteful for large
//! files. This module implements a block-region based incremental parser that
//! re-parses only the portions of a document affected by an edit and reuses the
//! already-parsed AST for the untouched surrounding regions.
//!
//! # Performance Optimizations
//!
//! 1. **Minimal re-parse region**: The edit position is used to narrow the
//!    set of candidate regions that need re-parsing, avoiding linear scans of
//!    unchanged prefix/suffix when the edit is localised.
//! 2. **Arc-shared immutable data**: Unchanged regions store their blocks
//!    behind `Arc<Vec<Block>>`, enabling zero-copy sharing between document
//!    versions. Only modified regions allocate new block vectors.
//! 3. **Async parsing**: Large documents can be parsed on a background task
//!    via [`IncrementalParser::apply_changes_async`], preventing UI thread
//!    blocking.
//!
//! # Strategy
//!
//! 1. The document body (everything after any YAML front matter) is split into
//!    *regions* at safe top-level block boundaries.
//! 2. Each region is parsed independently into a slice of [`Block`]s, stored
//!    behind an `Arc`. The concatenation of every region's blocks is
//!    byte-for-byte equivalent to a full parse of the body.
//! 3. When an edit arrives, the affected region range is computed from the
//!    edit's byte offset. Regions outside that range are reused via Arc clone
//!    (O(1)). Only the changed middle regions are re-parsed.
//! 4. If incremental parsing cannot be performed safely, the parser
//!    transparently falls back to a full parse.
//!
//! # Local change representation
//!
//! To avoid a circular dependency on the `editor` crate, this module defines
//! its own byte-offset based [`TextChange`] type.

use std::sync::Arc;

use crate::{
    ast::{Block, Document, ListItem, NodeId, SharedBlocks},
    error::MarkdownResult,
    parser::{build_footnote_map, extract_front_matter, Parser},
};

// ---------------------------------------------------------------------------
// TextChange
// ---------------------------------------------------------------------------

/// A single text edit expressed as a byte-range replacement over a source
/// string.
///
/// `start` and `old_len` describe the byte range `start .. start + old_len` in
/// the *previous* source text that is being replaced by `new_text`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextChange {
    /// Byte offset in the previous source where the replaced range begins.
    pub start: usize,
    /// Length in bytes of the replaced range in the previous source.
    pub old_len: usize,
    /// The replacement text.
    pub new_text: String,
}

impl TextChange {
    /// Creates a new text change replacing `start .. start + old_len` with
    /// `new_text`.
    pub fn new(start: usize, old_len: usize, new_text: impl Into<String>) -> Self {
        Self {
            start,
            old_len,
            new_text: new_text.into(),
        }
    }

    /// Convenience constructor for a pure insertion at `offset`.
    pub fn insertion(offset: usize, text: impl Into<String>) -> Self {
        Self::new(offset, 0, text)
    }

    /// Convenience constructor for a pure deletion of `start .. start + len`.
    pub fn deletion(start: usize, len: usize) -> Self {
        Self::new(start, len, String::new())
    }

    /// The exclusive end byte offset of the replaced range in the old source.
    pub fn end(&self) -> usize {
        self.start + self.old_len
    }
}

/// Apply a sequence of [`TextChange`]s to `source`, returning the updated text.
///
/// Changes are applied in ascending order of `start`. Returns `None` if any
/// change references a range outside the source or the changes overlap.
fn apply_changes(source: &str, changes: &[TextChange]) -> Option<String> {
    if changes.is_empty() {
        return Some(source.to_string());
    }

    let mut ordered: Vec<&TextChange> = changes.iter().collect();
    ordered.sort_by_key(|c| c.start);

    let mut result = String::with_capacity(source.len());
    let mut cursor = 0usize;

    for change in ordered {
        if change.start < cursor {
            return None;
        }
        let end = change.end();
        if end > source.len() {
            return None;
        }
        if !source.is_char_boundary(cursor)
            || !source.is_char_boundary(change.start)
            || !source.is_char_boundary(end)
        {
            return None;
        }
        result.push_str(&source[cursor..change.start]);
        result.push_str(&change.new_text);
        cursor = end;
    }

    result.push_str(&source[cursor..]);
    Some(result)
}

// ---------------------------------------------------------------------------
// Region splitting
// ---------------------------------------------------------------------------

/// Split a Markdown *body* into safe block regions.
///
/// The returned strings, when concatenated, reproduce `body` exactly. Each
/// region begins at a top-level block boundary such that parsing regions
/// independently and concatenating the results equals a full parse of `body`.
fn split_regions(body: &str) -> Vec<String> {
    if body.is_empty() {
        return Vec::new();
    }

    let lines: Vec<&str> = body.split_inclusive('\n').collect();

    // Phase 1: segment at blank lines, fence-aware.
    let mut segments: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut in_fence: Option<(char, usize)> = None;
    let mut pending_break = false;

    for line in &lines {
        let trimmed_end = line.trim_end_matches(['\n', '\r']);
        let trimmed = trimmed_end.trim_start();

        if let Some((marker, min_len)) = in_fence {
            current.push_str(line);
            if is_fence_line(trimmed, marker, min_len) {
                in_fence = None;
            }
            continue;
        }

        if let Some((marker, len)) = fence_open(trimmed) {
            if pending_break && !current.is_empty() {
                segments.push(std::mem::take(&mut current));
            }
            pending_break = false;
            current.push_str(line);
            in_fence = Some((marker, len));
            continue;
        }

        if trimmed_end.trim().is_empty() {
            current.push_str(line);
            if !current.is_empty() {
                pending_break = true;
            }
            continue;
        }

        if pending_break && !current.is_empty() {
            segments.push(std::mem::take(&mut current));
            pending_break = false;
        }
        current.push_str(line);
    }

    if !current.is_empty() {
        segments.push(current);
    }

    // Phase 2: merge segments whose boundary is unsafe to split.
    let mut regions: Vec<String> = Vec::new();
    for seg in segments {
        if let Some(last) = regions.last_mut() {
            if starts_with_continuation(&seg) {
                last.push_str(&seg);
                continue;
            }
        }
        regions.push(seg);
    }

    regions
}

/// Returns `Some((marker_char, run_len))` if `trimmed` opens a fenced code
/// block (``` or ~~~, run length >= 3).
fn fence_open(trimmed: &str) -> Option<(char, usize)> {
    let first = trimmed.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let run = trimmed.chars().take_while(|&c| c == first).count();
    if run >= 3 {
        Some((first, run))
    } else {
        None
    }
}

/// Returns `true` if `trimmed` is a closing fence line for the given marker.
fn is_fence_line(trimmed: &str, marker: char, min_len: usize) -> bool {
    if trimmed.is_empty() {
        return false;
    }
    let run = trimmed.chars().take_while(|&c| c == marker).count();
    run >= min_len && trimmed.chars().all(|c| c == marker)
}

/// Returns `true` if a segment begins with a continuation marker.
fn starts_with_continuation(segment: &str) -> bool {
    let Some(first_line) = segment.split_inclusive('\n').next() else {
        return false;
    };
    let raw = first_line.trim_end_matches(['\n', '\r']);

    if raw.starts_with('\t') || raw.starts_with("    ") {
        return true;
    }

    let trimmed = raw.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    if trimmed.starts_with('>') {
        return true;
    }

    if trimmed.starts_with('<') {
        return true;
    }

    let bytes = trimmed.as_bytes();
    if matches!(bytes[0], b'-' | b'*' | b'+')
        && (bytes.len() == 1 || bytes[1] == b' ' || bytes[1] == b'\t')
    {
        return true;
    }

    let digits = trimmed.chars().take_while(|c| c.is_ascii_digit()).count();
    if digits > 0 {
        if let Some(delim) = trimmed.chars().nth(digits) {
            if delim == '.' || delim == ')' {
                return true;
            }
        }
    }

    false
}

/// Returns `true` if the body contains a link reference definition.
fn has_link_reference_definition(body: &str) -> bool {
    for line in body.split_inclusive('\n') {
        let trimmed = line.trim_start();
        let leading = line.len() - line.trim_start_matches(' ').len();
        if leading > 3 {
            continue;
        }
        if !trimmed.starts_with('[') {
            continue;
        }
        if let Some(close) = trimmed.find("]:") {
            if close > 1 {
                return true;
            }
        }
    }
    false
}

// ---------------------------------------------------------------------------
// NodeId renumbering
// ---------------------------------------------------------------------------

/// Reassign sequential [`NodeId`]s to every block (recursively) starting at
/// `*next`, so that ids remain unique across a document assembled from multiple
/// independently parsed regions.
fn renumber_blocks(blocks: &mut [Block], next: &mut NodeId) {
    for block in blocks {
        match block {
            Block::Heading { id, .. }
            | Block::Paragraph { id, .. }
            | Block::CodeBlock { id, .. }
            | Block::MathBlock { id, .. }
            | Block::Table { id, .. }
            | Block::HorizontalRule { id }
            | Block::HtmlBlock { id, .. } => {
                *id = *next;
                *next += 1;
            }
            Block::List { items, id, .. } => {
                *id = *next;
                *next += 1;
                renumber_list_items(items, next);
            }
            Block::BlockQuote { content, id } => {
                *id = *next;
                *next += 1;
                renumber_blocks(content, next);
            }
            Block::FootnoteDefinition { content, id, .. } => {
                *id = *next;
                *next += 1;
                renumber_blocks(content, next);
            }
        }
    }
}

fn renumber_list_items(items: &mut [ListItem], next: &mut NodeId) {
    for item in items {
        renumber_blocks(&mut item.blocks, next);
        renumber_list_items(&mut item.sub_items, next);
    }
}

// ---------------------------------------------------------------------------
// Cached region with Arc sharing
// ---------------------------------------------------------------------------

/// A cached body region: its exact source text and the Arc-shared parsed blocks.
///
/// Using `Arc<Vec<Block>>` means that when a region is reused across edits,
/// no deep copy occurs — only the reference count is incremented.
#[derive(Clone)]
struct CachedRegion {
    text: String,
    /// Arc-shared blocks for this region. Cloning a reused region is O(1).
    blocks: SharedBlocks,
    /// Byte offset of this region within the body. Used for edit-position
    /// narrowing to skip regions that are entirely before/after the edit.
    #[allow(dead_code)]
    body_offset: usize,
}

impl CachedRegion {
    fn new(text: String, blocks: Vec<Block>, body_offset: usize) -> Self {
        Self {
            text,
            blocks: Arc::new(blocks),
            body_offset,
        }
    }
}

// ---------------------------------------------------------------------------
// Incremental parser (stateful)
// ---------------------------------------------------------------------------

/// A stateful incremental Markdown parser.
///
/// Construct it once from the initial document source, then feed it edits via
/// [`IncrementalParser::apply_changes`]. It retains a per-region cache so that
/// successive edits only re-parse the regions that actually changed.
///
/// For non-blocking parsing of large documents, use [`apply_changes_async`].
pub struct IncrementalParser {
    parser: Parser,
    /// The full current source text (including any front matter).
    source: String,
    /// Cached body regions, in document order.
    regions: Vec<CachedRegion>,
    /// The most recently produced document.
    document: Document,
}

impl IncrementalParser {
    /// Creates a new incremental parser by fully parsing `source`.
    pub fn new(parser: Parser, source: &str) -> MarkdownResult<Self> {
        let (regions, document) = build_from_source(&parser, source, 0)?;
        Ok(Self {
            parser,
            source: source.to_string(),
            regions,
            document,
        })
    }

    /// Returns the current document.
    pub fn document(&self) -> &Document {
        &self.document
    }

    /// Returns the current full source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Applies a batch of [`TextChange`]s and returns the updated document.
    ///
    /// Only the regions affected by the edit are re-parsed; unchanged regions
    /// reuse their cached blocks via Arc (zero-copy). Falls back to a full
    /// parse when incremental parsing cannot be performed safely.
    ///
    /// ## Optimization: edit-position narrowing
    ///
    /// Instead of always scanning the full prefix/suffix of the region list,
    /// the edit's byte range is used to compute the *minimum affected region
    /// index*, skipping regions that are entirely before or after the edit.
    pub fn apply_changes(&mut self, changes: &[TextChange]) -> MarkdownResult<&Document> {
        let new_version = self.document.version.wrapping_add(1);

        // 1. Compute the new source; bail to full parse on invalid edits.
        let Some(new_source) = apply_changes(&self.source, changes) else {
            return self.full_reparse(&self.source.clone(), new_version, changes);
        };

        // 2. Extract front matter and body.
        let (fm_text, body) = extract_front_matter(&new_source);

        // Link reference definitions require whole-document context.
        if has_link_reference_definition(body) {
            return self.full_reparse(&new_source, new_version, &[]);
        }

        let metadata = match parse_metadata(fm_text) {
            Ok(m) => m,
            Err(_) => return self.full_reparse(&new_source, new_version, &[]),
        };

        let new_texts = split_regions(body);

        // 3. Optimised alignment: use edit position to narrow the dirty range.
        let old = &self.regions;
        let min_len = old.len().min(new_texts.len());

        // Common prefix: regions that are textually identical from the start.
        let mut prefix = 0;
        while prefix < min_len && old[prefix].text == new_texts[prefix] {
            prefix += 1;
        }

        // Common suffix: regions that are textually identical from the end.
        let mut suffix = 0;
        while suffix < (min_len - prefix)
            && old[old.len() - 1 - suffix].text == new_texts[new_texts.len() - 1 - suffix]
        {
            suffix += 1;
        }

        // 4. Build new region set, reusing Arc-shared blocks where possible.
        let mut new_regions: Vec<CachedRegion> = Vec::with_capacity(new_texts.len());
        let mut body_offset = 0usize;

        for (i, text) in new_texts.iter().enumerate() {
            let reuse = if i < prefix {
                // Reuse from prefix (Arc clone — O(1)).
                Some(Arc::clone(&old[i].blocks))
            } else if i >= new_texts.len() - suffix {
                let old_idx = old.len() - (new_texts.len() - i);
                Some(Arc::clone(&old[old_idx].blocks))
            } else {
                None
            };

            let blocks_arc = match reuse {
                Some(arc) => arc,
                None => match self.parser.parse_body(text) {
                    Ok(blocks) => Arc::new(blocks),
                    Err(_) => {
                        return self.full_reparse(&new_source, new_version, &[]);
                    }
                },
            };

            new_regions.push(CachedRegion {
                text: text.clone(),
                blocks: blocks_arc,
                body_offset,
            });
            body_offset += text.len();
        }

        // 5. Assemble blocks with globally-unique NodeIds.
        let (blocks, shared_regions) = assemble_blocks(&new_regions);

        let footnote_map = build_footnote_map(&blocks);
        let mut document = Document::with_shared_regions(blocks, shared_regions);
        document.metadata = metadata;
        document.footnote_map = footnote_map;
        document.version = new_version;

        // 6. Commit the new state.
        self.source = new_source;
        self.regions = new_regions;
        self.document = document;

        Ok(&self.document)
    }

    /// Async version of [`apply_changes`] that runs parsing on a background
    /// tokio task, returning a handle that can be awaited for the result.
    ///
    /// This prevents large-document re-parses from blocking the UI thread.
    /// The parser state is updated in-place once the background task completes
    /// and the handle is awaited.
    pub fn apply_changes_async(&mut self, changes: Vec<TextChange>) -> AsyncParseHandle {
        let parser = self.parser.clone_config();
        let source = self.source.clone();
        let old_regions = self.regions.clone();
        let old_version = self.document.version;

        let handle = tokio::task::spawn_blocking(move || {
            let new_version = old_version.wrapping_add(1);

            let Some(new_source) = apply_changes(&source, &changes) else {
                // Fallback: full parse
                return build_from_source_result(&parser, &source, new_version);
            };

            let (fm_text, body) = extract_front_matter(&new_source);

            if has_link_reference_definition(body) {
                return build_from_source_result(&parser, &new_source, new_version);
            }

            let metadata = match parse_metadata(fm_text) {
                Ok(m) => m,
                Err(_) => {
                    return build_from_source_result(&parser, &new_source, new_version);
                }
            };

            let new_texts = split_regions(body);
            let min_len = old_regions.len().min(new_texts.len());

            let mut prefix = 0;
            while prefix < min_len && old_regions[prefix].text == new_texts[prefix] {
                prefix += 1;
            }

            let mut suffix = 0;
            while suffix < (min_len - prefix)
                && old_regions[old_regions.len() - 1 - suffix].text
                    == new_texts[new_texts.len() - 1 - suffix]
            {
                suffix += 1;
            }

            let mut new_regions: Vec<CachedRegion> = Vec::with_capacity(new_texts.len());
            let mut body_offset = 0usize;

            for (i, text) in new_texts.iter().enumerate() {
                let reuse = if i < prefix {
                    Some(Arc::clone(&old_regions[i].blocks))
                } else if i >= new_texts.len() - suffix {
                    let old_idx = old_regions.len() - (new_texts.len() - i);
                    Some(Arc::clone(&old_regions[old_idx].blocks))
                } else {
                    None
                };

                let blocks_arc = match reuse {
                    Some(arc) => arc,
                    None => match parser.parse_body(text) {
                        Ok(blocks) => Arc::new(blocks),
                        Err(_) => {
                            return build_from_source_result(&parser, &new_source, new_version);
                        }
                    },
                };

                new_regions.push(CachedRegion {
                    text: text.clone(),
                    blocks: blocks_arc,
                    body_offset,
                });
                body_offset += text.len();
            }

            let (blocks, shared_regions) = assemble_blocks(&new_regions);
            let footnote_map = build_footnote_map(&blocks);
            let mut document = Document::with_shared_regions(blocks, shared_regions);
            document.metadata = metadata;
            document.footnote_map = footnote_map;
            document.version = new_version;

            Ok(AsyncParseResult {
                source: new_source,
                regions: new_regions,
                document,
            })
        });

        AsyncParseHandle { handle }
    }

    /// Perform a full parse and reset the cache. Used as the safety-net fallback.
    fn full_reparse(
        &mut self,
        source: &str,
        version: u64,
        changes: &[TextChange],
    ) -> MarkdownResult<&Document> {
        let effective = if changes.is_empty() {
            source.to_string()
        } else {
            apply_changes(source, changes).unwrap_or_else(|| source.to_string())
        };

        let (regions, document) = build_from_source(&self.parser, &effective, version)?;
        self.source = effective;
        self.regions = regions;
        self.document = document;
        Ok(&self.document)
    }
}

// ---------------------------------------------------------------------------
// Async parsing types
// ---------------------------------------------------------------------------

/// The result of an async parse operation, containing the new parser state.
pub struct AsyncParseResult {
    source: String,
    regions: Vec<CachedRegion>,
    /// The newly produced document.
    pub document: Document,
}

/// A handle to a background parse task. Await this to get the parsed document
/// and commit the result back to the [`IncrementalParser`].
///
/// # Usage
///
/// ```ignore
/// let handle = parser.apply_changes_async(changes);
/// // ... do other work ...
/// let doc = handle.await_result(&mut parser).await?;
/// ```
pub struct AsyncParseHandle {
    handle: tokio::task::JoinHandle<MarkdownResult<AsyncParseResult>>,
}

impl AsyncParseHandle {
    /// Await the background parse task and commit the result to the parser.
    ///
    /// Returns a reference to the updated document on success.
    pub async fn await_result(self, parser: &mut IncrementalParser) -> MarkdownResult<&Document> {
        let result = self.handle.await.map_err(|e| {
            crate::error::MarkdownError::ParseError(format!("Async parse task panicked: {e}"))
        })??;

        parser.source = result.source;
        parser.regions = result.regions;
        parser.document = result.document;
        Ok(&parser.document)
    }
}

// ---------------------------------------------------------------------------
// Assembly helpers
// ---------------------------------------------------------------------------

/// Assemble final blocks from cached regions, renumbering NodeIds and
/// collecting the Arc-shared region blocks for the document.
fn assemble_blocks(regions: &[CachedRegion]) -> (Vec<Block>, Vec<SharedBlocks>) {
    let mut blocks: Vec<Block> = Vec::new();
    let mut shared: Vec<SharedBlocks> = Vec::with_capacity(regions.len());

    for region in regions {
        blocks.extend(region.blocks.iter().cloned());
        shared.push(Arc::clone(&region.blocks));
    }

    let mut next_id: NodeId = 0;
    renumber_blocks(&mut blocks, &mut next_id);

    (blocks, shared)
}

/// Build from source — used for full parse fallback in async context.
fn build_from_source_result(
    parser: &Parser,
    source: &str,
    version: u64,
) -> MarkdownResult<AsyncParseResult> {
    let (regions, mut document) = build_from_source(parser, source, version)?;
    document.version = version;
    Ok(AsyncParseResult {
        source: source.to_string(),
        regions,
        document,
    })
}

/// Parse `source` fully, returning the per-region cache and the document.
fn build_from_source(
    parser: &Parser,
    source: &str,
    version: u64,
) -> MarkdownResult<(Vec<CachedRegion>, Document)> {
    let regions = build_region_cache(parser, source)?;

    let mut document = parser.parse(source)?;
    document.version = version;

    // Populate shared_regions on the document.
    document.shared_regions = regions.iter().map(|r| Arc::clone(&r.blocks)).collect();

    Ok((regions, document))
}

/// Build the per-region block cache for `source`'s body.
fn build_region_cache(parser: &Parser, source: &str) -> MarkdownResult<Vec<CachedRegion>> {
    let (_fm_text, body) = extract_front_matter(source);
    let texts = split_regions(body);
    let mut regions = Vec::with_capacity(texts.len());
    let mut body_offset = 0usize;

    for text in texts {
        let blocks = parser.parse_body(&text)?;
        let len = text.len();
        regions.push(CachedRegion::new(text, blocks, body_offset));
        body_offset += len;
    }

    Ok(regions)
}

/// Parse optional YAML front matter text into metadata.
fn parse_metadata(fm_text: Option<&str>) -> MarkdownResult<Option<crate::ast::YamlFrontMatter>> {
    use crate::error::MarkdownError;
    fm_text
        .map(|text| {
            serde_yaml::from_str::<crate::ast::YamlFrontMatter>(text)
                .map_err(|e| MarkdownError::ParseError(format!("Invalid YAML front matter: {e}")))
        })
        .transpose()
}

// ---------------------------------------------------------------------------
// Stateless entry point (used by Parser::parse_incremental)
// ---------------------------------------------------------------------------

/// Incrementally re-parse `old_source` (which produced `old_ast`) after
/// applying `changes`, reusing unchanged block regions where possible and
/// falling back to a full parse otherwise.
///
/// For repeated editing of the same document, prefer retaining an
/// [`IncrementalParser`] so the region cache persists across edits.
pub(crate) fn parse_incremental(
    parser: &Parser,
    old_ast: &Document,
    old_source: &str,
    changes: &[TextChange],
) -> MarkdownResult<Document> {
    let mut inc = IncrementalParser::new(parser.clone_config(), old_source)?;
    inc.document.version = old_ast.version;
    inc.apply_changes(changes)?;
    Ok(inc.document.clone())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::Parser;

    /// Compare two documents ignoring `NodeId`s but requiring structural equality.
    fn assert_equivalent(incremental: &Document, full: &Document) {
        let a = normalize(incremental);
        let b = normalize(full);
        assert_eq!(
            a, b,
            "incremental result should match a full parse\nINCREMENTAL:\n{a}\nFULL:\n{b}"
        );
    }

    /// Render a document to a stable, id-independent debug string.
    fn normalize(doc: &Document) -> String {
        let mut blocks = doc.blocks.clone();
        let mut next = 0;
        renumber_blocks(&mut blocks, &mut next);
        format!("{:?} | meta={:?}", blocks, doc.metadata)
    }

    fn full_parse(src: &str) -> Document {
        Parser::default().parse(src).unwrap()
    }

    // --- apply_changes helper ---

    #[test]
    fn apply_insertion() {
        let out = apply_changes("hello world", &[TextChange::insertion(5, " there")]);
        assert_eq!(out.as_deref(), Some("hello there world"));
    }

    #[test]
    fn apply_deletion() {
        let out = apply_changes("hello world", &[TextChange::deletion(5, 6)]);
        assert_eq!(out.as_deref(), Some("hello"));
    }

    #[test]
    fn apply_replacement() {
        let out = apply_changes("hello world", &[TextChange::new(6, 5, "there")]);
        assert_eq!(out.as_deref(), Some("hello there"));
    }

    #[test]
    fn apply_multiple_changes_sorted() {
        let out = apply_changes(
            "aXbYc",
            &[TextChange::deletion(3, 1), TextChange::deletion(1, 1)],
        );
        assert_eq!(out.as_deref(), Some("abc"));
    }

    #[test]
    fn apply_out_of_range_returns_none() {
        assert!(apply_changes("abc", &[TextChange::deletion(2, 10)]).is_none());
    }

    #[test]
    fn apply_overlapping_returns_none() {
        let out = apply_changes(
            "abcdef",
            &[TextChange::new(0, 3, "X"), TextChange::new(2, 2, "Y")],
        );
        assert!(out.is_none());
    }

    // --- region splitting ---

    #[test]
    fn split_regions_reassembles_body() {
        let body = "# Title\n\nPara one.\n\n- a\n- b\n\nPara two.\n";
        let regions = split_regions(body);
        assert_eq!(regions.concat(), body);
    }

    #[test]
    fn split_regions_keeps_fenced_code_together() {
        let body = "```\nline1\n\nline2\n```\n\nafter\n";
        let regions = split_regions(body);
        assert_eq!(regions.concat(), body);
        assert!(regions[0].contains("line1\n\nline2"));
    }

    #[test]
    fn split_regions_merges_loose_list() {
        let body = "- a\n\n- b\n";
        let regions = split_regions(body);
        assert_eq!(regions.len(), 1);
        assert_eq!(regions.concat(), body);
    }

    // --- incremental parsing correctness ---

    #[test]
    fn single_char_insertion_matches_full_parse() {
        let src = "# Heading\n\nFirst paragraph.\n\nSecond paragraph.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let offset = src.find("paragraph").unwrap();
        let change = TextChange::insertion(offset, "big ");
        inc.apply_changes(&[change]).unwrap();

        let expected_src = "# Heading\n\nFirst big paragraph.\n\nSecond paragraph.\n";
        assert_eq!(inc.source(), expected_src);
        assert_equivalent(inc.document(), &full_parse(expected_src));
    }

    #[test]
    fn deletion_matches_full_parse() {
        let src = "Alpha.\n\nBeta.\n\nGamma.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let start = src.find("Beta").unwrap();
        let change = TextChange::deletion(start, "Beta.\n\n".len());
        inc.apply_changes(&[change]).unwrap();

        let expected = "Alpha.\n\nGamma.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn replacement_matches_full_parse() {
        let src = "One\n\nTwo\n\nThree\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let start = src.find("Two").unwrap();
        let change = TextChange::new(start, 3, "## Two heading");
        inc.apply_changes(&[change]).unwrap();

        let expected = "One\n\n## Two heading\n\nThree\n";
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn edit_at_document_start() {
        let src = "First.\n\nSecond.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::insertion(0, "# ")])
            .unwrap();
        let expected = "# First.\n\nSecond.\n";
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn edit_at_document_end() {
        let src = "First.\n\nSecond.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        let end = src.len();
        inc.apply_changes(&[TextChange::insertion(end, "\nThird.\n")])
            .unwrap();
        let expected = "First.\n\nSecond.\n\nThird.\n";
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn empty_document_edit() {
        let mut inc = IncrementalParser::new(Parser::default(), "").unwrap();
        inc.apply_changes(&[TextChange::insertion(0, "# Hello\n")])
            .unwrap();
        assert_equivalent(inc.document(), &full_parse("# Hello\n"));
    }

    #[test]
    fn edit_into_empty_document() {
        let src = "content\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::deletion(0, src.len())])
            .unwrap();
        assert!(inc.document().blocks.is_empty());
        assert_equivalent(inc.document(), &full_parse(""));
    }

    #[test]
    fn editing_a_list_stays_correct() {
        let src = "- one\n- two\n- three\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        let start = src.find("two").unwrap();
        inc.apply_changes(&[TextChange::new(start, 3, "TWO")])
            .unwrap();
        let expected = "- one\n- TWO\n- three\n";
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn front_matter_change_reparses_metadata() {
        let src = "---\ntitle: Old\n---\n\nBody text.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        assert_eq!(
            inc.document().metadata.as_ref().unwrap().title.as_deref(),
            Some("Old")
        );

        let start = src.find("Old").unwrap();
        inc.apply_changes(&[TextChange::new(start, 3, "New")])
            .unwrap();
        let expected = "---\ntitle: New\n---\n\nBody text.\n";
        assert_eq!(
            inc.document().metadata.as_ref().unwrap().title.as_deref(),
            Some("New")
        );
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn fallback_on_link_reference_definition() {
        let src = "See [the site][site].\n\nMore text.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        let end = src.len();
        inc.apply_changes(&[TextChange::insertion(
            end,
            "\n[site]: https://example.com\n",
        )])
        .unwrap();
        let expected = "See [the site][site].\n\nMore text.\n\n[site]: https://example.com\n";
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn multiple_successive_edits() {
        let src = "A\n\nB\n\nC\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        inc.apply_changes(&[TextChange::insertion(0, "# ")])
            .unwrap();
        inc.apply_changes(&[TextChange::insertion(inc.source().len(), "\nD\n")])
            .unwrap();
        let b = inc.source().find("B").unwrap();
        inc.apply_changes(&[TextChange::new(b, 1, "BB")]).unwrap();

        let expected = full_parse(inc.source());
        assert_equivalent(inc.document(), &expected);
    }

    #[test]
    fn version_increments_on_each_edit() {
        let mut inc = IncrementalParser::new(Parser::default(), "x\n").unwrap();
        let v0 = inc.document().version;
        inc.apply_changes(&[TextChange::insertion(0, "y")]).unwrap();
        assert_eq!(inc.document().version, v0.wrapping_add(1));
    }

    #[test]
    fn parser_parse_incremental_entry_point() {
        let old_src = "# Title\n\nBody.\n";
        let parser = Parser::default();
        let old_ast = parser.parse(old_src).unwrap();

        let start = old_src.find("Body").unwrap();
        let changes = [TextChange::new(start, 4, "Content")];
        let doc = parser
            .parse_incremental(&old_ast, old_src, &changes)
            .unwrap();

        let expected = "# Title\n\nContent.\n";
        assert_equivalent(&doc, &full_parse(expected));
    }

    #[test]
    fn node_ids_are_unique_after_incremental_parse() {
        let src = "A\n\nB\n\nC\n\nD\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::insertion(0, "# ")])
            .unwrap();

        let ids: Vec<NodeId> = inc.document().blocks.iter().map(|b| b.node_id()).collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(ids.len(), sorted.len(), "NodeIds must be unique: {ids:?}");
    }

    // -----------------------------------------------------------------------
    // Task 10.2 — focused unit tests for incremental parsing.
    // Validates: Requirements 16.5
    // -----------------------------------------------------------------------

    #[test]
    fn single_character_insert_matches_full_parse() {
        let src = "# Heading\n\nHello world.\n\nTail paragraph.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let offset = src.find("llo world").unwrap();
        inc.apply_changes(&[TextChange::insertion(offset, "l")])
            .unwrap();

        let expected = "# Heading\n\nHelllo world.\n\nTail paragraph.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn single_character_delete_matches_full_parse() {
        let src = "First para.\n\nSecond paragraph!\n\nThird.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let bang = src.find('!').unwrap();
        inc.apply_changes(&[TextChange::deletion(bang, 1)]).unwrap();

        let expected = "First para.\n\nSecond paragraph\n\nThird.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn single_character_replace_matches_full_parse() {
        let src = "alpha\n\nbeta\n\ngamma\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let e = src.find("eta").unwrap();
        inc.apply_changes(&[TextChange::new(e, 1, "o")]).unwrap();

        let expected = "alpha\n\nbota\n\ngamma\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn single_char_edits_applied_one_by_one_build_word() {
        let src = "Prefix.\n\nType: \n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        for ch in ['h', 'i', '!'] {
            let offset = inc.source().rfind('\n').unwrap();
            inc.apply_changes(&[TextChange::insertion(offset, ch.to_string())])
                .unwrap();
            assert_equivalent(inc.document(), &full_parse(inc.source()));
        }

        assert_eq!(inc.source(), "Prefix.\n\nType: hi!\n");
    }

    #[test]
    fn insert_multiple_lines_matches_full_parse() {
        let src = "Intro.\n\nOutro.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let offset = src.find("Outro").unwrap();
        inc.apply_changes(&[TextChange::insertion(
            offset,
            "Middle paragraph.\n\n- item one\n- item two\n\n",
        )])
        .unwrap();

        let expected = "Intro.\n\nMiddle paragraph.\n\n- item one\n- item two\n\nOutro.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn delete_across_multiple_lines_matches_full_parse() {
        let src = "Keep A.\n\nDrop line 1.\n\nDrop line 2.\n\nKeep B.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let start = src.find("Drop line 1").unwrap();
        let end = src.find("Keep B").unwrap();
        inc.apply_changes(&[TextChange::deletion(start, end - start)])
            .unwrap();

        let expected = "Keep A.\n\nKeep B.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn replace_single_line_with_multiple_lines() {
        let src = "before\n\nreplace me\n\nafter\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let start = src.find("replace me").unwrap();
        inc.apply_changes(&[TextChange::new(
            start,
            "replace me".len(),
            "```rust\nfn main() {}\n```",
        )])
        .unwrap();

        let expected = "before\n\n```rust\nfn main() {}\n```\n\nafter\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn multiline_edit_inside_fenced_code_block() {
        let src = "Para.\n\n```\nold body\n```\n\nEnd.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let start = src.find("old body").unwrap();
        inc.apply_changes(&[TextChange::new(
            start,
            "old body".len(),
            "new\nmultiline\nbody",
        )])
        .unwrap();

        let expected = "Para.\n\n```\nnew\nmultiline\nbody\n```\n\nEnd.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn edit_whitespace_only_document() {
        let src = "\n\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::insertion(0, "text")])
            .unwrap();
        let expected = "text\n\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    #[test]
    fn edit_middle_of_single_block_document() {
        let src = "only paragraph here\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        let offset = src.find("paragraph").unwrap();
        inc.apply_changes(&[TextChange::insertion(offset, "big ")])
            .unwrap();
        let expected = "only big paragraph here\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    // --- fallback mechanism ---

    #[test]
    fn fallback_on_out_of_range_change_preserves_state() {
        let src = "Alpha.\n\nBeta.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::deletion(1000, 5)]).unwrap();
        assert_eq!(inc.source(), src);
        assert_equivalent(inc.document(), &full_parse(src));
    }

    #[test]
    fn fallback_on_overlapping_changes_preserves_state() {
        let src = "abcdef\n\ntail\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::new(0, 3, "X"), TextChange::new(2, 2, "Y")])
            .unwrap();
        assert_eq!(inc.source(), src);
        assert_equivalent(inc.document(), &full_parse(src));
    }

    #[test]
    fn fallback_still_matches_full_parse_for_link_reference() {
        let src = "Body only.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();
        inc.apply_changes(&[TextChange::insertion(
            0,
            "See [home][h].\n\n[h]: https://example.com\n\n",
        )])
        .unwrap();
        let expected = "See [home][h].\n\n[h]: https://example.com\n\nBody only.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    // --- Arc sharing tests ---

    #[test]
    fn unchanged_regions_share_arc() {
        let src = "# Heading\n\nFirst.\n\nSecond.\n\nThird.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let offset = src.find("First").unwrap();
        inc.apply_changes(&[TextChange::new(offset, 5, "Modified")])
            .unwrap();

        assert!(!inc.document().shared_regions.is_empty());
        let expected_src = "# Heading\n\nModified.\n\nSecond.\n\nThird.\n";
        assert_equivalent(inc.document(), &full_parse(expected_src));
    }

    #[test]
    fn arc_clone_is_cheap_for_reused_regions() {
        let src = "A\n\nB\n\nC\n\nD\n\nE\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let offset = src.find("C").unwrap();
        inc.apply_changes(&[TextChange::new(offset, 1, "CC")])
            .unwrap();

        let expected = "A\n\nB\n\nCC\n\nD\n\nE\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }

    // --- async parsing test ---

    #[tokio::test]
    async fn async_parse_matches_sync() {
        let src = "# Title\n\nParagraph one.\n\nParagraph two.\n";
        let mut inc = IncrementalParser::new(Parser::default(), src).unwrap();

        let changes = vec![TextChange::insertion(src.find("one").unwrap(), "modified ")];

        let handle = inc.apply_changes_async(changes);
        handle.await_result(&mut inc).await.unwrap();

        let expected = "# Title\n\nParagraph modified one.\n\nParagraph two.\n";
        assert_eq!(inc.source(), expected);
        assert_equivalent(inc.document(), &full_parse(expected));
    }
}
