## 1. Compatibility corpus and reader selection

- [ ] 1.1 Assemble 20–30 redistributable/sanitized representative DOCX files plus small edge-case fixtures; record Word/WPS/LibreOffice producer versions and provenance, expected accepted-view text/order/structures, images, and diagnostics in `docs/docx-import-compatibility.md`. Include native Markion and MarkNice HTML-compatibility exports without treating either as the sole oracle.
- [x] 1.2 Evaluate a pinned released `docx-rs` reader in a disposable headless harness against required styles, numbering, relationships, footnotes, revisions, OMML access, unknown-content inventory, bounded decompression, cancellation, and memory behavior; record observed gaps and optional comparisons with the existing MarkNice parser/Pandoc.
- [x] 1.3 Record the selected reader, exact version/features/license, limits evidence, and rationale in the design/compatibility report; choose the focused `zip` + `quick-xml` reader if library gaps defeat the contract, and remove disposable or rejected alternatives before integration.

## 2. Isolated importer and bounded package reading

- [x] 2.1 Add GUI-free `markion-docx-import` under `crates/docx-import`, the root dependency and optimized dev-profile override, and typed prepared-import/assets/diagnostics/errors/cancellation boundaries; verify the member builds and tests without GPUI.
- [x] 2.2 Implement package-type and main-document discovery using content types, relationships, and namespace URIs; test supported packages, misleading extensions, DOC/DOCM/encryption, missing required parts, duplicate conflicting entries, and malformed XML.
- [ ] 2.3 Enforce compressed/actual decompressed/XML/output/entry/structure limits plus cancellation and deadlines during bounded work; test exact boundaries, misleading archive metadata, deep structures, and cancellation inside reader/serializer stages.
- [x] 2.4 Implement package-local relationship resolution and unsupported-content inventory; test traversal, host-file/external-image references, unsafe links, entities, `altChunk`, fields, and embedded objects with no external I/O or execution.

## 3. Text, styles, numbering, and tables

- [x] 3.1 Convert paragraphs and runs with text/order/meaningful whitespace preservation and context-aware Markdown escaping; cover Chinese/Latin text, delimiters, explicit breaks, and adjacent mixed formatting in golden semantic tests.
- [x] 3.2 Resolve direct/inherited heading and run styles with cycle/depth bounds; serialize headings and supported emphasis/underline/sup/sub with diagnostics for deeper headings and normalized layout.
- [x] 3.3 Map safe hyperlinks and internal bookmarks to stable destinations; test Unicode and delimiter-containing URLs, missing bookmarks, and rejected schemes retaining readable link text.
- [x] 3.4 Implement nested ordered/unordered lists, start/restart/continuation and significant custom labels; test lists separated by paragraphs, multi-digit markers, inherited numbering and Roman/Chinese labels without flattening hierarchy.
- [x] 3.5 Convert rectangular/headerless tables to escaped GFM, preserving body rows and supported cell content; verify output through the shared Markdown parser.
- [x] 3.6 Convert supported merged tables into one safe HTML block; test horizontal/vertical spans and LF/CRLF output through existing table consumers, with explicit fallback/diagnostics for nested or unsupported structures.

## 4. Images, notes, equations, and fidelity

- [x] 4.1 Read and validate supported embedded raster assets with byte/count/dimension/decoder limits, deduplicate identical bytes, and preserve alt text/order; test broken media, unsupported formats, floating placement, external images, and bounded animated-image handling.
- [x] 4.2 Convert footnotes with deterministic labels, repeated references and multi-paragraph supported definitions; test separator-note exclusion and missing/unsupported note content diagnostics.
- [x] 4.3 Convert the defined OMML subset to inline/display LaTeX and validate with the existing math path; add exact expected fixtures and visible diagnosed fallbacks for unsupported structures.
- [x] 4.4 Apply accepted-view inline insert/delete/move/current-format semantics and emit a revision-policy summary; test content expectations independent of converter-generated output.
- [x] 4.5 Handle accepted-view paragraph/block boundary revisions and detect ambiguous revisions; test paragraph joins/splits and move wrappers without omission or duplicate moved text.
- [x] 4.6 Complete informational/formatting/content-loss diagnostic classification and source-location context, unsupported nonempty-node inventory and readable fallbacks; test comments/endnotes/headers/text boxes/objects/fields and fatal placeholder-only or empty results.

