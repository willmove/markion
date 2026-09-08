## Context

The native application stores Markdown as its canonical document source. Existing exports consume the derived Markdown model; they cannot be reversed into a general DOCX reader. The MarkNice browser workspace already contains `JSZip`, a custom DOCX-to-HTML parser, and HTML-to-Markdown conversion, but it intentionally has no write-back route. That parser is useful conversion-reference material, not a proven compatibility oracle: it depends on the DOM, emits some list numbering as paragraph text, and needs additional validation for style inheritance, footnotes, and revisions.

Relevant stable capabilities are `document-resources`, `reliable-file-persistence`, `workspace`, `markdown-editing`, `html-table-rendering`, `crate-architecture`, `ui-i18n`, and `project-documentation`. The existing resource code supplies safe document-relative filenames and per-file atomic writes. It does not currently provide a multi-file import transaction. Other active work, including Git sync, must retain its own changes and contracts.

## Goals / Non-Goals

**Goals:**

- Make a common Word/WPS manuscript into a portable, saved Markdown document entirely offline.
- Preserve supported content semantics and make unsupported content visible before publication.
- Keep large or malformed inputs off the UI/typing path, with bounded work and cancellation.
- Reuse native tab, resource, persistence, localization, and document-cache lifecycles.
- Establish a real-producer compatibility corpus before presenting native import as supported.

**Non-Goals:**

- Page-layout reproduction, editing Word packages, or reversible Word round trips.
- Legacy `.doc`, macro-enabled `.docm`, encrypted packages, PDF/OCR, batch import, drag/drop import, or file associations.
- A Pandoc import preference, embedded JS runtime/WebView, browser write-back, or changes to current exports.
- A second persistent rich-text model, image hosting, or extending Git synchronization policy.

## Decisions

### 1. A separate Rust crate owns conversion; the application owns side effects

Add `crates/docx-import` with package name `markion-docx-import`, no GPUI/GUI dependency, and an explicit optimized dev profile. The public boundary accepts bounded source data, import options/limits, and a cancellation/deadline context. It returns either a typed fatal error or a prepared import containing:

- a transient semantic tree and deterministic Markdown serialization with typed asset references;
- embedded image bytes plus suggested names, verified media kinds, dimensions, and opaque asset IDs;
- structured diagnostics (code, severity, package part, paragraph/table/footnote location, and bounded text context);
- counts and resource usage needed for the report.

The exact type layout may be refined during implementation. The important boundary is that the crate cannot write arbitrary filesystem destinations, fetch relationships, access GPUI, or create editor documents. Resource URLs are materialized from typed references after the destination is chosen, never through global replacement of guessed placeholder strings in prose. Diagnostic codes are stable; the root i18n layer translates their presentation without translating document content.

```text
File → Import Word (.docx)
          │ source picker
          ▼
background bounded package reader → transient semantics → prepared import
          │                                                 │
          │ fatal/cancel: release temporary state            ▼
          │                                           report/review
          │                                                 │ choose new .md path
          │                                                 ▼
          │                                      staged assets + Markdown
          │                                                 │ create-only commit
          └─────────────────────────────────────────────────▼
                                             normal new saved document tab
                                             version-derived caches once
```

Existing tabs retain their bytes, version, dirty state, undo history, syntax memoization, and cached text/derived `Arc` identities. The new tab initializes through the normal saved-document path once. Import reports and temporary conversion models are not attached to every document version or undo snapshot. The filesystem commit is not an editor undo operation; subsequent text edits use ordinary undo/redo.

### 2. Select a reader through a bounded compatibility gate

The first implementation milestone evaluates a pinned released `docx-rs` reader against the required corpus, recording the version, features, license, coverage, unknown-content behavior, peak memory, and cancellation/resource controls. Prefer the library when it preserves the data needed by our semantic conversion. Disable unnecessary image preview generation where the selected release supports that option.

Adoption requires namespace-aware package reading, usable styles/numbering/relationships/footnotes/revisions, a way to inventory relevant unsupported content, and enforcement of this design's limits during reads and conversion. A package preflight alone does not prove bounded decompression or make a non-interruptible parser cancellable. Add a bounded package inventory pass where a reader would otherwise silently ignore content.

