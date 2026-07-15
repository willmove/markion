//! Markdown parser wrapping pulldown-cmark.
//!
//! Handles YAML front matter extraction, then delegates to pulldown-cmark
//! for CommonMark + GFM parsing.

use pulldown_cmark::{
    Alignment as PdAlignment, Event, HeadingLevel, Options, Parser as PdParser, Tag, TagEnd,
};

use crate::{
    ast::{Alignment, Block, Document, Inline, ListItem, TableCell, YamlFrontMatter},
    error::{MarkdownError, MarkdownResult},
};

// ---------------------------------------------------------------------------
// ParserOptions
// ---------------------------------------------------------------------------

/// Configuration flags for the Markdown parser.
#[derive(Debug, Clone)]
pub struct ParserOptions {
    /// Enable GitHub Flavored Markdown extensions.
    pub enable_gfm: bool,
    /// Enable GFM table syntax.
    pub enable_tables: bool,
    /// Enable footnote syntax.
    pub enable_footnotes: bool,
    /// Enable task-list checkbox syntax (`- [ ]` / `- [x]`).
    pub enable_task_lists: bool,
    /// Enable GFM strikethrough (`~~text~~`).
    pub enable_strikethrough: bool,
    /// Enable HTML pass-through (preserve HTML tags in output).
    pub enable_html: bool,
    /// Enable `$...$` / `$$...$$` math parsing into
    /// [`Inline::InlineMath`]/[`Block::MathBlock`] nodes. (Available since
    /// the pulldown-cmark 0.13 unification; previously math stayed plain
    /// text.)
    pub enable_math: bool,
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self {
            enable_gfm: true,
            enable_tables: true,
            enable_footnotes: true,
            enable_task_lists: true,
            enable_strikethrough: true,
            enable_html: true,
            enable_math: true,
        }
    }
}

impl ParserOptions {
    /// Build the pulldown-cmark `Options` bitset from this config.
    fn to_pd_options(&self) -> Options {
        let mut opts = Options::empty();
        if self.enable_tables {
            opts |= Options::ENABLE_TABLES;
        }
        if self.enable_footnotes {
            opts |= Options::ENABLE_FOOTNOTES;
        }
        if self.enable_task_lists {
            opts |= Options::ENABLE_TASKLISTS;
        }
        if self.enable_strikethrough {
            opts |= Options::ENABLE_STRIKETHROUGH;
        }
        if self.enable_math {
            opts |= Options::ENABLE_MATH;
        }
        // GFM flag in pulldown-cmark 0.11
        if self.enable_gfm {
            opts |= Options::ENABLE_GFM;
        }
        opts
    }
}

// ---------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------

/// The Markdown parser.
pub struct Parser {
    options: ParserOptions,
}

impl Default for Parser {
    /// Creates a parser with all extensions enabled.
    fn default() -> Self {
        Self::new(ParserOptions::default())
    }
}

impl Parser {
    /// Creates a new parser with the given options.
    pub fn new(options: ParserOptions) -> Self {
        Self { options }
    }

    /// Returns a new [`Parser`] with the same options as this one.
    pub(crate) fn clone_config(&self) -> Parser {
        Parser {
            options: self.options.clone(),
        }
    }

    /// Parse a Markdown string into a [`Document`].
    ///
    /// YAML front matter (a `---`-delimited block at the start) is extracted
    /// and parsed separately before the Markdown body is processed.
    pub fn parse(&self, markdown: &str) -> MarkdownResult<Document> {
        // 1. Strip YAML front matter
        let (yaml_fm, body) = extract_front_matter(markdown);

        let metadata = yaml_fm
            .map(|fm_text| {
                serde_yaml::from_str::<YamlFrontMatter>(fm_text).map_err(|e| {
                    MarkdownError::ParseError(format!("Invalid YAML front matter: {e}"))
                })
            })
            .transpose()?;

        // 2. Parse the markdown body into blocks (URL post-processing included).
        let blocks = self.parse_body(body)?;

        // 3. Build footnote map
        let footnote_map = build_footnote_map(&blocks);

        let mut doc = Document::new(blocks);
        doc.metadata = metadata;
        doc.footnote_map = footnote_map;

        Ok(doc)
    }

    /// Parse a Markdown *body* fragment (with no YAML front matter handling)
    /// into a list of top-level [`Block`]s.
    ///
    /// This is the shared core used by both [`Parser::parse`] and the
    /// incremental parser. It intentionally does **not** strip front matter,
    /// so that body fragments beginning with `---` are treated as thematic
    /// breaks / headings rather than metadata.
    ///
    /// NodeIds are assigned starting from `0`; callers that combine fragments
    /// are responsible for renumbering ids to keep them unique across the
    /// whole document (see [`crate::incremental`]).
    pub(crate) fn parse_body(&self, body: &str) -> MarkdownResult<Vec<Block>> {
        let opts = self.options.to_pd_options();
        let events: Vec<Event<'_>> = PdParser::new_ext(body, opts).collect();

        let mut builder = AstBuilder::new();
        let mut blocks = builder.build(&events)?;

        // Post-process: detect and convert bare URLs in text nodes to links.
        post_process_url_detection_in_blocks(&mut blocks);

        Ok(blocks)
    }

    /// Incrementally re-parse a document after a set of [`TextChange`]s.
    ///
    /// `old_source` is the Markdown text that produced `old_ast`, and `changes`
    /// describes edits applied to that text. The method re-parses only the
    /// affected block regions and reuses the unchanged surrounding blocks.
    ///
    /// If incremental parsing cannot be performed safely (for example the
    /// change offsets are out of range, or the front matter boundary shifts),
    /// this transparently falls back to a full [`Parser::parse`] of the updated
    /// source. The returned [`Document`] is always equivalent to a full parse.
    ///
    /// [`TextChange`]: crate::incremental::TextChange
    pub fn parse_incremental(
        &self,
        old_ast: &Document,
        old_source: &str,
        changes: &[crate::incremental::TextChange],
    ) -> MarkdownResult<Document> {
        crate::incremental::parse_incremental(self, old_ast, old_source, changes)
    }
}

