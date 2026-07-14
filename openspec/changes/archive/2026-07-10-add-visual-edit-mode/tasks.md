## 1. Mode Wiring

- [x] 1.1 Add `VisualEdit` to `ViewMode`, default cycling, status mapping, and mode-preservation tests
- [x] 1.2 Add Visual Edit actions, menu item, shortcut, shortcut-reference text, and localized strings
- [x] 1.3 Update render layout width/visibility logic so Visual Edit shows one editing surface while Edit, Split Preview, and Read remain unchanged
- [x] 1.4 Add tests for direct mode switching and cycle order across Edit, Visual Edit, Split Preview, and Read

## 2. Source-Ranged Visual Model

- [x] 2.1 Define visual edit block/run data types with source ranges, content ranges, style metadata, and conservative fallback markers
- [x] 2.2 Build visual blocks from the current Markdown source for headings, paragraphs, blockquotes, lists/task lists, rules, links, images, inline styles, and plain text
- [x] 2.3 Add conservative source-island representation for fenced code blocks, math blocks, HTML/front matter, images where needed, and unsupported or ambiguous syntax
- [x] 2.4 Cache visual edit blocks per `MarkdownDocument.version()` and invalidate only on text mutation
- [x] 2.5 Add unit tests for visual block source ranges, inline run mapping, nested inline fallback behavior, and cache reuse/invalidation

## 3. Visual Edit Rendering

- [x] 3.1 Implement a Visual Edit GPUI element or surface that renders visual blocks with theme-consistent typography and spacing
- [x] 3.2 Render headings, paragraphs, blockquotes, unordered/ordered/task lists, inline strong/emphasis/code/link/image syntax, and rules visually
- [x] 3.3 Render source islands for complex blocks with source text, cursor affordance, and existing code highlighting where practical
- [x] 3.4 Preserve large-document behavior by avoiding full visual-block recomputation on cursor, hover, selection, or focus-only updates
- [x] 3.5 Add visual render tests or focused helper tests for block classification and unsupported-syntax fallback

## 4. Editing, Selection, and Input

- [x] 4.1 Map pointer and keyboard focus in Visual Edit from visual positions to source ranges
- [x] 4.2 Route text insertion, deletion, paste, IME composition, and select-all through existing source-backed mutation paths
- [x] 4.3 Reuse existing formatting actions in Visual Edit and verify they update Markdown markers in source
- [x] 4.4 Reveal focused Markdown syntax or enter source islands for precise editing of inline styles, links, images, code, math, and unsupported constructs
- [x] 4.5 Preserve undo/redo, dirty state, autosave/recovery scheduling, and per-tab selection isolation after Visual Edit mutations

## 5. Table Behavior

- [x] 5.1 Render GFM tables as visual grids in Visual Edit using the same source table data as the preview
- [x] 5.2 Wire Visual Edit table toolbar buttons to the existing source table edit commands
- [x] 5.3 Keep direct cell-level visual editing out of v1 and provide a source-backed edit/fallback path for table cell text
- [x] 5.4 Add tests for Visual Edit table toolbar operations updating source Markdown and preserving table command behavior elsewhere

## 6. Verification

- [x] 6.1 Run `openspec validate add-visual-edit-mode`
- [x] 6.2 Run targeted Rust tests for view modes, visual block mapping, formatting actions, and table edit helpers
- [x] 6.3 Run `cargo test`
- [x] 6.4 Manually verify mode switching, visual prose editing, focused syntax reveal, source islands, table toolbar operations, and unchanged Split Preview behavior