If the library cannot meet these conditions without a broad fork, use a focused reader based on `zip` and `quick-xml` behind the same boundary. Implement only the supported DOCX subset, while inventorying unsupported content and diagnosing loss. Record the decision before UI integration; do not ship two native readers or leave dead alternatives in the workspace. This is an implementation selection task, not a blocker requiring a new product decision.

The current JS parser supplies examples and regression cases. Pandoc is an optional development comparator, never a runtime dependency or the ground truth for the accepted-revision semantics. Source-visible content and explicit expected structures determine correctness. A DOCX produced by MarkNice's HTML compatibility export can contain `altChunk`; detect and report that case rather than claiming round-trip compatibility.

Primary references reviewed during exploration:

- [Microsoft: WordprocessingML package structure](https://learn.microsoft.com/en-us/office/open-xml/word/structure-of-a-wordprocessingml-document)
- [docx-rs: reader API and supported capabilities](https://github.com/bokuweb/docx-rs#read-a-document-with-rust)
- [Pandoc: DOCX reader, tracked changes, media extraction](https://pandoc.org/MANUAL.html)
- [Mammoth: deprecated direct Markdown conversion](https://github.com/mwilliamson/mammoth.js#api)

### 3. Convert semantics directly into Markion-compatible Markdown

Read package relationships and declared content types, including the main-document relationship; do not depend only on conventional filenames or namespace prefixes. Validate namespace URIs and identify unsupported OOXML dialects explicitly. Resolve style inheritance with cycle detection and bounded depth. Preserve authored text and meaningful whitespace before applying context-aware Markdown escaping.

| Input | First-release conversion |
| --- | --- |
| Paragraphs and explicit line breaks | Markdown paragraphs and hard breaks; retain significant text and ordering. |
| Heading levels 1–6 | ATX headings resolved through paragraph properties and inherited styles; never infer solely from font size. |
| Deeper headings | Level 6 with a structural-simplification diagnostic; retain heading text. |
| Bold, italic, strike, underline, superscript/subscript | Native Markdown where supported; limited safe inline HTML for underline/sup/sub. |
| Hyperlinks | Preserve safe HTTP(S), mailto, and resolvable internal bookmark links; retain text and report unsupported destinations. |
| Lists | Preserve hierarchy, start/restart and continuation. Use Markdown list markers; when a significant Roman/Chinese/custom label cannot be expressed, retain the label as item text and report the normalization. |
| Rectangular tables | GFM tables with correct escaping. Retain all body rows; if the source has no header, use an empty header rather than relabeling the first data row as a header. |
| Supported merged tables | A single safe HTML table with rowspan/colspan, no blank lines that split the CommonMark HTML block. Preserve text and cell placement. Nested/unsupported structures get a content or structure diagnostic and readable fallback. |
| PNG/JPEG/GIF/WebP embedded images | Keep supported original bytes as managed assets, preserve alt text, use deterministic relative references; floating positioning becomes document-order placement with a diagnostic. |
| Footnotes | Deterministic Markdown footnote labels and definitions, including paragraphs and supported inline content. Exclude separator notes and preserve repeated references. |
| Native OMML equations | Tested subset: text/symbol runs, fractions, radicals, sub/superscripts, delimiters, sums/products/integrals with limits, composed inline and display expressions. Validate emitted LaTeX through the existing math path. Unsupported constructs retain readable text when available and receive content-loss diagnostics rather than fabricated equations. |
| Revisions | Accepted view: include inserted/move-to content, omit deleted/move-from content, use current formatting, and report that revisions were resolved. Handle both inline and block/paragraph-boundary changes; ambiguous revision structures receive content-loss diagnostics. |
| Comments, endnotes, headers/footers, text boxes, fields, drawings and embedded objects | Inventory explicitly. Preserve readable content where a documented fallback exists, retain cached field results without execution, and report omitted content. Unsupported media is represented by a visible placeholder with available alt text. |

Unrecognized nonempty content-bearing nodes cannot silently disappear. Present normalizations separately from actual omissions. Page geometry, fonts, colors, and spacing are summarized as layout normalization, not repeated warnings for each run. Rasterized previews must not masquerade as editable equations or structured text. Automatic headings/TOCs and unsupported `altChunk` bodies need explicit coverage because exported Word files do not necessarily contain ordinary Word paragraphs.

### 4. A report makes fidelity decisions concrete

The UI has one import job at a time: selecting → converting → reviewing → choosing destination → committing → completed/canceled/failed. Switching or editing tabs is allowed while converting and never changes the target into the active tab.

Every nonfatal result opens a report showing source name, content/image counts, revision policy, and diagnostic groups. Use four classes:

- `info`: normalization such as discarded fonts or accepted revisions;
- `formatting-loss`: structure/presentation simplified while content remains;
- `content-loss`: omitted/unrepresentable content, missing embedded images, unsupported formula or ambiguous revision semantics;
- fatal error: not a prepared import; no Save action.

For info/formatting-only output, the primary action is Save as Markdown. If content loss was detected, the primary action explicitly says to continue with the listed losses and is not activated by a default Enter keypress; Cancel remains available. Show source-location context and any readable fallback for each affected group. A document with no recoverable non-whitespace text, table content, image, or math is a fatal empty-result error. Do not count generated placeholders alone as recovered content.

After the report, the native save dialog suggests `<source-stem>.md`. First release accepts only a new `.md` destination not already represented by an open tab. Both the Markdown path and the derived asset directory must be absent; conflicts offer another name rather than overwrite. Original DOCX bytes and existing tabs are unchanged. On success open and activate a new clean tab, register it in normal session/recent-file state, and report the saved paths. If opening the tab fails after durable publication, report the saved Markdown path as recoverable rather than rolling back a completed save.

All native and in-app labels, report categories, errors, progress, and success states use the current i18n layer in every supported language. No default keyboard shortcut or new preference is needed. Existing File → Open and drag/drop classifications remain unchanged.

### 5. Bounded, offline work is enforced below the UI

Initial implementation budgets are internal constants, not new preferences:

| Budget | Initial limit |
| --- | --- |
| Compressed source | 20 MiB |
| Package entries | 4,096 |
| Actual cumulative decompressed bytes | 128 MiB |
| Individual XML part | 16 MiB |
| XML nesting / style inheritance depth | 128 / 64 |
| Extracted images | 200 distinct assets |
| Image bytes | 16 MiB each, 64 MiB aggregate |
| Image dimensions | 40 megapixels per decoded frame; bound aggregate animated-frame decoding too |
| Generated Markdown | 16 MiB |
| Conversion deadline | 30 seconds |

Check actual streamed consumption in addition to ZIP metadata, plus structure/item counters to keep memory bounded inside byte limits. Check cancellation and deadlines between bounded read/event/conversion operations; the library choice must not introduce an opaque stage that defeats these checks. Move any necessary media validation off the UI thread and apply decoder allocation limits before decoding. Refuse encrypted/macro-enabled/non-DOCX package types even when the extension is misleading; malformed XML, conflicting duplicate ZIP parts, broken required relationships, and exceeded budgets are fatal.

Do not extract the ZIP using authored filesystem names. Resolve relationships in package space with normalized containment checks; forbid escape into the host filesystem. Never execute fields, macros, OLE, or external entities. External image/file relationships are not fetched or emitted as automatically loading remote images: use a visible fallback and diagnostic. Safe user-clicked hyperlinks remain links. Importing and then previewing the imported output must not cause document-directed network/file reads outside its managed resources.

Cancellation releases conversion data and temporary files. Closing the application before commit abandons the job. During the short publication boundary cancellation is disabled, with a visible saving state, to avoid an ambiguous completed-save/canceled result. A canceled/stale job cannot later create files or a tab.

### 6. Publish resources before Markdown with explicit rollback ownership

This workflow extends `document-resources`; ordinary paste/drop behavior remains unchanged. The saved-document base path is selected before managed resources are stored, consistent with the untitled-resource rule.

1. Validate a new target `.md` and absent derived asset directory, reject source/path aliases and destinations already open in the application, and stage files in a private sibling location on the target filesystem.
2. Reuse safe resource naming and relative-URL rules, adding a batch staging adapter. Deduplicate identical image bytes within the import; keep typed asset-to-URL mapping. Never write into an existing asset directory or follow a substituted link/junction outside the validated destination.
3. Persist and close prepared image files and final Markdown bytes. Recheck destination identities and publish using create-only/no-clobber operations; an earlier existence check followed by an overwriting rename is insufficient.
4. Publish assets before Markdown. The create-only publication of complete Markdown is the commit point; no visible successful `.md` may reference uncommitted resources. A text-only import creates no empty `.assets` directory.
5. On handled failure or precommit cancellation, remove only transaction-owned staging and outputs whose identities still match. If cleanup itself fails, report the retained locations accurately and never delete unrelated files. After commit, ownership passes to the user and ordinary persistence/recovery applies.

Two sibling paths cannot be claimed as one universally atomic transaction. An unexpected process/OS termination before the Markdown commit can leave a staging directory or newly published asset directory, but must not expose a partial Markdown document or corrupt existing files. Automatic orphan cleanup and a persistent transaction journal are outside this change; documentation describes possible remnants. A retry uses a fresh name or explicit user cleanup. Existing single-file persistence remains the basis for complete-file publication, with a distinct no-clobber primitive where required.

### 7. Compatibility evidence precedes the public feature

Assemble 20–30 representative, redistributable/sanitized documents spanning Word, WPS, and LibreOffice producers, plus small synthetic fixtures for exact edge cases. Record producer/version and provenance, covered features, expected accepted-view body text/order, warnings, and asset identities. Include a native Markion export and a MarkNice HTML-compatibility DOCX; neither is sufficient as the only interoperability source.

Release criteria:

- All must-preserve fixtures retain expected content and semantics; no silent omission is accepted.
- Every out-of-scope fixture produces its expected diagnostic/fatal category and continuation behavior.
- Saved outputs reopen with local images, including Chinese/space-containing parent paths, and supported tables/math/footnotes render across Source, Split, Read, and Visual Edit as applicable.
- Corrupt, adversarial, oversized, canceled, and colliding imports preserve existing files and tabs; test failures injected before/after each publication stage and crash-before-Markdown behavior.
- Record elapsed time and peak memory for small, image-heavy, and near-limit inputs. Verify the UI remains responsive and cancellation stops publication. Record tested hardware/platform rather than claiming unmeasured universal performance.
- Headless crate tests and root integration tests pass, followed by `cargo test --workspace`, build checks, and native picker/import smoke evidence on Windows, macOS, and Linux before claiming those platforms verified.

## Risks / Trade-offs

- [Reader drops unsupported content before our serializer sees it] → Require package inventory and loss diagnostics at the selection gate; reject a reader that cannot meet the contract.
- [Scope expands toward Word fidelity] → Fixed semantic matrix, documented unsupported constructs, no page-layout engine or reverse-save promise.
- [Revision handling hides or duplicates manuscript text] → Test accepted content and paragraph boundaries against source-visible expectations; report ambiguous cases as content loss.
- [HTML table or equation output differs across editor modes] → Reuse current Markdown/HTML/math consumers and add renderer-facing fixtures; degrade explicitly when the supported subset is exceeded.
- [Images enlarge disk use and Git sync payloads] → Deduplication and budgets, normal managed-resource paths; no automatic Git commits/pushes or sync policy changes introduced by import.
- [Native and browser import drift] → Share compatible fixture expectations and document intentional differences; do not couple native import to the browser service.
- [Termination leaves an asset directory] → Markdown-last commit, no-clobber ownership, explicit crash limitations; no unsupported promise of cross-file atomicity.
- [Arbitrary initial limits reject legitimate manuscripts] → Measure the corpus and adjust constants with documented evidence and matching tests before release; do not silently bypass limits.

## Migration Plan

1. Complete the corpus/reader gate and record results in `docs/docx-import-compatibility.md`.
2. Implement the isolated converter and resource transaction, then integrate the native report/picker/tab lifecycle.
3. Add localization and `docs/word-import.md`, update bilingual READMEs, and complete tests/platform evidence.
4. Validate this change before archival; sync its deltas only through the OpenSpec workflow.

No existing files or preferences are migrated. Rollback removes the import entry point and engine; produced Markdown and `.assets` files remain ordinary user-owned resources usable by older Markion versions. Existing browser import and export behavior remain available.

## Open Questions

- Which pinned reader/version passes the content-preservation, inventory, and bounded-work gate? Resolved by tasks 1.1–1.3 before integration; the pure-Rust boundary and product scope are already decided.
- Do the initial budgets cover the representative producer corpus with acceptable memory use? Record evidence and adjust before release if needed. Compatibility and performance have not yet been measured; this proposal does not assert a pass.