/// Build a footnote label → definition NodeId map from a list of blocks.
pub(crate) fn build_footnote_map(
    blocks: &[Block],
) -> std::collections::HashMap<String, crate::ast::NodeId> {
    let mut footnote_map = std::collections::HashMap::new();
    for block in blocks {
        if let Block::FootnoteDefinition { label, id, .. } = block {
            footnote_map.insert(label.clone(), *id);
        }
    }
    footnote_map
}

// ---------------------------------------------------------------------------
// Front-matter extraction
// ---------------------------------------------------------------------------

/// Splits off an optional YAML front-matter block from the start of a document.
///
/// Returns `(Some(yaml_text), rest)` when the document starts with `---\n`,
/// or `(None, full_markdown)` when no front matter is present.
pub(crate) fn extract_front_matter(src: &str) -> (Option<&str>, &str) {
    // Must start with exactly `---` on its own line
    let Some(after_open) = src
        .strip_prefix("---\n")
        .or_else(|| src.strip_prefix("---\r\n"))
    else {
        return (None, src);
    };

    // Find the closing `---` line
    let close = after_open
        .find("\n---\n")
        .or_else(|| after_open.find("\n---\r\n"))
        .or_else(|| {
            // Handle trailing `---` at end-of-string
            if after_open.ends_with("\n---") {
                Some(after_open.len() - 4)
            } else {
                None
            }
        });

    match close {
        Some(end) => {
            let yaml = &after_open[..end];
            // Skip past the closing `---` delimiter (with optional CRLF)
            let rest_start = end + 1; // skip '\n'
            let rest_after_dashes = &after_open[rest_start..];
            let rest = rest_after_dashes
                .strip_prefix("---\r\n")
                .or_else(|| rest_after_dashes.strip_prefix("---\n"))
                .unwrap_or(rest_after_dashes);
            (Some(yaml), rest)
        }
        None => (None, src),
    }
}

// ---------------------------------------------------------------------------
// AstBuilder — converts pulldown-cmark events to our Block/Inline AST
// ---------------------------------------------------------------------------

struct AstBuilder {
    next_id: usize,
}

impl AstBuilder {
    fn new() -> Self {
        Self { next_id: 0 }
    }

