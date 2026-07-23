use std::{
    collections::HashMap,
    fs, io,
    ops::Range,
    path::{Path, PathBuf},
};

use pulldown_cmark::{Alignment, CodeBlockKind, CowStr, Event, Parser, Tag, TagEnd, html};
use regex::RegexBuilder;

mod diagram;
mod editing;
mod escape;
mod export;
mod frontmatter;
mod highlight;
pub mod i18n;
mod math;
pub mod model;
mod parse;
mod paths;
mod render;
mod source_mapped;
mod storage;
mod table;
mod text_util;
mod visual;

/// Markdown shown in the first in-memory document when Markion starts.
///
/// This is document content rather than localized UI chrome, so it stays in
/// English regardless of the selected application language.
pub const DEFAULT_WELCOME_MARKDOWN: &str = r#"# Welcome to Markion

This starter document is a quick tour of Markdown in Markion. Edit the source on the left and see the preview update on the right.

## Heading hierarchy

### Level three heading

#### Level four heading

##### Level five heading

###### Level six heading

## Inline formatting

Write with *italic*, **bold**, ***bold italic***, ~~strikethrough~~, `inline code`, ==highlighted text==, H~2~O, and x^2^. Visit the [Markion project page](https://github.com/willmove/markion), or use the reference link below.[^links]

![Local image placeholder](markion-example.png "Replace with your image")

## Quotes and lists

> Markdown keeps ideas easy to read in plain text and in preview.

---

- Unordered list item
  - Nested list item
    - A deeper detail
1. Ordered list item
   1. Nested ordered item
2. Another ordered item

- [x] Draft in Markdown
- [x] Review in preview
- [ ] Export when ready

## Table

| Syntax | Example | Purpose |
| :--- | :---: | ---: |
| `**bold**` | **bold** | Emphasis |
| `[text](url)` | [link](https://github.com/willmove/markion) | Navigation |
| `- [ ] task` | - [ ] | Checklist |

## Code and math

Use `cargo run` to start Markion locally.

```rust
fn greeting(name: &str) -> String {
    format!("Welcome, {name}!")
}
```

Inline math: $E = mc^2$.

$$
\sum_{n=1}^{10} n = 55
$$

## Notes

Reference-style links work too: [Markion repository][markion-repo].

[^links]: Links can point to project pages, files, and useful references.

[markion-repo]: https://github.com/willmove/markion
"#;

pub use model::{
    AppPreferences, AutoSavePreferences, AutosaveOutcome, DEFAULT_EDITOR_FONT_SIZE,
    DEFAULT_HEADING_MENU_MAX_LEVEL, DEFAULT_PARAGRAPH_SPACING, DEFAULT_RENDERED_FONT_SIZE,
    DocumentStats, EXTENDED_HEADING_MENU_MAX_LEVEL, ExportBackend, ExportFormat, ExportPreferences,
    Footnote, FrontMatterError, Heading, HighlightKind, HighlightedSpan, InlineSpan, InlineStyle,
    MAX_EDITOR_FONT_SIZE, MAX_PARAGRAPH_SPACING, MAX_RENDERED_FONT_SIZE, MIN_EDITOR_FONT_SIZE,
    MIN_PARAGRAPH_SPACING, MIN_RENDERED_FONT_SIZE, MarkdownFormat, MathDelimiter, MathExpression,
    MathLayoutStyle, MathSource, PreviewBlock, RecoveryDocument, RenderedMath, ReplaceResult,
    RichText, SearchError, SearchMatch, SearchMatchRange, SearchOptions, SidebarTab,
    TableAlignment, TableEdit, TableEditResult, ThemeColors, ThemeDefinition, ViewMode,
    VisualBlock, VisualBlockEdit, VisualBlockEditor, VisualBlockId, VisualBlockKind,
    VisualBlockPrefix, VisualBlockPrefixKind, VisualBoundaryCandidates, VisualCaretAffinity,
    VisualEditorField, VisualEditorFieldKind, VisualInlineRun, VisualNavigationTarget,
    VisualProjection, VisualProjectionSegment, VisualProjectionSpan, VisualRevealGroup,
    VisualRevealKind, VisualSourceIslandKind, VisualStructuralEdit, VisualTableCell,
    YamlFrontMatter, builtin_theme_definitions, normalize_editor_font_size,
    normalize_heading_menu_max_level, normalize_paragraph_spacing, normalize_rendered_font_size,
};
pub use visual::{build_visual_projection, build_visual_projection_with_marked_range};

pub use diagram::{builtin_diagram_registry, diagram_backend_id};
pub use highlight::{highlight_code, supported_highlight_languages, warm_highlighter};
pub use i18n::{
    Language, Msg, ShortcutAction, ShortcutCatalog, ShortcutCategory, ShortcutPlatform,
    ShortcutSection, shortcut_catalog, sidebar_tab_label, t, tf,
};
pub use math::{render_math, validate_latex};
pub use parse::{HtmlPreviewPart, html_preview_parts, html_preview_plain_text};

pub use storage::{
    FileTree, FileTreeEntry, FileTreeEntryKind, MARKDOWN_EXTENSIONS, delete_recovery_file,
    init_logging, is_markdown_path, list_recovery_files, list_theme_definitions,
    load_app_preferences, load_recovery_file, load_theme_definition, parse_app_preferences,
    parse_legacy_app_preferences, parse_theme_definition, render_app_preferences,
    render_theme_definition, save_app_preferences, save_theme_definition,
};

use table::{
    TableDraft, format_markdown_table, formatted_table_cell_range, parse_markdown_table,
    table_cell_source_ranges, table_position_at, table_range_at as table_range_at_fn,
    table_ranges as table_ranges_fn,
};

use parse::{
    ImageDraft, InlineStateDraft, ListItemDraft, ListLevelDraft, append_span, clean_preview_text,
    finish_rich_text, flush_list_item, heading_level_to_u8, markdown_options, push_nonempty_block,
    push_preview_math, push_preview_rich, render_extended_html_text_nodes, slugify,
};

use diagram::collect_html_diagrams;
use render::{
    DEFAULT_CSS, annotate_math_html, collect_html_math, escape_latex, escape_latex_path,
    latex_listing_language, push_latex_list_item, render_latex_rich_text, render_latex_table,
};

use export::{write_docx, write_image_snapshot, write_pdf};

use frontmatter::{parse_front_matter, split_front_matter};

use editing::{
    adjust_offset_for_line_insert, adjust_offset_for_line_marker_removal, heading_level_at,
    heading_marker_len_at, is_empty_list_marker, line_outdent_len, markdown_continuation,
    paragraph_range_at, selected_line_starts,
};

use storage::recovery::recovery_file_path;

#[derive(Debug)]
pub struct MarkdownDocument {
    text: String,
    path: Option<PathBuf>,
    dirty: bool,
    // --- Derived-state cache (lazily computed, invalidated on text change) ---
    // Parsing markdown is the dominant per-frame cost during typing: a single
    // render used to trigger up to five full pulldown-cmark passes plus a
    // table-range scan. We memoize the three heaviest derived values behind a
    // monotonically increasing `text_version` so each is parsed at most once
    // between edits, no matter how many times the render path asks for it.
    // The caches use interior mutability so they can be populated from the
    // `&self` accessors used throughout the render path.
    text_version: u64,
    cached_preview_blocks: std::cell::RefCell<Option<Cached<std::sync::Arc<Vec<PreviewBlock>>>>>,
    cached_visual_blocks: std::cell::RefCell<Option<Cached<std::sync::Arc<Vec<VisualBlock>>>>>,
    cached_outline: std::cell::RefCell<Option<Cached<Vec<Heading>>>>,
    cached_stats: std::cell::RefCell<Option<Cached<DocumentStats>>>,
    cached_line_count: std::cell::Cell<Option<(u64, usize)>>,
    source_mapped_cache: std::cell::RefCell<Option<source_mapped::SourceMappedCache>>,
    pending_source_edits: std::cell::RefCell<source_mapped::PendingSourceEdits>,
}

/// Cloning a document (undo/redo snapshots take one per edit) must stay cheap:
/// only the text and metadata are copied, never the derived caches. The clone
/// lazily recomputes derived state if it is ever rendered.
impl Clone for MarkdownDocument {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
            path: self.path.clone(),
            dirty: self.dirty,
            text_version: self.text_version,
            cached_preview_blocks: std::cell::RefCell::new(None),
            cached_visual_blocks: std::cell::RefCell::new(None),
            cached_outline: std::cell::RefCell::new(None),
            cached_stats: std::cell::RefCell::new(None),
            cached_line_count: std::cell::Cell::new(None),
            source_mapped_cache: std::cell::RefCell::new(None),
            pending_source_edits: std::cell::RefCell::new(source_mapped::PendingSourceEdits::Full),
        }
    }
}

/// A value paired with the `text_version` it was computed for. A request with
/// a matching version reuses the stored value; any text mutation bumps the
/// version and discards stale caches.
#[derive(Debug, Clone)]
struct Cached<T> {
    version: u64,
    value: T,
}

impl Default for MarkdownDocument {
    fn default() -> Self {
        Self::new()
    }
}

impl MarkdownDocument {
    /// Monotonic counter shared across every `MarkdownDocument` instance so
    /// that freshly created/opened documents never reuse an older document's
    /// `text_version`. The editor caches derived values (wrapped-text layout,
    /// shared text handle) keyed on `version()`; if a brand-new document
    /// restarted at `0`, it could collide with a stale cache entry left by the
    /// previous document and render a blank editor even though the preview
    /// (which reads `text()` directly) showed the new content.
    fn next_text_version() -> u64 {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT: AtomicU64 = AtomicU64::new(1);
        NEXT.fetch_add(1, Ordering::Relaxed)
    }

    fn with_state(text: String, path: Option<PathBuf>, dirty: bool) -> Self {
        Self {
            text,
            path,
            dirty,
            text_version: Self::next_text_version(),
            cached_preview_blocks: std::cell::RefCell::new(None),
            cached_visual_blocks: std::cell::RefCell::new(None),
            cached_outline: std::cell::RefCell::new(None),
            cached_stats: std::cell::RefCell::new(None),
            cached_line_count: std::cell::Cell::new(None),
            source_mapped_cache: std::cell::RefCell::new(None),
            pending_source_edits: std::cell::RefCell::new(source_mapped::PendingSourceEdits::Full),
        }
    }

    pub fn new() -> Self {
        Self::with_state(String::new(), None, false)
    }

    pub fn from_text(text: impl Into<String>) -> Self {
        Self::with_state(text.into(), None, false)
    }

