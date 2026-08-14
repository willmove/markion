## 1. Preview Heading Lookup

- [x] 1.1 Add a pure helper that maps an outline heading source offset to the exact cached `PreviewBlock::Heading` list index without parsing or allocating a second derived cache.
- [x] 1.2 Add lookup coverage for front matter offsets, formatted heading text, duplicate heading titles, and a missing exact match.

## 2. Context-Aware Outline Navigation

- [x] 2.1 Add an outline-specific application navigation method that preserves the canonical source-position update and existing non-Read reveal behavior.
- [x] 2.2 In Read mode, scroll the persistent preview `ListState` to the matched heading item with a zero in-item offset, without changing document text, version, dirty state, or history.
- [x] 2.3 Route outline row clicks through the context-aware method while retaining application focus, status feedback, and current behavior in Edit, Visual Edit, and Split Preview modes.

## 3. Interaction Verification

- [x] 3.1 Add focused GPUI coverage showing that a Read-mode outline navigation updates the canonical cursor and preview logical scroll target for the clicked heading.
- [x] 3.2 Add regression coverage showing that navigation remains non-mutating and that Edit, Visual Edit, and Split Preview retain their existing navigation paths.
- [x] 3.3 Run `cargo fmt --check`, `cargo test`, and `openspec validate outline-preview-jump`.
