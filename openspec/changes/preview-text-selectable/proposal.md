## Why

The rendered preview pane (Split Preview and Read modes) currently paints text with non-selectable GPUI text elements, so users cannot drag-select or copy quoted passages, code, or headings from the preview. That forces a detour through the source editor even when the user only wants the rendered wording. Enabling selection and copy in the preview closes that gap without turning the preview into an editor.

Non-goals: this change does not make the preview editable (no insert/delete/cut/paste-into-document), does not add rich HTML/Markdown clipboard formats, does not change Markdown parsing or derived-state caching, and does not require cross-block multi-selection spanning the entire document in one gesture if GPUI constraints make that impractical in v1.

## What Changes

- Make rendered preview text selectable by mouse drag (and platform-appropriate keyboard selection where feasible) in Split Preview and Read modes.
- Allow copying the selected preview text to the system clipboard via the existing Copy shortcut / Edit→Copy path when the preview holds the selection focus.
- Keep Read/Split preview non-editable: selection and copy MUST NOT mutate document text, dirty state, or undo history.
- Preserve existing preview interactions that already work (link clicks via `InteractiveText`, scrolling, outline jump targets) alongside selection.
- Cover the common textual block types users copy from (headings, paragraphs, list item text, blockquotes, code blocks, math fallback/source text, table cell text, image captions/URLs as shown). Decorative markers (list bullets, code line numbers) MAY be omitted from the copied plain text when that yields cleaner output.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Extends the editor view-mode / preview-pane requirements so the rendered preview supports text selection and copy while remaining non-editable.

## Impact

- Affected code: primarily `src/main.rs` preview rendering (`preview_block_view`, `rich_text_element`, code/math/table text helpers) and any Copy / focus routing that today assumes only the source editor owns selection; possibly small helpers in `src/lib.rs` / model types if plain-text extraction for a selection range is needed.
- No new crate dependencies expected; solution should stay within GPUI text/selection primitives (or a thin app-level selection overlay if stock `StyledText` cannot select).
- Invariants to preserve: derived Markdown state (preview blocks, outline, stats) remains cached per document version and shared via `Arc`; syntax highlighting stays memoized; preview `ListState` virtualization/splice behavior must not be forced into full rebuilds on every selection change; Read mode still MUST NOT edit through the preview.