    pub fn recovered(text: impl Into<String>, path: Option<PathBuf>) -> Self {
        Self::with_state(text.into(), path, true)
    }

    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        Ok(Self::with_state(
            fs::read_to_string(path)?,
            Some(path.to_path_buf()),
            false,
        ))
    }

    pub fn save(&mut self) -> io::Result<()> {
        let path = self
            .path
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "document has no path"))?;
        fs::write(path, &self.text)?;
        self.dirty = false;
        Ok(())
    }

    pub fn save_as(&mut self, path: impl AsRef<Path>) -> io::Result<()> {
        let path = path.as_ref();
        fs::write(path, &self.text)?;
        self.path = Some(path.to_path_buf());
        self.dirty = false;
        Ok(())
    }

    pub fn export_to(
        &self,
        path: impl AsRef<Path>,
        format: ExportFormat,
    ) -> io::Result<ExportBackend> {
        self.export_to_with(path, format, &ExportPreferences::default())
    }

    /// Exports with explicit export settings (the app passes the `[export]`
    /// config values). Returns which backend produced the file: PDF/DOCX try
    /// the Typune pandoc engine first and fall back to the built-in writers
    /// on any failure, so export never needs external tools; every other
    /// format is always built-in.
    pub fn export_to_with(
        &self,
        path: impl AsRef<Path>,
        format: ExportFormat,
        settings: &ExportPreferences,
    ) -> io::Result<ExportBackend> {
        let path = path.as_ref();
        match format {
            ExportFormat::Markdown => {
                fs::write(path, &self.text)?;
                Ok(ExportBackend::BuiltIn)
            }
            ExportFormat::Html => {
                fs::write(path, self.render_html_document())?;
                Ok(ExportBackend::BuiltIn)
            }
            ExportFormat::PlainHtml => {
                fs::write(path, self.render_plain_html_document())?;
                Ok(ExportBackend::BuiltIn)
            }
            ExportFormat::Pdf => match export::engine_pdf(&self.text, &settings.pdf_engine) {
                Some(bytes) => {
                    fs::write(path, bytes)?;
                    Ok(ExportBackend::PandocEngine)
                }
                None => {
                    let mut file = fs::File::create(path)?;
                    write_pdf(&mut file, &self.plain_text_preview())?;
                    Ok(ExportBackend::BuiltIn)
                }
            },
            ExportFormat::Latex => {
                fs::write(path, self.render_latex_document())?;
                Ok(ExportBackend::BuiltIn)
            }
            ExportFormat::Docx => match export::engine_docx(&self.text) {
                Some(bytes) => {
                    fs::write(path, bytes)?;
                    Ok(ExportBackend::PandocEngine)
                }
                None => {
                    write_docx(path, self)?;
                    Ok(ExportBackend::BuiltIn)
                }
            },
            ExportFormat::Png => {
                write_image_snapshot(path, &self.plain_text_preview(), image::ImageFormat::Png)?;
                Ok(ExportBackend::BuiltIn)
            }
            ExportFormat::Jpeg => {
                write_image_snapshot(path, &self.plain_text_preview(), image::ImageFormat::Jpeg)?;
                Ok(ExportBackend::BuiltIn)
            }
        }
    }

    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn refresh_dirty_from_disk(&mut self) {
        let Some(path) = self.path.as_ref() else {
            return;
        };

        self.dirty = fs::read_to_string(path).map_or(true, |saved_text| saved_text != self.text);
    }

    pub fn front_matter(&self) -> Result<Option<YamlFrontMatter>, FrontMatterError> {
        let Some((raw, _body_start)) = split_front_matter(&self.text) else {
            return Ok(None);
        };
        parse_front_matter(raw).map(Some)
    }

    pub fn body_text(&self) -> &str {
        self.body_text_and_offset().0
    }

    fn body_text_and_offset(&self) -> (&str, usize) {
        split_front_matter(&self.text)
            .map(|(_, body_start)| (&self.text[body_start..], body_start))
            .unwrap_or((&self.text, 0))
    }

    pub fn set_text(&mut self, text: impl Into<String>) {
        let text = text.into();
        if self.text != text {
            self.text = text;
            self.invalidate_derived();
        }
    }

    pub fn insert(&mut self, byte_index: usize, text: &str) {
        let index = clamp_to_char_boundary(&self.text, byte_index);
        self.replace_source_range(index..index, text);
    }

    pub fn replace_range(&mut self, range: std::ops::Range<usize>, text: &str) {
        let start = clamp_to_char_boundary(&self.text, range.start);
        let end = clamp_to_char_boundary(&self.text, range.end).max(start);
        self.replace_source_range(start..end, text);
    }

    fn replace_source_range(&mut self, range: Range<usize>, replacement: &str) {
        let old_version = self.text_version;
        let new_version = old_version.wrapping_add(1);
        let edit = source_mapped::SourceEdit::new(
            &self.text,
            range.clone(),
            replacement.len(),
            old_version,
            new_version,
        );
        self.text.replace_range(range, replacement);
        self.dirty = true;
        self.text_version = new_version;
        *self.cached_preview_blocks.borrow_mut() = None;
        *self.cached_outline.borrow_mut() = None;
        *self.cached_stats.borrow_mut() = None;
        self.cached_line_count.set(None);
        if let Some(edit) = edit {
            self.pending_source_edits.borrow_mut().record(edit);
        } else {
            *self.pending_source_edits.borrow_mut() = source_mapped::PendingSourceEdits::Full;
        }
    }

    /// Marks the document as modified and discards any cached derived state.
    /// Called from every text mutation so the version-gated caches always
    /// reflect the current text.
    fn invalidate_derived(&mut self) {
        self.dirty = true;
        self.text_version = self.text_version.wrapping_add(1);
        *self.cached_preview_blocks.borrow_mut() = None;
        *self.cached_visual_blocks.borrow_mut() = None;
        *self.cached_outline.borrow_mut() = None;
        *self.cached_stats.borrow_mut() = None;
        self.cached_line_count.set(None);
        *self.source_mapped_cache.borrow_mut() = None;
        *self.pending_source_edits.borrow_mut() = source_mapped::PendingSourceEdits::Full;
    }

    /// Monotonically increasing counter bumped by every text mutation. Callers
    /// can key their own derived caches on this value.
    pub fn version(&self) -> u64 {
        self.text_version
    }

    /// Number of logical lines (newline count + 1), cached per text version.
    /// The editor layout asks for this every frame.
    pub fn line_count(&self) -> usize {
        if let Some((version, count)) = self.cached_line_count.get()
            && version == self.text_version
        {
            return count;
        }
        let count = self.text.bytes().filter(|byte| *byte == b'\n').count() + 1;
        self.cached_line_count.set(Some((self.text_version, count)));
        count
    }

    pub fn line_start_at(&self, byte_index: usize) -> usize {
        let index = clamp_to_char_boundary(&self.text, byte_index);
        self.text[..index].rfind('\n').map_or(0, |index| index + 1)
    }

    pub fn line_end_at(&self, byte_index: usize) -> usize {
        let index = clamp_to_char_boundary(&self.text, byte_index);
        self.text[index..]
            .find('\n')
            .map_or(self.text.len(), |line_end| index + line_end)
    }

    pub fn paragraph_range_at(&self, byte_index: usize) -> Range<usize> {
        paragraph_range_at(&self.text, byte_index)
    }

    pub fn previous_line_offset(&self, byte_index: usize) -> usize {
        let index = clamp_to_char_boundary(&self.text, byte_index);
        let current_line_start = self.line_start_at(index);
        if current_line_start == 0 {
            return self.line_start_at(index);
        }

        let column = self.text[current_line_start..index].chars().count();
        let previous_line_end = current_line_start - 1;
        let previous_line_start = self.line_start_at(previous_line_end);
        self.offset_at_line_column(previous_line_start, column)
    }

    pub fn next_line_offset(&self, byte_index: usize) -> usize {
        let index = clamp_to_char_boundary(&self.text, byte_index);
        let current_line_start = self.line_start_at(index);
        let current_line_end = self.line_end_at(index);
        if current_line_end == self.text.len() {
            return self.line_end_at(index);
        }

        let column = self.text[current_line_start..index].chars().count();
        self.offset_at_line_column(current_line_end + 1, column)
    }

    pub fn indent_lines(&mut self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        let line_starts = selected_line_starts(&self.text, range.clone());
        if line_starts.is_empty() {
            return range;
        }

        let mut inserted = 0usize;
        for line_start in line_starts.iter().copied() {
            self.text.insert_str(line_start + inserted, "    ");
            inserted += 4;
        }
        self.invalidate_derived();

        let first_line_start = *line_starts.first().unwrap();
        let new_start = if range.start <= first_line_start {
            range.start
        } else {
            range.start + 4
        };
        new_start..range.end + inserted
    }

    pub fn outdent_lines(&mut self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        let line_starts = selected_line_starts(&self.text, range.clone());
        if line_starts.is_empty() {
            return range;
        }

        let mut removed_before_start = 0usize;
        let mut removed_before_end = 0usize;
        let mut removed_total = 0usize;
        for line_start in line_starts.iter().copied() {
            let adjusted_line_start = line_start - removed_total;
            let remove_len = line_outdent_len(&self.text, adjusted_line_start);
            if remove_len == 0 {
                continue;
            }

            self.text
                .replace_range(adjusted_line_start..adjusted_line_start + remove_len, "");
            if line_start < range.start {
                removed_before_start += remove_len;
            }
            if line_start < range.end {
                removed_before_end += remove_len;
            }
            removed_total += remove_len;
        }

        if removed_total > 0 {
            self.invalidate_derived();
        }

        range.start.saturating_sub(removed_before_start)
            ..range.end.saturating_sub(removed_before_end)
    }

    pub fn apply_markdown_format(
        &mut self,
        range: std::ops::Range<usize>,
        format: MarkdownFormat,
    ) -> std::ops::Range<usize> {
        let range = clamp_range_to_char_boundaries(&self.text, range);
        match format {
            MarkdownFormat::Bold => self.wrap_inline(range, "**", "**", "bold"),
            MarkdownFormat::Italic => self.wrap_inline(range, "*", "*", "italic"),
            MarkdownFormat::InlineCode => self.wrap_inline(range, "`", "`", "code"),
            MarkdownFormat::Link => self.wrap_link(range, false),
            MarkdownFormat::Image => self.wrap_link(range, true),
            MarkdownFormat::Heading(level) => self.apply_heading(range, level.clamp(1, 6)),
            MarkdownFormat::UnorderedList => self.prefix_lines(range, |_, _| "- ".to_string()),
            MarkdownFormat::OrderedList => {
                self.prefix_lines(range, |line_index, _| format!("{}. ", line_index + 1))
            }
            MarkdownFormat::TaskList => self.prefix_lines(range, |_, _| "- [ ] ".to_string()),
            MarkdownFormat::BlockQuote => self.prefix_lines(range, |_, _| "> ".to_string()),
            MarkdownFormat::CodeFence => self.wrap_code_fence(range),
        }
    }

    pub fn table_range_at(&self, byte_index: usize) -> Option<Range<usize>> {
        table_range_at_fn(&self.text, byte_index)
    }

    pub fn table_ranges(&self) -> Vec<Range<usize>> {
        table_ranges_fn(&self.text)
    }

    pub fn edit_table_at(&mut self, byte_index: usize, edit: TableEdit) -> Option<TableEditResult> {
        let byte_index = clamp_to_char_boundary(&self.text, byte_index);
        let table_range = self.table_range_at(byte_index)?;
        let table_source = &self.text[table_range.clone()];
        let table_position = table_position_at(table_source, byte_index - table_range.start)?;
        let mut table = parse_markdown_table(table_source)?;
        let mut selected_row = table_position.row.min(table.rows.len().saturating_sub(1));
        let mut selected_column = table_position
            .column
            .min(table.column_count().saturating_sub(1));

        match edit {
            TableEdit::Format => {}
            TableEdit::AddRow => {
                selected_row = selected_row.max(1);
                let insert_at = (selected_row + 1).min(table.rows.len());
                table
                    .rows
                    .insert(insert_at, vec![String::new(); table.column_count()]);
                selected_row = insert_at;
                selected_column = 0;
            }
            TableEdit::DeleteRow => {
                if selected_row == 0 || table.rows.len() <= 1 {
                    return None;
                }
                table.rows.remove(selected_row);
                selected_row = selected_row.min(table.rows.len().saturating_sub(1)).max(1);
                selected_column = selected_column.min(table.column_count().saturating_sub(1));
            }
            TableEdit::MoveRowUp => {
                if selected_row <= 1 {
                    return None;
                }
                table.rows.swap(selected_row, selected_row - 1);
                selected_row -= 1;
            }
            TableEdit::MoveRowDown => {
                if selected_row == 0 || selected_row + 1 >= table.rows.len() {
                    return None;
                }
                table.rows.swap(selected_row, selected_row + 1);
                selected_row += 1;
            }
            TableEdit::AddColumn => {
                let insert_at = (selected_column + 1).min(table.column_count());
                for row in &mut table.rows {
                    row.insert(insert_at, String::new());
                }
                table.alignments.insert(insert_at, TableAlignment::Default);
                selected_column = insert_at;
            }
            TableEdit::DeleteColumn => {
                if table.column_count() <= 1 {
                    return None;
                }
                for row in &mut table.rows {
                    row.remove(selected_column);
                }
                table.alignments.remove(selected_column);
                selected_column = selected_column.min(table.column_count().saturating_sub(1));
            }
        }

        table.normalize();
        let replacement = format_markdown_table(&table);
        let selection_in_table =
            formatted_table_cell_range(&table, selected_row, selected_column).unwrap_or(0..0);
        let selected_range = table_range.start + selection_in_table.start
            ..table_range.start + selection_in_table.end;

        if replacement != table_source {
            self.replace_source_range(table_range.clone(), &replacement);
        }

        Some(TableEditResult {
            table_range: table_range.start..table_range.start + replacement.len(),
            selected_range,
            row: selected_row,
            column: selected_column,
        })
    }

    fn wrap_inline(
        &mut self,
        range: std::ops::Range<usize>,
        prefix: &str,
        suffix: &str,
        placeholder: &str,
    ) -> std::ops::Range<usize> {
        if range.start >= prefix.len()
            && range.end + suffix.len() <= self.text.len()
            && self.text.is_char_boundary(range.start - prefix.len())
            && self.text.is_char_boundary(range.end + suffix.len())
            && &self.text[range.start - prefix.len()..range.start] == prefix
            && &self.text[range.end..range.end + suffix.len()] == suffix
        {
            let replacement = self.text[range.clone()].to_string();
            self.replace_source_range(
                range.start - prefix.len()..range.end + suffix.len(),
                &replacement,
            );
            return range.start - prefix.len()..range.end - prefix.len();
        }

        if range.end - range.start >= prefix.len() + suffix.len()
            && self.text[range.clone()].starts_with(prefix)
            && self.text[range.clone()].ends_with(suffix)
        {
            let replacement =
                self.text[range.start + prefix.len()..range.end - suffix.len()].to_string();
            self.replace_source_range(range.clone(), &replacement);
            return range.start..range.end - prefix.len() - suffix.len();
        }

        let selected = &self.text[range.clone()];
        let inner = if selected.is_empty() {
            placeholder
        } else {
            selected
        };
        let replacement = format!("{prefix}{inner}{suffix}");
        let inner_start = range.start + prefix.len();
        let inner_end = inner_start + inner.len();
        self.replace_source_range(range, &replacement);
        inner_start..inner_end
    }

    fn wrap_link(&mut self, range: std::ops::Range<usize>, image: bool) -> std::ops::Range<usize> {
        let selected = self.text[range.clone()].to_string();
        let selected_is_empty = selected.is_empty();
        let prefix = if image { "![" } else { "[" };
        let label_placeholder = if image { "alt" } else { "text" };
        let url_placeholder = if image { "image.png" } else { "url" };
        let label = if selected_is_empty {
            label_placeholder
        } else {
            &selected
        };
        let replacement = format!("{prefix}{label}]({url_placeholder})");
        let label_start = range.start + prefix.len();
        let label_end = label_start + label.len();
        let url_start = label_end + "](".len();
        let url_end = url_start + url_placeholder.len();

        self.replace_source_range(range, &replacement);

        if selected_is_empty {
            label_start..label_end
        } else {
            url_start..url_end
        }
    }

    fn wrap_code_fence(&mut self, range: std::ops::Range<usize>) -> std::ops::Range<usize> {
        let selected = &self.text[range.clone()];
        let inner = if selected.is_empty() {
            "code"
        } else {
            selected.trim_matches('\n')
        };
        let replacement = format!("```\n{inner}\n```");
        let inner_start = range.start + "```\n".len();
        let inner_end = inner_start + inner.len();
        self.replace_source_range(range, &replacement);
        inner_start..inner_end
    }

    fn prefix_lines(
        &mut self,
        range: std::ops::Range<usize>,
        mut prefix_for_line: impl FnMut(usize, &str) -> String,
    ) -> std::ops::Range<usize> {
        let line_starts = selected_line_starts(&self.text, range.clone());
        if line_starts.is_empty() {
            let prefix = prefix_for_line(0, "");
            self.replace_source_range(range.start..range.start, &prefix);
            return range.start + prefix.len()..range.start + prefix.len();
        }

        let mut inserted = 0usize;
        let mut inserted_before_start = 0usize;
        let mut inserted_before_end = 0usize;
        for (line_index, line_start) in line_starts.iter().copied().enumerate() {
            let adjusted_line_start = line_start + inserted;
            let line_end = self.line_end_at(adjusted_line_start);
            let prefix = prefix_for_line(line_index, &self.text[adjusted_line_start..line_end]);
            self.text.insert_str(adjusted_line_start, &prefix);
            if line_start < range.start || (range.is_empty() && line_start == range.start) {
                inserted_before_start += prefix.len();
            }
            if line_start < range.end || (range.is_empty() && line_start == range.end) {
                inserted_before_end += prefix.len();
            }
            inserted += prefix.len();
        }
        self.invalidate_derived();
        range.start + inserted_before_start..range.end + inserted_before_end
    }

    fn apply_heading(
        &mut self,
        range: std::ops::Range<usize>,
        level: u8,
    ) -> std::ops::Range<usize> {
        let line_starts = selected_line_starts(&self.text, range.clone());
        if line_starts.is_empty() {
            let prefix = format!("{} ", "#".repeat(level as usize));
            self.replace_source_range(range.start..range.start, &prefix);
            return range.start + prefix.len()..range.start + prefix.len();
        }

        let all_same_level = line_starts
            .iter()
            .copied()
            .all(|line_start| heading_level_at(&self.text, line_start) == Some(level));

        let prefix = (!all_same_level).then(|| format!("{} ", "#".repeat(level as usize)));
        let mut delta: isize = 0;
        let mut start_delta: isize = 0;
        let mut end_delta: isize = 0;

        for line_start in line_starts.iter().copied() {
            let adjusted_line_start = (line_start as isize + delta) as usize;
            let existing_len = heading_marker_len_at(&self.text, adjusted_line_start);
            if existing_len > 0 {
                self.text
                    .replace_range(adjusted_line_start..adjusted_line_start + existing_len, "");
                adjust_offset_for_line_marker_removal(
                    range.start,
                    line_start,
                    existing_len,
                    &mut start_delta,
                );
                adjust_offset_for_line_marker_removal(
                    range.end,
                    line_start,
                    existing_len,
                    &mut end_delta,
                );
                delta -= existing_len as isize;
            }
            if let Some(prefix) = prefix.as_ref() {
                // Insert the new prefix at the line's *current* start, which
                // (after any marker removal above) is `adjusted_line_start`.
                // Using `line_start + delta` here underflows when delta went
                // negative from removing an existing marker, and the resulting
                // huge index panics inside `insert_str` on a char-boundary check.
                self.text.insert_str(adjusted_line_start, prefix);
                adjust_offset_for_line_insert(
                    range.start,
                    line_start,
                    prefix.len(),
                    range.is_empty(),
                    &mut start_delta,
                );
                adjust_offset_for_line_insert(
                    range.end,
                    line_start,
                    prefix.len(),
                    range.is_empty(),
                    &mut end_delta,
                );
                delta += prefix.len() as isize;
            }
        }

        self.invalidate_derived();
        let start = offset_with_delta(range.start, start_delta);
        let end = offset_with_delta(range.end, end_delta).max(start);
        start..end
    }

    fn offset_at_line_column(&self, line_start: usize, column: usize) -> usize {
        let line_start = clamp_to_char_boundary(&self.text, line_start);
        let line_end = self.line_end_at(line_start);
        let mut chars = self.text[line_start..line_end].char_indices();
        let mut offset = line_start;

        for _ in 0..column {
            match chars.next() {
                Some((index, ch)) => offset = line_start + index + ch.len_utf8(),
                None => return line_end,
            }
        }

        offset.min(line_end)
    }

    pub fn insert_markdown_newline(&mut self, byte_index: usize) -> usize {
        let cursor = clamp_to_char_boundary(&self.text, byte_index);
        let line_start = self.text[..cursor].rfind('\n').map_or(0, |index| index + 1);
        let line_end = self.text[cursor..]
            .find('\n')
            .map_or(self.text.len(), |index| cursor + index);
        let before_cursor = &self.text[line_start..cursor];
        let after_cursor = &self.text[cursor..line_end];

        if after_cursor.trim().is_empty() && is_empty_list_marker(before_cursor) {
            self.replace_source_range(line_start..cursor, "");
            return line_start;
        }

        let continuation = markdown_continuation(before_cursor);
        let insertion = format!("\n{continuation}");
        self.replace_source_range(cursor..cursor, &insertion);
        cursor + insertion.len()
    }

    /// Compute a single source edit for a Visual Edit structural Enter.
    /// Returning an edit rather than mutating here lets the application take
    /// exactly one undo snapshot and perform exactly one document replacement.
    pub fn visual_enter_edit(&self, byte_index: usize) -> Option<VisualStructuralEdit> {
        let cursor = clamp_to_char_boundary(&self.text, byte_index);
        let prefix = visual::structural_prefix_at(&self.text, cursor)?;
        let line_start = self.line_start_at(cursor);
        let line_end = self.line_end_at(cursor);
        if prefix.source_range.start != line_start || cursor < prefix.source_range.end {
            return None;
        }
        let content_is_empty = self.text[prefix.source_range.end..line_end]
            .trim()
            .is_empty();
        let exits_when_empty = matches!(
            prefix.kind,
            VisualBlockPrefixKind::BlockQuote { .. }
                | VisualBlockPrefixKind::UnorderedList { .. }
                | VisualBlockPrefixKind::OrderedList { .. }
                | VisualBlockPrefixKind::TaskList { .. }
        );
        let empty_structure_exit = content_is_empty && exits_when_empty;

        let blocks = self.visual_blocks_shared();
        let active_block = blocks.iter().find(|block| {
            block.source_range.contains(&cursor)
                || (cursor == self.text.len() && cursor == block.source_range.end)
        })?;
        let supported_block = matches!(
            active_block.kind,
            VisualBlockKind::Heading { .. }
                | VisualBlockKind::BlockQuote
                | VisualBlockKind::ListItem { .. }
        );
        // pulldown-cmark does not emit preview content for an empty blockquote
        // or ordinary list item, so the visual model conservatively represents
        // a line such as `> ` or `- ` as an unsupported gap. Permit only the
        // exact empty-structure exit in that gap; explicit code, HTML, math,
        // front-matter, image, and table source islands remain excluded.
        let orphaned_empty_structure = empty_structure_exit
            && matches!(active_block.kind, VisualBlockKind::Unsupported)
            && active_block.source_island == Some(VisualSourceIslandKind::Unsupported)
            && active_block.source_range.start <= line_start
            && active_block.source_range.end >= line_end;
        if !supported_block && !orphaned_empty_structure {
            return None;
        }
        if empty_structure_exit {
            return Some(VisualStructuralEdit {
                range: prefix.source_range,
                replacement: String::new(),
                selection_after: line_start..line_start,
            });
        }

        let continuation = markdown_continuation(&self.text[line_start..cursor]);
        let replacement = format!("\n{continuation}");
        let next = cursor + replacement.len();
        Some(VisualStructuralEdit {
            range: cursor..cursor,
            replacement,
            selection_after: next..next,
        })
    }

    /// Compute a single source edit for Backspace at the first visible content
    /// position of a supported Visual Edit block.
    pub fn visual_backspace_edit(&self, byte_index: usize) -> Option<VisualStructuralEdit> {
        let cursor = clamp_to_char_boundary(&self.text, byte_index);
        let blocks = self.visual_blocks_shared();
        let active_block = blocks.iter().find(|block| {
            block.source_range.contains(&cursor)
                || (cursor == self.text.len() && cursor == block.source_range.end)
        })?;
        if !matches!(
            active_block.kind,
            VisualBlockKind::Heading { .. }
                | VisualBlockKind::BlockQuote
                | VisualBlockKind::ListItem { .. }
        ) {
            return None;
        }
        let prefix = visual::structural_prefix_at(&self.text, cursor)?;
        if cursor != prefix.source_range.end {
            return None;
        }
        let nested_list = matches!(
            prefix.kind,
            VisualBlockPrefixKind::UnorderedList { .. }
                | VisualBlockPrefixKind::OrderedList { .. }
                | VisualBlockPrefixKind::TaskList { .. }
        ) && !prefix.indentation_range.is_empty();
        if nested_list {
            let remove_len = line_outdent_len(&self.text, prefix.indentation_range.start)
                .min(prefix.indentation_range.len());
            if remove_len == 0 {
                return None;
            }
            let next = cursor - remove_len;
            return Some(VisualStructuralEdit {
                range: prefix.indentation_range.start..prefix.indentation_range.start + remove_len,
                replacement: String::new(),
                selection_after: next..next,
            });
        }
        let next = prefix.source_range.start;
        Some(VisualStructuralEdit {
            range: prefix.source_range,
            replacement: String::new(),
            selection_after: next..next,
        })
    }

    pub fn render_html_fragment(&self) -> String {
        let body = self.body_text();
        let parser = Parser::new_ext(body, markdown_options());
        let (events, formulas) = collect_html_math(parser.into_offset_iter(), body);
        let (events, diagrams) = collect_html_diagrams(events);
        let mut output = String::new();
        html::push_html(&mut output, events.into_iter());
        output = annotate_math_html(&render_extended_html_text_nodes(&output));
        for formula in formulas {
            formula.apply(&mut output);
        }
        for diagram in diagrams {
            diagram.apply(&mut output);
        }
        output
    }

    pub fn render_html_document(&self) -> String {
        self.render_html_document_with_style(true)
    }

    pub fn render_plain_html_document(&self) -> String {
        self.render_html_document_with_style(false)
    }

    pub fn render_latex_document(&self) -> String {
        let metadata = self.front_matter().ok().flatten();
        let title = metadata
            .as_ref()
            .and_then(|metadata| metadata.title.as_deref())
            .unwrap_or("Untitled");
        let author = metadata
            .as_ref()
            .and_then(|metadata| metadata.author.as_deref())
            .unwrap_or("");
        let date = metadata
            .as_ref()
            .and_then(|metadata| metadata.date.as_deref())
            .unwrap_or("\\today");

        format!(
            "\\documentclass{{article}}\n\\usepackage[utf8]{{inputenc}}\n\\usepackage{{hyperref}}\n\\usepackage{{graphicx}}\n\\usepackage{{longtable}}\n\\usepackage[normalem]{{ulem}}\n\\usepackage{{soul}}\n\\usepackage{{listings}}\n\\usepackage{{amssymb}}\n\\title{{{}}}\n\\author{{{}}}\n\\date{{{}}}\n\\begin{{document}}\n\\maketitle\n\n{}\\end{{document}}\n",
            escape_latex(title),
            escape_latex(author),
            if date == "\\today" {
                date.to_string()
            } else {
                escape_latex(date)
            },
            self.render_latex_body()
        )
    }

    fn render_latex_body(&self) -> String {
        let mut output = String::new();
        let mut blocks = self.preview_blocks().into_iter().peekable();
        while let Some(block) = blocks.next() {
            match block {
                PreviewBlock::Heading { level, text, .. } => {
                    let command = match level {
                        1 => "section",
                        2 => "subsection",
                        3 => "subsubsection",
                        _ => "paragraph",
                    };
                    output.push_str(&format!("\\{command}{{{}}}\n\n", escape_latex(&text.text)));
                }
                PreviewBlock::Paragraph { text, .. } => {
                    output.push_str(&render_latex_rich_text(&text));
                    output.push_str("\n\n");
                }
                PreviewBlock::ListItem {
                    ordered,
                    checked,
                    text,
                    ..
                } => {
                    let environment = if ordered { "enumerate" } else { "itemize" };
                    output.push_str(&format!("\\begin{{{environment}}}\n"));
                    push_latex_list_item(&mut output, checked, &text);
                    // Consecutive same-kind items share one environment.
                    while let Some(PreviewBlock::ListItem {
                        ordered: next_ordered,
                        ..
                    }) = blocks.peek()
                    {
                        if *next_ordered != ordered {
                            break;
                        }
                        let Some(PreviewBlock::ListItem { checked, text, .. }) = blocks.next()
                        else {
                            unreachable!("peeked a list item");
                        };
                        push_latex_list_item(&mut output, checked, &text);
                    }
                    output.push_str(&format!("\\end{{{environment}}}\n\n"));
                }
                PreviewBlock::BlockQuote { text, .. } => {
                    output.push_str("\\begin{quote}\n");
                    output.push_str(&render_latex_rich_text(&text));
                    output.push_str("\n\\end{quote}\n\n");
                }
                PreviewBlock::CodeBlock { language, code, .. } => {
                    let options = latex_listing_language(language.as_deref())
                        .map(|name| format!("[language={name}]"))
                        .unwrap_or_default();
                    output.push_str(&format!("\\begin{{lstlisting}}{options}\n"));
                    output.push_str(&code);
                    if !code.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push_str("\\end{lstlisting}\n\n");
                }
                PreviewBlock::MathBlock { latex, .. } => {
                    output.push_str("\\[\n");
                    output.push_str(latex.trim());
                    output.push_str("\n\\]\n\n");
                }
                PreviewBlock::Html { html, .. } => {
                    for part in html_preview_parts(&html) {
                        match part {
                            parse::HtmlPreviewPart::Text { text, .. } => {
                                output.push_str(&render_latex_rich_text(&text));
                                output.push_str("\n\n");
                            }
                            parse::HtmlPreviewPart::Image { alt, url, .. } => {
                                output.push_str("\\begin{figure}[h]\n\\centering\n");
                                output.push_str(&format!(
                                    "\\includegraphics[width=\\linewidth]{{{}}}\n",
                                    escape_latex_path(&url)
                                ));
                                if !alt.is_empty() {
                                    output.push_str(&format!(
                                        "\\caption{{{}}}\n",
                                        escape_latex(&alt)
                                    ));
                                }
                                output.push_str("\\end{figure}\n\n");
                            }
                        }
                    }
                }
                PreviewBlock::Image { alt, url, .. } => {
                    output.push_str("\\begin{figure}[h]\n\\centering\n");
                    output.push_str(&format!(
                        "\\includegraphics[width=\\linewidth]{{{}}}\n",
                        escape_latex_path(&url)
                    ));
                    if !alt.is_empty() {
                        output.push_str(&format!("\\caption{{{}}}\n", escape_latex(&alt)));
                    }
                    output.push_str("\\end{figure}\n\n");
                }
                PreviewBlock::Rule { .. } => output.push_str("\\hrule\n\n"),
                PreviewBlock::FootnoteDefinition { label, text, .. } => {
                    output.push_str(&format!(
                        "[{}] {}\n\n",
                        escape_latex(&label),
                        render_latex_rich_text(&text).trim()
                    ));
                }
                PreviewBlock::Table {
                    rows, alignments, ..
                } => {
                    output.push_str(&render_latex_table(&rows, &alignments));
                    output.push_str("\n\n");
                }
            }
        }

        output
    }

    fn render_html_document_with_style(&self, styled: bool) -> String {
        let metadata = self.front_matter().ok().flatten();
        let title = metadata
            .as_ref()
            .and_then(|metadata| metadata.title.as_deref())
            .or_else(|| {
                self.path
                    .as_ref()
                    .and_then(|path| path.file_stem())
                    .and_then(|stem| stem.to_str())
            })
            .unwrap_or("Untitled");
        let author = metadata
            .as_ref()
            .and_then(|metadata| metadata.author.as_deref())
            .map(|author| {
                format!(
                    "\n<meta name=\"author\" content=\"{}\">",
                    escape_html_attribute(author)
                )
            })
            .unwrap_or_default();
        let date = metadata
            .as_ref()
            .and_then(|metadata| metadata.date.as_deref())
            .map(|date| {
                format!(
                    "\n<meta name=\"date\" content=\"{}\">",
                    escape_html_attribute(date)
                )
            })
            .unwrap_or_default();
        let style = if styled {
            format!("\n<style>{DEFAULT_CSS}</style>")
        } else {
            String::new()
        };

        format!(
            "<!doctype html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n<title>{}</title>{author}{date}{style}\n</head>\n<body>\n{}\n</body>\n</html>\n",
            escape_html_text(title),
            self.render_html_fragment()
        )
    }

    pub fn plain_text_preview(&self) -> String {
        let mut output = String::new();
        for event in Parser::new_ext(self.body_text(), markdown_options()) {
            match event {
                Event::Text(text) | Event::Code(text) => output.push_str(&text),
                Event::Html(text) | Event::InlineHtml(text) => {
                    output.push_str(&html_preview_plain_text(&text));
                }
                Event::SoftBreak | Event::HardBreak => output.push('\n'),
                Event::End(TagEnd::Paragraph | TagEnd::Heading(_)) => output.push_str("\n\n"),
                Event::End(TagEnd::Item) => output.push('\n'),
                _ => {}
            }
        }
        output.trim().to_string()
    }

    pub fn preview_blocks(&self) -> Vec<PreviewBlock> {
        (*self.preview_blocks_shared()).clone()
    }

    /// Cached preview blocks behind an `Arc`, so the per-frame render path can
    /// take a reference-counted handle instead of deep-copying every block.
    pub fn preview_blocks_shared(&self) -> std::sync::Arc<Vec<PreviewBlock>> {
        if let Some(cached) = self.cached_preview_blocks.borrow().as_ref()
            && cached.version == self.text_version
        {
            return cached.value.clone();
        }

        // One pulldown pass yields both the preview blocks and the outline
        // headings; cache both so an open outline panel this frame is a cache
        // hit rather than a second full parse.
        let (blocks, headings) = Self::derive_preview_and_outline(&self.text);
        let blocks = std::sync::Arc::new(blocks);
        let version = self.text_version;
        *self.cached_preview_blocks.borrow_mut() = Some(Cached {
            version,
            value: blocks.clone(),
        });
        *self.cached_outline.borrow_mut() = Some(Cached {
            version,
            value: headings,
        });
        blocks
    }

    pub fn visual_blocks(&self) -> Vec<VisualBlock> {
        (*self.visual_blocks_shared()).clone()
    }

    /// Source-ranged Visual Edit model cached strictly by document version.
    /// Cursor, selection, hover, and focus changes therefore reuse the same
    /// allocation and do not trigger another Markdown parse.
    pub fn visual_blocks_shared(&self) -> std::sync::Arc<Vec<VisualBlock>> {
        if let Some(cached) = self.cached_visual_blocks.borrow().as_ref()
            && cached.version == self.text_version
        {
            return cached.value.clone();
        }
        let previous_cache = self.source_mapped_cache.borrow_mut().take();
        let pending = self.pending_source_edits.borrow().clone();
        let current_cached_full = self
            .cached_preview_blocks
            .borrow()
            .as_ref()
            .filter(|cached| cached.version == self.text_version)
            .and_then(|preview| {
                self.cached_outline
                    .borrow()
                    .as_ref()
                    .filter(|cached| cached.version == self.text_version)
                    .map(|outline| (preview.value.clone(), outline.value.clone()))
            });
        let cache = match (previous_cache.as_ref(), pending.edits()) {
            (Some(previous), Some(edits)) if !edits.is_empty() => {
                source_mapped::SourceMappedCache::update(
                    previous,
                    &self.text,
                    self.text_version,
                    edits,
                )
            }
            (Some(previous), Some(_))
                if previous.version == self.text_version
                    && previous.source.as_ref() == self.text =>
            {
                previous.clone()
            }
            _ => current_cached_full.map_or_else(
                || source_mapped::SourceMappedCache::derive_full(&self.text, self.text_version),
                |(blocks, headings)| {
                    source_mapped::SourceMappedCache::from_cached_full(
                        &self.text,
                        self.text_version,
                        blocks,
                        headings,
                    )
                },
            ),
        };

        let mut blocks =
            visual::build_visual_blocks(&self.text, &cache.blocks, VisualBlockId::fresh);
        if let (Some(previous), Some(edits)) = (previous_cache.as_ref(), pending.edits())
            && !edits.is_empty()
            && let Some(old_visual) = self.cached_visual_blocks.borrow().as_ref()
            && old_visual.version == previous.version
        {
            source_mapped::reconcile_visual_block_ids(
                previous.source.as_ref(),
                &self.text,
                &old_visual.value,
                &mut blocks,
                edits,
            );
        }
        let blocks = std::sync::Arc::new(blocks);
        *self.cached_preview_blocks.borrow_mut() = Some(Cached {
            version: self.text_version,
            value: cache.blocks.clone(),
        });
        *self.cached_outline.borrow_mut() = Some(Cached {
            version: self.text_version,
            value: cache.headings.clone(),
        });
        *self.source_mapped_cache.borrow_mut() = Some(cache);
        self.pending_source_edits.borrow_mut().reset_incremental();
        *self.cached_visual_blocks.borrow_mut() = Some(Cached {
            version: self.text_version,
            value: blocks.clone(),
        });
        blocks
    }

    /// Build one validated replacement for a dedicated Visual Edit block
    /// field. Prose and complete source islands intentionally return `None`
    /// and keep using the ordinary source-selection path.
    pub fn direct_visual_block_edit(
        &self,
        range: Range<usize>,
        replacement: &str,
    ) -> Option<VisualBlockEdit> {
        if range.start > range.end
            || range.end > self.text.len()
            || !self.text.is_char_boundary(range.start)
            || !self.text.is_char_boundary(range.end)
        {
            return None;
        }
        let blocks = self.visual_blocks_shared();
        let (block, field) = blocks.iter().find_map(|block| {
            block
                .editor
                .as_ref()?
                .field_containing(&range)
                .map(|field| (block, field))
        })?;
        let replacement = sanitize_visual_field_replacement(
            &self.text,
            field.kind,
            &field.source_range,
            replacement,
        );
        if let VisualEditorFieldKind::TableCell { row, column } = field.kind {
            let table_source = self.text.get(block.source_range.clone())?;
            let authored_cells = table_cell_source_ranges(table_source)?;
            let authored_cell = authored_cells
                .iter()
                .find(|cell| cell.row == row && cell.column == column)?;
            let authored_range = block.source_range.start + authored_cell.source_range.start
                ..block.source_range.start + authored_cell.source_range.end;
            if authored_range != field.source_range {
                return None;
            }
            let relative =
                range.start - field.source_range.start..range.end - field.source_range.start;
            let mut value = self.text[field.source_range.clone()].to_string();
            value.replace_range(relative.clone(), &replacement);
            let mut table = parse_markdown_table(table_source)?;
            if table.rows.get(row)?.get(column)? != &self.text[field.source_range.clone()] {
                return None;
            }
            table.rows.get_mut(row)?.get_mut(column)?.clone_from(&value);
            let formatted = format_markdown_table(&table);
            let new_cell = formatted_table_cell_range(&table, row, column)?;
            let inserted_start = block.source_range.start + new_cell.start + relative.start;
            let inserted_end = inserted_start + replacement.len();
            return Some(VisualBlockEdit {
                document_version: self.text_version,
                block_id: block.id,
                field: field.clone(),
                range: block.source_range.clone(),
                replacement: formatted,
                inserted_range_after: inserted_start..inserted_end,
                selection_after: inserted_end..inserted_end,
            });
        }
        let inserted_start = range.start;
        let inserted_end = inserted_start + replacement.len();
        Some(VisualBlockEdit {
            document_version: self.text_version,
            block_id: block.id,
            field: field.clone(),
            range,
            replacement,
            inserted_range_after: inserted_start..inserted_end,
            selection_after: inserted_end..inserted_end,
        })
    }

    pub fn validate_visual_block_edit(&self, edit: &VisualBlockEdit) -> bool {
        edit.document_version == self.text_version
            && self.visual_blocks_shared().iter().any(|block| {
                if block.id != edit.block_id {
                    return false;
                }
                let Some(editor) = block.editor.as_ref() else {
                    return false;
                };
                if !editor
                    .fields()
                    .into_iter()
                    .any(|field| field == &edit.field)
                {
                    return false;
                }
                match edit.field.kind {
                    VisualEditorFieldKind::TableCell { .. } => edit.range == block.source_range,
                    _ => {
                        edit.range.start >= edit.field.source_range.start
                            && edit.range.end <= edit.field.source_range.end
                    }
                }
            })
    }

    /// Return the dedicated field that owns the complete canonical selection.
    pub fn visual_editor_field_at(&self, range: &Range<usize>) -> Option<VisualEditorField> {
        self.visual_blocks_shared()
            .iter()
            .find_map(|block| block.editor.as_ref()?.field_containing(range).cloned())
    }

    /// Resolve Tab traversal for multi-field visual editors. Code and math
    /// retain ordinary indentation behavior because each owns one payload.
    pub fn visual_editor_tab_target(
        &self,
        range: &Range<usize>,
        forward: bool,
    ) -> Option<Range<usize>> {
        let blocks = self.visual_blocks_shared();
        let (block_index, editor, field_index) =
            blocks.iter().enumerate().find_map(|(block_index, block)| {
                let editor = block.editor.as_ref()?;
                if !matches!(editor, VisualBlockEditor::Table { .. }) {
                    return None;
                }
                let fields = editor.fields();
                let field_index = fields.iter().position(|field| {
                    range.start >= field.source_range.start && range.end <= field.source_range.end
                })?;
                Some((block_index, editor, field_index))
            })?;
        let fields = editor.fields();
        if forward {
            if let Some(field) = fields.get(field_index + 1) {
                return Some(field.source_range.clone());
            }
            let boundary = blocks
                .get(block_index + 1)
                .map_or(fields[field_index].source_range.end, |block| {
                    block.source_range.start
                });
            Some(boundary..boundary)
        } else {
            if field_index > 0 {
                return Some(fields[field_index - 1].source_range.clone());
            }
            let boundary = block_index
                .checked_sub(1)
                .and_then(|previous| blocks.get(previous))
                .map_or(fields[field_index].source_range.start, |block| {
                    block.source_range.end
                });
            Some(boundary..boundary)
        }
    }

    /// Keep arrow/deletion movement out of hidden block delimiters. A field
    /// edge hands off to its sibling field or the adjacent visual block.
    pub fn visual_editor_edge_target(&self, offset: usize, forward: bool) -> Option<usize> {
        let range = offset..offset;
        let blocks = self.visual_blocks_shared();
        let (block_index, editor, field_index) =
            blocks.iter().enumerate().find_map(|(block_index, block)| {
                let editor = block.editor.as_ref()?;
                let fields = editor.fields();
                let field_index = fields.iter().position(|field| {
                    range.start >= field.source_range.start && range.end <= field.source_range.end
                })?;
                Some((block_index, editor, field_index))
            })?;
        let fields = editor.fields();
        let field = fields[field_index];
        if forward {
            if offset != field.source_range.end {
                return None;
            }
            Some(
                fields
                    .get(field_index + 1)
                    .map(|next| next.source_range.start)
                    .or_else(|| {
                        blocks
                            .get(block_index + 1)
                            .map(|block| block.source_range.start)
                    })
                    .unwrap_or(offset),
            )
        } else {
            if offset != field.source_range.start {
                return None;
            }
            Some(
                field_index
                    .checked_sub(1)
                    .map(|previous| fields[previous].source_range.end)
                    .or_else(|| {
                        block_index
                            .checked_sub(1)
                            .and_then(|previous| blocks.get(previous))
                            .map(|block| block.source_range.end)
                    })
                    .unwrap_or(offset),
            )
        }
    }

    #[cfg(test)]
    fn source_mapped_derivation_counters(&self) -> source_mapped::DerivationCounters {
        self.source_mapped_cache
            .borrow()
            .as_ref()
            .map(|cache| cache.counters)
            .unwrap_or_default()
    }

    /// Fold derived state computed elsewhere (a background thread running
    /// [`Self::derive_preview_and_outline`] on a snapshot of this document's
    /// text taken at `version`) into the caches. Dropped when the document has
    /// changed since the snapshot, so a slow parse can never overwrite the
    /// derived state of newer text.
    pub fn install_derived(
        &self,
        version: u64,
        blocks: std::sync::Arc<Vec<PreviewBlock>>,
        headings: Vec<Heading>,
    ) {
        if version != self.text_version {
            return;
        }
        *self.cached_preview_blocks.borrow_mut() = Some(Cached {
            version,
            value: blocks,
        });
        *self.cached_outline.borrow_mut() = Some(Cached {
            version,
            value: headings,
        });
    }

    /// Single pulldown pass producing both the preview blocks and the outline
    /// headings. Merging them avoids a second full parse for the outline. The
    /// heading offsets/titles are accumulated exactly as `compute_outline_only`
    /// does (from every `Text`/`Code` event inside a heading) so the two paths
    /// stay byte-identical — an invariant locked by a unit test.
    ///
    /// Takes the text instead of `&self` so a background thread can run it on
    /// a snapshot of the document; the result is folded back into the caches
    /// via [`Self::install_derived`].
    pub fn derive_preview_and_outline(text: &str) -> (Vec<PreviewBlock>, Vec<Heading>) {
        let (body, body_offset) = split_front_matter(text)
            .map(|(_, body_start)| (&text[body_start..], body_start))
            .unwrap_or((text, 0));
        let mut blocks = Vec::new();
        let mut headings: Vec<Heading> = Vec::new();
        let mut outline_current: Option<(u8, usize, String)> = None;
        let mut heading: Option<(u8, Vec<InlineSpan>, std::ops::Range<usize>)> = None;
        let mut paragraph: Option<(Vec<InlineSpan>, std::ops::Range<usize>)> = None;
        let mut quote_depth = 0usize;
        let mut quote: Vec<InlineSpan> = Vec::new();
        let mut quote_source_range: Option<std::ops::Range<usize>> = None;
        let mut list_stack: Vec<ListLevelDraft> = Vec::new();
        let mut list_item: Option<ListItemDraft> = None;
        let mut image: Option<ImageDraft> = None;
        let mut code: Option<(Option<String>, String, std::ops::Range<usize>)> = None;
        let mut table: Option<TableDraft> = None;
        let mut inline = InlineStateDraft::default();
        let mut table_ranges = table_ranges_fn(text).into_iter();
        let mut footnote: Option<(String, Vec<InlineSpan>, std::ops::Range<usize>)> = None;

        for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
            let source_range = body_offset + range.start..body_offset + range.end;
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    let level = heading_level_to_u8(level);
                    heading = Some((level, Vec::new(), source_range.clone()));
                    outline_current = Some((level, source_range.start, String::new()));
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((level, spans, heading_range)) = heading.take() {
                        push_nonempty_block(
                            &mut blocks,
                            PreviewBlock::Heading {
                                level,
                                text: finish_rich_text(spans),
                                source_range: heading_range,
                            },
                        );
                    }
                    if let Some((level, offset, title)) = outline_current.take() {
                        headings.push(Heading {
                            level,
                            anchor: slugify(&title),
                            offset,
                            title,
                        });
                    }
                }
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    footnote = Some((label.to_string(), Vec::new(), source_range));
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    if let Some((label, spans, _)) = footnote.take() {
                        blocks.push(PreviewBlock::FootnoteDefinition {
                            label,
                            text: finish_rich_text(spans),
                            source_range,
                        });
                    }
                }
                Event::Start(Tag::Paragraph) => {
                    paragraph = Some((Vec::new(), source_range));
                }
                Event::End(TagEnd::Paragraph) => {
                    if let Some((spans, paragraph_range)) = paragraph.take() {
                        if let Some((_, footnote_spans, _)) = footnote.as_mut() {
                            if !footnote_spans.is_empty() && !spans.is_empty() {
                                append_span(
                                    footnote_spans,
                                    "\n",
                                    InlineStyle::default(),
                                    None,
                                );
                            }
                            footnote_spans.extend(spans);
                        } else if list_item.is_none() && quote_depth == 0 && table.is_none() {
                            if html_only_paragraph_source(&text[paragraph_range.clone()]) {
                                push_html_block(
                                    &mut blocks,
                                    text[paragraph_range.clone()].to_string(),
                                    paragraph_range,
                                );
                            } else {
                                push_nonempty_block(
                                    &mut blocks,
                                    PreviewBlock::Paragraph {
                                        text: finish_rich_text(spans),
                                        source_range: paragraph_range,
                                    },
                                );
                            }
                        } else if let Some(item) = list_item.as_mut() {
                            // Keep a line break between sibling paragraphs that get
                            // flattened into one list item or blockquote block.
                            append_span(&mut item.spans, "\n", InlineStyle::default(), None);
                        } else if quote_depth > 0 {
                            append_span(&mut quote, "\n", InlineStyle::default(), None);
                        }
                    }
                }
                Event::Start(Tag::BlockQuote(_)) => {
                    quote_depth += 1;
                    if quote_depth == 1 {
                        quote.clear();
                        quote_source_range = Some(source_range);
                    }
                }
                Event::End(TagEnd::BlockQuote(_)) => {
                    if quote_depth == 1 {
                        let text = finish_rich_text(std::mem::take(&mut quote));
                        if !text.is_empty() {
                            blocks.push(PreviewBlock::BlockQuote {
                                text,
                                source_range: quote_source_range.take().unwrap_or(source_range),
                            });
                        } else {
                            quote_source_range = None;
                        }
                    }
                    quote_depth = quote_depth.saturating_sub(1);
                }
                Event::Start(Tag::List(start)) => {
                    list_stack.push(ListLevelDraft {
                        ordered: start.is_some(),
                        next_index: start.unwrap_or(1),
                    });
                }
                Event::End(TagEnd::List(_)) => {
                    list_stack.pop();
                }
                Event::Start(Tag::Item) => {
                    // A new item can begin while the previous one is still
                    // open (a nested list follows the item's own text). Flush
                    // the open draft so the parent item is not lost.
                    flush_list_item(&mut blocks, list_item.take());
                    let index = list_stack.last_mut().and_then(|level| {
                        level.ordered.then(|| {
                            let index = level.next_index;
                            level.next_index = level.next_index.saturating_add(1);
                            index
                        })
                    });
                    list_item = Some(ListItemDraft {
                        level: list_stack.len().max(1),
                        ordered: list_stack
                            .last()
                            .map(|level| level.ordered)
                            .unwrap_or(false),
                        index,
                        checked: None,
                        spans: Vec::new(),
                        source_range,
                    });
                }
                Event::End(TagEnd::Item) => {
                    if let Some(item) = list_item.as_mut() {
                        item.source_range = source_range;
                    }
                    flush_list_item(&mut blocks, list_item.take());
                }
                Event::TaskListMarker(checked) => {
                    if let Some(item) = list_item.as_mut() {
                        item.checked = Some(checked);
                    }
                }
                Event::Start(Tag::Strong) => inline.bold += 1,
                Event::End(TagEnd::Strong) => inline.bold = inline.bold.saturating_sub(1),
                Event::Start(Tag::Emphasis) => inline.italic += 1,
                Event::End(TagEnd::Emphasis) => inline.italic = inline.italic.saturating_sub(1),
                Event::Start(Tag::Strikethrough) => inline.strikethrough += 1,
                Event::End(TagEnd::Strikethrough) => {
                    inline.strikethrough = inline.strikethrough.saturating_sub(1);
                }
                Event::Start(Tag::Link { dest_url, .. }) => {
                    inline.links.push(dest_url.to_string());
                }
                Event::End(TagEnd::Link) => {
                    inline.links.pop();
                }
                Event::Start(Tag::Image {
                    dest_url, title, ..
                }) => {
                    image = Some(ImageDraft {
                        alt: String::new(),
                        url: dest_url.to_string(),
                        title: (!title.is_empty()).then(|| title.to_string()),
                        source_range,
                    });
                }
                Event::End(TagEnd::Image) => {
                    if let Some(image) = image.take() {
                        blocks.push(PreviewBlock::Image {
                            alt: clean_preview_text(&image.alt),
                            url: image.url,
                            title: image.title,
                            source_range: image.source_range,
                        });
                    }
                }
                Event::Start(Tag::CodeBlock(kind)) => {
                    let language = match kind {
                        CodeBlockKind::Fenced(info) => info
                            .split_whitespace()
                            .next()
                            .filter(|language| !language.is_empty())
                            .map(ToOwned::to_owned),
                        CodeBlockKind::Indented => None,
                    };
                    code = Some((language, String::new(), source_range));
                }
                Event::End(TagEnd::CodeBlock) => {
                    if let Some((language, code, code_range)) = code.take() {
                        let code = code.trim_end_matches('\n').to_string();
                        if language
                            .as_deref()
                            .is_some_and(|language| language.eq_ignore_ascii_case("math"))
                        {
                            let authored = text
                                .get(code_range.clone())
                                .map_or_else(|| code.clone(), str::to_string);
                            blocks.push(PreviewBlock::MathBlock {
                                error: validate_latex(&code).err(),
                                latex: code,
                                authored,
                                delimiter: MathDelimiter::Fenced,
                                source_range: code_range,
                            });
                        } else {
                            blocks.push(PreviewBlock::CodeBlock {
                                language,
                                code,
                                source_range: code_range,
                            });
                        }
                    }
                }
                Event::Rule => blocks.push(PreviewBlock::Rule { source_range }),
                Event::Start(Tag::Table(alignments)) => {
                    let draft = TableDraft {
                        alignments: alignments
                            .iter()
                            .map(|alignment| match alignment {
                                Alignment::Left => TableAlignment::Left,
                                Alignment::Center => TableAlignment::Center,
                                Alignment::Right => TableAlignment::Right,
                                Alignment::None => TableAlignment::Default,
                            })
                            .collect(),
                        ..TableDraft::default()
                    };
                    table = Some(draft);
                }
                Event::End(TagEnd::Table) => {
                    if let Some(table) = table.take()
                        && !table.rows.is_empty()
                    {
                        blocks.push(PreviewBlock::Table {
                            rows: table.rows,
                            alignments: table.alignments,
                            source_range: table_ranges.next().unwrap_or(0..0),
                        });
                    }
                }
                Event::Start(Tag::TableHead) => {
                    if let Some(table) = table.as_mut() {
                        table.current_row = Some(Vec::new());
                    }
                }
                Event::End(TagEnd::TableHead) => {
                    if let Some(table) = table.as_mut()
                        && let Some(row) = table.current_row.take()
                    {
                        table.rows.push(row);
                    }
                }
                Event::Start(Tag::TableRow) => {
                    if let Some(table) = table.as_mut() {
                        table.current_row = Some(Vec::new());
                    }
                }
                Event::End(TagEnd::TableRow) => {
                    if let Some(table) = table.as_mut()
                        && let Some(row) = table.current_row.take()
                    {
                        table.rows.push(row);
                    }
                }
                Event::Start(Tag::TableCell) => {
                    if let Some(table) = table.as_mut() {
                        table.current_cell.clear();
                    }
                }
                Event::End(TagEnd::TableCell) => {
                    if let Some(table) = table.as_mut()
                        && let Some(row) = table.current_row.as_mut()
                    {
                        row.push(finish_rich_text(std::mem::take(&mut table.current_cell)));
                    }
                }
                Event::Text(text) => {
                    push_preview_rich(
                        &mut heading,
                        &mut paragraph,
                        &mut quote,
                        quote_depth,
                        &mut list_item,
                        &mut image,
                        &mut code,
                        &mut table,
                        &text,
                        inline.style(),
                        inline.link(),
                        true,
                    );
                    if let Some((_, _, title)) = outline_current.as_mut() {
                        title.push_str(&text);
                    }
                }
                Event::Code(text) => {
                    let mut style = inline.style();
                    style.code = true;
                    if let Some((_, _, title)) = outline_current.as_mut() {
                        title.push_str(&text);
                    }
                    push_preview_rich(
                        &mut heading,
                        &mut paragraph,
                        &mut quote,
                        quote_depth,
                        &mut list_item,
                        &mut image,
                        &mut code,
                        &mut table,
                        &text,
                        style,
                        inline.link(),
                        false,
                    );
                }
                Event::Html(text) | Event::InlineHtml(text) => {
                    let standalone_html = heading.is_none()
                        && paragraph.is_none()
                        && quote_depth == 0
                        && list_item.is_none()
                        && image.is_none()
                        && code.is_none()
                        && table.is_none();
                    if standalone_html {
                        push_html_block(&mut blocks, text.to_string(), source_range);
                    } else {
                        let text = html_preview_plain_text(&text);
                        if !text.is_empty() {
                            push_preview_rich(
                                &mut heading,
                                &mut paragraph,
                                &mut quote,
                                quote_depth,
                                &mut list_item,
                                &mut image,
                                &mut code,
                                &mut table,
                                &text,
                                inline.style(),
                                inline.link(),
                                false,
                            );
                        }
                    }
                }
                Event::FootnoteReference(text) => {
                    let mut style = inline.style();
                    style.superscript = true;
                    push_preview_rich(
                        &mut heading,
                        &mut paragraph,
                        &mut quote,
                        quote_depth,
                        &mut list_item,
                        &mut image,
                        &mut code,
                        &mut table,
                        &text,
                        style,
                        inline.link(),
                        false,
                    );
                }
                Event::InlineMath(latex) => {
                    let authored = text
                        .get(source_range.clone())
                        .map_or_else(|| format!("${latex}$"), str::to_string);
                    push_preview_math(
                        &mut heading,
                        &mut paragraph,
                        &mut quote,
                        quote_depth,
                        &mut list_item,
                        &mut image,
                        &mut code,
                        &mut table,
                        MathSource {
                            latex: latex.to_string(),
                            authored,
                            style: MathLayoutStyle::Text,
                            delimiter: MathDelimiter::InlineDollar,
                            source_range,
                        },
                        inline.style(),
                        inline.link(),
                    );
                }
                Event::DisplayMath(latex) => {
                    let standalone = heading.is_none()
                        && list_item.is_none()
                        && image.is_none()
                        && code.is_none()
                        && table.is_none()
                        && quote_depth == 0
                        && paragraph.as_ref().is_some_and(|(paragraph, _)| {
                            paragraph.iter().all(|span| span.text.trim().is_empty())
                        });
                    if standalone {
                        paragraph.take();
                        let authored = text
                            .get(source_range.clone())
                            .map_or_else(|| format!("$${latex}$$"), str::to_string);
                        blocks.push(PreviewBlock::MathBlock {
                            latex: latex.to_string(),
                            authored,
                            delimiter: MathDelimiter::DisplayDollar,
                            error: validate_latex(&latex).err(),
                            source_range,
                        });
                    } else {
                        let authored = text
                            .get(source_range.clone())
                            .map_or_else(|| format!("$${latex}$$"), str::to_string);
                        push_preview_math(
                            &mut heading,
                            &mut paragraph,
                            &mut quote,
                            quote_depth,
                            &mut list_item,
                            &mut image,
                            &mut code,
                            &mut table,
                            MathSource {
                                latex: latex.to_string(),
                                authored,
                                style: MathLayoutStyle::Display,
                                delimiter: MathDelimiter::DisplayDollar,
                                source_range,
                            },
                            inline.style(),
                            inline.link(),
                        );
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    push_preview_rich(
                        &mut heading,
                        &mut paragraph,
                        &mut quote,
                        quote_depth,
                        &mut list_item,
                        &mut image,
                        &mut code,
                        &mut table,
                        "\n",
                        InlineStyle::default(),
                        None,
                        false,
                    );
                }
                _ => {}
            }
        }

        (blocks, headings)
    }

    pub fn footnotes(&self) -> Vec<Footnote> {
        let mut references: HashMap<String, usize> = HashMap::new();
        let mut definitions = Vec::new();
        let mut current_definition: Option<(String, String)> = None;

        for event in Parser::new_ext(self.body_text(), markdown_options()) {
            match event {
                Event::FootnoteReference(label) => {
                    *references.entry(label.to_string()).or_insert(0) += 1;
                }
                Event::Start(Tag::FootnoteDefinition(label)) => {
                    current_definition = Some((label.to_string(), String::new()));
                }
                Event::End(TagEnd::FootnoteDefinition) => {
                    if let Some((label, text)) = current_definition.take() {
                        definitions.push((label, clean_preview_text(&text)));
                    }
                }
                Event::Text(text)
                | Event::Code(text)
                | Event::Html(text)
                | Event::InlineHtml(text) => {
                    if let Some((_, definition)) = current_definition.as_mut() {
                        definition.push_str(&text);
                    }
                }
                Event::InlineMath(text) | Event::DisplayMath(text) => {
                    if let Some((_, definition)) = current_definition.as_mut() {
                        definition.push('$');
                        definition.push_str(&text);
                        definition.push('$');
                    }
                }
                Event::SoftBreak | Event::HardBreak => {
                    if let Some((_, definition)) = current_definition.as_mut() {
                        definition.push('\n');
                    }
                }
                _ => {}
            }
        }

        definitions
            .into_iter()
            .map(|(label, text)| Footnote {
                references: references.get(&label).copied().unwrap_or(0),
                label,
                text,
            })
            .collect()
    }

    pub fn math_expressions(&self) -> Vec<MathExpression> {
        Parser::new_ext(self.body_text(), markdown_options())
            .filter_map(|event| match event {
                Event::InlineMath(latex) => {
                    let latex = latex.trim().to_string();
                    Some(MathExpression {
                        error: validate_latex(&latex).err(),
                        latex,
                        display: false,
                    })
                }
                Event::DisplayMath(latex) => {
                    let latex = latex.trim().to_string();
                    Some(MathExpression {
                        error: validate_latex(&latex).err(),
                        latex,
                        display: true,
                    })
                }
                _ => None,
            })
            .collect()
    }

    pub fn outline(&self) -> Vec<Heading> {
        if let Some(cached) = self.cached_outline.borrow().as_ref()
            && cached.version == self.text_version
        {
            return cached.value.clone();
        }

        // Reached only when the outline is needed but the (much heavier) preview
        // derive did not run this version — e.g. Edit mode with the outline
        // panel open. A heading-only pass is far cheaper than the full derive.
        let headings = self.compute_outline_only();
        let version = self.text_version;
        *self.cached_outline.borrow_mut() = Some(Cached {
            version,
            value: headings.clone(),
        });
        headings
    }

    /// Heading-only pulldown pass. Kept byte-identical to the outline produced
    /// as a side effect of [`Self::derive_preview_and_outline`]; the two paths are asserted
    /// equal by a unit test so either can populate `cached_outline`.
    fn compute_outline_only(&self) -> Vec<Heading> {
        let (body, body_offset) = self.body_text_and_offset();
        let mut headings = Vec::new();
        let mut current: Option<(u8, usize, String)> = None;

        for (event, range) in Parser::new_ext(body, markdown_options()).into_offset_iter() {
            match event {
                Event::Start(Tag::Heading { level, .. }) => {
                    current = Some((
                        heading_level_to_u8(level),
                        body_offset + range.start,
                        String::new(),
                    ));
                }
                Event::Text(text) | Event::Code(text) => {
                    if let Some((_, _, title)) = current.as_mut() {
                        title.push_str(&text);
                    }
                }
                Event::End(TagEnd::Heading(_)) => {
                    if let Some((level, offset, title)) = current.take() {
                        headings.push(Heading {
                            level,
                            anchor: slugify(&title),
                            offset,
                            title,
                        });
                    }
                }
                _ => {}
            }
        }
        headings
    }

    pub fn current_heading_index(&self, offset: usize) -> Option<usize> {
        self.outline()
            .iter()
            .enumerate()
            .take_while(|(_, heading)| heading.offset <= offset)
            .map(|(index, _)| index)
            .last()
    }

    pub fn search(&self, needle: &str) -> Vec<SearchMatch> {
        self.find_matches(&SearchOptions::literal(needle))
            .unwrap_or_default()
            .into_iter()
            .map(|found| SearchMatch {
                line: found.line,
                column: found.column,
                snippet: found.snippet,
            })
            .collect()
    }

    pub fn find_matches(
        &self,
        options: &SearchOptions,
    ) -> Result<Vec<SearchMatchRange>, SearchError> {
        if options.query.is_empty() {
            return Ok(Vec::new());
        }

        let pattern = if options.regex {
            options.query.clone()
        } else {
            regex::escape(&options.query)
        };
        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(!options.case_sensitive)
            .build()
            .map_err(|err| SearchError {
                message: err.to_string(),
            })?;

        Ok(regex
            .find_iter(&self.text)
            .map(|found| self.search_match_for_range(found.start()..found.end()))
            .collect())
    }

    pub fn find_next_match(
        &self,
        options: &SearchOptions,
        after: usize,
        wrap: bool,
    ) -> Result<Option<SearchMatchRange>, SearchError> {
        let matches = self.find_matches(options)?;
        let next = matches
            .iter()
            .find(|found| found.range.start >= after)
            .cloned()
            .or_else(|| wrap.then(|| matches.first().cloned()).flatten());
        Ok(next)
    }

    pub fn find_previous_match(
        &self,
        options: &SearchOptions,
        before: usize,
        wrap: bool,
    ) -> Result<Option<SearchMatchRange>, SearchError> {
        let matches = self.find_matches(options)?;
        let previous = matches
            .iter()
            .rev()
            .find(|found| found.range.end <= before)
            .cloned()
            .or_else(|| wrap.then(|| matches.last().cloned()).flatten());
        Ok(previous)
    }

    pub fn replace_current_match(
        &mut self,
        range: Range<usize>,
        options: &SearchOptions,
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        let matches = self.find_matches(options)?;
        if !matches.iter().any(|found| found.range == range) {
            return Ok(ReplaceResult {
                replacements: 0,
                selected_range: None,
            });
        }

        let replacement_text = if options.regex {
            let regex = RegexBuilder::new(&options.query)
                .case_insensitive(!options.case_sensitive)
                .build()
                .map_err(|err| SearchError {
                    message: err.to_string(),
                })?;
            regex
                .replace(&self.text[range.clone()], replacement)
                .to_string()
        } else {
            replacement.to_string()
        };
        let selected_range = range.start..range.start + replacement_text.len();
        self.replace_source_range(range, &replacement_text);

        Ok(ReplaceResult {
            replacements: 1,
            selected_range: Some(selected_range),
        })
    }

    pub fn replace_all_matches(
        &mut self,
        options: &SearchOptions,
        replacement: &str,
    ) -> Result<ReplaceResult, SearchError> {
        if options.query.is_empty() {
            return Ok(ReplaceResult {
                replacements: 0,
                selected_range: None,
            });
        }

        let pattern = if options.regex {
            options.query.clone()
        } else {
            regex::escape(&options.query)
        };
        let regex = RegexBuilder::new(&pattern)
            .case_insensitive(!options.case_sensitive)
            .build()
            .map_err(|err| SearchError {
                message: err.to_string(),
            })?;
        let replacements = regex.find_iter(&self.text).count();
        if replacements == 0 {
            return Ok(ReplaceResult {
                replacements: 0,
                selected_range: None,
            });
        }

        let text = if options.regex {
            regex.replace_all(&self.text, replacement).to_string()
        } else {
            regex
                .replace_all(&self.text, regex::NoExpand(replacement))
                .to_string()
        };
        self.set_text(text);

        Ok(ReplaceResult {
            replacements,
            selected_range: None,
        })
    }

    fn search_match_for_range(&self, range: Range<usize>) -> SearchMatchRange {
        let (line, column) = line_column_at(&self.text, range.start);
        SearchMatchRange {
            range,
            line,
            column,
            snippet: line_snippet_at(&self.text, line),
        }
    }

    pub fn autosave(&mut self, recovery_dir: impl AsRef<Path>) -> io::Result<AutosaveOutcome> {
        if !self.dirty {
            return Ok(AutosaveOutcome::NoChanges);
        }

        if let Some(path) = self.path.clone() {
            self.save()?;
            Ok(AutosaveOutcome::SavedFile(path))
        } else {
            let path = self.save_recovery_copy(recovery_dir)?;
            Ok(AutosaveOutcome::SavedRecovery(path))
        }
    }

    pub fn save_recovery_copy(&self, dir: impl AsRef<Path>) -> io::Result<PathBuf> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir)?;
        let path = recovery_file_path(dir, self.path.as_deref());
        let original_path = self
            .path
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_default();
        let payload = format!(
            "markion-recovery-v1\npath:{original_path}\n---\n{}",
            self.text
        );
        fs::write(&path, payload)?;
        Ok(path)
    }

    pub fn stats(&self) -> DocumentStats {
        if let Some(cached) = self.cached_stats.borrow().as_ref()
            && cached.version == self.text_version
        {
            return cached.value.clone();
        }

        let stats = DocumentStats {
            bytes: self.text.len(),
            chars: self.text.chars().count(),
            words: self.text.split_whitespace().count(),
            lines: self.text.lines().count().max(1),
            headings: self.outline().len(),
        };
        let version = self.text_version;
        *self.cached_stats.borrow_mut() = Some(Cached {
            version,
            value: stats.clone(),
        });
        stats
    }
}

