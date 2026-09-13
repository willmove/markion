## Context

Visual Edit already has collapsible payload editors for HTML blocks, inline images, diagrams, and Ready math. Four remaining everyday constructs still take the whole-block island path:

1. YAML is forced through `source_island(..., FrontMatter)` and `always_source`.
2. `visual_block_editor` for images requires a closing `)`, so `![alt][ref]` stays `editor: None` and `focused_conservative` islands it.
3. `table_cell_source_ranges` returns `None` when a row’s cell count ≠ separator columns, so ragged tables lose `VisualBlockEditor::Table`.
4. Display math with `editor: None`, or the MathBlock arm’s Pending/Error match, calls `visual_source_island_view` even though `visual_math_editor` already paints Pending/Error with a forced-open payload.

## Goals / Non-Goals

**Goals:** Close roadmap gaps 1, 4 (reference-style only), 5, and 11 so focus never replaces those four with island chrome.

**Non-Goals:** Indented/unclosed code, definition lists, multiline images, TOML/JSON front matter, Read-mode YAML chrome, converting reference images to inline on presentation edits.

## Decisions

### YAML is a collapsible header, not an island

Add `VisualBlockKind::FrontMatter` + `VisualBlockEditor::FrontMatter { payload }` covering `0..body_start` (complete authored `---` … `---`/`...` including delimiters). Collapsed chrome is a compact header (localized YAML label plus parsed `title` when present). Expanded / caret-in-payload uses the same collapsible source widget as HTML. Invalid YAML still edits; it does not island. `always_source` drops `FrontMatter`. Turn Into stays unavailable (`visual_block_transform` already returns `None` for unmatched kinds).

### Reference-style block images reuse the Image payload editor

Extend the whole-span proof to single-line reference forms: `![alt][label]`, `![alt][]`, and shortcut `![alt]`. The payload is the complete authored span; destination fingerprint still comes from `PreviewBlock::Image` identity. `inline_image_at` stays `LinkType::Inline` only, so width/alignment controls remain hidden and cannot serialize a reference into parentheses form. Multiline and unclosed destination scans stay islands.

### Ragged tables pad cell ranges

`table_cell_source_ranges` uses `max(separator columns, max body-row cell counts)` (same as `MarkdownTable::normalize`). Short rows get empty insertion ranges at the line’s content end (UTF-8 boundary, before the newline). Extra cells beyond the separator are kept. `visual_block_editor` then attaches `VisualBlockEditor::Table` whenever preview rows exist. Unfocused grid behavior is unchanged.

### Math Pending/Error never islands

If delimiter/payload split fails, still attach `VisualBlockEditor::Math` with `payload = source_range` (empty opening/closing). The MathBlock view always routes through `visual_math_editor`, which already forces the payload open until Ready. Delete the Pending/Error `visual_source_island_view` fallback.

## Risks / Trade-offs

- **[YAML whole-span payload includes delimiters]** → Deleting `---` can drop the header on the next parse; same contract as HTML source. Prefer this over a payload-only editor that cannot repair a broken closing fence.
- **[Empty padded table cells insert at line end]** → Typing in a missing cell may not insert a `|` until Format/toolbar reflow. Acceptable: the row stays a table, not an island; Format already normalizes.
- **[Shortcut `![alt]` vs empty ATX-like ambiguity]** → Only prove when the preview classified the block as `PreviewBlock::Image`.

## Migration

None. Existing documents keep the same source. Visual Edit presentation of those four constructs changes.
