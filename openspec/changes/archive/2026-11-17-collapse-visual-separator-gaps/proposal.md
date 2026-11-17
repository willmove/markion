## Why

Visual Edit paints the single blank line CommonMark uses to separate blocks as a full body line. Typora and Markion's Read / Split Preview treat that newline as paragraph spacing, so the same file looks like it has an extra empty row in Visual Edit (GitHub issue #19). The source newline is structural, not an extra authored blank line.

## What Changes

- A whitespace gap that sits between two blocks and contains exactly one newline occupies paragraph spacing only. It stays a source-backed caret target: click and Up/Down still land on that newline, and typing there inserts into the existing source without adding another newline.
- Each newline beyond that separator still occupies one body line, up to the existing pathological bound.
- A normal trailing newline no longer paints an extra blank line. The end-of-document insertion row stays addressable. Authored trailing blank lines (additional newlines after the block's own line ending) stay visible, and Enter at the end still grows them.
- Leading blank lines stay visible. Source bytes are unchanged. There is no new preference.
- Typing into a collapsed separator grows a real line, so content below moves down. That replaces the previous invariant that typing into every blank line kept the following content fixed.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Visual Edit whitespace rows, vertical navigation across a separator gap, and list-tail blank lines distinguish a structural separator from additional authored blank lines.

## Impact

- Visual Edit layout and caret mapping in `src/app/preview.rs` and `src/app/editing.rs`. Whitespace `height_signature` in `src/visual.rs` changes with the painted line count so the virtual list remeasures. Derived Markdown caches stay per document version; this does not recompute them on each keystroke.
- Read mode, Split Preview, and source editing are unchanged.
- Non-goals: no preference toggle, no source rewrite, no HTML-block margin restyle, no change to how many newlines CommonMark requires between blocks.
