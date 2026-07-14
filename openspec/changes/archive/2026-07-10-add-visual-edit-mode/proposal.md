## Why

Markion currently offers source editing, split preview, and read-only preview, but users who prefer an Obsidian-like writing flow still have to choose between raw Markdown control and a separate rendered pane. A Visual Edit mode gives users a single editing surface that looks close to the rendered document while keeping Markdown source text as the canonical document format.

Non-goals: this change does not attempt full Typora parity, rich-document storage, arbitrary rendered-tree editing, direct table cell editing, or perfect Reading-view fidelity for every Markdown extension in the first version.

## What Changes

- Add a fourth view mode, Visual Edit, alongside Edit, Split Preview, and Read.
- In Visual Edit, render an Obsidian Live Preview-style single editing surface: Markdown formatting is shown visually where feasible, while the underlying syntax remains available around the cursor or inside focused edit islands.
- Keep `MarkdownDocument.text` as the single source of truth; all visual edits mutate the existing Markdown text and use the existing undo/redo, dirty-state, autosave, recovery, and tab isolation paths.
- Scope v1 to source-backed visual editing for common prose blocks: headings, paragraphs, inline emphasis/strong/code/link/image syntax, blockquotes, unordered/ordered/task lists, and horizontal rules.
- Treat complex constructs as conservative source-backed islands or existing preview interactions in v1: fenced code blocks, math blocks, HTML/front matter, images, and GFM tables do not require direct rich editing to ship the first mode.
- Preserve the current split-preview workflow unchanged for users who want simultaneous source and rendered panes.
- Preserve core performance invariants: derived Markdown state stays cached per document version and shared via `Arc`, syntax highlighting remains memoized, and the editor continues reusing cached text handles per version.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `markdown-editing`: Extend editor view modes with Visual Edit and define source-backed visual editing behavior.
- `tables-outline`: Clarify how existing preview table controls behave in Visual Edit while direct cell-level visual table editing remains out of scope.

## Impact

- Affected code: `src/model.rs` (`ViewMode`), `src/main.rs` rendering/input/action routing, preview block/source mapping helpers, cursor/selection hit testing, menu/shortcut/status strings, and tests around view modes and editing operations.
- Possible supporting changes in `src/lib.rs` / parsing helpers to expose source ranges for visual-editable blocks and inline spans without changing persisted Markdown files.
- No new storage format or file migration is expected.
- No new heavy dependency is expected for v1; GPUI elements and existing Markdown parsing/rendering should be reused where practical.