## 5. Managed resource staging and create-only publication

- [x] 5.1 Add a batch staging adapter reusing managed-resource naming and relative-URL rules; materialize typed asset references for the chosen target, with tests proving authored text cannot be changed by placeholder substitution.
- [x] 5.2 Validate new `.md` destinations, absent asset paths, source/path aliases, open-tab conflicts, and link/junction containment; add collision tests including Chinese and space-containing parent paths.
- [x] 5.3 Implement same-filesystem staging and no-clobber assets-first/Markdown-last publication; test races after initial validation, complete-file publication, repeated imports, deduplication, and text-only imports without empty asset directories.
- [x] 5.4 Implement ownership-aware rollback and accurate retained-path diagnostics; inject failures at each precommit stage, simulate termination before Markdown commit, and verify cleanup never deletes substituted or unrelated files.

## 6. Native import flow and editor integration

- [x] 6.1 Add localized Import Word actions to native and in-app File menus plus a DOCX picker and `.md` save-dialog profile; keep File → Open, drag/drop, shortcuts, export preferences, and browser import behavior unchanged.
- [x] 6.2 Add a dedicated import coordinator with one bounded background job, progress, cancellation, stale-result rejection, and the non-cancellable saving boundary; test repeated invocation, tab switching and application/job shutdown.
- [x] 6.3 Build the pre-save conversion report with counts, revision policy, severity groups and location context; test fatal/no-content states, ordinary Save as Markdown, and deliberate continuation with content losses without default Enter acceptance.
- [ ] 6.4 Connect report acceptance, destination selection, staging and commit; test source-picker/report/save-picker cancellation, recoverable destination conflicts and persistence error presentation.
- [ ] 6.5 Open committed output in a new clean tab through the normal document/session/recent-file lifecycle; test active dirty-tab preservation, normal subsequent undo/save/recovery, and reporting of an already-saved path if tab creation fails.
- [x] 6.6 Complete all supported language strings through the current i18n entry point and verify menu/report/status parity while leaving document content and diagnostic excerpts untranslated.
- [ ] 6.7 Add GPUI/document invariant tests for success, review, cancellation, errors and stale results: existing text/version/dirty/selection/undo/syntax memoization/cached-text/derived `Arc` identities remain unchanged, and only the new tab initializes normal caches after commit.

## 7. Documentation and verification

- [x] 7.1 Add `docs/word-import.md` and equivalent README/README.zh-CN.md coverage for scope, accepted revisions, the report, new destinations, `.assets` portability, limits, unsupported formats, cancellation/cleanup/crash remnants, and the distinction from browser-session import.
- [ ] 7.2 Run the complete semantic corpus and saved-output reopen checks, including Source/Split/Read/Visual Edit tables, footnotes, math and images; record producer/version results and diagnose every out-of-scope case without silent loss.
- [ ] 7.3 Record diagnostic timings and peak memory on small, image-heavy and near-limit documents; verify UI responsiveness, cooperative cancellation and finite resource consumption, adjusting documented constants and boundary tests only with evidence.
- [ ] 7.4 Record Windows native menu/picker/report/import/reopen smoke evidence, including Chinese paths, existing-output collision, content-loss cancellation, and offline execution without external converters.
- [ ] 7.5 Record macOS and Linux native menu/picker/import/reopen/offline/cancellation smoke evidence; leave this task incomplete while required platform verification is unavailable rather than claiming a pass.
- [x] 7.6 Run formatting, the headless importer suite, `cargo test --workspace`, `cargo build`, and the repository's documented quality gate; keep ignored external-tool checks and unrelated pre-existing failures explicitly identified in the compatibility evidence.
- [ ] 7.7 Run strict non-interactive `openspec validate add-builtin-docx-import`, reconcile proposal/design/specs/tasks with the implemented scope and evidence, and confirm readiness for later OpenSpec archival without directly editing stable specs.