fn sanitize_visual_field_replacement(
    source: &str,
    kind: VisualEditorFieldKind,
    field_range: &Range<usize>,
    replacement: &str,
) -> String {
    match kind {
        VisualEditorFieldKind::CodePayload | VisualEditorFieldKind::MathPayload => {
            replacement.to_string()
        }
        VisualEditorFieldKind::ImageAlt => {
            escape_unescaped_visual_terminators(&replacement.replace(['\r', '\n'], " "), |ch| {
                ch == ']'
            })
        }
        VisualEditorFieldKind::ImageDestination => {
            let normalized = replacement
                .chars()
                .filter_map(|ch| match ch {
                    '\r' | '\n' => None,
                    ch if ch.is_whitespace() => Some("%20".to_string()),
                    ch => Some(ch.to_string()),
                })
                .collect::<String>();
            escape_unescaped_visual_terminators(&normalized, |ch| ch == ')')
        }
        VisualEditorFieldKind::ImageTitle => {
            let delimiter = field_range
                .start
                .checked_sub(1)
                .and_then(|offset| source.as_bytes().get(offset))
                .copied()
                .unwrap_or(b'"') as char;
            escape_unescaped_visual_terminators(&replacement.replace(['\r', '\n'], " "), |ch| {
                ch == delimiter
            })
        }
        VisualEditorFieldKind::TableCell { .. } => {
            escape_unescaped_visual_terminators(&replacement.replace(['\r', '\n'], " "), |ch| {
                ch == '|'
            })
        }
    }
}

