## Why

Visual Edit currently treats authored blank lines (`VisualBlockKind::Whitespace`) as dead inter-block spacing: clicks do nothing, Up/Down skip them, and the row is only 12px tall with a caret that is easy to miss. Source mode still lets the user click and arrow into those same bytes. That mismatch is especially painful in heading-heavy documents (changelogs, outlines) where the only place to insert a new paragraph is the blank line between headings. Canonical Markdown is the source of truth — if a `\n` exists in the document, Visual Edit must be able to walk into it without inserting another one.

## What Changes

- Treat Visual Edit `Whitespace` rows as first-class empty lines, not CSS-like gaps: they occupy body paragraph line height, show a visible insertion caret when they own the caret, and accept pointer and keyboard entry.
- Restore click-to-enter: clicking a blank-line row moves the caret onto an existing source offset in that range. The click MUST NOT insert a newline or otherwise mutate the document.
- Restore arrow-to-enter: Up/Down (and their selection variants) land on a blank-line row as a navigation stop instead of skipping it. A second press continues into the block on the far side. Preferred horizontal coordinate is retained.
- Paint before navigation: each covered newline uses the rendered body paragraph line height (not the current 12px constant), so parking the caret on a blank line is visibly an empty paragraph, not a no-op.
- Reconcile the contradictory `markdown-editing` requirements that currently both forbid gap clicks and claim blank lines are reachable by clicking.
- Update the Visual Edit coverage matrix row for blank lines. This is still rendered WYSIWYG — not a roadmap-gap closure.

### Non-goals

- Notion-style “click the gap to insert a new paragraph” (click is caret placement only).
- Changing Source, Split Preview source pane, or Read/Split Preview compact block spacing.
- Collapsing or rewriting extra blank lines on save.
- Empty ATX headings / empty list items (already handled by `keep-empty-structure-visual-and-soften-islands`).
- Moving heading `mt`/`mb` into the whitespace row, or inventing empty-paragraph AST nodes.
- Quote-prefix structural edits beyond landing the caret on the existing whitespace source.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Visual Edit whitespace is an editable empty line (body line height, click and arrows enter, caret on existing `\n`). Replaces the passive-gap-click and skip-gap-row contracts.
- `document-typography`: Visual Edit authored blank lines use the rendered body paragraph line height. Changing that size reflows blank-line rows without mutating source.

## Impact

- **Code:** `src/app/preview.rs` (Whitespace render arm, pointer handlers, `whitespace_row_height` / list height estimates); `src/app/editing.rs` (`move_visual_vertical` skip-gap loop); tests in `src/app/tests.rs` that currently lock passive clicks and skip-gap arrows; `docs/visual-editing-quality.md` matrix row.
- **Invariants:** Derived preview / visual-block / outline / stats caches stay per document version and `Arc`-shared. Clicking or arrowing into a blank line is interaction-only (selection + caret ownership) and MUST NOT bump document version or rebuild those caches. Typing still goes through canonical `MarkdownDocument.text`. `crates/*` stay GPUI-free.
- **Compatibility:** Source bytes are unchanged until the user types. Documents with many heading-to-heading blank lines will grow vertically in Visual Edit because 12px gaps become body line height.