    fn alloc_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// Convert a flat slice of pulldown-cmark events to a `Vec<Block>`.
    fn build<'a>(&mut self, events: &[Event<'a>]) -> MarkdownResult<Vec<Block>> {
        let mut pos = 0;
        let mut blocks = Vec::new();
        while pos < events.len() {
            match &events[pos] {
                Event::Start(Tag::Heading { level, .. }) => {
                    let (heading, consumed) = self.parse_heading(events, pos, *level)?;
                    blocks.push(heading);
                    pos += consumed;
                }
                Event::Start(Tag::Paragraph) => {
                    let (para, consumed) = self.parse_paragraph(events, pos)?;
                    blocks.push(para);
                    pos += consumed;
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let s = lang.to_string();
                            if s.is_empty() { None } else { Some(s) }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                    let (cb, consumed) = self.parse_code_block(events, pos, lang)?;
                    blocks.push(cb);
                    pos += consumed;
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    let (bq, consumed) = self.parse_block_quote(events, pos)?;
                    blocks.push(bq);
                    pos += consumed;
                }
                Event::Start(Tag::List(start)) => {
                    let ordered = start.is_some();
                    let start_num = *start;
                    let (list, consumed) = self.parse_list(events, pos, ordered, start_num)?;
                    blocks.push(list);
                    pos += consumed;
                }
                Event::Start(Tag::Table(alignments)) => {
                    let aligns: Vec<Alignment> = alignments.iter().map(convert_alignment).collect();
                    let (table, consumed) = self.parse_table(events, pos, aligns)?;
                    blocks.push(table);
                    pos += consumed;
                }
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    let label_str = label.to_string();
                    let (footnote, consumed) =
                        self.parse_footnote_definition(events, pos, label_str)?;
                    blocks.push(footnote);
                    pos += consumed;
                }
                Event::Rule => {
                    blocks.push(Block::HorizontalRule {
                        id: self.alloc_id(),
                    });
                    pos += 1;
                }
                Event::Html(html) => {
                    // Create HtmlBlock for block-level HTML
                    blocks.push(Block::HtmlBlock {
                        content: html.to_string(),
                        id: self.alloc_id(),
                    });
                    pos += 1;
                }
                // Math blocks emitted as code blocks with lang `math` by some processors;
                // we also handle `$$…$$` via Text events inside a paragraph check below.
                _ => {
                    pos += 1;
                }
            }
        }
        Ok(blocks)
    }

    // -----------------------------------------------------------------------
    // Block parsers
    // -----------------------------------------------------------------------

    fn parse_heading<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
        level: HeadingLevel,
    ) -> MarkdownResult<(Block, usize)> {
        let level_u8 = heading_level_to_u8(level);
        let id = self.alloc_id();
        let end_tag = TagEnd::Heading(level);

        let mut pos = start + 1;
        let inner = collect_until_end(events, &mut pos, end_tag);
        let content = self.parse_inlines(inner)?;

        Ok((
            Block::Heading {
                level: level_u8,
                content,
                id,
            },
            pos - start,
        ))
    }

    fn parse_paragraph<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let mut pos = start + 1;
        let inner = collect_until_end(events, &mut pos, TagEnd::Paragraph);
        let content = self.parse_inlines(inner)?;
        Ok((Block::Paragraph { content, id }, pos - start))
    }

    fn parse_code_block<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
        lang: Option<String>,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let mut pos = start + 1;
        let mut code = String::new();
        while pos < events.len() {
            match &events[pos] {
                Event::End(TagEnd::CodeBlock) => {
                    pos += 1;
                    break;
                }
                Event::Text(t) => {
                    code.push_str(t);
                    pos += 1;
                }
                _ => {
                    pos += 1;
                }
            }
        }
        // Check for math block: lang is "math" or "$$"
        if lang.as_deref() == Some("math") || lang.as_deref() == Some("$$") {
            return Ok((Block::MathBlock { latex: code, id }, pos - start));
        }
        Ok((Block::CodeBlock { lang, code, id }, pos - start))
    }

    fn parse_block_quote<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let mut pos = start + 1;

        // Gather inner events until the matching End(BlockQuote)
        let mut depth = 1usize;
        let inner_start = pos;
        while pos < events.len() {
            match &events[pos] {
                Event::Start(Tag::BlockQuote(_)) => {
                    depth += 1;
                    pos += 1;
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    depth -= 1;
                    pos += 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    pos += 1;
                }
            }
        }

        let inner_events = &events[inner_start..pos - 1];
        let content = self.build(inner_events)?;
        Ok((Block::BlockQuote { content, id }, pos - start))
    }

    fn parse_list<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
        ordered: bool,
        list_start: Option<u64>,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let start_num = list_start.map(|n| n as u32);
        let mut pos = start + 1;
        let mut items: Vec<ListItem> = Vec::new();

        while pos < events.len() {
            match &events[pos] {
                Event::End(TagEnd::List(_)) => {
                    pos += 1;
                    break;
                }
                Event::Start(Tag::Item) => {
                    let (item, consumed) = self.parse_list_item(events, pos)?;
                    items.push(item);
                    pos += consumed;
                }
                _ => {
                    pos += 1;
                }
            }
        }

        Ok((
            Block::List {
                items,
                ordered,
                start: start_num,
                id,
            },
            pos - start,
        ))
    }

    fn parse_list_item<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
    ) -> MarkdownResult<(ListItem, usize)> {
        let mut pos = start + 1;
        let mut inline_content: Vec<Inline> = Vec::new();
        let mut block_content: Vec<Block> = Vec::new();
        let mut checked: Option<bool> = None;
        let mut sub_items: Vec<ListItem> = Vec::new();

        while pos < events.len() {
            match &events[pos] {
                Event::End(TagEnd::Item) => {
                    pos += 1;
                    break;
                }
                Event::TaskListMarker(checked_val) => {
                    checked = Some(*checked_val);
                    pos += 1;
                }
                Event::Start(Tag::Paragraph) => {
                    let mut p_pos = pos + 1;
                    let inner = collect_until_end(events, &mut p_pos, TagEnd::Paragraph);
                    let inlines = self.parse_inlines(inner)?;
                    if inline_content.is_empty() && block_content.is_empty() {
                        inline_content.extend(inlines);
                    } else {
                        block_content.push(Block::Paragraph {
                            content: inlines,
                            id: self.alloc_id(),
                        });
                    }
                    pos = p_pos;
                }
                Event::Start(Tag::List(start_num)) => {
                    let ordered = start_num.is_some();
                    let sn = *start_num;
                    let (nested_list, consumed) = self.parse_list(events, pos, ordered, sn)?;
                    // Flatten sub-items into sub_items
                    if let Block::List { items, .. } = nested_list {
                        sub_items.extend(items);
                    }
                    pos += consumed;
                }
                // Nested block-level content: parse via the block helpers and
                // push into the item's `blocks` so the nesting hierarchy is
                // preserved instead of being flattened into inline content.
                Event::Start(Tag::CodeBlock(kind)) => {
                    let lang = match kind {
                        pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                            let s = lang.to_string();
                            if s.is_empty() { None } else { Some(s) }
                        }
                        pulldown_cmark::CodeBlockKind::Indented => None,
                    };
                    let (cb, consumed) = self.parse_code_block(events, pos, lang)?;
                    block_content.push(cb);
                    pos += consumed;
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    let (bq, consumed) = self.parse_block_quote(events, pos)?;
                    block_content.push(bq);
                    pos += consumed;
                }
                Event::Start(Tag::Table(alignments)) => {
                    let aligns: Vec<Alignment> = alignments.iter().map(convert_alignment).collect();
                    let (table, consumed) = self.parse_table(events, pos, aligns)?;
                    block_content.push(table);
                    pos += consumed;
                }
                Event::Start(Tag::Heading { level, .. }) => {
                    let (heading, consumed) = self.parse_heading(events, pos, *level)?;
                    block_content.push(heading);
                    pos += consumed;
                }
                Event::Rule => {
                    block_content.push(Block::HorizontalRule {
                        id: self.alloc_id(),
                    });
                    pos += 1;
                }
                // Handle inline formatting events directly (for task list items without paragraphs)
                Event::Start(Tag::Strong) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Strong);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inline_content.push(Inline::Strong(inner_inlines));
                }
                Event::Start(Tag::Emphasis) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Emphasis);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inline_content.push(Inline::Emphasis(inner_inlines));
                }
                Event::Start(Tag::Strikethrough) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Strikethrough);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inline_content.push(Inline::Strikethrough(inner_inlines));
                }
                Event::Start(Tag::Link {
                    dest_url, title, ..
                }) => {
                    let url = dest_url.to_string();
                    let title_opt = if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    };
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Link);
                    let text = self.parse_inlines(inner)?;
                    inline_content.push(Inline::Link {
                        text,
                        url,
                        title: title_opt,
                    });
                }
                Event::Start(Tag::Image {
                    dest_url, title, ..
                }) => {
                    let url = dest_url.to_string();
                    let title_opt = if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    };
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Image);
                    let alt = inner
                        .iter()
                        .filter_map(|e| {
                            if let Event::Text(t) = e {
                                Some(t.as_ref())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    inline_content.push(Inline::Image {
                        alt,
                        url,
                        title: title_opt,
                    });
                }
                Event::Text(t) => {
                    inline_content.push(Inline::Text(t.to_string()));
                    pos += 1;
                }
                Event::Code(c) => {
                    inline_content.push(Inline::Code(c.to_string()));
                    pos += 1;
                }
                Event::SoftBreak => {
                    inline_content.push(Inline::Text(" ".into()));
                    pos += 1;
                }
                Event::HardBreak => {
                    inline_content.push(Inline::LineBreak);
                    pos += 1;
                }
                Event::FootnoteReference(label) => {
                    inline_content.push(Inline::FootnoteReference(label.to_string()));
                    pos += 1;
                }
                Event::Html(h) | Event::InlineHtml(h) => {
                    inline_content.push(Inline::HtmlInline(h.to_string()));
                    pos += 1;
                }
                _ => {
                    pos += 1;
                }
            }
        }

        let item = ListItem {
            content: inline_content,
            blocks: block_content,
            checked,
            sub_items,
        };
        Ok((item, pos - start))
    }

    fn parse_table<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
        alignment: Vec<Alignment>,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let mut pos = start + 1;
        let mut headers: Vec<TableCell> = Vec::new();
        let mut rows: Vec<Vec<TableCell>> = Vec::new();
        let mut in_header = false;
        let mut current_row: Vec<TableCell> = Vec::new();

        while pos < events.len() {
            match &events[pos] {
                Event::End(TagEnd::Table) => {
                    pos += 1;
                    break;
                }
                Event::Start(Tag::TableHead) => {
                    in_header = true;
                    pos += 1;
                }
                Event::End(TagEnd::TableHead) => {
                    headers = std::mem::take(&mut current_row);
                    in_header = false;
                    pos += 1;
                }
                Event::Start(Tag::TableRow) => {
                    current_row.clear();
                    pos += 1;
                }
                Event::End(TagEnd::TableRow) => {
                    if !in_header {
                        rows.push(std::mem::take(&mut current_row));
                    }
                    pos += 1;
                }
                Event::Start(Tag::TableCell) => {
                    let mut cell_pos = pos + 1;
                    let inner = collect_until_end(events, &mut cell_pos, TagEnd::TableCell);
                    let content = self.parse_inlines(inner)?;
                    current_row.push(TableCell { content });
                    pos = cell_pos;
                }
                _ => {
                    pos += 1;
                }
            }
        }

        Ok((
            Block::Table {
                headers,
                rows,
                alignment,
                id,
            },
            pos - start,
        ))
    }

    fn parse_footnote_definition<'a>(
        &mut self,
        events: &[Event<'a>],
        start: usize,
        label: String,
    ) -> MarkdownResult<(Block, usize)> {
        let id = self.alloc_id();
        let mut pos = start + 1;
        let mut depth = 1usize;
        let inner_start = pos;

        // Gather inner events until the matching End(FootnoteDefinition)
        while pos < events.len() {
            match &events[pos] {
                Event::Start(Tag::FootnoteDefinition(_)) => {
                    depth += 1;
                    pos += 1;
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    depth -= 1;
                    pos += 1;
                    if depth == 0 {
                        break;
                    }
                }
                _ => {
                    pos += 1;
                }
            }
        }

        let inner_events = &events[inner_start..pos - 1];
        let content = self.build(inner_events)?;
        Ok((
            Block::FootnoteDefinition { label, content, id },
            pos - start,
        ))
    }

    // -----------------------------------------------------------------------
    // Inline parser
    // -----------------------------------------------------------------------

    fn parse_inlines<'a>(&mut self, events: &[Event<'a>]) -> MarkdownResult<Vec<Inline>> {
        let mut inlines = Vec::new();
        let mut pos = 0;

        while pos < events.len() {
            match &events[pos] {
                Event::Text(t) => {
                    let s = t.to_string();
                    // With ENABLE_MATH, pulldown splits rejected math
                    // candidates (e.g. whitespace-edged `$ x $`) into
                    // separate `$`/content/`$` text events. Trial-merge a
                    // `$`-led run and consume it only if the whole run hits
                    // the `$…$` heuristic; otherwise keep per-event handling
                    // (merging unconditionally would break `\^` escape runs).
                    if s.starts_with('$') {
                        let mut merged = s.clone();
                        let mut next = pos + 1;
                        while let Some(Event::Text(more)) = events.get(next) {
                            merged.push_str(more);
                            next += 1;
                        }
                        if next > pos + 1 {
                            if let Some(math) = extract_inline_math(&merged) {
                                inlines.push(Inline::InlineMath(math.to_string()));
                                pos = next;
                                continue;
                            }
                        }
                    }
                    // Detect inline math: content wrapped in `$…$`
                    if let Some(math) = extract_inline_math(&s) {
                        inlines.push(Inline::InlineMath(math.to_string()));
                    } else {
                        // Apply extended inline parsing (superscript, subscript, highlight, emoji)
                        let extended = crate::extended_inline::parse_extended_inlines(&s);
                        inlines.extend(extended);
                    }
                    pos += 1;
                }
                Event::Code(c) => {
                    inlines.push(Inline::Code(c.to_string()));
                    pos += 1;
                }
                Event::SoftBreak => {
                    inlines.push(Inline::Text(" ".into()));
                    pos += 1;
                }
                Event::HardBreak => {
                    inlines.push(Inline::LineBreak);
                    pos += 1;
                }
                Event::Html(h) => {
                    // Create HtmlInline for inline HTML
                    inlines.push(Inline::HtmlInline(h.to_string()));
                    pos += 1;
                }
                Event::InlineHtml(h) => {
                    // Create HtmlInline for inline HTML (pulldown-cmark 0.11+)
                    inlines.push(Inline::HtmlInline(h.to_string()));
                    pos += 1;
                }
                Event::InlineMath(m) => {
                    inlines.push(Inline::InlineMath(m.to_string()));
                    pos += 1;
                }
                Event::DisplayMath(m) => {
                    // display math inside inline context — treat as block math text
                    inlines.push(Inline::InlineMath(m.to_string()));
                    pos += 1;
                }
                Event::Start(Tag::Strong) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Strong);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inlines.push(Inline::Strong(inner_inlines));
                }
                Event::Start(Tag::Emphasis) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Emphasis);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inlines.push(Inline::Emphasis(inner_inlines));
                }
                Event::Start(Tag::Strikethrough) => {
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Strikethrough);
                    let inner_inlines = self.parse_inlines(inner)?;
                    inlines.push(Inline::Strikethrough(inner_inlines));
                }
                Event::Start(Tag::Link {
                    dest_url, title, ..
                }) => {
                    let url = dest_url.to_string();
                    let title_opt = if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    };
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Link);
                    let text = self.parse_inlines(inner)?;
                    inlines.push(Inline::Link {
                        text,
                        url,
                        title: title_opt,
                    });
                }
                Event::Start(Tag::Image {
                    dest_url, title, ..
                }) => {
                    let url = dest_url.to_string();
                    let title_opt = if title.is_empty() {
                        None
                    } else {
                        Some(title.to_string())
                    };
                    pos += 1;
                    let inner = collect_until_inline_end(events, &mut pos, TagEnd::Image);
                    // Collect alt text from inner text events
                    let alt = inner
                        .iter()
                        .filter_map(|e| {
                            if let Event::Text(t) = e {
                                Some(t.as_ref())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("");
                    inlines.push(Inline::Image {
                        alt,
                        url,
                        title: title_opt,
                    });
                }
                Event::FootnoteReference(label) => {
                    inlines.push(Inline::FootnoteReference(label.to_string()));
                    pos += 1;
                }
                _ => {
                    pos += 1;
                }
            }
        }
        Ok(inlines)
    }
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