fn escape_unescaped_visual_terminators(
    input: &str,
    mut is_terminator: impl FnMut(char) -> bool,
) -> String {
    let mut output = String::with_capacity(input.len());
    let mut preceding_backslashes = 0usize;
    for ch in input.chars() {
        if ch == '\\' {
            output.push(ch);
            preceding_backslashes += 1;
            continue;
        }
        if is_terminator(ch) && preceding_backslashes.is_multiple_of(2) {
            output.push('\\');
        }
        output.push(ch);
        preceding_backslashes = 0;
    }
    output
}

pub fn default_recovery_dir() -> PathBuf {
    crate::paths::default_recovery_dir()
}

pub fn default_config_dir() -> PathBuf {
    crate::paths::default_config_dir()
}

pub fn default_preferences_path() -> PathBuf {
    crate::paths::default_preferences_path()
}

pub fn default_themes_dir() -> PathBuf {
    crate::paths::default_themes_dir()
}

fn escape_html_text(text: &str) -> String {
    crate::escape::escape_html_text(text)
}

fn escape_html_attribute(text: &str) -> String {
    crate::escape::escape_html_attribute(text)
}

fn line_column_at(text: &str, offset: usize) -> (usize, usize) {
    crate::text_util::line_column_at(text, offset)
}

fn line_snippet_at(text: &str, line_number: usize) -> String {
    crate::text_util::line_snippet_at(text, line_number)
}

fn clamp_to_char_boundary(text: &str, index: usize) -> usize {
    crate::text_util::clamp_to_char_boundary(text, index)
}

fn clamp_range_to_char_boundaries(
    text: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    crate::text_util::clamp_range_to_char_boundaries(text, range)
}

fn offset_with_delta(offset: usize, delta: isize) -> usize {
    crate::text_util::offset_with_delta(offset, delta)
}

fn push_html_block(blocks: &mut Vec<PreviewBlock>, html: String, source_range: Range<usize>) {
    if html.is_empty() {
        return;
    }
    if let Some(PreviewBlock::Html {
        html: existing_html,
        source_range: existing_range,
    }) = blocks.last_mut()
        && existing_range.end == source_range.start
    {
        existing_html.push_str(&html);
        existing_range.end = source_range.end;
        return;
    }

    blocks.push(PreviewBlock::Html { html, source_range });
}

fn html_only_paragraph_source(source: &str) -> bool {
    let mut index = 0;
    let mut depth = 0usize;
    let mut saw_tag = false;

    while index < source.len() {
        if source[index..].starts_with('<') {
            let Some(tag_end) = html_tag_end(source, index) else {
                return false;
            };
            let tag = &source[index..tag_end];
            let Some(parsed) = HtmlOnlyTag::parse(tag) else {
                return false;
            };
            saw_tag = true;
            if parsed.closing {
                depth = depth.saturating_sub(1);
            } else if !parsed.self_closing {
                depth += 1;
            }
            index = tag_end;
            continue;
        }

        let next_tag = source[index..]
            .find('<')
            .map_or(source.len(), |relative| index + relative);
        if depth == 0 && !source[index..next_tag].trim().is_empty() {
            return false;
        }
        index = next_tag;
    }

    saw_tag
}

fn html_tag_end(source: &str, start: usize) -> Option<usize> {
    let mut quote = None;
    for (relative, ch) in source[start..].char_indices() {
        match (quote, ch) {
            (Some(active), current) if current == active => quote = None,
            (None, '"' | '\'') => quote = Some(ch),
            (None, '>') => return Some(start + relative + ch.len_utf8()),
            _ => {}
        }
    }
    None
}

struct HtmlOnlyTag {
    closing: bool,
    self_closing: bool,
}

impl HtmlOnlyTag {
    fn parse(tag: &str) -> Option<Self> {
        let inner = tag.strip_prefix('<')?.strip_suffix('>')?.trim();
        if inner.starts_with('!') || inner.starts_with('?') {
            return Some(Self {
                closing: false,
                self_closing: true,
            });
        }

        let closing = inner.starts_with('/');
        let body = inner.trim_start_matches('/').trim_start();
        let name = body
            .split(|ch: char| ch.is_whitespace() || ch == '/')
            .next()?
            .to_ascii_lowercase();
        if name.is_empty()
            || !name
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_alphabetic())
        {
            return None;
        }

        Some(Self {
            closing,
            self_closing: body.trim_end().ends_with('/') || html_void_tag(&name),
        })
    }
}

