## Context

See `proposal.md` for motivation. The reports identify narrow defects in existing paths rather than missing product requirements. Several fixes interact with source ranges, focus routing, asynchronous preview layout, session snapshots, or generated-file internals, so each must preserve canonical source identity and existing derived-state caching.

## Goals / Non-Goals

**Goals:**
- Correct only the failing behavior in each issue while keeping adjacent controls and formats stable.
- Make source edits atomic and undoable, and keep navigation/find operations non-mutating.
- Feed preview, Visual Edit, search, and exports from equivalent parsed semantics where the reports expose divergent interpretations.
- Keep expensive derivation versioned and cached; use layout snapshots only for presentation and navigation.
- Verify exported OOXML/PDF structures, not merely successful file creation.

**Non-Goals:**
- No general parser replacement, PDF/UA conformance project, new UI focus framework, or session-format migration.
- No full-document comparison on every keystroke, synchronous formula rendering on the typing path, or unbounded history/export output.
- No dependency additions unless an existing declared capability is proven insufficient and the plan is revisited.

## Decisions

### Input dispatch and source edits (#107–#109, #114–#118, #123, #135–#139)

Route keyboard commands by the concrete focused control before applying document-level defaults. Replace All and related search buttons invoke their own actions; Find/Replace text fields retain match navigation. Inline name editing owns undo/copy/cut without mutating the active document.

Canonical byte ranges remain the unit of mutation. Ordered-list continuation accepts the same marker whitespace as the parser. Emoji completion extends the replacement over the recognized suffix only when it is part of the existing shortcode, preserving an independently authored colon. Image destinations replace the correct inner/outer range with exactly one delimiter pair, titles escape entity-opening ampersands for a lossless reparse, and slash-inserted standalone blocks establish safe boundaries on both sides. Backward word navigation segments sufficient whole-line context and selects the boundary covering the caret rather than reparsing only its prefix.

Drag/drop computes the moved source range first, then remaps the complete selection to the resulting range. Close decides from current text versus the saved baseline at the decision point rather than trusting an edit-only dirty transition; undo/redo retain their existing efficient baseline updates.

### Math, reference links, preview text, and Find (#110–#112, #118, #120, #122, #124–#126, #134)

Keep exactly delimited display math as a semantic block even when its payload is invalid, allowing the existing local renderer error and payload editor to remain. Validation must ignore escaped literal braces and defer actual TeX validity to the renderer. Reference links use document-level parsed definitions in Visual Edit, and HTML copy supplies relevant original definitions to the selected fragment rather than reparsing it without context.

Preview text cleanup trims Markdown layout whitespace without discarding significant U+00A0. Definition-list terms and definitions expose their visible text to the existing body-run search. Search reveal considers both match endpoints for folded code and scrolls the active end into view; when pending preview blocks become ready, installing matches also performs the normal current-result reveal. Multiline table-cell rendering retains fragment-to-source offsets so hit testing does not collapse a whole cell to one source unit.

A leading UTF-8 BOM is treated as an encoding signature for Markdown derivation and is preserved on save; interior U+FEFF remains content. Derived blocks, outline, stats, and search continue to be produced once per document version and shared through the existing `Arc` caches. No extra full parse is introduced for layout, search navigation, or cursor movement.

### Paging and image resizing (#115–#116, #121)

Source paging explicitly reveals the new caret/selection endpoint in the source viewport independently of Visual Edit and typewriter scrolling. Visual paging carries unresolved vertical distance through pending layout work instead of consuming iterations against unmeasured blocks. Tests exercise repeated forward/backward paging and selection paging after fresh layout.

Image resize captures the dragged wrapper's identity and initial width at drag start; move events use that captured measurement rather than whichever wrapper appears first. Saving still updates only the target image's width attribute.

### Workspace sessions and Git history (#127–#129)

A workspace switch snapshots the outgoing root before removing its clean tabs, changes the selected root before restored-tab focus can persist a session, and makes initial-tab replacement a one-shot decision based on the original tab rather than newly restored clean tabs. Retained dirty documents from other roots remain untouched. Existing session data remains compatible.

File-history entries carry the path used at each commit alongside the current worktree path. Detail and comparison use the historical path for the historical side and the current path for the working side, retaining literal path arguments, rename detection, output bounds, and non-mutating Git reads. `crates/git-sync` remains GPUI-free.

### Code copy and built-in exports (#119, #130–#133, #140)

All code-copy surfaces use parsed code payload semantics: trailing blank lines remain, while up to the opening fence's indentation margin is removed from each payload line according to CommonMark. Visual Edit retains exact authored source for editing even though Copy uses the semantic payload.

Built-in DOCX ordered-list instances carry their resolved start number into numbering definitions/overrides, and nested bullets do not terminate an enclosing ordered group. Tests inspect effective numbering XML for top-level, nested, separated, and mixed-kind lists.

Built-in PDF embeds local-image alt text in a minimal tagged structure/alt-description path without changing image pixels or claiming full PDF/UA conformance. Missing-image fallback remains visible text. Text shaping applies font mark offsets once in glyph space and converts to points once, preserving advances and precomposed controls. The PDF crate remains GPUI-free and deterministic apart from already-established metadata behavior.

## Data flow and caching

Canonical document edits update the document version once through existing edit transactions. Versioned derivation then supplies preview blocks, outline, search text, and exports; unchanged versions reuse cached `Arc` results and syntax/text handles. Focus, folding, drag measurements, paging snapshots, and session restoration remain presentation or application state and never create document versions. Git history and export fixes operate on already-derived or repository data and do not invalidate unrelated tabs.

## Risks / Trade-offs

- [Global Markdown context or range remapping is broader than the fixture] → Compare incremental derivation with a fresh parse and test neighboring blocks, Unicode, LF/CRLF, undo, and redo for each affected editor path.
- [Asynchronous layout makes navigation tests timing-sensitive] → Use the existing test-platform frame/layout pumping and assert both caret and viewport after pending work settles.
- [Session-order changes could overwrite another root's snapshot] → Test repeated A→B→A→B switches, multiple remembered tabs, retained dirty external tabs, and active-path restoration.
- [DOCX/PDF generator changes may alter unrelated output] → Inspect package/dictionary structure and control fixtures, and run the complete member-crate suites.
- [Historical paths may not map cleanly through every rename/edit combination] → Test pure renames, rename-plus-edit, ordinary same-name edits, missing historical blobs, and bounded/binary fallbacks.

## Migration Plan

Implement and verify issues sequentially in independent, testable chunks. No persisted-format migration is required. If a chunk regresses adjacent behavior, revert only that chunk; earlier verified fixes remain valid. Finish with formatting, whitespace checks, root tests, and `cargo test --workspace` before reporting completion.