/// Convert a `HeadingLevel` to a `u8` (1–6).
fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Map a pulldown-cmark `Alignment` to our `Alignment`.
fn convert_alignment(a: &PdAlignment) -> Alignment {
    match a {
        PdAlignment::None => Alignment::None,
        PdAlignment::Left => Alignment::Left,
        PdAlignment::Center => Alignment::Center,
        PdAlignment::Right => Alignment::Right,
    }
}

/// Collect events between the current position and a matching `End(tag)`,
/// advancing `pos` past the end event. Returns the inner slice.
fn collect_until_end<'a, 'b>(
    events: &'b [Event<'a>],
    pos: &mut usize,
    end_tag: TagEnd,
) -> &'b [Event<'a>] {
    let start = *pos;
    while *pos < events.len() {
        if let Event::End(t) = &events[*pos] {
            if *t == end_tag {
                let inner = &events[start..*pos];
                *pos += 1; // consume the End event
                return inner;
            }
        }
        *pos += 1;
    }
    &events[start..*pos]
}

/// Same as `collect_until_end` but for inline-level tag ends.
fn collect_until_inline_end<'a, 'b>(
    events: &'b [Event<'a>],
    pos: &mut usize,
    end_tag: TagEnd,
) -> &'b [Event<'a>] {
    collect_until_end(events, pos, end_tag)
}

