## Why

Visual Edit presents inline images with a three-row editable field table (Alt Text, Destination, optional Title) beneath the preview. This table is visually noisy and exposes low-level Markdown syntax (escaped URLs, quoting) that most users do not want to edit inline. The same information is already available—and more naturally edited—in source mode. Simplifying the visual surface to a single read-only caption lets the preview stay focused on content.

## What Changes

- Remove the editable Alt Text / Destination / Title field table from the Visual Edit image block.
- Show a single read-only caption line beneath the image: the image **Title** when one is authored, otherwise the **Alt Text** (when non-empty). No caption line is shown when neither exists.
- Editing any image field (alt, destination, title) now requires switching to source editing. Visual Edit no longer hosts direct text controls for image fields.
- **BREAKING** (spec-level): the "Direct Markdown image editing in Visual Edit" requirement is replaced by a "Read-only image caption in Visual Edit" requirement. Inline images are no longer directly editable in Visual Edit.
- The `VisualBlockEditor::Image` editor variant and its field-range parsing (`image_field_ranges`) are removed. Inline images fall back to the existing source-backed image island rendering, augmented with the caption line.

## Non-goals

- Changing how images render in Split Preview or Read mode.
- Changing image parsing, source mapping, or HTML export.
- Removing the `ImageAlt` / `ImageDestination` / `ImageTitle` `VisualEditorFieldKind` variants or their sanitization logic from the codebase (they remain for potential future use; only the `Image` editor variant that consumed them is removed).

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `markdown-editing`: Replace the "Direct Markdown image editing in Visual Edit" requirement with a read-only caption requirement. Images are no longer directly editable in Visual Edit.

## Impact

- **`src/visual.rs`**: Remove `VisualBlockEditor::Image` construction from `visual_block_editor`; remove `image_field_ranges`. Update tests that assert on `VisualBlockEditor::Image`.
- **`src/model.rs`**: Remove `VisualBlockEditor::Image` variant; update `fields()` match.
- **`src/app/preview.rs`**: Remove `visual_image_editor` and `visual_image_field_row`. Add caption rendering to the image fallback path. Remove the `VisualBlockKind::Image` branch that delegated to `visual_image_editor`.
- **`src/app/editing.rs`**: Remove `ImageAlt`/`ImageDestination`/`ImageTitle` from the `insert_newline` field match (they are no longer reachable through an image editor).
- **`src/lib.rs`**: Update `visual_editor_tab_target` / `visual_editor_edge_target` (no more `VisualBlockEditor::Image`). Update tests that destructure or assert on `VisualBlockEditor::Image`.
- **`src/app/tests.rs`**: Update the image Tab-traversal test (`visual_direct_image_fields_traverse_sanitize_and_stay_tab_local`) which relied on editable image fields.
- **`openspec/specs/markdown-editing/spec.md`**: Delta spec replaces the direct-editing requirement.
- **Invariants touched**: none of the caching/identity invariants change; only the Visual Edit presentation and editing surface for images changes.
