## 1. Image preferences and operation model

- [x] 1.1 Add source-kind, insertion-policy, uploader, and resource-directory preference types in `src/model.rs`, keeping provider execution out of the model; verify the local/clipboard/remote policy matrix and copy/save/keep defaults with unit tests.
- [x] 1.2 Add optional `[images]` and provider-specific TOML parsing/rendering in `src/storage/preferences.rs`, including timeout validation, per-field error isolation, reset, and sanitized summaries; verify old configs, partial/invalid tables, round-trips, and unrelated preference preservation with storage tests.
- [x] 1.3 Define immutable job configuration, occurrence identity, per-item result, cancellation generation, and recovery-record types in pure image-operation modules; verify that jobs capture their configuration and that no GPUI types enter reusable logic or existing member crates.

## 2. Configurable local resource publication

- [x] 2.1 Add document-relative template resolution for `{document}.assets`, `assets`, and nested paths without changing the default helper used by transactional DOCX import; verify safe stems, full nested URLs, Unicode/space escaping, unknown variables, absolute paths, and traversal cases.
- [x] 2.2 Add policy-aware image publication with canonical ancestor checks, Windows junction/symlink containment, content reuse, and no-clobber collision handling; verify repeated content, equal-name/different-content files, directory substitution, and publication failure in temporary-directory tests.
- [x] 2.3 Separate resource planning/staging from final publication and coordinate the latter with Git write admission; verify busy/conflicted repositories cannot publish uncoordinated resources, later admission revalidates the base, and original files remain intact.

## 3. Exact image occurrence planning

- [x] 3.1 Extend the explicit-action image scanner around `src/lib.rs` and `src/inline_edit.rs` to return structured inline Markdown and HTML `src` occurrences, syntax-specific escaping, and skip reasons; verify nested blocks/tables, quoted/escaped URLs, HTML attributes, and exclusion of code, front matter, and ordinary links.
- [x] 3.2 Add resolved reference-style image planning and equivalent inline materialization without changing shared definitions; verify full/collapsed/shortcut image references, formatted alt content, titles, duplicate uses, and definitions also used by normal hyperlinks.
- [x] 3.3 Add single-occurrence, current-document, and inserted-fragment selection plus per-operation input grouping; verify one-image actions affect only the selected occurrence, automatic paste selects only inserted images, repeated inputs share transfer work, and missing/unsupported sources are reported before transfer.

## 4. Bounded materialization and uploader adapters

- [x] 4.1 Add background local/data/remote materialization with private staging, supported-content checks, streaming byte limits, bounded redirects, timeouts, and a bounded queue on the existing HTTP runtime; verify malformed/oversized inputs, MIME/extension mismatch, changing local sources, stalled bodies, and concurrency limits with fixtures.
- [x] 4.2 Implement a reusable external-command runner with literal argv, null stdin, bounded concurrent stdout/stderr draining, timeout/cancellation, owned process-tree cleanup, and hidden Windows execution; verify Unicode/metacharacter filenames, hanging children, excessive output, and nonzero exits using fixture programs.
- [x] 4.3 Implement PicGo HTTP's loopback-only one-file JSON POST adapter, proxy bypass, no-redirect rule, and strict result attribution; verify success, server absence, false/malformed success, wrong result count, redirects, and mid-response cancellation with a loopback server.
- [x] 4.4 Implement PicGo Core executable/launcher/config-path invocation and successful-output parsing; verify supported captured output fixtures, ANSI/log handling, missing programs, ambiguous output, and explicit Windows Node launcher configuration without shell interpolation.
- [x] 4.5 Implement the custom-program `{file}` argument and one-line stdout URL contract with shared URL validation/redaction; verify blank/log/Markdown/multiple-line output rejection, unsupported schemes, signed-query preservation, and exclusion of secrets/raw output from routine diagnostics.

## 5. Asynchronous targeting, application, and recovery

- [x] 5.1 Add operation-owned anchors and edit events at canonical mutation acceptance, covering source/visual edits and tab undo/redo paths; verify unrelated edits rebase anchors, overlapping edits invalidate them, zero-width insertion affinity is deterministic, and no parsing/I/O occurs during anchor updates.
- [x] 5.2 Implement the app controller's staged execution, captured settings, bounded deduplication, progress, cancellation generations, and explicit uncertain-result retry; verify with fake providers that cancellation blocks later automatic edits and retry does not repeat already known successful inputs unnecessarily.
- [x] 5.3 Apply verified results to the originating tab through one checked current-version mutation with independent undo capture and selection rebasing; verify inactive-tab completion, intervening typing, duplicate URLs, partial/stale targets, and one-step batch undo without changing the active tab.
- [x] 5.4 Invalidate automatic application across close/reopen, external reload, missing edit history, rename/Save As, and undo/redo; verify that successful outputs remain available and no completion recreates a deleted target or writes to a different document/base.
- [x] 5.5 Add atomic private recovery manifests and owned-input retention for failed/canceled/unapplied jobs, with no automatic restart/resume; verify restart recovery, corrupt/missing recovery entries, safe explicit discard, and that final document files/cloud objects are never deleted by cleanup or undo.
- [x] 5.6 Add explicit Retry, Save locally, Copy URL, Reveal file, Save recovered input, and Apply known results transitions; verify stale-target revalidation, required save locations, per-item errors, and no transfer during redo or result copying.