/// If the text is a simple `$...$` inline math expression (single-dollar delimited),
/// return the interior. This handles cases where pulldown-cmark emits the delimiters
/// as part of a Text event rather than as dedicated math events.
fn extract_inline_math(text: &str) -> Option<&str> {
    let trimmed = text.trim();
    if trimmed.starts_with('$') && trimmed.ends_with('$') && trimmed.len() > 2 {
        let inner = &trimmed[1..trimmed.len() - 1];
        // Avoid matching `$$` (block math)
        if !inner.starts_with('$') {
            return Some(inner);
        }
    }
    None
}

/// Detect URLs in text and convert them to Link inline nodes.
/// Supports http://, https://, and www. prefixed URLs.
/// Returns a vector of Inline elements (Text and Link nodes).
fn detect_and_convert_urls(text: &str) -> Vec<Inline> {
    let mut result = Vec::new();
    let mut remaining = text;

    while !remaining.is_empty() {
        // Find the earliest URL pattern
        let http_pos = remaining.find("http://");
        let https_pos = remaining.find("https://");
        let www_pos = remaining.find("www.");

        // Find the earliest match
        let earliest = [http_pos, https_pos, www_pos]
            .iter()
            .filter_map(|&pos| pos)
            .min();

        match earliest {
            None => {
                // No URLs found, push remaining text
                if !remaining.is_empty() {
                    result.push(Inline::Text(remaining.to_string()));
                }
                break;
            }
            Some(start_pos) => {
                // Push text before URL
                if start_pos > 0 {
                    result.push(Inline::Text(remaining[..start_pos].to_string()));
                }

                // Find the end of the URL
                let url_start = &remaining[start_pos..];
                let end_pos = find_url_end(url_start);
                let url = &url_start[..end_pos];

                // Convert www. URLs to https://www.
                let full_url = if url.starts_with("www.") {
                    format!("https://{}", url)
                } else {
                    url.to_string()
                };

                // Create Link node
                result.push(Inline::Link {
                    text: vec![Inline::Text(url.to_string())],
                    url: full_url,
                    title: None,
                });

                // Move to next part
                remaining = &remaining[start_pos + end_pos..];
            }
        }
    }

    result
}

/// Find the end position of a URL in text.
/// URLs end at whitespace, punctuation (except certain characters), or end of string.
fn find_url_end(url_start: &str) -> usize {
    let chars: Vec<char> = url_start.chars().collect();
    let mut end = 0;

    for (i, &ch) in chars.iter().enumerate() {
        // URL ends at whitespace
        if ch.is_whitespace() {
            break;
        }

        // Handle period: if it's at the end or followed by whitespace, exclude it
        if ch == '.' {
            let next_is_whitespace_or_end = i + 1 >= chars.len() || chars[i + 1].is_whitespace();
            if next_is_whitespace_or_end {
                // Don't include the trailing period
                break;
            }
        }

        // Handle other sentence-ending punctuation
        if i > 0 && matches!(ch, '!' | ')' | '>' | '"' | '\'' | ',' | ';' | '?') {
            let next_is_whitespace_or_end = i + 1 >= chars.len() || chars[i + 1].is_whitespace();
            if next_is_whitespace_or_end {
                // Don't include the trailing punctuation
                break;
            }
        }

        // Allow common URL characters
        if ch.is_alphanumeric()
            || matches!(
                ch,
                '-' | '_'
                    | '.'
                    | '/'
                    | '?'
                    | '&'
                    | '='
                    | '%'
                    | '#'
                    | '+'
                    | ':'
                    | '@'
                    | '~'
                    | '('
                    | ')'
                    | '!'
            )
        {
            end = i + 1;
        } else {
            break;
        }
    }

    // Ensure we captured at least something
    if end == 0 {
        end = 1;
    }

    end
}