fn html_void_tag(name: &str) -> bool {
    matches!(
        name,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

pub fn title_from_path(path: Option<&Path>) -> CowStr<'static> {
    path.and_then(Path::file_name)
        .and_then(|name| name.to_str())
        .unwrap_or("Untitled.md")
        .to_string()
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_common_markdown_to_html() {
        let doc = MarkdownDocument::from_text(
            "# Hello\n\n- [x] shipped\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        );
        let html = doc.render_html_fragment();

        assert!(html.contains("<h1>Hello</h1>"));
        assert!(html.contains("checkbox"));
        assert!(html.contains("<table>"));
    }

    #[test]
    fn preview_keeps_raw_html_blocks_for_rendering() {
        let doc = MarkdownDocument::from_text(
            r#"<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">
</p>

# Markion"#,
        );
        let blocks = doc.preview_blocks();

        assert!(
            matches!(
                blocks.first(),
                Some(PreviewBlock::Html { html, .. })
                    if html.contains("assets/markion-logo.svg")
            ),
            "raw HTML blocks should reach the rendered preview instead of disappearing"
        );
        assert!(matches!(blocks.get(1), Some(PreviewBlock::Heading { .. })));
    }

    #[test]
    fn html_preview_parts_render_common_readme_html() {
        let html = r#"<p align="center">
  <img src="assets/markion-logo.svg" alt="Markion logo" width="128" height="128">
</p>

<p align="center">
  <strong>English</strong> · <a href="README.zh-CN.md">简体中文</a>
</p>"#;
        let parts = html_preview_parts(html);

        assert!(matches!(
            &parts[0],
            HtmlPreviewPart::Image { url, alt, centered, .. }
                if url == "assets/markion-logo.svg" && alt == "Markion logo" && *centered
        ));
        let HtmlPreviewPart::Text { text, centered } = &parts[1] else {
            panic!("expected text part");
        };
        assert!(*centered);
        assert_eq!(text.text, "English · 简体中文");
        assert!(
            text.spans
                .iter()
                .any(|span| span.text == "English" && span.style.bold)
        );
        assert!(
            text.spans
                .iter()
                .any(|span| span.text == "简体中文"
                    && span.link.as_deref() == Some("README.zh-CN.md"))
        );
    }

    #[test]
    fn visual_edit_keeps_html_as_source_island() {
        let doc = MarkdownDocument::from_text("<p><strong>HTML</strong></p>\n\nText");
        let blocks = doc.visual_blocks();

        assert!(matches!(
            blocks.first(),
            Some(VisualBlock {
                source_island: Some(VisualSourceIslandKind::Html),
                ..
            })
        ));
    }

    #[test]
    fn extracts_outline_with_stable_anchors() {
        let doc = MarkdownDocument::from_text("# One\n\n## Two Things!\n\n### Rust & GPUI");
        let outline = doc.outline();

        assert_eq!(outline[0].anchor, "one");
        assert_eq!(outline[1].anchor, "two-things");
        assert_eq!(outline[2].level, 3);
    }

    #[test]
    fn edits_at_utf8_boundaries() {
        let mut doc = MarkdownDocument::from_text("a文c");
        doc.insert(2, "字");
        doc.replace_range(1..4, "b");

        assert_eq!(doc.text(), "ab文c");
        assert!(doc.is_dirty());
    }

    #[test]
    fn search_is_case_insensitive_and_line_based() {
        let doc = MarkdownDocument::from_text("Alpha\nbeta alpha");
        assert_eq!(
            doc.search("ALPHA"),
            vec![
                SearchMatch {
                    line: 1,
                    column: 1,
                    snippet: "Alpha".into()
                },
                SearchMatch {
                    line: 2,
                    column: 6,
                    snippet: "beta alpha".into()
                }
            ]
        );
    }

    #[test]
    fn saves_and_exports_all_formats() {
        let dir = tempfile::tempdir().unwrap();
        let markdown = dir.path().join("note.md");
        let html = dir.path().join("note.html");
        let pdf = dir.path().join("note.pdf");
        let docx = dir.path().join("note.docx");
        let png = dir.path().join("note.png");
        let jpeg = dir.path().join("note.jpg");
        let mut doc = MarkdownDocument::from_text("# Export\n\nBody");

        doc.save_as(&markdown).unwrap();
        doc.export_to(&html, ExportFormat::Html).unwrap();
        doc.export_to(&pdf, ExportFormat::Pdf).unwrap();
        doc.export_to(&docx, ExportFormat::Docx).unwrap();
        doc.export_to(&png, ExportFormat::Png).unwrap();
        doc.export_to(&jpeg, ExportFormat::Jpeg).unwrap();

        assert_eq!(fs::read_to_string(markdown).unwrap(), "# Export\n\nBody");
        assert!(
            fs::read_to_string(html)
                .unwrap()
                .contains("<h1>Export</h1>")
        );
        // Version-agnostic prefix: the built-in fallback writes PDF-1.4, while
        // the pandoc engine path (when pandoc + xelatex are installed) emits a
        // newer PDF version.
        assert!(fs::read(pdf).unwrap().starts_with(b"%PDF-"));
        assert!(fs::read(docx).unwrap().starts_with(b"PK\x03\x04"));
        assert!(fs::read(png).unwrap().starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(fs::read(jpeg).unwrap().starts_with(&[0xff, 0xd8, 0xff]));
    }

    #[test]
    fn docx_export_contains_metadata_blocks_and_tables() {
        let dir = tempfile::tempdir().unwrap();
        let docx = dir.path().join("paper.docx");
        let doc = MarkdownDocument::from_text(
            "---\ntitle: Research Note\nauthor: Ada\ndate: 2026-06-30\n---\n# Findings\n\nBody & details\n\n```rust\nfn main() {}\n```\n\n$$\na^2 + b^2\n$$\n\n| Name | Score |\n|---|---|\n| Ada | 10 |",
        );

        // Exercise the built-in fallback writer directly: `export_to` prefers
        // the pandoc engine when pandoc is installed, whose deflate-compressed
        // package would hide these XML markers from raw-byte inspection.
        export::write_docx(&docx, &doc).unwrap();
        let bytes = fs::read(docx).unwrap();
        let package = String::from_utf8_lossy(&bytes);

        assert!(bytes.starts_with(b"PK\x03\x04"));
        assert!(package.contains("[Content_Types].xml"));
        assert!(package.contains("word/document.xml"));
        assert!(package.contains("<dc:title>Research Note</dc:title>"));
        assert!(package.contains("<dc:creator>Ada</dc:creator>"));
        assert!(package.contains("<w:pStyle w:val=\"Heading1\"/>"));
        assert!(package.contains("Body &amp; details"));
        assert!(package.contains("fn main() {}"));
        assert!(package.contains("Math: a^2 + b^2"));
        assert!(package.contains("<w:tbl>"));
        assert!(package.contains("<w:t xml:space=\"preserve\">Ada</w:t>"));
    }

    #[test]
    fn parses_markdown_preview_blocks() {
        let doc = MarkdownDocument::from_text(
            "# Title\n\nParagraph with **bold** text.\n\n- [x] Done\n- [ ] Next\n\n> Quote\n\n```rust\nfn main() {}\n```\n\n---\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        );
        let table_start = doc.text().find("| A").unwrap();
        let table_range = table_start..doc.text().len();

        let blocks = doc.preview_blocks();
        assert_eq!(blocks.len(), 8);
        assert!(matches!(
            &blocks[0],
            PreviewBlock::Heading {
                level: 1,
                text,
                ..
            } if text.text == "Title"
        ));
        let PreviewBlock::Paragraph { text: para, .. } = &blocks[1] else {
            panic!("expected paragraph, got {:?}", blocks[1]);
        };
        assert_eq!(
            para,
            &RichText {
                text: "Paragraph with bold text.".into(),
                spans: vec![
                    InlineSpan {
                        text: "Paragraph with ".into(),
                        ..InlineSpan::default()
                    },
                    InlineSpan {
                        text: "bold".into(),
                        style: InlineStyle {
                            bold: true,
                            ..InlineStyle::default()
                        },
                        link: None,
                        math: None,
                    },
                    InlineSpan {
                        text: " text.".into(),
                        ..InlineSpan::default()
                    },
                ],
            }
        );
        assert!(matches!(
            &blocks[2],
            PreviewBlock::ListItem {
                level: 1,
                ordered: false,
                index: None,
                checked: Some(true),
                text,
                ..
            } if text.text == "Done"
        ));
        assert!(matches!(
            &blocks[3],
            PreviewBlock::ListItem {
                level: 1,
                ordered: false,
                index: None,
                checked: Some(false),
                text,
                ..
            } if text.text == "Next"
        ));
        assert!(matches!(
            &blocks[4],
            PreviewBlock::BlockQuote { text, .. } if text.text == "Quote"
        ));
        assert!(matches!(
            &blocks[5],
            PreviewBlock::CodeBlock {
                language: Some(lang),
                code,
                ..
            } if lang == "rust" && code == "fn main() {}"
        ));
        assert!(matches!(&blocks[6], PreviewBlock::Rule { .. }));
        assert_eq!(
            &blocks[7],
            &PreviewBlock::Table {
                rows: vec![vec!["A".into(), "B".into()], vec!["1".into(), "2".into()]],
                alignments: vec![TableAlignment::Default, TableAlignment::Default],
                source_range: table_range,
            }
        );
    }

    #[test]
    fn table_cells_parse_inline_formatting() {
        let doc = MarkdownDocument::from_text(
            "| Syntax | Example |\n|---|---|\n| `**bold**` | **bold** |\n| `[text](url)` | [link](https://github.com/willmove/markion) |",
        );
        let blocks = doc.preview_blocks();
        assert_eq!(blocks.len(), 1);
        let PreviewBlock::Table { rows, .. } = &blocks[0] else {
            panic!("expected table, got {:?}", blocks[0]);
        };
        // Header row: plain text cells.
        assert_eq!(rows[0][0].text, "Syntax");
        assert_eq!(rows[0][1].text, "Example");
        // Row 1, col 0: inline code span (the backtick fence).
        assert_eq!(rows[1][0].text, "**bold**");
        assert!(rows[1][0].spans.iter().any(|span| span.style.code));
        // Row 1, col 1: bold.
        assert_eq!(rows[1][1].text, "bold");
        assert!(rows[1][1].spans.iter().any(|span| span.style.bold));
        // Row 2, col 0: plain text of the link syntax example.
        assert_eq!(rows[2][0].text, "[text](url)");
        // Row 2, col 1: rendered link.
        assert_eq!(rows[2][1].text, "link");
        assert!(rows[2][1]
            .spans
            .iter()
            .any(|span| span.link.as_deref() == Some("https://github.com/willmove/markion")));
    }

    #[test]
    fn markdown_newline_inserts_real_line_break() {
        let mut doc = MarkdownDocument::from_text("AlphaBeta");
        let cursor = doc.insert_markdown_newline(5);

        assert_eq!(doc.text(), "Alpha\nBeta");
        assert_eq!(cursor, 6);
        assert!(doc.is_dirty());
    }

    #[test]
    fn markdown_newline_continues_lists() {
        let mut unordered = MarkdownDocument::from_text("- item");
        let cursor = unordered.insert_markdown_newline(unordered.text().len());
        assert_eq!(unordered.text(), "- item\n- ");
        assert_eq!(cursor, unordered.text().len());

        let mut ordered = MarkdownDocument::from_text("9. item");
        ordered.insert_markdown_newline(ordered.text().len());
        assert_eq!(ordered.text(), "9. item\n10. ");

        let mut task = MarkdownDocument::from_text("- [x] done");
        task.insert_markdown_newline(task.text().len());
        assert_eq!(task.text(), "- [x] done\n- [ ] ");
    }

    #[test]
    fn markdown_newline_continues_blockquotes() {
        let mut doc = MarkdownDocument::from_text("> quoted");
        doc.insert_markdown_newline(doc.text().len());

        assert_eq!(doc.text(), "> quoted\n> ");
    }

    #[test]
    fn markdown_newline_exits_empty_list_marker() {
        let mut doc = MarkdownDocument::from_text("- ");
        let cursor = doc.insert_markdown_newline(doc.text().len());

        assert_eq!(doc.text(), "");
        assert_eq!(cursor, 0);
    }

    #[test]
    fn visual_enter_edits_cover_supported_block_transitions() {
        let cases = [
            ("# 标题", "# 标题\n"),
            ("> 引用", "> 引用\n> "),
            ("- item", "- item\n- "),
            ("3. item", "3. item\n4. "),
            ("- [x] done", "- [x] done\n- [ ] "),
        ];
        for (source, expected) in cases {
            let mut doc = MarkdownDocument::from_text(source);
            let version = doc.version();
            let edit = doc
                .visual_enter_edit(source.len())
                .unwrap_or_else(|| panic!("missing structural Enter for {source:?}"));
            doc.replace_range(edit.range, &edit.replacement);
            assert_eq!(doc.text(), expected, "source: {source:?}");
            assert_eq!(edit.selection_after, expected.len()..expected.len());
            assert!(doc.version() > version);
            assert!(doc.is_dirty());
        }

        for source in ["> ", "- ", "- [ ] ", "1. "] {
            let mut doc = MarkdownDocument::from_text(source);
            let edit = doc
                .visual_enter_edit(source.len())
                .unwrap_or_else(|| panic!("missing empty-structure exit for {source:?}"));
            doc.replace_range(edit.range, &edit.replacement);
            assert_eq!(doc.text(), "", "source: {source:?}");
            assert_eq!(edit.selection_after, 0..0);
        }

        let mut middle = MarkdownDocument::from_text("before\n> \nafter");
        let cursor = middle.text().find("> ").unwrap() + 2;
        let edit = middle
            .visual_enter_edit(cursor)
            .expect("empty blockquote in the middle of a document should exit");
        middle.replace_range(edit.range, &edit.replacement);
        assert_eq!(middle.text(), "before\n\nafter");
        assert_eq!(edit.selection_after, 7..7);

        let code = MarkdownDocument::from_text("```text\n> \n- \n1. \n- [ ] \n```\n");
        for source in ["> ", "- ", "1. ", "- [ ] "] {
            let cursor = code.text().find(source).unwrap() + source.len();
            assert!(
                code.visual_enter_edit(cursor).is_none(),
                "source-island prefix must remain literal: {source:?}"
            );
        }
    }

    #[test]
    fn visual_backspace_demotes_top_level_blocks_and_outdents_nested_lists() {
        let cases = [
            ("# 标题", 2, "标题"),
            ("> quote", 2, "quote"),
            ("- item", 2, "item"),
            ("4. item", 3, "item"),
            ("- [x] done", 6, "done"),
        ];
        for (source, cursor, expected) in cases {
            let mut doc = MarkdownDocument::from_text(source);
            let edit = doc
                .visual_backspace_edit(cursor)
                .unwrap_or_else(|| panic!("missing structural Backspace for {source:?}"));
            doc.replace_range(edit.range, &edit.replacement);
            assert_eq!(doc.text(), expected, "source: {source:?}");
            assert_eq!(edit.selection_after, 0..0);
        }

        let mut nested = MarkdownDocument::from_text("  - 项目");
        let edit = nested.visual_backspace_edit(4).unwrap();
        nested.replace_range(edit.range, &edit.replacement);
        assert_eq!(nested.text(), "- 项目");
        assert_eq!(edit.selection_after, 2..2);
        assert!(nested.visual_backspace_edit(nested.text().len()).is_none());
    }

    #[test]
    fn visual_structural_helpers_are_non_mutating_until_applied() {
        let doc = MarkdownDocument::from_text("- item");
        let version = doc.version();
        let blocks = doc.visual_blocks_shared();
        assert!(doc.visual_enter_edit(doc.text().len()).is_some());
        assert!(doc.visual_backspace_edit(2).is_some());
        assert_eq!(doc.version(), version);
        assert!(std::sync::Arc::ptr_eq(&blocks, &doc.visual_blocks_shared()));

        let code = MarkdownDocument::from_text("```text\n- item\n> quote\n```\n");
        let list_cursor = code.text().find("item").unwrap();
        let quote_cursor = code.text().find("quote").unwrap();
        assert!(code.visual_enter_edit(list_cursor).is_none());
        assert!(code.visual_backspace_edit(list_cursor).is_none());
        assert!(code.visual_enter_edit(quote_cursor).is_none());
        assert!(code.visual_backspace_edit(quote_cursor).is_none());
    }

    #[test]
    fn preview_marks_ordered_list_items() {
        let doc = MarkdownDocument::from_text("1. First\n2. Second");

        let blocks = doc.preview_blocks();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            &blocks[0],
            PreviewBlock::ListItem {
                level: 1,
                ordered: true,
                index: Some(1),
                checked: None,
                text,
                ..
            } if text.text == "First"
        ));
        assert!(matches!(
            &blocks[1],
            PreviewBlock::ListItem {
                level: 1,
                ordered: true,
                index: Some(2),
                checked: None,
                text,
                ..
            } if text.text == "Second"
        ));
    }

    #[test]
    fn preview_numbers_ordered_lists_from_start_attribute() {
        let doc = MarkdownDocument::from_text("3. Third\n4. Fourth");
        let indexes: Vec<Option<u64>> = doc
            .preview_blocks()
            .iter()
            .map(|block| match block {
                PreviewBlock::ListItem { index, .. } => *index,
                _ => None,
            })
            .collect();

        assert_eq!(indexes, vec![Some(3), Some(4)]);
    }

    #[test]
    fn preview_keeps_parent_item_of_nested_lists() {
        let doc = MarkdownDocument::from_text("- parent\n  - child");
        let blocks = doc.preview_blocks();

        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            &blocks[0],
            PreviewBlock::ListItem {
                level: 1,
                ordered: false,
                index: None,
                checked: None,
                text,
                ..
            } if text.text == "parent"
        ));
        assert!(matches!(
            &blocks[1],
            PreviewBlock::ListItem {
                level: 2,
                ordered: false,
                index: None,
                checked: None,
                text,
                ..
            } if text.text == "child"
        ));
    }

    #[test]
    fn preview_tracks_inline_styles_for_bold_italic_code_links() {
        let doc = MarkdownDocument::from_text(
            "**bold** *italic* ~~gone~~ `code` [Zed](https://zed.dev) ==mark==",
        );
        let blocks = doc.preview_blocks();
        let PreviewBlock::Paragraph { text: rich, .. } = &blocks[0] else {
            panic!("expected paragraph, got {blocks:?}");
        };

        assert_eq!(rich.text, "bold italic gone code Zed mark");
        let span_for = |needle: &str| {
            rich.spans
                .iter()
                .find(|span| span.text == needle)
                .unwrap_or_else(|| panic!("missing span {needle:?} in {:?}", rich.spans))
        };
        assert!(span_for("bold").style.bold);
        assert!(span_for("italic").style.italic);
        assert!(span_for("gone").style.strikethrough);
        assert!(span_for("code").style.code);
        assert_eq!(span_for("Zed").link.as_deref(), Some("https://zed.dev"));
        assert!(span_for("mark").style.highlight);
        assert!(span_for("bold").link.is_none());
    }

    #[test]
    fn preview_rich_text_concatenates_spans_into_plain_text() {
        let doc = MarkdownDocument::from_text("# Head **strong**\n\n> quoted *soft*\n");
        let blocks = doc.preview_blocks();

        for block in &blocks {
            let rich = match block {
                PreviewBlock::Heading { text, .. } => text,
                PreviewBlock::BlockQuote { text, .. } => text,
                _ => continue,
            };
            let joined: String = rich.spans.iter().map(|span| span.text.as_str()).collect();
            assert_eq!(joined, rich.text);
        }
        assert!(matches!(
            &blocks[0],
            PreviewBlock::Heading { level: 1, text, .. } if text.text == "Head strong"
        ));
    }

    #[test]
    fn preview_extracts_markdown_images() {
        let doc = MarkdownDocument::from_text(
            "Intro\n\n![Architecture Diagram](images/arch.png \"System overview\")",
        );

        let blocks = doc.preview_blocks();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(
            &blocks[0],
            PreviewBlock::Paragraph { text, .. } if text.text == "Intro"
        ));
        assert!(matches!(
            &blocks[1],
            PreviewBlock::Image {
                alt,
                url,
                title: Some(title),
                ..
            } if alt == "Architecture Diagram"
                && url == "images/arch.png"
                && title == "System overview"
        ));
    }

    #[test]
    fn preview_preserves_fragment_bearing_remote_image_destination() {
        let authored =
            "https://mmbiz.qpic.cn/sz_mmbiz_png/example/640?wx_fmt=png&from=appmsg#imgIndex=0";
        let doc = MarkdownDocument::from_text(format!("![Image]({authored})"));

        let blocks = doc.preview_blocks();
        assert!(matches!(
            blocks.as_slice(),
            [PreviewBlock::Image { url, .. }] if url == authored
        ));
    }

    #[test]
    fn preview_renders_extended_inline_markdown_as_readable_text() {
        let doc = MarkdownDocument::from_text(
            "Water H~2~O, ==marked text==, x^2^, :smile:, and https://example.com/docs.",
        );

        let blocks = doc.preview_blocks();
        let PreviewBlock::Paragraph { text: rich, .. } = &blocks[0] else {
            panic!("expected paragraph, got {blocks:?}");
        };

        assert_eq!(
            rich.text,
            "Water H2O, marked text, x2, 🙂, and https://example.com/docs."
        );
        let span_for = |needle: &str| {
            rich.spans
                .iter()
                .find(|span| span.text == needle)
                .unwrap_or_else(|| panic!("missing span {needle:?} in {:?}", rich.spans))
        };
        assert!(span_for("marked text").style.highlight);
        assert!(
            rich.spans
                .iter()
                .any(|span| span.text == "2" && span.style.subscript)
        );
        assert!(
            rich.spans
                .iter()
                .any(|span| span.text == "2" && span.style.superscript)
        );
        assert_eq!(
            span_for("https://example.com/docs").link.as_deref(),
            Some("https://example.com/docs")
        );
    }

    #[test]
    fn html_export_renders_extended_inline_semantics() {
        let doc = MarkdownDocument::from_text(
            "Water H~2~O, ==marked text==, x^2^, :rocket:, and www.example.com.\n\n`==code==`\n\n```text\n==code==\n```",
        );
        let html = doc.render_html_fragment();

        assert!(html.contains("H<sub>2</sub>O"));
        assert!(html.contains("<mark>marked text</mark>"));
        assert!(html.contains("x<sup>2</sup>"));
        assert!(html.contains("🚀"));
        assert!(html.contains("<a href=\"https://www.example.com\">www.example.com</a>"));
        assert!(html.contains("<code>==code==</code>"));
        assert!(html.contains("<pre><code class=\"language-text\">==code==\n</code></pre>"));
    }

    #[test]
    fn footnotes_track_definitions_references_and_extended_text() {
        let doc = MarkdownDocument::from_text(
            "See the note[^details] and again[^details].\n\n[^details]: Footnote with ==mark== and :check:",
        );

        assert_eq!(
            doc.footnotes(),
            vec![Footnote {
                label: "details".into(),
                text: "Footnote with mark and ✅".into(),
                references: 2,
            }]
        );

        let html = doc.render_html_fragment();
        assert!(html.contains("class=\"footnote-reference\""));
        assert!(html.contains("<mark>mark</mark>"));
    }

    #[test]
    fn line_navigation_uses_logical_lines_and_utf8_columns() {
        let doc = MarkdownDocument::from_text("alpha\n中文ab\nz");

        let second_line_b = doc.text().find('b').unwrap();
        assert_eq!(doc.line_start_at(second_line_b), 6);
        assert_eq!(doc.line_end_at(second_line_b), "alpha\n中文ab".len());
        assert_eq!(doc.previous_line_offset(second_line_b), "alp".len());

        let first_line_p = doc.text().find('p').unwrap();
        assert_eq!(doc.next_line_offset(first_line_p), "alpha\n中文".len());
    }

    #[test]
    fn indents_and_outdents_selected_lines() {
        let mut doc = MarkdownDocument::from_text("one\ntwo\nthree");
        let range = 1.."one\ntwo".len();
        let range = doc.indent_lines(range);

        assert_eq!(doc.text(), "    one\n    two\nthree");
        assert_eq!(range, 5.."    one\n    two".len());

        let range = doc.outdent_lines(range);
        assert_eq!(doc.text(), "one\ntwo\nthree");
        assert_eq!(range, 1.."one\ntwo".len());
    }

    #[test]
    fn outdent_removes_up_to_four_spaces_or_one_tab() {
        let mut doc = MarkdownDocument::from_text("  two\n\tthree\nplain");
        doc.outdent_lines(0..doc.text().len());

        assert_eq!(doc.text(), "two\nthree\nplain");
    }

    #[test]
    fn outdent_keeps_empty_cursor_range_valid_at_line_start() {
        let mut doc = MarkdownDocument::from_text("top\n    nested");
        let cursor = "top\n".len();
        let range = doc.outdent_lines(cursor..cursor);

        assert_eq!(doc.text(), "top\nnested");
        assert_eq!(range, cursor..cursor);
    }

    #[test]
    fn markdown_format_wraps_and_unwraps_inline_selection() {
        let mut doc = MarkdownDocument::from_text("write text");
        let range = doc.apply_markdown_format(6..10, MarkdownFormat::Bold);

        assert_eq!(doc.text(), "write **text**");
        assert_eq!(range, 8..12);

        let range = doc.apply_markdown_format(range, MarkdownFormat::Bold);
        assert_eq!(doc.text(), "write text");
        assert_eq!(range, 6..10);
    }

    #[test]
    fn markdown_format_inserts_placeholder_for_empty_inline_selection() {
        let mut doc = MarkdownDocument::new();
        let range = doc.apply_markdown_format(0..0, MarkdownFormat::InlineCode);

        assert_eq!(doc.text(), "`code`");
        assert_eq!(range, 1..5);
        assert!(doc.is_dirty());
    }

    #[test]
    fn markdown_format_inserts_link_and_selects_next_placeholder() {
        let mut selected = MarkdownDocument::from_text("OpenAI");
        let range = selected.apply_markdown_format(0..6, MarkdownFormat::Link);

        assert_eq!(selected.text(), "[OpenAI](url)");
        assert_eq!(range, 9..12);

        let mut empty = MarkdownDocument::new();
        let range = empty.apply_markdown_format(0..0, MarkdownFormat::Link);

        assert_eq!(empty.text(), "[text](url)");
        assert_eq!(range, 1..5);
    }

    #[test]
    fn markdown_format_inserts_image_and_selects_next_placeholder() {
        let mut selected = MarkdownDocument::from_text("Diagram");
        let range = selected.apply_markdown_format(0..7, MarkdownFormat::Image);

        assert_eq!(selected.text(), "![Diagram](image.png)");
        assert_eq!(range, 11..20);

        let mut empty = MarkdownDocument::new();
        let range = empty.apply_markdown_format(0..0, MarkdownFormat::Image);

        assert_eq!(empty.text(), "![alt](image.png)");
        assert_eq!(range, 2..5);
    }

    #[test]
    fn markdown_format_sets_and_toggles_headings() {
        let mut doc = MarkdownDocument::from_text("Title\nBody");
        let range = doc.apply_markdown_format(0..5, MarkdownFormat::Heading(2));

        assert_eq!(doc.text(), "## Title\nBody");
        assert_eq!(range, 0..8);

        let range = doc.apply_markdown_format(range, MarkdownFormat::Heading(2));
        assert_eq!(doc.text(), "Title\nBody");
        assert_eq!(range, 0..5);
    }

    #[test]
    fn markdown_format_keeps_partial_line_selection_on_same_text() {
        let mut list = MarkdownDocument::from_text("hello world");
        let range = list.apply_markdown_format(6..11, MarkdownFormat::UnorderedList);
        assert_eq!(list.text(), "- hello world");
        assert_eq!(&list.text()[range.clone()], "world");

        let mut heading = MarkdownDocument::from_text("hello world");
        let range = heading.apply_markdown_format(6..11, MarkdownFormat::Heading(1));
        assert_eq!(heading.text(), "# hello world");
        assert_eq!(&heading.text()[range.clone()], "world");

        let range = heading.apply_markdown_format(range, MarkdownFormat::Heading(1));
        assert_eq!(heading.text(), "hello world");
        assert_eq!(&heading.text()[range], "world");
    }

    #[test]
    fn repro_switch_heading_level_with_cursor_on_heading() {
        // 光标停留在 H1 标题行内（无选区），然后把该行切换为 H2/H3。
        // 文本 "# Title"：offset 5 落在 "Title" 的 'i' 上。切换 marker 长度
        // 变化时光标随之平移：H1 marker 是 2 字节，H{target} 是 target+1 字节。
        for target in [2u8, 3, 4, 5, 6] {
            let mut doc = MarkdownDocument::from_text("# Title\nBody");
            let range = doc.apply_markdown_format(5..5, MarkdownFormat::Heading(target));
            let expected = format!("{} Title\nBody", "#".repeat(target as usize));
            assert_eq!(doc.text(), expected, "switching H1 -> H{target}");
            let expected_cursor = 5 + target as usize - 1;
            assert_eq!(
                range,
                expected_cursor..expected_cursor,
                "cursor for H{target}"
            );
        }

        // 反向：光标在 H3 行（marker 4 字节），切换为 H1/H2，光标左移。
        for target in [1u8, 2] {
            let mut doc = MarkdownDocument::from_text("### Title\nBody");
            let range = doc.apply_markdown_format(7..7, MarkdownFormat::Heading(target));
            let expected = format!("{} Title\nBody", "#".repeat(target as usize));
            assert_eq!(doc.text(), expected, "switching H3 -> H{target}");
            let expected_cursor = (7 + (target as isize - 3)) as usize;
            assert_eq!(
                range,
                expected_cursor..expected_cursor,
                "cursor for H{target}"
            );
        }
    }

    #[test]
    fn markdown_format_prefixes_selected_lines() {
        let mut doc = MarkdownDocument::from_text("one\ntwo");
        let range = doc.apply_markdown_format(0..doc.text().len(), MarkdownFormat::TaskList);

        assert_eq!(doc.text(), "- [ ] one\n- [ ] two");
        assert_eq!(range, 0..doc.text().len());
    }

    #[test]
    fn markdown_format_wraps_selection_in_code_fence() {
        let mut doc = MarkdownDocument::from_text("fn main() {}");
        let range = doc.apply_markdown_format(0..doc.text().len(), MarkdownFormat::CodeFence);

        assert_eq!(doc.text(), "```\nfn main() {}\n```");
        assert_eq!(range, 4.."```\nfn main() {}".len());
    }

    #[test]
    fn table_edit_finds_formats_and_adds_rows() {
        let mut doc = MarkdownDocument::from_text(
            "Intro\n\n| Name | Score |\n|---|---|\n| Ada | 10 |\n| Linus | 9 |\n\nDone",
        );
        let cursor = doc.text().find("Ada").unwrap();
        let result = doc.edit_table_at(cursor, TableEdit::AddRow).unwrap();

        assert_eq!(
            doc.table_range_at(result.selected_range.start),
            Some("Intro\n\n".len().."Intro\n\n| Name  | Score |\n| ----- | ----- |\n| Ada   | 10    |\n|       |       |\n| Linus | 9     |".len())
        );
        assert_eq!(
            doc.text(),
            "Intro\n\n| Name  | Score |\n| ----- | ----- |\n| Ada   | 10    |\n|       |       |\n| Linus | 9     |\n\nDone"
        );
        assert_eq!(result.row, 2);
        assert_eq!(result.column, 0);
        assert!(doc.is_dirty());
    }

    #[test]
    fn table_ranges_track_multiple_source_tables() {
        let doc = MarkdownDocument::from_text(
            "Intro\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\nText\n\n| C | D |\n|---|---|\n| 3 | 4 |",
        );
        let ranges = doc.table_ranges();

        assert_eq!(ranges.len(), 2);
        assert_eq!(
            &doc.text()[ranges[0].clone()],
            "| A | B |\n|---|---|\n| 1 | 2 |"
        );
        assert_eq!(
            &doc.text()[ranges[1].clone()],
            "| C | D |\n|---|---|\n| 3 | 4 |"
        );

        let tables = doc
            .preview_blocks()
            .into_iter()
            .filter_map(|block| match block {
                PreviewBlock::Table { source_range, .. } => Some(source_range),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(tables, ranges);
    }

    #[test]
    fn preview_blocks_carry_source_ranges_for_common_blocks() {
        let source = "# Heading\n\nParagraph text.\n\n- list item\n\n```rs\nlet x = 1;\n```\n\n> quote\n\n---\n";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.preview_blocks();
        assert!(
            blocks.iter().all(|block| !block.source_range().is_empty()),
            "every content block should expose a non-empty source_range"
        );

        let heading = blocks.iter().find_map(|block| match block {
            PreviewBlock::Heading { source_range, .. } => Some(source_range.clone()),
            _ => None,
        });
        let paragraph = blocks.iter().find_map(|block| match block {
            PreviewBlock::Paragraph { source_range, .. } => Some(source_range.clone()),
            _ => None,
        });
        let list = blocks.iter().find_map(|block| match block {
            PreviewBlock::ListItem { source_range, .. } => Some(source_range.clone()),
            _ => None,
        });
        let code = blocks.iter().find_map(|block| match block {
            PreviewBlock::CodeBlock { source_range, .. } => Some(source_range.clone()),
            _ => None,
        });
        let quote = blocks.iter().find_map(|block| match block {
            PreviewBlock::BlockQuote { source_range, .. } => Some(source_range.clone()),
            _ => None,
        });
        let rule = blocks.iter().find_map(|block| match block {
            PreviewBlock::Rule { source_range } => Some(source_range.clone()),
            _ => None,
        });

        assert_eq!(source[heading.expect("heading")].trim(), "# Heading");
        assert_eq!(
            source[paragraph.expect("paragraph")].trim(),
            "Paragraph text."
        );
        assert!(source[list.expect("list")].contains("list item"));
        assert!(source[code.expect("code")].contains("let x = 1;"));
        assert!(source[quote.expect("quote")].contains("quote"));
        assert_eq!(source[rule.expect("rule")].trim(), "---");
    }

    #[test]
    fn table_edit_deletes_columns_and_moves_data_rows() {
        let mut doc = MarkdownDocument::from_text(
            "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |",
        );
        let cursor = doc.text().find('2').unwrap();
        doc.edit_table_at(cursor, TableEdit::DeleteColumn).unwrap();

        assert_eq!(
            doc.text(),
            "| A   | C   |\n| --- | --- |\n| 1   | 3   |\n| 4   | 6   |"
        );

        let cursor = doc.text().find('1').unwrap();
        let result = doc.edit_table_at(cursor, TableEdit::MoveRowDown).unwrap();

        assert_eq!(
            doc.text(),
            "| A   | C   |\n| --- | --- |\n| 4   | 6   |\n| 1   | 3   |"
        );
        assert_eq!(result.row, 2);
    }

    #[test]
    fn table_edit_preserves_column_alignment_markers() {
        let mut doc =
            MarkdownDocument::from_text("| A | B | C |\n| :--- | :---: | ---: |\n| 1 | 2 | 3 |");
        let cursor = doc.text().find('2').unwrap();
        doc.edit_table_at(cursor, TableEdit::Format).unwrap();

        assert_eq!(
            doc.text(),
            "| A   | B   | C   |\n| :--- | :---: | ---: |\n| 1   | 2   | 3   |"
        );

        let cursor = doc.text().find('2').unwrap();
        doc.edit_table_at(cursor, TableEdit::DeleteColumn).unwrap();
        assert_eq!(doc.text(), "| A   | C   |\n| :--- | ---: |\n| 1   | 3   |");
    }

    #[test]
    fn table_edit_handles_utf8_cursor_boundaries() {
        let mut doc = MarkdownDocument::from_text("| 名 | 值 |\n|---|---|\n| 文 | 1 |");
        let cursor_inside_utf8 = doc.text().find("文").unwrap() + 1;

        doc.edit_table_at(cursor_inside_utf8, TableEdit::AddColumn)
            .unwrap();

        assert_eq!(
            doc.text(),
            "| 名   |     | 值   |\n| --- | --- | --- |\n| 文   |     | 1   |"
        );
    }

    #[test]
    fn table_edit_returns_none_outside_tables_or_invalid_moves() {
        let mut doc = MarkdownDocument::from_text("A | B but not a table");
        assert_eq!(doc.edit_table_at(0, TableEdit::Format), None);

        let mut table = MarkdownDocument::from_text("| A | B |\n|---|---|\n| 1 | 2 |");
        let cursor = table.text().find('1').unwrap();
        assert_eq!(table.edit_table_at(cursor, TableEdit::MoveRowUp), None);
    }

    #[test]
    fn paragraph_range_tracks_current_focus_block() {
        let doc = MarkdownDocument::from_text(
            "# Title\n\nFirst paragraph.\nStill first.\n\nSecond 文本.",
        );

        let first = doc.text().find("Still").unwrap();
        assert_eq!(
            doc.paragraph_range_at(first),
            "# Title\n\n".len().."# Title\n\nFirst paragraph.\nStill first.".len()
        );

        let second_inside_utf8 = doc.text().find("文本").unwrap() + 1;
        assert_eq!(
            doc.paragraph_range_at(second_inside_utf8),
            "# Title\n\nFirst paragraph.\nStill first.\n\n".len()..doc.text().len()
        );
    }

    #[test]
    fn view_mode_cycles_through_all_modes() {
        assert_eq!(ViewMode::default_mode(), ViewMode::VisualEdit);
        assert_eq!(ViewMode::default(), ViewMode::VisualEdit);
        assert_eq!(ViewMode::Edit.next(), ViewMode::VisualEdit);
        assert_eq!(ViewMode::VisualEdit.next(), ViewMode::Split);
        assert_eq!(ViewMode::Split.next(), ViewMode::Read);
        assert_eq!(ViewMode::Read.next(), ViewMode::Edit);
    }

    #[test]
    fn outline_tracks_source_offsets_and_current_heading() {
        let doc = MarkdownDocument::from_text("---\ntitle: Doc\n---\n# One\n\nText\n\n## Two");
        let outline = doc.outline();

        assert_eq!(outline[0].title, "One");
        assert_eq!(outline[0].offset, "---\ntitle: Doc\n---\n".len());
        assert_eq!(outline[1].title, "Two");
        assert_eq!(
            doc.current_heading_index(doc.text().find("Text").unwrap()),
            Some(0)
        );
        assert_eq!(
            doc.current_heading_index(doc.text().find("## Two").unwrap()),
            Some(1)
        );
    }

    #[test]
    fn merged_and_standalone_outline_paths_agree() {
        // The outline is produced two ways — folded into `derive_preview_and_outline`
        // (used when the preview parses) and via `compute_outline_only` (the
        // Edit-mode fallback). They must be byte-identical. Exercise headings
        // with front matter, inline code, styling, and an image (whose alt text
        // both paths must fold into the title) to stress the accumulation.
        let source = "---\ntitle: T\n---\n\
             # Plain heading\n\ntext\n\n\
             ## Sub `code` and **bold**\n\n\
             ### With ![alt words](img.png) image\n\nbody\n";

        // `outline()` after a preview parse returns the derive-produced outline.
        let derived = MarkdownDocument::from_text(source);
        let _ = derived.preview_blocks_shared();
        let via_derive = derived.outline();

        // A fresh document with no preview parse takes the heading-only path.
        let standalone = MarkdownDocument::from_text(source);
        let via_standalone = standalone.outline();

        assert_eq!(via_derive, via_standalone);
        // Sanity: the image alt text is part of the title on both paths.
        assert_eq!(via_derive[2].title, "With alt words image");
    }

    #[test]
    fn install_derived_is_version_gated() {
        let mut doc = MarkdownDocument::from_text("# One\n");
        let stale_version = doc.version();
        let (stale_blocks, stale_headings) =
            MarkdownDocument::derive_preview_and_outline(doc.text());

        // Simulates a background parse landing after the document changed: the
        // snapshot's version no longer matches, so the install is dropped.
        doc.insert(doc.text().len(), "\n# Two\n");
        doc.install_derived(
            stale_version,
            std::sync::Arc::new(stale_blocks),
            stale_headings,
        );
        assert_eq!(doc.outline().len(), 2, "stale install must not stick");

        // A matching version is accepted, and both caches serve the installed
        // values (`preview_blocks_shared` returns the very same Arc).
        let version = doc.version();
        let (blocks, headings) = MarkdownDocument::derive_preview_and_outline(doc.text());
        let blocks = std::sync::Arc::new(blocks);
        doc.install_derived(version, blocks.clone(), headings.clone());
        assert!(std::sync::Arc::ptr_eq(
            &doc.preview_blocks_shared(),
            &blocks
        ));
        assert_eq!(doc.outline(), headings);
    }

    #[test]
    fn search_supports_case_sensitive_regex_and_utf8_ranges() {
        let doc = MarkdownDocument::from_text("Alpha\nalpha 文本\nbeta");

        let insensitive = doc.find_matches(&SearchOptions::literal("ALPHA")).unwrap();
        assert_eq!(insensitive.len(), 2);

        let sensitive = doc
            .find_matches(&SearchOptions {
                query: "ALPHA".into(),
                case_sensitive: true,
                regex: false,
            })
            .unwrap();
        assert!(sensitive.is_empty());

        let regex = doc
            .find_matches(&SearchOptions {
                query: r"a\w+".into(),
                case_sensitive: false,
                regex: true,
            })
            .unwrap();
        assert_eq!(regex[0].range, 0..5);

        let unicode = doc.find_matches(&SearchOptions::literal("文本")).unwrap();
        assert_eq!(&doc.text()[unicode[0].range.clone()], "文本");
        assert_eq!(unicode[0].line, 2);
        assert_eq!(unicode[0].column, 7);

        assert!(
            doc.find_matches(&SearchOptions {
                query: "(".into(),
                case_sensitive: false,
                regex: true,
            })
            .is_err()
        );
    }

    #[test]
    fn replace_current_and_all_matches_update_document() {
        let mut doc = MarkdownDocument::from_text("one two one");
        let options = SearchOptions::literal("one");
        let first = doc.find_matches(&options).unwrap()[0].range.clone();

        let result = doc.replace_current_match(first, &options, "ONE").unwrap();
        assert_eq!(result.replacements, 1);
        assert_eq!(doc.text(), "ONE two one");
        assert_eq!(result.selected_range, Some(0..3));

        let result = doc.replace_all_matches(&options, "1").unwrap();
        assert_eq!(result.replacements, 2);
        assert_eq!(doc.text(), "1 two 1");
    }

    #[test]
    fn regex_replace_all_supports_captures() {
        let mut doc = MarkdownDocument::from_text("2026-06-30");
        let result = doc
            .replace_all_matches(
                &SearchOptions {
                    query: r"(\d{4})-(\d{2})-(\d{2})".into(),
                    case_sensitive: true,
                    regex: true,
                },
                "$2/$3/$1",
            )
            .unwrap();

        assert_eq!(result.replacements, 1);
        assert_eq!(doc.text(), "06/30/2026");
    }

    #[test]
    fn autosave_writes_existing_file_and_recovery_copy() {
        let dir = tempfile::tempdir().unwrap();
        let file_path = dir.path().join("note.md");
        let recovery_dir = dir.path().join("recovery");

        let mut saved = MarkdownDocument::from_text("old");
        saved.save_as(&file_path).unwrap();
        let undo_snapshot = saved.clone();
        saved.set_text("new");
        let outcome = saved.autosave(&recovery_dir).unwrap();
        assert_eq!(outcome, AutosaveOutcome::SavedFile(file_path.clone()));
        assert_eq!(fs::read_to_string(&file_path).unwrap(), "new");
        assert!(!saved.is_dirty());

        let mut restored = undo_snapshot;
        restored.refresh_dirty_from_disk();
        assert!(restored.is_dirty());

        let mut unsaved = MarkdownDocument::new();
        unsaved.set_text("# Draft");
        let outcome = unsaved.autosave(&recovery_dir).unwrap();
        let AutosaveOutcome::SavedRecovery(recovery_path) = outcome else {
            panic!("expected recovery save");
        };
        assert!(unsaved.is_dirty());

        let recovered = load_recovery_file(&recovery_path).unwrap();
        assert_eq!(recovered.text, "# Draft");
        assert_eq!(recovered.original_path, None);
        assert_eq!(
            list_recovery_files(&recovery_dir).unwrap(),
            vec![recovery_path.clone()]
        );
        delete_recovery_file(recovery_path).unwrap();
        assert!(list_recovery_files(&recovery_dir).unwrap().is_empty());
    }

    #[test]
    fn preferences_parse_save_and_load_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let preferences = AppPreferences {
            theme: "Ink".to_string(),
            custom_theme: Some("Midnight".to_string()),
            focus_mode: true,
            typewriter_mode: true,
            code_line_numbers: false,
            preview_adaptive_width: true,
            editor_font_size: 18,
            rendered_font_size: 20,
            paragraph_spacing: 16,
            heading_menu_max_level: EXTENDED_HEADING_MENU_MAX_LEVEL,
            sync_scroll: true,
            sidebar_visible: false,
            sidebar_tab: SidebarTab::Outline,
            language: "zh".to_string(),
            auto_save: AutoSavePreferences {
                enabled: false,
                delay_secs: 30,
            },
            export: ExportPreferences {
                pdf_engine: "tectonic".to_string(),
            },
        };

        save_app_preferences(&path, &preferences).unwrap();
        assert_eq!(load_app_preferences(&path).unwrap(), preferences);

        // The on-disk format is TOML with [auto_save] and [export] tables.
        let written = fs::read_to_string(&path).unwrap();
        assert!(written.contains("theme = \"Ink\""));
        assert!(written.contains("preview_adaptive_width = true"));
        assert!(written.contains("editor_font_size = 18"));
        assert!(written.contains("rendered_font_size = 20"));
        assert!(written.contains("paragraph_spacing = 16"));
        assert!(written.contains("heading_menu_max_level = 6"));
        assert!(written.contains("sync_scroll = true"));
        assert!(written.contains("[auto_save]"));
        assert!(written.contains("delay_secs = 30"));
        assert!(written.contains("[export]"));
        assert!(written.contains("pdf_engine = \"tectonic\""));

        // Partial TOML files take defaults for missing fields.
        let parsed =
            parse_app_preferences("theme = \"Forest\"\n\n[auto_save]\ndelay_secs = 9\n").unwrap();
        assert_eq!(parsed.theme, "Forest");
        assert_eq!(parsed.custom_theme, None);
        assert_eq!(parsed.language, "en");
        assert!(!parsed.preview_adaptive_width);
        assert_eq!(parsed.editor_font_size, DEFAULT_EDITOR_FONT_SIZE);
        assert_eq!(parsed.rendered_font_size, DEFAULT_RENDERED_FONT_SIZE);
        assert_eq!(parsed.paragraph_spacing, DEFAULT_PARAGRAPH_SPACING);
        assert!(parsed.auto_save.enabled);
        assert_eq!(parsed.auto_save.delay_secs, 9);

        // An empty file is all defaults; unknown sidebar tabs fall back to
        // Files.
        assert_eq!(
            parse_app_preferences("").unwrap(),
            AppPreferences::default()
        );
        let parsed_unknown = parse_app_preferences("sidebar_tab = \"bogus\"").unwrap();
        assert_eq!(parsed_unknown.sidebar_tab, SidebarTab::Files);
        let parsed_invalid_adaptive_width =
            parse_app_preferences("preview_adaptive_width = \"wide\"").unwrap();
        assert!(!parsed_invalid_adaptive_width.preview_adaptive_width);

        let parsed_extended = parse_app_preferences("heading_menu_max_level = 6").unwrap();
        assert_eq!(
            parsed_extended.heading_menu_max_level,
            EXTENDED_HEADING_MENU_MAX_LEVEL
        );
        let parsed_invalid_heading_depth =
            parse_app_preferences("heading_menu_max_level = 4").unwrap();
        assert_eq!(
            parsed_invalid_heading_depth.heading_menu_max_level,
            DEFAULT_HEADING_MENU_MAX_LEVEL
        );

        // No config file and no legacy sibling → defaults.
        let empty_dir = tempfile::tempdir().unwrap();
        assert_eq!(
            load_app_preferences(empty_dir.path().join("config.toml")).unwrap(),
            AppPreferences::default()
        );
    }

    #[test]
    fn legacy_preferences_migrate_to_toml_once() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        fs::write(
            dir.path().join("preferences.conf"),
            "# comment\ntheme=Forest\ncustom_theme=\nfocus_mode=on\ntypewriter_mode=no\ncode_line_numbers=1\nsidebar_visible=off\nsidebar_tab=outline\nlanguage=zh\nunknown=ignored",
        )
        .unwrap();

        let migrated = load_app_preferences(&config_path).unwrap();
        assert_eq!(migrated.theme, "Forest");
        assert_eq!(migrated.custom_theme, None);
        assert!(migrated.focus_mode);
        assert!(!migrated.typewriter_mode);
        assert!(migrated.code_line_numbers);
        assert!(!migrated.sidebar_visible);
        assert_eq!(migrated.sidebar_tab, SidebarTab::Outline);
        assert_eq!(migrated.language, "zh");
        // Legacy files predate auto-save configurability → defaults.
        assert_eq!(migrated.auto_save, AutoSavePreferences::default());

        // The migration wrote config.toml; later loads read it and ignore
        // the legacy file even if it changes.
        assert!(config_path.exists());
        fs::write(dir.path().join("preferences.conf"), "theme=Rose").unwrap();
        assert_eq!(load_app_preferences(&config_path).unwrap().theme, "Forest");
    }

    #[test]
    fn preferences_report_invalid_values() {
        // Invalid TOML.
        let err = parse_app_preferences("theme = ").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        // The legacy migration reader keeps its strict error reporting.
        let err = parse_legacy_app_preferences("focus_mode=maybe").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        let err = parse_legacy_app_preferences("not a pair").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn theme_definition_parse_save_and_list_roundtrip() {
        let dir = tempfile::tempdir().unwrap();
        let theme = parse_theme_definition(
            "name = \"Midnight\"\nis_dark = true\n[colors]\napp_bg = \"#10131a\"\npanel_bg = \"#171b24\"\nsurface_bg = \"#0f1720\"\ntext = \"#e5edf5\"\nmuted = \"#91a4b7\"\nborder = \"#2b3544\"\nactive_bg = \"#23304a\"\nactive_text = \"#9ec5ff\"",
        )
        .unwrap();

        assert_eq!(theme.name, "Midnight");
        assert!(theme.is_dark);
        assert_eq!(theme.colors.app_bg, 0x10131a);
        assert_eq!(theme.colors.active_text, 0x9ec5ff);

        let path = dir.path().join("midnight.toml");
        save_theme_definition(&path, &theme).unwrap();
        assert_eq!(load_theme_definition(&path).unwrap(), theme);
        assert_eq!(list_theme_definitions(dir.path()).unwrap(), vec![theme]);
    }

    #[test]
    fn theme_definition_reports_invalid_values() {
        // Missing required `name`.
        let err = parse_theme_definition("[colors]\napp_bg = \"#ffffff\"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);

        // Malformed color value.
        let err = parse_theme_definition("name = \"Bad\"\n[colors]\ntext = \"blue\"").unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn partial_toml_theme_loads_with_fallback_palette() {
        // Only two colors set; the rest take the default palette.
        let theme = parse_theme_definition(
            "name = \"Sparse\"\nis_dark = true\n[colors]\napp_bg = \"#10131a\"\ntext = \"#e5edf5\"",
        )
        .unwrap();
        assert_eq!(theme.name, "Sparse");
        assert_eq!(theme.colors.app_bg, 0x10131a);
        assert_eq!(theme.colors.text, 0xe5edf5);
        // Unset keys fall back to the historical default palette.
        assert_eq!(theme.colors.panel_bg, 0xffffff);
        assert_eq!(theme.colors.border, 0xdbe4ee);
    }

    #[test]
    fn legacy_theme_migrates_to_toml_once() {
        let dir = tempfile::tempdir().unwrap();
        let legacy_text = "name=Midnight\nis_dark=true\napp_bg=#10131a\npanel_bg=#171b24\nsurface_bg=#0f1720\ntext=#e5edf5\nmuted=#91a4b7\nborder=#2b3544\nactive_bg=#23304a\nactive_text=#9ec5ff";
        let legacy_path = dir.path().join("midnight.theme");
        fs::write(&legacy_path, legacy_text).unwrap();

        // No `.toml` exists yet — `list_theme_definitions` migrates and lists.
        let themes = list_theme_definitions(dir.path()).unwrap();
        assert_eq!(themes.len(), 1);
        assert_eq!(themes[0].name, "Midnight");
        assert_eq!(themes[0].colors.app_bg, 0x10131a);
        assert_eq!(themes[0].colors.active_text, 0x9ec5ff);

        // The `.toml` was written beside the legacy file.
        let toml_path = dir.path().join("midnight.toml");
        assert!(toml_path.exists());
        // The legacy `.theme` is left in place.
        assert!(legacy_path.exists());

        // Rewrite the legacy file — on the next load the `.toml` wins and the
        // legacy change is ignored (migration is idempotent / one-shot).
        fs::write(&legacy_path, "name=Changed\nis_dark=false\n").unwrap();
        let themes_again = list_theme_definitions(dir.path()).unwrap();
        assert_eq!(themes_again.len(), 1);
        assert_eq!(themes_again[0].name, "Midnight");
    }

    #[test]
    fn yaml_front_matter_is_hidden_from_preview_and_used_in_html() {
        let doc = MarkdownDocument::from_text(
            "---\ntitle: My Doc\nauthor: Me\ndate: 2026-06-30\n---\n# Body\n\nText",
        );

        let metadata = doc.front_matter().unwrap().unwrap();
        assert_eq!(metadata.title.as_deref(), Some("My Doc"));
        assert_eq!(metadata.author.as_deref(), Some("Me"));
        assert_eq!(metadata.date.as_deref(), Some("2026-06-30"));

        assert!(matches!(
            doc.preview_blocks().first(),
            Some(PreviewBlock::Heading {
                level: 1,
                text,
                ..
            }) if text.text == "Body"
        ));

        let html = doc.render_html_document();
        assert!(html.contains("<title>My Doc</title>"));
        assert!(html.contains(r#"<meta name="author" content="Me">"#));
        assert!(!html.contains("title: My Doc"));
    }

    #[test]
    fn invalid_yaml_front_matter_returns_error() {
        let doc = MarkdownDocument::from_text("---\ntitle: [oops\n---\n# Body");

        assert!(doc.front_matter().is_err());
    }

    #[test]
    fn plain_html_export_omits_default_css() {
        let doc = MarkdownDocument::from_text("# Plain");
        let html = doc.render_plain_html_document();

        assert!(html.contains("<h1>Plain</h1>"));
        assert!(!html.contains("<style>"));
    }

    #[test]
    fn highlights_code_keywords_strings_numbers_and_comments() {
        let lines = highlight_code(
            r#"fn main() { let answer = "42"; let n = 7; // ok }"#,
            Some("rust"),
        );
        let spans = &lines[0];

        assert!(
            spans
                .iter()
                .any(|span| span.text == "fn" && span.kind == HighlightKind::Keyword)
        );
        assert!(
            spans
                .iter()
                .any(|span| span.text == "let" && span.kind == HighlightKind::Keyword)
        );
        assert!(
            spans
                .iter()
                .any(|span| span.text == r#""42""# && span.kind == HighlightKind::String)
        );
        assert!(
            spans
                .iter()
                .any(|span| span.text == "7" && span.kind == HighlightKind::Number)
        );
        assert!(
            spans
                .iter()
                .any(|span| span.text == "// ok }" && span.kind == HighlightKind::Comment)
        );
    }

    #[test]
    fn highlighter_advertises_more_than_fifty_languages() {
        let languages = supported_highlight_languages();

        assert!(languages.len() >= 50);
        assert!(languages.contains(&"rust"));
        assert!(languages.contains(&"typescript"));
        assert!(languages.contains(&"python"));
        assert!(languages.contains(&"sql"));
    }

    #[test]
    fn highlighter_normalizes_language_aliases_and_common_syntax() {
        let typescript = highlight_code("export type User = string", Some("ts"));
        assert!(
            typescript[0]
                .iter()
                .any(|span| span.text == "export" && span.kind == HighlightKind::Keyword)
        );
        assert!(
            typescript[0]
                .iter()
                .any(|span| span.text == "string" && span.kind == HighlightKind::Type)
        );

        let sql = highlight_code("SELECT name FROM users -- comment", Some("sql"));
        assert!(
            sql[0]
                .iter()
                .any(|span| span.text == "SELECT" && span.kind == HighlightKind::Keyword)
        );
        assert!(
            sql[0]
                .iter()
                .any(|span| span.text == "-- comment" && span.kind == HighlightKind::Comment)
        );

        let shell = highlight_code("echo ok # comment", Some("sh"));
        assert!(
            shell[0]
                .iter()
                .any(|span| span.text == "# comment" && span.kind == HighlightKind::Comment)
        );
    }

    #[test]
    fn highlights_multiline_constructs_across_lines() {
        // The syntect path keeps parser state across lines, so a block
        // comment stays a comment on its middle lines (the legacy line-local
        // lexer could not do this).
        let lines = highlight_code("/* first\nmiddle line\n*/\nlet x = 1;", Some("rust"));

        assert_eq!(lines.len(), 4);
        assert!(
            lines[1]
                .iter()
                .all(|span| span.kind == HighlightKind::Comment)
        );
        assert!(
            lines[3]
                .iter()
                .any(|span| span.text == "let" && span.kind == HighlightKind::Keyword)
        );
    }

    #[test]
    fn highlight_keeps_empty_line_contract_on_syntect_path() {
        let lines = highlight_code("fn a() {}\n\nfn b() {}", Some("rust"));

        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines[1],
            vec![HighlightedSpan {
                text: String::new(),
                kind: HighlightKind::Plain,
            }]
        );
    }

    #[test]
    fn extended_set_language_uses_syntect_path() {
        // TypeScript is absent from syntect's bundled defaults and only
        // arrives with the two-face extended set; a block comment keeping its
        // color across lines proves the grammar path (the legacy lexer is
        // line-local and cannot do this).
        let lines = highlight_code("/* first\nmiddle\n*/", Some("typescript"));
        assert_eq!(lines.len(), 3);
        assert!(
            lines[1]
                .iter()
                .all(|span| span.kind == HighlightKind::Comment)
        );
    }

    #[test]
    fn registry_uncovered_language_falls_back_to_legacy_lexer() {
        // "wasm" is advertised but not covered even by the extended grammar
        // set, so the hand-written lexer must keep coloring it.
        let lines = highlight_code("const answer = 42; // ok", Some("wasm"));

        assert!(
            lines[0]
                .iter()
                .any(|span| span.text == "const" && span.kind == HighlightKind::Keyword)
        );
        assert!(
            lines[0]
                .iter()
                .any(|span| span.text == "42" && span.kind == HighlightKind::Number)
        );
        assert!(
            lines[0]
                .iter()
                .any(|span| span.text == "// ok" && span.kind == HighlightKind::Comment)
        );
    }

    #[test]
    fn math_is_parsed_for_preview_and_html_export() {
        let doc = MarkdownDocument::from_text("Inline $a+b$.\n\n$$\n\\frac{1}{2}\n$$");
        let blocks = doc.preview_blocks();

        assert!(matches!(
            &blocks[0],
            PreviewBlock::Paragraph { text, .. } if text.text == "Inline $a+b$."
        ));
        assert_eq!(
            doc.math_expressions(),
            vec![
                MathExpression {
                    latex: "a+b".into(),
                    display: false,
                    error: None,
                },
                MathExpression {
                    latex: "\\frac{1}{2}".into(),
                    display: true,
                    error: None,
                }
            ]
        );
        assert!(blocks.iter().any(|block| {
            matches!(
                block,
                PreviewBlock::MathBlock { latex, error, .. }
                    if latex.contains("\\frac{1}{2}") && error.is_none()
            )
        }));

        let html = doc.render_html_fragment();
        assert!(html.contains("math math-inline"));
        assert!(html.contains("math math-display"));
        assert!(html.contains("data-latex=\"a+b\""));
        assert!(html.contains("data-style=\"text\""));
        assert!(html.contains("data-valid=\"true\""));
        assert_eq!(html.matches("<svg aria-hidden=\"true\"").count(), 2);
        assert!(!html.contains("<text"));
    }

    #[test]
    fn invalid_math_block_reports_preview_error() {
        let doc = MarkdownDocument::from_text("$$\n\\begin{matrix} x\n$$");
        let blocks = doc.preview_blocks();

        assert!(blocks.iter().any(|block| {
            matches!(
                block,
                PreviewBlock::MathBlock { error: Some(error), .. }
                    if error.contains("environment")
            )
        }));

        let html = doc.render_html_fragment();
        assert!(html.contains("math-error"));
        assert!(html.contains("data-valid=\"false\""));
        assert!(html.contains("\\begin{matrix} x"));
        assert!(!html.contains("<svg"));
    }

    #[test]
    fn html_math_export_is_static_safe_source_faithful_and_shared_by_both_modes() {
        let source = "Inline $a^2+b^2=c^2$ and x^2^ outside.\n\n```MaTh linenos\n\\begin{matrix}a&b\\\\c&d\\end{matrix}\n```";
        let doc = MarkdownDocument::from_text(source);
        let fragment = doc.render_html_fragment();
        let styled = doc.render_html_document();
        let plain = doc.render_plain_html_document();

        for html in [&fragment, &styled, &plain] {
            assert!(html.contains("data-latex=\"a^2+b^2=c^2\""));
            assert!(html.contains("data-style=\"text\" data-valid=\"true\""));
            assert!(html.contains("data-style=\"display\" data-valid=\"true\""));
            assert!(html.contains("aria-label=\"$a^2+b^2=c^2$\""));
            assert!(html.contains("```MaTh linenos"));
            assert!(html.contains("<svg aria-hidden=\"true\""));
            let lower = html.to_ascii_lowercase();
            assert!(!lower.contains("<script"));
            assert!(!lower.contains("javascript:"));
            assert!(!lower.contains(" onload="));
            assert!(!lower.contains("https://cdn"));
        }
        assert!(fragment.contains("x<sup>2</sup>"));
        assert!(styled.contains("<style>"));
        assert!(!plain.contains("<style>"));
    }

    #[test]
    fn formula_heavy_semantic_derivation_stays_cached_and_renderer_free() {
        use std::time::Instant;

        let source = (0..500)
            .map(|index| format!("row {index}: $x_{index}^2+y_{index}^2$\n\n"))
            .collect::<String>();
        let doc = MarkdownDocument::from_text(source);
        let version = doc.version();
        let started = Instant::now();
        let preview = doc.preview_blocks_shared();
        let visual = doc.visual_blocks_shared();
        let cold = started.elapsed();
        let preview_again = doc.preview_blocks_shared();
        let visual_again = doc.visual_blocks_shared();

        assert_eq!(doc.version(), version);
        assert!(std::sync::Arc::ptr_eq(&preview, &preview_again));
        assert!(std::sync::Arc::ptr_eq(&visual, &visual_again));
        assert_eq!(preview.len(), 500);
        assert!(preview.iter().all(|block| matches!(
            block,
            PreviewBlock::Paragraph { text, .. }
                if text.spans.iter().any(|span| span.math.is_some())
        )));
        assert!(
            cold.as_secs_f32() < 5.0,
            "semantic derivation took {cold:?}"
        );
        eprintln!(
            "500-formula semantic derivation: {cold:?}; cached preview/visual reads reuse Arc"
        );
    }

    #[test]
    fn inline_math_retains_semantic_style_delimiter_and_utf8_source_range() {
        let source = "速度 $E=mc^2$ 和 **$\\frac{a}{b}$**.";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.preview_blocks_shared();
        let PreviewBlock::Paragraph { text, .. } = &blocks[0] else {
            panic!("expected paragraph");
        };
        let math = text
            .spans
            .iter()
            .filter_map(|span| span.math.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(math.len(), 2);
        assert_eq!(math[0].latex, "E=mc^2");
        assert_eq!(math[0].style, MathLayoutStyle::Text);
        assert_eq!(math[0].delimiter, MathDelimiter::InlineDollar);
        assert_eq!(&source[math[0].source_range.clone()], math[0].authored);
        assert_eq!(math[1].latex, "\\frac{a}{b}");
        assert_eq!(&source[math[1].source_range.clone()], math[1].authored);
        assert!(std::sync::Arc::ptr_eq(
            &blocks,
            &doc.preview_blocks_shared()
        ));
    }

    #[test]
    fn display_math_inside_prose_remains_a_display_style_atom() {
        let source = "before $$\\sum_i x_i$$ after";
        let doc = MarkdownDocument::from_text(source);
        let PreviewBlock::Paragraph { text, .. } = &doc.preview_blocks()[0] else {
            panic!("expected paragraph");
        };
        let math = text
            .spans
            .iter()
            .find_map(|span| span.math.as_ref())
            .expect("display math atom");
        assert_eq!(math.style, MathLayoutStyle::Display);
        assert_eq!(math.delimiter, MathDelimiter::DisplayDollar);
        assert_eq!(&source[math.source_range.clone()], "$$\\sum_i x_i$$");
    }

    #[test]
    fn fenced_math_dispatches_without_losing_the_authored_fence() {
        let source = "```MaTh linenos\n\\begin{matrix}a&b\\\\c&d\\end{matrix}\n```";
        let doc = MarkdownDocument::from_text(source);
        let block = doc
            .preview_blocks()
            .into_iter()
            .find(|block| matches!(block, PreviewBlock::MathBlock { .. }))
            .expect("fenced math block");
        let PreviewBlock::MathBlock {
            latex,
            delimiter,
            source_range,
            error,
            ..
        } = block
        else {
            unreachable!()
        };
        assert_eq!(delimiter, MathDelimiter::Fenced);
        assert!(latex.contains("\\begin{matrix}"));
        assert!(error.is_none(), "{error:?}");
        assert_eq!(&source[source_range], source);
        assert!(
            doc.visual_blocks()
                .iter()
                .any(|block| matches!(block.editor, Some(VisualBlockEditor::Math { .. })))
        );
    }

    #[test]
    fn malformed_dollar_source_stays_conservative_plain_text() {
        let source = "before $\\frac{a}{b after";
        let doc = MarkdownDocument::from_text(source);
        let PreviewBlock::Paragraph { text, .. } = &doc.preview_blocks()[0] else {
            panic!("expected paragraph");
        };
        assert!(text.spans.iter().all(|span| span.math.is_none()));
        assert!(text.text.contains("$\\frac{a}{b"));
    }

    #[test]
    fn math_renderer_degrades_common_latex_to_readable_text() {
        let rendered = render_math("\\alpha + \\beta \\leq \\frac{x}{2}", false);

        assert_eq!(rendered.text, "α + β ≤ x⁄2");
        assert_eq!(rendered.error, None);

        let invalid = render_math("\\frac{1}{2", true);
        assert!(invalid.error.unwrap().contains("unclosed brace"));
    }

    #[test]
    fn file_tree_scans_markdown_files_and_supports_basic_operations() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes");
        fs::create_dir(&notes).unwrap();
        fs::write(dir.path().join("root.md"), "# Root").unwrap();
        fs::write(notes.join("child.markdown"), "# Child").unwrap();
        fs::write(notes.join("ignored.txt"), "plain").unwrap();
        fs::create_dir(dir.path().join("target")).unwrap();
        fs::write(dir.path().join("target").join("skip.md"), "# Skip").unwrap();

        let mut tree = FileTree::scan(dir.path()).unwrap();
        assert!(
            tree.entries
                .iter()
                .any(|entry| entry.name == "root.md" && entry.is_markdown)
        );
        assert!(
            tree.entries
                .iter()
                .any(|entry| entry.name == "child.markdown" && entry.depth == 1)
        );
        assert!(!tree.entries.iter().any(|entry| entry.name == "skip.md"));

        let draft = tree.create_file(&notes, "draft.md").unwrap();
        assert!(draft.exists());
        let renamed = tree.rename(&draft, "renamed.md").unwrap();
        assert!(renamed.exists());
        let folder = tree.create_directory(dir.path(), "archive").unwrap();
        assert!(folder.is_dir());
        fs::write(notes.join("existing.md"), "keep").unwrap();
        assert!(tree.create_file(&notes, "existing.md").is_err());
        assert_eq!(
            fs::read_to_string(notes.join("existing.md")).unwrap(),
            "keep"
        );
        assert!(tree.create_file(&notes, "../escape.md").is_err());
        assert!(tree.create_directory(&notes, "nested/archive").is_err());
        assert!(tree.rename(&folder, "../escape").is_err());
        tree.delete(&renamed).unwrap();
        tree.delete(&folder).unwrap();
        assert!(!renamed.exists());
        assert!(!folder.exists());
    }

    #[test]
    fn file_tree_filters_uniquely_names_and_moves_entries() {
        let dir = tempfile::tempdir().unwrap();
        let notes = dir.path().join("notes");
        let archive = dir.path().join("archive");
        fs::create_dir(&notes).unwrap();
        fs::create_dir(&archive).unwrap();
        fs::write(notes.join("daily.md"), "# Daily").unwrap();
        fs::write(notes.join("untitled.md"), "# Existing").unwrap();

        let mut tree = FileTree::scan(dir.path()).unwrap();
        let matches = tree.filtered_entries("daily");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].name, "daily.md");

        let created = tree.create_unique_file(&notes, "untitled.md").unwrap();
        assert_eq!(
            created.file_name().and_then(|name| name.to_str()),
            Some("untitled 1.md")
        );
        assert!(created.exists());

        let folder = tree
            .create_unique_directory(dir.path(), "New:Folder?")
            .unwrap();
        assert_eq!(
            folder.file_name().and_then(|name| name.to_str()),
            Some("New-Folder-")
        );

        let renamed = tree.rename_unique(&created, "daily.md").unwrap();
        assert_eq!(
            renamed.file_name().and_then(|name| name.to_str()),
            Some("daily 1.md")
        );

        let moved = tree.move_entry(&renamed, &archive).unwrap();
        assert_eq!(
            moved
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str()),
            Some("archive")
        );
        assert!(moved.exists());
        assert!(!renamed.exists());
    }

    #[test]
    fn latex_export_includes_metadata_blocks_math_code_and_tables() {
        let doc = MarkdownDocument::from_text(
            "---\ntitle: Export Doc\nauthor: Writer\ndate: 2026-06-30\n---\n# Intro\n\nInline $a+b$.\n\n```rust\nfn main() {}\n```\n\n$$\nx^2\n$$\n\n| A | B |\n|---|---|\n| 1 | 2 |",
        );
        let latex = doc.render_latex_document();

        assert!(latex.contains("\\title{Export Doc}"));
        assert!(latex.contains("\\author{Writer}"));
        assert!(latex.contains("\\date{2026-06-30}"));
        assert!(latex.contains("\\section{Intro}"));
        assert!(latex.contains("Inline $a+b$."));
        assert!(latex.contains("\\begin{lstlisting}\n"));
        assert!(latex.contains("fn main() {}"));
        assert!(latex.contains("\\[\nx^2\n\\]"));
        assert!(latex.contains("\\begin{longtable}{ll}"));
        assert!(latex.contains("A & B"));
    }

    #[test]
    fn inline_math_survives_extended_superscript_extension() {
        let doc = MarkdownDocument::from_text("Inline $a^2+b^2=c^2$ and x^2^ outside.");
        let html = doc.render_html_fragment();

        // The math payload must reach the annotator untouched...
        assert!(html.contains("data-latex=\"a^2+b^2=c^2\""));
        assert!(!html.contains("data-latex=\"a<sup>"));
        // ...while extended superscript still applies outside math.
        assert!(html.contains("x<sup>2</sup>"));
    }

    #[test]
    fn latex_export_preserves_inline_styles_alignment_and_task_lists() {
        let doc = MarkdownDocument::from_text(
            "Text **bold** *it* ~~gone~~ ==mark== x^2^ H~2~O `code` [link](https://e.com/p).\n\n| L | C | R |\n|:--|:-:|--:|\n| a | b | c |\n\n- [x] done\n- [ ] open\n\n```python\nprint(1)\n```\n",
        );
        let latex = doc.render_latex_document();

        assert!(latex.contains("\\textbf{bold}"));
        assert!(latex.contains("\\textit{it}"));
        assert!(latex.contains("\\sout{gone}"));
        assert!(latex.contains("\\hl{mark}"));
        assert!(latex.contains("\\textsuperscript{2}"));
        assert!(latex.contains("\\textsubscript{2}"));
        assert!(latex.contains("\\texttt{code}"));
        assert!(latex.contains("\\href{https://e.com/p}{link}"));
        assert!(latex.contains("\\begin{longtable}{lcr}"));
        // Consecutive task items share one environment, with checkbox symbols.
        assert_eq!(latex.matches("\\begin{itemize}").count(), 1);
        assert!(latex.contains("\\item $\\boxtimes$ done"));
        assert!(latex.contains("\\item $\\square$ open"));
        // listings-known language is named; the preamble carries the packages.
        assert!(latex.contains("\\begin{lstlisting}[language=Python]"));
        assert!(latex.contains("\\usepackage[normalem]{ulem}"));
        assert!(latex.contains("\\usepackage{listings}"));
    }

    // ── Supplementary tests: find_next_match / find_previous_match ──

    #[test]
    fn find_next_match_with_wrap_around() {
        let doc = MarkdownDocument::from_text("aaa bbb aaa bbb");
        let opts = SearchOptions::literal("aaa");

        // cursor at 0 → first match at 0
        let m = doc.find_next_match(&opts, 0, false).unwrap().unwrap();
        assert_eq!(m.range, 0..3);

        // cursor after first match → second match at 8
        let m = doc.find_next_match(&opts, 4, false).unwrap().unwrap();
        assert_eq!(m.range, 8..11);

        // cursor past last match, no wrap → None
        assert!(doc.find_next_match(&opts, 12, false).unwrap().is_none());

        // cursor past last match, wrap → back to first match
        let m = doc.find_next_match(&opts, 12, true).unwrap().unwrap();
        assert_eq!(m.range, 0..3);
    }

    #[test]
    fn find_previous_match_with_wrap_around() {
        let doc = MarkdownDocument::from_text("aaa bbb aaa bbb");
        let opts = SearchOptions::literal("aaa");

        // before=15 (end of text) → last match at 8
        let m = doc.find_previous_match(&opts, 15, false).unwrap().unwrap();
        assert_eq!(m.range, 8..11);

        // before=7 → first match at 0
        let m = doc.find_previous_match(&opts, 7, false).unwrap().unwrap();
        assert_eq!(m.range, 0..3);

        // before=0, no wrap → None
        assert!(doc.find_previous_match(&opts, 0, false).unwrap().is_none());

        // before=0, wrap → last match
        let m = doc.find_previous_match(&opts, 0, true).unwrap().unwrap();
        assert_eq!(m.range, 8..11);
    }

    #[test]
    fn find_next_and_previous_with_no_matches() {
        let doc = MarkdownDocument::from_text("hello world");
        let opts = SearchOptions::literal("xyz");

        assert!(doc.find_next_match(&opts, 0, true).unwrap().is_none());
        assert!(doc.find_previous_match(&opts, 11, true).unwrap().is_none());
    }

    // ── validate_latex ──

    #[test]
    fn validate_latex_accepts_valid_formula() {
        assert!(validate_latex("E = mc^{2}").is_ok());
        assert!(validate_latex("\\begin{pmatrix} a & b \\\\ c & d \\end{pmatrix}").is_ok());
    }

    #[test]
    fn validate_latex_rejects_empty() {
        assert!(validate_latex("").is_err());
        assert!(validate_latex("   ").is_err());
    }

    #[test]
    fn validate_latex_detects_unmatched_braces() {
        // extra closing brace
        let err = validate_latex("a}b").unwrap_err();
        assert!(err.contains("unmatched closing brace"));

        // unclosed opening brace
        let err = validate_latex("{a + b").unwrap_err();
        assert!(err.contains("unclosed brace"));
    }

    #[test]
    fn validate_latex_detects_mismatched_environments() {
        // 2 \begin{ but only 1 \end{ → count mismatch
        let err = validate_latex("\\begin{matrix} \\begin{matrix} a \\end{matrix}").unwrap_err();
        assert!(err.contains("mismatched"));
    }

    // ── MarkdownDocument::open / save / recovered ──

    #[test]
    fn open_reads_file_from_disk() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.md");
        fs::write(&path, "# Opened").unwrap();

        let doc = MarkdownDocument::open(&path).unwrap();
        assert_eq!(doc.text(), "# Opened");
        assert_eq!(doc.path(), Some(path.as_path()));
        assert!(!doc.is_dirty());
    }

    #[test]
    fn open_returns_error_for_missing_file() {
        let result = MarkdownDocument::open("/nonexistent/path/to/file.md");
        assert!(result.is_err());
    }

    #[test]
    fn save_writes_to_existing_path() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("save_test.md");

        let mut doc = MarkdownDocument::from_text("initial");
        doc.save_as(&path).unwrap();
        doc.set_text("modified");
        doc.save().unwrap();

        assert_eq!(fs::read_to_string(&path).unwrap(), "modified");
        assert!(!doc.is_dirty());
    }

    #[test]
    fn save_errors_when_no_path_set() {
        let mut doc = MarkdownDocument::from_text("no path");
        let err = doc.save().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidInput);
    }

    #[test]
    fn recovered_constructor_marks_dirty() {
        let doc = MarkdownDocument::recovered("# Recovered", None);
        assert_eq!(doc.text(), "# Recovered");
        assert!(doc.is_dirty());
        assert!(doc.path().is_none());

        let with_path = MarkdownDocument::recovered("text", Some(PathBuf::from("/tmp/old.md")));
        assert_eq!(with_path.path(), Some(Path::new("/tmp/old.md")));
        assert!(with_path.is_dirty());
    }

    // ── export_to: Markdown, PlainHtml, Latex ──

    #[test]
    fn export_to_markdown_writes_raw_text() {
        let dir = tempfile::tempdir().unwrap();
        let doc = MarkdownDocument::from_text("# Hello\n\nbody text");
        let path = dir.path().join("out.md");
        doc.export_to(&path, ExportFormat::Markdown).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "# Hello\n\nbody text");
    }

    #[test]
    fn export_to_plain_html_omits_css() {
        let dir = tempfile::tempdir().unwrap();
        let doc = MarkdownDocument::from_text("# Plain");
        let path = dir.path().join("out.html");
        doc.export_to(&path, ExportFormat::PlainHtml).unwrap();
        let html = fs::read_to_string(&path).unwrap();
        assert!(html.contains("<h1>Plain</h1>"));
        assert!(!html.contains("<style>"));
    }

    #[test]
    fn export_to_latex_produces_valid_document() {
        let dir = tempfile::tempdir().unwrap();
        let doc = MarkdownDocument::from_text("---\ntitle: Test\n---\n# Intro\n\nParagraph.");
        let path = dir.path().join("out.tex");
        doc.export_to(&path, ExportFormat::Latex).unwrap();
        let tex = fs::read_to_string(&path).unwrap();
        assert!(tex.contains("\\documentclass"));
        assert!(tex.contains("\\title{Test}"));
        assert!(tex.contains("\\section{Intro}"));
    }

    // ── Empty search query ──

    #[test]
    fn find_matches_returns_empty_for_empty_query() {
        let doc = MarkdownDocument::from_text("some text");
        let opts = SearchOptions::literal("");
        assert!(doc.find_matches(&opts).unwrap().is_empty());
    }

    #[test]
    fn replace_all_returns_zero_for_empty_query() {
        let mut doc = MarkdownDocument::from_text("some text");
        let opts = SearchOptions::literal("");
        let result = doc.replace_all_matches(&opts, "replacement").unwrap();
        assert_eq!(result.replacements, 0);
    }

    // ── Front matter edge cases ──

    #[test]
    fn front_matter_returns_none_for_plain_document() {
        let doc = MarkdownDocument::from_text("# Just a heading\n\nNo front matter here.");
        assert!(doc.front_matter().unwrap().is_none());
    }

    #[test]
    fn front_matter_with_dotdotdot_closing_delimiter() {
        let doc = MarkdownDocument::from_text("---\ntitle: Dots\n...\n# Body");
        let fm = doc.front_matter().unwrap().unwrap();
        assert_eq!(fm.title.as_deref(), Some("Dots"));
    }

    #[test]
    fn front_matter_with_windows_line_endings() {
        let doc = MarkdownDocument::from_text("---\r\ntitle: CRLF\r\nauthor: Win\r\n---\r\n# Body");
        let fm = doc.front_matter().unwrap().unwrap();
        assert_eq!(fm.title.as_deref(), Some("CRLF"));
        assert_eq!(fm.author.as_deref(), Some("Win"));
    }

    // ── Code highlighting edge cases ──

    #[test]
    fn highlight_code_with_empty_input() {
        let lines = highlight_code("", Some("rust"));
        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0][0].text, "");
    }

    #[test]
    fn highlight_code_with_no_language() {
        let lines = highlight_code("let x = 1;", None);
        // Function should not panic and should return valid output
        assert!(!lines.is_empty());
        assert!(!lines[0].is_empty());
    }

    // ── title_from_path ──

    #[test]
    fn title_from_path_extracts_filename() {
        assert_eq!(
            title_from_path(Some(Path::new("/tmp/notes.md"))).as_ref(),
            "notes.md"
        );
        assert_eq!(title_from_path(None).as_ref(), "Untitled.md");
    }

    // ── Recovery file error paths ──

    #[test]
    fn load_recovery_file_rejects_bad_format() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.md");
        fs::write(&path, "wrong-header\n---\nbody").unwrap();

        let err = load_recovery_file(&path).unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::InvalidData);
    }

    #[test]
    fn list_recovery_files_returns_empty_for_nonexistent_dir() {
        let files = list_recovery_files("/nonexistent/recovery/dir").unwrap();
        assert!(files.is_empty());
    }

    #[test]
    fn delete_recovery_file_handles_missing_file() {
        // Should not panic on non-existent file
        let _ = delete_recovery_file(PathBuf::from("/nonexistent/file.md"));
    }

    // ── Document stats ──

    #[test]
    fn stats_counts_words_lines_and_headings() {
        let doc = MarkdownDocument::from_text("# Title\n\nTwo words here.\n\n## Second heading");
        let stats = doc.stats();
        assert_eq!(stats.headings, 2);
        assert_eq!(stats.lines, 5);
        assert!(stats.words >= 5); // "Title", "Two", "words", "here.", "Second", "heading"
    }

    #[test]
    fn derived_cache_invalidates_after_edit() {
        // Guards the version-gated cache: a mutation must drop the cached
        // preview_blocks/outline/stats so the next read reflects the new text.
        let mut doc = MarkdownDocument::from_text("# One\n\nbody");
        assert_eq!(doc.outline().len(), 1);
        assert_eq!(doc.stats().headings, 1);

        doc.set_text("# One\n# Two\n\nbody");
        assert_eq!(doc.outline().len(), 2, "outline cache must refresh on edit");
        assert_eq!(doc.stats().headings, 2, "stats cache must refresh on edit");

        // A small in-place edit via replace_range must also invalidate.
        doc.replace_range(0..12, ""); // drop both "# One\n# Two\n" heading lines
        let blocks = doc.preview_blocks();
        assert!(
            !blocks
                .iter()
                .any(|b| matches!(b, PreviewBlock::Heading { .. })),
            "preview_blocks cache must refresh on edit"
        );
    }

    #[test]
    fn derived_cache_is_reused_between_reads() {
        // Sanity: repeated reads with no edit return consistent results
        // (the cache path, not a fresh parse, is exercised on the 2nd call).
        let doc = MarkdownDocument::from_text("# Title\n\n## Sub");
        let first = doc.outline();
        let second = doc.outline();
        assert_eq!(first, second);
        assert_eq!(doc.stats(), doc.stats());
    }

    #[test]
    fn direct_code_edit_is_utf8_exact_and_preserves_authored_fences() {
        let mut doc = MarkdownDocument::from_text("~~~~  rust extra\nlet 名称 = 1;\n~~~~");
        let block = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.editor, Some(VisualBlockEditor::Code { .. })))
            .expect("direct code block");
        let VisualBlockEditor::Code { payload, .. } = block.editor.unwrap() else {
            unreachable!()
        };
        let name_start = doc.text()[payload.source_range.clone()]
            .find("名称")
            .unwrap()
            + payload.source_range.start;
        let edit = doc
            .direct_visual_block_edit(name_start..name_start + "名称".len(), "emoji_😀")
            .expect("validated direct edit");
        assert!(doc.validate_visual_block_edit(&edit));
        assert_eq!(edit.range, name_start..name_start + "名称".len());
        assert_eq!(
            edit.inserted_range_after,
            name_start..name_start + "emoji_😀".len()
        );
        doc.replace_range(edit.range, &edit.replacement);
        assert_eq!(doc.text(), "~~~~  rust extra\nlet emoji_😀 = 1;\n~~~~");
    }

    #[test]
    fn direct_table_edit_reflows_once_preserves_alignment_and_restores_logical_cell() {
        let source = "before\n\n| A | B |\n| :--- | ---: |\n| x | y |\n\nafter";
        let mut doc = MarkdownDocument::from_text(source);
        let block = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.editor, Some(VisualBlockEditor::Table { .. })))
            .expect("direct table");
        let VisualBlockEditor::Table { cells } = block.editor.unwrap() else {
            unreachable!()
        };
        let cell = cells
            .iter()
            .find(|cell| cell.row == 1 && cell.column == 0)
            .unwrap();
        let edit = doc
            .direct_visual_block_edit(cell.field.source_range.clone(), "宽字符|值")
            .expect("table edit");
        assert_eq!(edit.range, block.source_range);
        assert_eq!(edit.replacement.matches('\n').count(), 2);
        assert!(edit.replacement.contains("宽字符\\|值"));
        assert!(edit.replacement.lines().nth(1).unwrap().contains(':'));
        assert!(doc.validate_visual_block_edit(&edit));

        doc.replace_range(edit.range, &edit.replacement);
        let field = doc
            .visual_editor_field_at(&edit.selection_after)
            .expect("selection remains in a direct field");
        assert_eq!(
            field.kind,
            VisualEditorFieldKind::TableCell { row: 1, column: 0 }
        );
        let table = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::Table { .. }))
            .unwrap();
        let VisualBlockKind::Table { alignments, .. } = table.kind else {
            unreachable!()
        };
        assert_eq!(
            alignments,
            vec![TableAlignment::Left, TableAlignment::Right]
        );
        assert!(doc.text().starts_with("before\n\n"));
        assert!(doc.text().ends_with("\n\nafter"));
    }

    #[test]
    fn direct_table_tab_targets_follow_fields_then_handoff() {
        let source =
            "before\n\n| A | B |\n| --- | --- |\n| x | y |\n\nafter";
        let doc = MarkdownDocument::from_text(source);
        let blocks = doc.visual_blocks();
        let table = blocks
            .iter()
            .find(|block| matches!(block.editor, Some(VisualBlockEditor::Table { .. })))
            .unwrap();
        let table_fields = table.editor.as_ref().unwrap().fields();
        assert_eq!(
            doc.visual_editor_tab_target(&table_fields[0].source_range, true),
            Some(table_fields[1].source_range.clone())
        );
        let last = table_fields.last().unwrap();
        let after = blocks
            .iter()
            .skip_while(|block| block.id != table.id)
            .nth(1)
            .unwrap();
        assert_eq!(
            doc.visual_editor_tab_target(&last.source_range, true),
            Some(after.source_range.start..after.source_range.start)
        );
    }

    // ── Diagram rendering ──

    #[test]
    fn mermaid_html_export_embeds_sanitized_static_svg_in_both_html_modes() {
        let doc = MarkdownDocument::from_text("```mermaid\nflowchart LR\nA[Start] --> B[End]\n```");
        let fragment = doc.render_html_fragment();
        assert!(fragment.contains("<div class=\"markion-diagram\""));
        assert!(fragment.contains("data-diagram-backend=\"mermaid\""));
        assert!(fragment.contains("<svg"));
        assert!(!fragment.contains("markion-diagram-placeholder"));
        assert!(!fragment.contains("<script"));
        assert!(!fragment.contains("onload="));

        let styled = doc.render_html_document();
        let plain = doc.render_plain_html_document();
        assert!(styled.contains(".markion-diagram svg"));
        assert!(plain.contains("<div class=\"markion-diagram\""));
        assert!(!plain.contains("<style>"));
    }

    #[test]
    fn mermaid_html_export_handles_multiple_diagrams_after_markdown_transforms() {
        let doc = MarkdownDocument::from_text(
            "```mermaid\nflowchart LR\nA[~~literal~~ ^label^] --> B\n```\n\nText ~~strike~~.\n\n```MERMAID extra\nsequenceDiagram\nAlice->>Bob: Hello\n```",
        );
        let html = doc.render_html_fragment();
        assert_eq!(html.matches("class=\"markion-diagram\"").count(), 2);
        assert!(html.contains("<del>strike</del>"));
        assert!(!html.contains("markion-diagram-placeholder"));
    }

    #[test]
    fn invalid_mermaid_html_falls_back_to_exact_escaped_source() {
        let source = "flowchart LR\nsubgraph MissingEnd\nA[\"<unsafe> & exact\"] --> B\n";
        let doc = MarkdownDocument::from_text(format!("```mermaid\n{source}```"));
        let html = doc.render_html_fragment();
        assert!(html.contains("<pre><code class=\"language-mermaid\">"));
        assert!(html.contains("A[\"&lt;unsafe&gt; &amp; exact\"] --&gt; B\n"));
        assert!(!html.contains("<svg"));
    }

    #[test]
    fn mermaid_keeps_code_block_ranges_copy_text_and_source_backed_payload_editor() {
        let markdown = "before\n\n```MerMaid linenos\nA --> B\n```\n\nafter";
        let doc = MarkdownDocument::from_text(markdown);
        let block = doc
            .preview_blocks()
            .into_iter()
            .find(|block| matches!(block, PreviewBlock::CodeBlock { .. }))
            .expect("code block");
        let PreviewBlock::CodeBlock {
            language,
            code,
            source_range,
        } = block
        else {
            unreachable!()
        };
        assert_eq!(language.as_deref(), Some("MerMaid"));
        assert_eq!(code, "A --> B");
        assert_eq!(&markdown[source_range], "```MerMaid linenos\nA --> B\n```");
        assert!(doc.plain_text_preview().contains("A --> B"));
        // A closed diagram fence is now source-backed through a Code payload
        // editor (so Visual Edit can layer the rendered diagram on top),
        // matching display-math blocks: `editor = Some(Code)`, no conservative
        // source island. The block's source range still fully covers the fence.
        let visual = doc
            .visual_blocks()
            .into_iter()
            .find(|block| matches!(block.kind, VisualBlockKind::CodeBlock { .. }))
            .expect("visual code block");
        let VisualBlockEditor::Code { payload, .. } =
            visual.editor.expect("diagram fence carries a Code editor")
        else {
            unreachable!()
        };
        let payload_range = payload.source_range.clone();
        assert_eq!(&markdown[payload_range.clone()], "A --> B\n");
        assert!(visual.source_range.start <= payload_range.start);
        assert!(payload_range.end <= visual.source_range.end);
        assert_eq!(&markdown[visual.source_range.clone()], "```MerMaid linenos\nA --> B\n```");
        assert!(visual.source_island.is_none());
    }

    #[test]
    fn mermaid_does_not_change_non_html_export_paths() {
        let doc = MarkdownDocument::from_text("```mermaid\nflowchart LR\nA --> B\n```");
        assert_eq!(doc.plain_text_preview(), "flowchart LR\nA --> B");
        let latex = doc.render_latex_document();
        assert!(latex.contains("flowchart LR"));
        assert!(!latex.contains("<svg"));
    }

    // ── replace_current_match edge case ──

    #[test]
    fn replace_current_match_ignores_stale_range() {
        let mut doc = MarkdownDocument::from_text("hello world");
        let opts = SearchOptions::literal("hello");
        // Pass a range that doesn't match any current result
        let result = doc.replace_current_match(100..105, &opts, "hi").unwrap();
        assert_eq!(result.replacements, 0);
        assert_eq!(doc.text(), "hello world"); // unchanged
    }

    #[test]
    fn visual_edit_documentation_contract_tracks_current_strategies() {
        let guide = include_str!("../docs/visual-editing-quality.md");
        for required in [
            "VisualBlockEditor::Code",
            "VisualBlockEditor::Math",
            "VisualBlockEditor::Table",
            "progressive source reveal",
            "YAML front matter",
            "Mermaid/registered diagrams",
            "pulldown-cmark",
            "SourceEdit",
            "VisualBlockId",
            "cargo test --workspace",
        ] {
            assert!(
                guide.contains(required),
                "missing contract marker: {required}"
            );
        }

        let english = include_str!("../README.md");
        let chinese = include_str!("../README.zh-CN.md");
        assert!(english.contains("docs/visual-editing-quality.md"));
        assert!(chinese.contains("docs/visual-editing-quality.md"));
        assert!(!english.contains("direct visual cell editing is not yet supported"));
        assert!(!english.contains("Direct cell-level Visual Edit table editing"));
        assert!(!chinese.contains("暂不支持直接可视化编辑单元格"));
        assert!(!chinese.contains("尚未实现直接可视化编辑表格单元格"));

        let context = include_str!("../openspec/config.yaml");
        assert!(context.contains("Root-package Cargo workspace"));
        assert!(!context.contains("Single crate, no workspace"));

        let editing_spec = include_str!("../openspec/specs/markdown-editing/spec.md");
        assert!(editing_spec.contains("WYSIWYG-oriented Visual Edit surface"));
        assert!(!editing_spec.contains("it is a future candidate"));

        let quality_spec = include_str!("../openspec/specs/engineering-quality/spec.md");
        assert!(quality_spec.contains("Visual Edit invariant evidence"));
        assert!(!quality_spec.contains("TBD - created by archiving change"));
    }
}
