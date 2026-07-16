## 1. Version-Cached Visual Syntax Metadata

- [x] 1.1 Add GPUI-independent reveal-group and structural-prefix types to the visual model for byte-exact strong, emphasis, strikethrough, inline-code, link, heading, blockquote, list, and task-list source ranges
- [x] 1.2 Populate reveal groups and block-prefix metadata during per-version visual block construction, rejecting non-boundary, nested, overlapping, or byte-inexact ranges through the conservative fallback path
- [x] 1.3 Add model tests for UTF-8 content, escapes, link destinations/titles, supported block prefixes, nested fallback, and unchanged `Arc<VisualBlock>` cache reuse

## 2. Mixed Rendered/Source Projection

- [x] 2.1 Implement an ephemeral per-visible-block projection that combines rendered content runs with identity-mapped local source groups selected from the active caret or selection endpoints
- [x] 2.2 Implement deterministic source/display boundary mapping for hidden markers plus projected selection and caret ranges without storing interaction state in `MarkdownDocument`
- [x] 2.3 Add focused projection tests for local marker reveal/hide, link source exposure, UTF-8 round trips, hidden-marker keyboard entry, cross-run selections, and cursor-only version/cache stability

## 3. Live Preview Rendering and Input

- [x] 3.1 Replace focus-driven full-block source islands with the mixed projection for supported paragraphs, headings, list items, and blockquotes while retaining explicit conservative islands for unsupported or ambiguous constructs
- [x] 3.2 Route pointer hit testing, drag selection, painted caret bounds, IME placement, and platform replacement through projected source mappings across reveal-induced layout changes
- [x] 3.3 Add GPUI rendered-window tests proving plain focused prose stays visual, only the active inline/link group exposes source, platform input edits canonical Markdown, and Read mode remains non-mutating

## 4. Structure-Aware Block Commands

- [x] 4.1 Add source-backed structural context helpers for heading splits, ordered/unordered/task-list continuation and exit, blockquote continuation and exit, and exact visible-content prefix boundaries
- [x] 4.2 Make Visual Edit Enter and collapsed-caret Backspace use the structural helpers, including nested-list outdent before top-level prefix removal, while preserving existing selection and non-Visual-mode paths
- [x] 4.3 Add source and application tests for every structural transition, UTF-8 boundary safety, one-step undo/redo, dirty/autosave behavior, per-tab isolation, and derived-cache invalidation only after text mutation

## 5. Regression and Performance Verification

- [x] 5.1 Confirm projections are built only for virtualized visible rows and add a large-document regression proving cursor-only reveal does not reparse Markdown or replace the cached visual-block `Arc`
- [x] 5.2 Run `cargo fmt --check`, `cargo test`, and `cargo test --workspace`
- [x] 5.3 Run `openspec validate improve-visual-edit-live-preview` and resolve all validation errors