## 6. Insertion and existing workflow integration

- [x] 6.1 Route clipboard image bytes and OS image drops through the controller while retaining original input order and default local behavior; verify named/untitled documents, successful initial save binding versus later Save As invalidation, save cancellation, tab switching during prompts, later clipboard changes, and real tiny-image fixtures replacing invalid placeholder bytes where necessary.
- [x] 6.2 Add explicit file/URL image insertion and route image replacement through the same policy pipeline while keeping syntax-only formatting available; verify keep/copy/upload choices, untitled upload-only success, metadata preservation, and picker callbacks bound to the original document.
- [x] 6.3 Process eligible image references after Markdown/rich-text paste using the inserted-fragment plan; verify immediate text insertion, later separate resource undo, local relative-path resolution, remote download/rehosting, bare-URL behavior, and unchanged existing HTML-converter handling of discarded data images.
- [x] 6.4 Reuse the executor from publishing/Git resource organization affordances and expose explicit full-document organization for template migration; verify the existing narrow publishing selection where retained, new directory-based selection, partial failures, and unchanged DOCX import/publishing resource resolution.

## 7. Preferences, actions, and localized feedback

- [x] 7.1 Add the Images preferences tab with policy controls, directory presets/input and resolved preview, validation, theme integration, and scrolling; verify defaults, untitled preview, invalid template handling, persistence/reset, and rendering without spawning work.
- [x] 7.2 Add PicGo HTTP/Core/custom-command controls, program/config pickers, argument editing, timeout, and explicit Test upload; verify only the selected provider's relevant controls appear, other configurations remain saved, and tests never mutate documents or run merely on tab opening.
- [x] 7.3 Add source/visual single-image context actions and current-document Images menu actions across the applicable menu surfaces; verify source mapping, shared-image occurrence isolation, availability states, and batch review/cancel with no transfer or publication side effects.
- [x] 7.4 Add operation progress, result/recovery views, eligible/unique/repeated/skip counts, actionable failures, and recovery controls through `src/i18n.rs`; verify all supported languages have labels, sensitive values stay out of routine status text, and background updates preserve document/cache identity.
- [x] 7.5 Group every image-related Format command beneath one localized Images submenu in both native and in-window menus; verify insertion, selected-image, and current-document actions remain wired and no image command is duplicated at the Format root.
- [x] 7.6 Move selected visual-image width, alignment, source editing, replacement, save-local, and upload controls into the image right-click menu while retaining applicable duplicate/move/delete actions; remove the inline button strip and verify real pointer right-clicks on both local and remote images, exact occurrence targeting, and undo behavior.
- [x] 7.7 Add the missing visible scrollbar to the Images preferences pane using its existing scroll handle; verify wheel and thumb scrolling reach the complete provider/recovery content and other preference tabs remain unchanged.

## 8. Integration verification and documentation

- [x] 8.1 Run the cross-layer GPUI regression matrix for all explicit entry points, manual single/batch actions, partial success, cancellation, editing/undo/tab/base races, restart recovery, and Git barriers; verify only accepted source edits change versions and existing per-version derived cache sharing remains intact.
- [x] 8.2 Add an image-handling guide and links from current user documentation covering directory templates, PicGo/OSS setup, Core launchers, literal custom commands, output format, supported image forms, rich-HTML data-image limitation, limits, recovery, and undo semantics; verify examples match persisted fields and adapter fixtures.
- [ ] 8.3 Perform and record Windows/macOS/Linux smoke checks for real PicGo HTTP/Core and a custom uploader, including packaged Windows no-console behavior and Unicode paths; verify usable output links and report unavailable platform/tool evidence explicitly instead of marking it passed.
- [ ] 8.4 Run `cargo test --workspace`, `cargo build`, and `openspec validate add-image-upload-and-resource-policies --strict`; verify all required checks pass and review the final diff for preserved defaults, no automatic transfers on open/render/save, no cloud credentials, and no direct stable-spec edits before requesting archive. Follow-up verification: workspace tests and strict validation pass; compilation/linking succeeded, but `cargo build` could not replace the running `target/debug/markion.exe` (Windows error 5). The newly linked executable was copied and hash-verified as `target/debug/markion-image-path-fix.exe`; rerun the standard build after closing the running app before archive.
- [x] 8.5 Fix preview resolution of percent-encoded local resource URLs, including Chinese document names, spaces, and literal percent signs; verify insertion/publication through Markdown parsing to actual image decoding, without decoding native file paths or remote URLs.
- [x] 8.6 Keep failed-image placeholders concise, localized, and contained within the document column; show a bounded readable filename instead of raw paths/system errors and verify narrow-window rendering and unchanged source/undo state.
