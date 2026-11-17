## Why

Colon emoji completion is fixed near the window's right edge, far from the text being edited, while slash completion already appears beside the caret. Matching their placement and presentation keeps block-start insertion predictable.

## What Changes

- Anchor emoji completion beside the current input using the same below-source placement and offset as slash completion, with viewport fitting.
- Share the compact themed popup shell and button dimensions with slash completion; show emoji glyph buttons with shortcode tooltips and an active-result caption.
- Preserve filtering, keyboard and pointer confirmation, Escape dismissal, source replacement, and undo behavior in Visual Edit and the source editor.
- Give recognized emoji shortcodes precedence over definition-list markers at line starts, preserving their glyph display in consecutive lines and while editing the owning block.
- Expand completion to the existing Markdown library's emoji catalog, retaining current aliases/glyphs and allowing keyboard navigation across all matching candidates through a compact candidate window.
- Increase the emoji popup's overall height to roughly twice the initial compact version by displaying six rows of six candidates (36 per window), retaining the existing width and button sizes.

Non-goals: changing slash actions, genuine definition lists, document persistence, or requiring network access for emoji lookup.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Colon completion uses a compact slash-consistent palette beside its input position.

## Impact

Touches application completion, shared Markdown event parsing and emoji lookup, with library/GPUI regression coverage. No external dependencies or data migrations change. Popup-only interactions must preserve document version and the existing shared per-version Markdown caches, syntax highlighting, and cached text handles.
