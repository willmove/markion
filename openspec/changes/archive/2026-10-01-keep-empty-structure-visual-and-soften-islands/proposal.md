## Why

In Visual Edit, a line that is only an ATX marker (`##`, `###     `, or any `#`–`######` plus optional spaces) — and the same family of empty list markers (`- `, `1. `) — is dropped from the preview model because its payload text is empty. Visual Edit then treats the uncovered bytes as an `Unsupported` source island: a bordered, padded, monospace gray box. The same box also wraps remaining true source-only blocks (front matter, unclosed fences, residual gaps), so a one-line heading placeholder looks like a code block. Typora and Obsidian keep these rows as headings/lists, reserve heading/list height when unfocused, and reveal the marker when focused. Markion should do the same, and the leftover island chrome should stop competing with prose.

## What Changes

- Keep empty ATX headings (levels 1–6, including trailing spaces only) in the derived preview and visual-block streams instead of dropping them. Visual Edit, Read, and Split Preview reserve heading-sized space for them when unfocused (no gray box, no disappearing row). Outline continues to expose them as jump targets.
- Keep empty unordered and ordered list items the same way. Empty task-list items already survive derivation; they stop promoting to a source island when they own the caret.
- When the caret (or a selection endpoint) is on an empty heading or empty list row, Visual Edit reveals the structural prefix (`## `, `- `, `1. `, task marker) through the existing progressive-reveal mapping, keeps heading/list typography, and paints a caret in that row. It does not replace the row with a whole-block source island.
- Stop routing “empty `editable_runs` plus caret” through `visual_source_island_view` for blocks that already have a rendered kind and no `source_island`. That gate stays only for genuine source-only blocks (and the existing whitespace / callout / reference-definition exemptions).
- Soften chrome for remaining source islands (front matter, indented/unclosed code, residual unsupported gaps that are not empty structure): keep an exact source-backed editing affordance, but drop the poster-like padded bordered box so height and font no longer jump like a code block inserted into prose.
- Close the WYSIWYG coverage-roadmap gap for empty list items, add empty ATX headings as a closed construct rather than a newly discovered gap, and update the coverage matrix so residual islands are a lighter transitional view, not the current heavy box.

### Non-goals

Closing other roadmap gaps (YAML front-matter form, indented-code payload editor, unclosed-fence highlighting, malformed tables, definition lists, task-checkbox click). Changing setext headings. Changing Source-mode editing. Guessing rendered-tree mutations for byte-ambiguous syntax. Redesigning fenced-code / math / diagram / HTML payload editors.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: Empty ATX headings and empty list items stay rendered (with reserved unfocused height) and reveal their structural prefix when focused; remaining source islands keep a source-backed affordance with lighter chrome; coverage matrix and roadmap update.

## Impact

- **Code:** `src/parse.rs` (`push_nonempty_block`, `flush_list_item`); `src/lib.rs` preview derivation (same nonempty gate on the full-document path); `src/visual.rs` (gap classification, heading/list prefix derivation, prefix reveal when the caret sits at prefix end); `src/app/preview.rs` (`focused_conservative` / `always_source`, heading and list render arms for empty payload, `visual_source_island_view` chrome); Read/Split Preview heading and list painting so empty rows still occupy layout; `docs/visual-editing-quality.md` matrix and roadmap.
- **Invariants:** Derived preview, outline, visual blocks, and stats remain cached per document version and shared via `Arc`. Caret/focus/reveal MUST NOT invalidate those caches. Mutations still flow through canonical `MarkdownDocument.text`. `crates/*` stay GPUI-free.
- **Tests:** Pure model tests for empty ATX headings and empty list items (kind, source range, no `Unsupported` island, prefix ranges); projection tests that focusing the row reveals the marker; GPUI tests that the focused/unfocused row is not a source-island box; Read/preview tests that empty headings/lists occupy space. Remaining islands still edit source and round-trip.