// ---------------------------------------------------------------------------
// URL Auto-Detection Post-Processing
// ---------------------------------------------------------------------------

/// Post-process a list of blocks to detect and convert bare URLs to Link nodes.
pub(crate) fn post_process_url_detection_in_blocks(blocks: &mut [Block]) {
    for block in blocks {
        post_process_url_in_block(block);
    }
}

fn post_process_url_in_block(block: &mut Block) {
    match block {
        Block::Paragraph { content, .. } => {
            *content = process_inlines_for_urls(std::mem::take(content));
        }
        Block::Heading { content, .. } => {
            *content = process_inlines_for_urls(std::mem::take(content));
        }
        Block::BlockQuote { content, .. } => {
            for inner_block in content {
                post_process_url_in_block(inner_block);
            }
        }
        Block::List { items, .. } => {
            for item in items {
                item.content = process_inlines_for_urls(std::mem::take(&mut item.content));
                for inner_block in &mut item.blocks {
                    post_process_url_in_block(inner_block);
                }
                // Recursively process sub-items
                for sub_item in &mut item.sub_items {
                    sub_item.content =
                        process_inlines_for_urls(std::mem::take(&mut sub_item.content));
                }
            }
        }
        Block::Table { headers, rows, .. } => {
            for cell in headers {
                cell.content = process_inlines_for_urls(std::mem::take(&mut cell.content));
            }
            for row in rows {
                for cell in row {
                    cell.content = process_inlines_for_urls(std::mem::take(&mut cell.content));
                }
            }
        }
        Block::FootnoteDefinition { content, .. } => {
            for inner_block in content {
                post_process_url_in_block(inner_block);
            }
        }
        _ => {}
    }
}

fn process_inlines_for_urls(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut result = Vec::new();
    let mut pending_text = String::new();

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                // Accumulate consecutive text nodes
                pending_text.push_str(&text);
            }
            other => {
                // Process any pending text before handling the non-text inline
                if !pending_text.is_empty() {
                    result.extend(detect_and_convert_urls(&pending_text));
                    pending_text.clear();
                }

                // Process the non-text inline recursively
                match other {
                    Inline::Strong(inner) => {
                        result.push(Inline::Strong(process_inlines_for_urls(inner)));
                    }
                    Inline::Emphasis(inner) => {
                        result.push(Inline::Emphasis(process_inlines_for_urls(inner)));
                    }
                    Inline::Strikethrough(inner) => {
                        result.push(Inline::Strikethrough(process_inlines_for_urls(inner)));
                    }
                    Inline::Superscript(inner) => {
                        result.push(Inline::Superscript(process_inlines_for_urls(inner)));
                    }
                    Inline::Subscript(inner) => {
                        result.push(Inline::Subscript(process_inlines_for_urls(inner)));
                    }
                    Inline::Highlight(inner) => {
                        result.push(Inline::Highlight(process_inlines_for_urls(inner)));
                    }
                    // Don't process URLs inside existing links
                    non_text => result.push(non_text),
                }
            }
        }
    }

    // Process any remaining text
    if !pending_text.is_empty() {
        result.extend(detect_and_convert_urls(&pending_text));
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Block, Inline};

    fn default_parser() -> Parser {
        Parser::default()
    }

    // --- Math (ENABLE_MATH via the pulldown-cmark 0.13 unification) ---

    #[test]
    fn inline_math_embedded_in_prose_parses_to_math_node() {
        let doc = default_parser().parse("prose $a^2+b^2$ end").unwrap();
        let Block::Paragraph { content, .. } = &doc.blocks[0] else {
            panic!("expected paragraph");
        };
        assert!(
            content
                .iter()
                .any(|inline| matches!(inline, Inline::InlineMath(m) if m == "a^2+b^2"))
        );
        assert!(
            content
                .iter()
                .any(|inline| matches!(inline, Inline::Text(t) if t.contains("prose")))
        );
    }

    #[test]
    fn whitespace_edged_dollar_run_still_hits_math_heuristic() {
        // pulldown rejects `$ x $` as math and splits it into `$`/` x `/`$`
        // text events; the coalescing heuristic must still see one run.
        let doc = default_parser().parse("$ x $").unwrap();
        let Block::Paragraph { content, .. } = &doc.blocks[0] else {
            panic!("expected paragraph");
        };
        assert!(
            content
                .iter()
                .any(|inline| matches!(inline, Inline::InlineMath(m) if m.trim() == "x"))
        );
    }

    // --- YAML front matter ---

    #[test]
    fn extract_front_matter_present() {
        let md = "---\ntitle: Hello\n---\n# Body\n";
        let (yaml, body) = extract_front_matter(md);
        // The yaml slice is everything between the opening and closing `---` lines.
        // It may or may not include the trailing newline depending on the find offset.
        let yaml_str = yaml.expect("should have yaml");
        assert!(yaml_str.contains("title: Hello"));
        assert_eq!(body, "# Body\n");
    }

    #[test]
    fn extract_front_matter_absent() {
        let md = "# No front matter\n";
        let (yaml, body) = extract_front_matter(md);
        assert!(yaml.is_none());
        assert_eq!(body, md);
    }

    #[test]
    fn parse_yaml_front_matter() {
        let md = "---\ntitle: My Title\nauthor: Alice\ntags:\n  - rust\n  - markdown\n---\nHello\n";
        let parser = default_parser();
        let doc = parser.parse(md).unwrap();
        let fm = doc.metadata.unwrap();
        assert_eq!(fm.title.as_deref(), Some("My Title"));
        assert_eq!(fm.author.as_deref(), Some("Alice"));
        assert_eq!(fm.tags, vec!["rust", "markdown"]);
    }

    // --- Headings ---

    #[test]
    fn parse_heading() {
        let doc = default_parser().parse("# Hello World\n").unwrap();
        assert_eq!(doc.blocks.len(), 1);
        if let Block::Heading { level, content, .. } = &doc.blocks[0] {
            assert_eq!(*level, 1);
            assert_eq!(content, &vec![Inline::Text("Hello World".into())]);
        } else {
            panic!("Expected Heading");
        }
    }

    #[test]
    fn parse_headings_all_levels() {
        let md = "# H1\n## H2\n### H3\n#### H4\n##### H5\n###### H6\n";
        let doc = default_parser().parse(md).unwrap();
        for (i, block) in doc.blocks.iter().enumerate() {
            if let Block::Heading { level, .. } = block {
                assert_eq!(*level, (i + 1) as u8);
            } else {
                panic!("Expected Heading at index {i}");
            }
        }
    }

    // --- Paragraph and inline elements ---

    #[test]
    fn parse_paragraph() {
        let doc = default_parser().parse("Hello world\n").unwrap();
        assert!(matches!(&doc.blocks[0], Block::Paragraph { .. }));
    }

    #[test]
    fn parse_strong_and_emphasis() {
        let doc = default_parser().parse("**bold** and *italic*\n").unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(content.iter().any(|i| matches!(i, Inline::Strong(_))));
            assert!(content.iter().any(|i| matches!(i, Inline::Emphasis(_))));
        }
    }

    #[test]
    fn parse_strikethrough() {
        let doc = default_parser().parse("~~struck~~\n").unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(
                content
                    .iter()
                    .any(|i| matches!(i, Inline::Strikethrough(_)))
            );
        }
    }

    #[test]
    fn parse_inline_code() {
        let doc = default_parser().parse("`code`\n").unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(content.iter().any(|i| matches!(i, Inline::Code(_))));
        }
    }

    // --- Code blocks ---

    #[test]
    fn parse_code_block_with_lang() {
        let doc = default_parser()
            .parse("```rust\nfn main() {}\n```\n")
            .unwrap();
        if let Block::CodeBlock { lang, code, .. } = &doc.blocks[0] {
            assert_eq!(lang.as_deref(), Some("rust"));
            assert!(code.contains("fn main()"));
        } else {
            panic!("Expected CodeBlock");
        }
    }

    #[test]
    fn parse_code_block_preserves_whitespace() {
        let code = "  indented\n\ttabbed\n";
        let md = format!("```\n{code}```\n");
        let doc = default_parser().parse(&md).unwrap();
        if let Block::CodeBlock { code: c, .. } = &doc.blocks[0] {
            assert_eq!(c, code);
        } else {
            panic!("Expected CodeBlock");
        }
    }

    // --- Lists ---

    #[test]
    fn parse_unordered_list() {
        let doc = default_parser().parse("- item 1\n- item 2\n").unwrap();
        if let Block::List { items, ordered, .. } = &doc.blocks[0] {
            assert!(!ordered);
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected List");
        }
    }

    #[test]
    fn parse_ordered_list() {
        let doc = default_parser().parse("1. first\n2. second\n").unwrap();
        if let Block::List {
            items,
            ordered,
            start,
            ..
        } = &doc.blocks[0]
        {
            assert!(ordered);
            assert_eq!(*start, Some(1));
            assert_eq!(items.len(), 2);
        } else {
            panic!("Expected ordered List");
        }
    }

    #[test]
    fn parse_task_list() {
        let doc = default_parser().parse("- [x] done\n- [ ] todo\n").unwrap();
        if let Block::List { items, .. } = &doc.blocks[0] {
            assert_eq!(items[0].checked, Some(true));
            assert_eq!(items[1].checked, Some(false));
        } else {
            panic!("Expected List");
        }
    }

    // --- Block quote ---

    #[test]
    fn parse_block_quote() {
        let doc = default_parser().parse("> quoted text\n").unwrap();
        assert!(matches!(&doc.blocks[0], Block::BlockQuote { .. }));
    }

    // --- Horizontal rule ---

    #[test]
    fn parse_horizontal_rule() {
        let doc = default_parser().parse("---\n").unwrap();
        assert!(matches!(&doc.blocks[0], Block::HorizontalRule { .. }));
    }

    // --- Links and images ---

    #[test]
    fn parse_link() {
        let doc = default_parser()
            .parse("[text](https://example.com)\n")
            .unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(content.iter().any(|i| matches!(i, Inline::Link { .. })));
        }
    }

    #[test]
    fn parse_image() {
        let doc = default_parser().parse("![alt](img.png)\n").unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(content.iter().any(|i| matches!(i, Inline::Image { .. })));
        }
    }

    // --- GFM Tables ---

    #[test]
    fn parse_gfm_table() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { headers, rows, .. } = &doc.blocks[0] {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 1);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_extracts_headers() {
        let md = "| Name | Age | City |\n|---|---|---|\n| Alice | 30 | NYC |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { headers, .. } = &doc.blocks[0] {
            assert_eq!(headers.len(), 3);
            assert_eq!(headers[0].content, vec![Inline::Text("Name".into())]);
            assert_eq!(headers[1].content, vec![Inline::Text("Age".into())]);
            assert_eq!(headers[2].content, vec![Inline::Text("City".into())]);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_extracts_rows_and_cells() {
        let md = "| A | B |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { headers, rows, .. } = &doc.blocks[0] {
            assert_eq!(headers.len(), 2);
            assert_eq!(rows.len(), 2);
            assert_eq!(rows[0].len(), 2);
            assert_eq!(rows[1].len(), 2);
            assert_eq!(rows[0][0].content, vec![Inline::Text("1".into())]);
            assert_eq!(rows[0][1].content, vec![Inline::Text("2".into())]);
            assert_eq!(rows[1][0].content, vec![Inline::Text("3".into())]);
            assert_eq!(rows[1][1].content, vec![Inline::Text("4".into())]);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_alignment_left() {
        let md = "| A | B |\n|:---|:---|\n| 1 | 2 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { alignment, .. } = &doc.blocks[0] {
            assert_eq!(alignment.len(), 2);
            assert_eq!(alignment[0], Alignment::Left);
            assert_eq!(alignment[1], Alignment::Left);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_alignment_center() {
        let md = "| A | B |\n|:---:|:---:|\n| 1 | 2 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { alignment, .. } = &doc.blocks[0] {
            assert_eq!(alignment.len(), 2);
            assert_eq!(alignment[0], Alignment::Center);
            assert_eq!(alignment[1], Alignment::Center);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_alignment_right() {
        let md = "| A | B |\n|---:|---:|\n| 1 | 2 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { alignment, .. } = &doc.blocks[0] {
            assert_eq!(alignment.len(), 2);
            assert_eq!(alignment[0], Alignment::Right);
            assert_eq!(alignment[1], Alignment::Right);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_alignment_mixed() {
        let md = "| A | B | C |\n|:---|:---:|---:|\n| 1 | 2 | 3 |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { alignment, .. } = &doc.blocks[0] {
            assert_eq!(alignment.len(), 3);
            assert_eq!(alignment[0], Alignment::Left);
            assert_eq!(alignment[1], Alignment::Center);
            assert_eq!(alignment[2], Alignment::Right);
        } else {
            panic!("Expected Table");
        }
    }

    #[test]
    fn parse_table_with_inline_formatting() {
        let md = "| **Bold** | *Italic* |\n|---|---|\n| `code` | text |\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Table { headers, rows, .. } = &doc.blocks[0] {
            // Header should contain Strong inline
            assert!(matches!(&headers[0].content[0], Inline::Strong(_)));
            // Header should contain Emphasis inline
            assert!(matches!(&headers[1].content[0], Inline::Emphasis(_)));
            // Row should contain Code inline
            assert_eq!(rows[0][0].content, vec![Inline::Code("code".into())]);
        } else {
            panic!("Expected Table");
        }
    }

    // --- Footnotes ---

    #[test]
    fn parse_footnote_definition() {
        let md = "[^1]: This is a footnote.\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::FootnoteDefinition { label, content, .. } = &doc.blocks[0] {
            assert_eq!(label, "1");
            assert_eq!(content.len(), 1);
            if let Block::Paragraph {
                content: para_content,
                ..
            } = &content[0]
            {
                assert_eq!(
                    para_content,
                    &vec![Inline::Text("This is a footnote.".into())]
                );
            } else {
                panic!("Expected paragraph in footnote");
            }
        } else {
            panic!("Expected FootnoteDefinition");
        }
    }

    #[test]
    fn parse_footnote_reference() {
        // A footnote reference (`[^id]`) is only recognized as such when a
        // matching definition exists; otherwise the underlying CommonMark
        // parser treats `[^id]` as literal text. Include the definition so the
        // reference is parsed into an `Inline::FootnoteReference`.
        let md = "This is text[^1].\n\n[^1]: The note.\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            assert!(
                content
                    .iter()
                    .any(|i| matches!(i, Inline::FootnoteReference(_)))
            );
            // Verify the label
            for inline in content {
                if let Inline::FootnoteReference(label) = inline {
                    assert_eq!(label, "1");
                }
            }
        } else {
            panic!("Expected Paragraph");
        }
    }

    #[test]
    fn parse_footnote_reference_and_definition() {
        let md = "Text with footnote[^note].\n\n[^note]: Footnote content.\n";
        let doc = default_parser().parse(md).unwrap();

        // Check the paragraph contains the reference
        if let Block::Paragraph { content, .. } = &doc.blocks[0] {
            let has_ref = content
                .iter()
                .any(|i| matches!(i, Inline::FootnoteReference(label) if label == "note"));
            assert!(has_ref, "Should contain footnote reference");
        } else {
            panic!("Expected Paragraph as first block");
        }

        // Check the footnote definition exists
        let has_def = doc
            .blocks
            .iter()
            .any(|b| matches!(b, Block::FootnoteDefinition { label, .. } if label == "note"));
        assert!(has_def, "Should contain footnote definition");

        // Check the footnote map
        assert!(doc.footnote_map.contains_key("note"));
    }

    #[test]
    fn parse_multiple_footnotes() {
        let md = "First[^1] and second[^2].\n\n[^1]: First note.\n[^2]: Second note.\n";
        let doc = default_parser().parse(md).unwrap();

        // Should have 2 footnote definitions
        let footnote_count = doc
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::FootnoteDefinition { .. }))
            .count();
        assert_eq!(footnote_count, 2);

        // Footnote map should have both
        assert_eq!(doc.footnote_map.len(), 2);
        assert!(doc.footnote_map.contains_key("1"));
        assert!(doc.footnote_map.contains_key("2"));
    }

    #[test]
    fn parse_footnote_with_complex_content() {
        let md = "[^complex]: This footnote has **bold** and *italic* text.\n";
        let doc = default_parser().parse(md).unwrap();
        if let Block::FootnoteDefinition { label, content, .. } = &doc.blocks[0] {
            assert_eq!(label, "complex");
            if let Block::Paragraph {
                content: para_content,
                ..
            } = &content[0]
            {
                assert!(para_content.iter().any(|i| matches!(i, Inline::Strong(_))));
                assert!(
                    para_content
                        .iter()
                        .any(|i| matches!(i, Inline::Emphasis(_)))
                );
            }
        } else {
            panic!("Expected FootnoteDefinition");
        }
    }
}
