## Context

See `proposal.md` for motivation and scope. This design is necessary because image handling spans asynchronous I/O, source mutation, filesystem publication, preferences, and external processes.

Observed implementation:

- `src/app/editing.rs::request_image_import` and `insert_image_inputs` currently capture bytes, require a named document, import synchronously, and insert one Markdown edit. `PendingImageInput` stores no source path or document identity. Replacement also resolves against the active tab after a picker returns.
- `src/storage/resources.rs::document_asset_dir` hardcodes `<sanitized-stem>.assets`; `import_image_bytes` assumes a one-component directory name when constructing the URL. It has content-based filenames and reuse, but custom nested targets need a full relative-path calculation and publication-time containment checks.
- `src/app/publishing.rs` already offers local resource organization. It selects references outside the publishing scope, which is different from moving all resources into the user's chosen image directory. Its confirmation/completion flow also needs originating-document identity.
- `MarkdownDocument::publishing_image_references`, `image_destination_matches`, and `rewrite_image_destinations` already scan on explicit actions and handle inline Markdown and raw HTML destinations. They are useful foundations, but URL-wide replacement cannot implement a single-occurrence action, and reference-style images are currently skipped by the destination locator.
- `CheckedMutation` binds an edit to a document instance, version, and expected source. Preserve that final acceptance gate; an asynchronous job must not operate on whichever tab is active when it finishes.
- `src/app/network.rs` owns a reusable HTTP runtime. Its existing helpers collect whole response bodies before checking export size and start all export URLs at once; the new pipeline needs a streaming limit and a bounded queue rather than calling those helpers unchanged.
- `PreferencesFile` is optional/defaulted TOML; the panel already has General, Appearance, Shortcuts, and Export tabs. `crates/html-import` is a pure converter: it retains remote image references but deliberately drops HTML data-image payloads to alt text. This change processes its retained references and does not broaden that converter behavior.

Specification reconciliation: the durable `document-resources` spec requires unconditional local import and save-before-import; this change explicitly replaces those rules with policy-dependent behavior. The durable preferences overview excludes uploader credentials: external-tool settings are added, while cloud credentials stay external. Unrelated older wording about fonts/auto-save in that overview is not a mandate to remove currently implemented settings. Existing unarchived changes for data images, Git sync, and publishing already have code present; implementation must preserve their resource decoding, write admission, and publishing scope, without archiving or editing their artifacts as part of this change.

## Goals / Non-Goals

**Goals:**

- Use one planning/execution/application pipeline for insertion and existing-image transformations.
- Keep source text canonical; isolate progress and pending work from per-version Markdown caches and undo snapshots.
- Maintain exact occurrence attribution across unrelated edits, and retain results whenever automatic application is unsafe.
- Keep provider protocols testable without cloud accounts and keep existing offline defaults.

**Non-Goals:**

- A general plugin runtime, cloud SDK, credential vault, or automatic installer for external tools.
- Rewriting preview caches into an upload database, changing image presentation syntax, or hooking transfers into ordinary parse/render/save.
- Changing DOCX import's dedicated transactional publication rules or the WeChat browser workspace's network behavior. Users can run image actions on the resulting Markdown explicitly.

## Decisions

### 1. Represent source, operation, and provider independently

Add image preference enums in `src/model.rs` and optional serde-facing tables in `src/storage/preferences.rs`. Proposed persisted shape:

```toml
[images]
local_policy = "copy"              # keep | copy | upload
clipboard_policy = "save"          # save | upload
remote_policy = "keep"             # keep | download | upload
directory = "{document}.assets"
uploader = "none"                  # none | picgo-http | picgo-core | command
timeout_secs = 60

[images.picgo_http]
endpoint = "http://127.0.0.1:36677/upload"

[images.picgo_core]
# executable = "..."
# launcher_args = []
# config_path = "..."

[images.command]
# executable = "..."
# args = ["...", "{file}"]
```

Keep one selected provider and saved configurations for each adapter. Global application preferences are sufficient for v1; documents/front matter do not configure commands. Missing settings retain compatibility; invalid enum values use defaults, while invalid directory/provider values disable the affected operation and produce visible validation feedback. Invalid nested TOML values must not invalidate unrelated preferences.

Input variants distinguish a local path, captured bytes, a remote URL, and a data URI. A plan records input identity, intended transformation, source occurrences, provider configuration, destination base/template, and document instance. Explicit actions override insertion policy; they do not modify preferences. Embedded data references are retained on paste and included in manual upload/localization actions. Keep-reference insertion computes a relative URL where representable and otherwise emits an escaped absolute path; native paths and file URIs are not conflated.

Alternative rejected: one global upload toggle. It cannot express keeping web images while copying screenshots, and makes enabling an uploader unexpectedly rehost every image source.

### 2. Separate pure planning, bounded I/O, and checked application

Put pure source classification, occurrence planning, and replacement construction in root-library modules alongside existing resource/inline helpers. Add an app-side image-operation controller and bounded transfer/provider implementation, using the existing runtime. Keep GPUI types out of reusable pure logic; a new workspace crate is unnecessary at this scale.

```text
File/URL insert --+
Paste/drop ------+--> capture inputs + document identity + policies
Single/batch ----+                  |
                                   v
                    source plan / batch review
                                   |
                                   v
                  private staging + bounded workers
                   /             |              \
             copy/decode     download      provider upload
                   \             |              /
                    +------------+-------------+
                                 |
                                 v
                  validate targets + Git admission
                                 |
                                 v
                   publish local files + one edit
                                 |
                                 v
                  normal version/cache invalidation

Progress / failures --> operation state and recoverable results
```

A command-time scan runs on a captured source snapshot in the background. Metadata-only batch review performs no uploads, remote downloads, or final resource publication. After confirmation, jobs capture bounded input bytes to private files; source file changes during copying are detected and reported. Duplicate captured content within a single operation shares one transfer, while retaining a list of selected occurrences. No cross-session/global upload-result cache is introduced because host settings and remote retention can change.

Run at most two materializations concurrently and serialize calls into external uploaders initially, since their plugins/configuration may have global state. Use a 32 MiB per-input/download limit enforced while reading, 15-second connection timeout, configurable 60-second transfer/process timeout (validated to 5–600 seconds), five HTTP download redirects, and 1 MiB per process output/HTTP upload-result stream. Iterate queued items and spool to disk so a large document does not keep every image's bytes in memory. Check content rather than trusting URL suffixes or a server MIME header; preserve original supported image bytes instead of decoding/re-encoding them for upload. Reuse bounded existing data-URI validation; reject unsupported/malformed inputs with item-level errors.

Alternatives rejected: synchronous reuse of `insert_image_inputs` for network I/O; using decoded preview bitmaps as upload inputs (loses original resolution/animation/format); using the current whole-response fetch helper unchanged.

### 3. Build three explicit provider adapters

**PicGo HTTP:** POST JSON `{"list":["<absolute staged file>"]}` to a loopback-only configurable endpoint, defaulting to `http://127.0.0.1:36677/upload`. Use one file/request, no redirects, and bypass proxy settings for loopback. Require successful HTTP status, `success: true`, and one valid URL. Never send an empty list, which could cause PicGo to read a different clipboard image. V1 targets PicGo GUI's local server; a separately authenticated server protocol is not implied.

**PicGo Core:** configure a program, optional launcher arguments, and optional config file; append its `upload` invocation and one staged path. Parse the documented successful-upload output separately from logs and require zero exit status. ANSI styling is stripped only for protocol recognition. A result is accepted only when exactly one URL follows the successful-upload marker; fixtures must cover the supported installed CLI versions. On Windows, allow `node.exe` plus the installed PicGo CLI entry script as launcher arguments. Do not implicitly interpolate a `.cmd`/`.bat` shim; unsupported launchers produce guidance to configure the interpreter directly. No Node/PicGo download is performed.

**Custom command:** program plus argv, with `{file}` exactly once as a complete argument. A script is invoked through its explicit interpreter, e.g. `pwsh.exe -NoProfile -NonInteractive -File upload.ps1 {file}`. Set stdin to null, drain stdout/stderr concurrently under limits, use a private operation working directory, and hide the Windows console. A zero exit code plus exactly one non-empty stdout URL is the protocol; diagnostics go to stderr. No implicit shell, JSON/Markdown result guessing, or clipboard probing. Environment inheritance supports the user's externally managed credentials; preference UI/documentation advise against embedding secrets in argument strings, and diagnostics never dump argv or environment values. Adopt cancellation/process-tree ownership patterns where suitable, without coupling to Git-specific environment sanitation.

All adapters validate absolute HTTP/HTTPS URL syntax, host, control characters, and absence of userinfo. A successful response is not followed by a mandatory GET: private or signed URLs and CDN propagation need not be immediately publicly fetchable. Preserve query parameters in source/results but redact them in routine logs.

Protocol references checked during exploration: [PicGo GUI HTTP/CLI](https://docs.picgo.app/gui/guide/advance), [PicGo Core commands](https://docs.picgo.app/core/guide/commands), and [PicGo host configuration](https://docs.picgo.app/gui/guide/config). Providers such as Aliyun OSS are configured there rather than in Markion.

Alternative rejected: only a generic shell command. It would force ordinary PicGo users to solve logging/output parsing and quoting themselves; explicit adapters make errors attributable and testable.

### 4. Generalize resource storage without changing unrelated import transactions

Introduce a resolved resource destination argument for image import helpers while retaining the current default helper for callers such as DOCX import that need their own fixed transaction. `{document}` uses the existing sanitized document stem. Normalize optional leading `./` and nested segments, reject absolute/drive/UNC paths, `..`, unknown variables, and a destination equal to the document directory. Validate both lexical structure and canonical existing ancestors, including Windows junctions, before creating missing directories and again before publication. A configured destination must stay beneath the named document directory.

Construct a URL from the full relative destination path using forward slashes and encoding appropriate for Markdown or HTML, not `asset_dir.file_name()`. Reuse equal bytes safely; use content fingerprints and no-clobber publication to prevent races from overwriting different files. Do not move/delete the original input. A source already in the desired directory can be reused after validation.

Preview resolves authored local destinations through the shared local-reference resolver before generating cache keys or reading files. Percent-decoding happens exactly once, after separating URL query/fragment delimiters, so Unicode directory names, spaces, encoded hashes, and literal percent sequences resolve to the published file. Native file-viewer paths bypass URL decoding; HTTP(S) destinations retain their request encoding. Failed-image placeholders use a localized short message and a bounded, decoded filename with ellipsis and explicit width/overflow constraints. Full paths and raw system errors remain outside the rendered placeholder, and the existing source editor/right-click replacement affordances remain available.

Final resource publication takes repository write admission only for the short write/application phase; no repository lock is held across network requests or an external command. Recheck the document base, destination containment, and conflict ownership then. A busy barrier keeps results pending; after admission is available, validate again. If publication succeeds but source acceptance fails, keep and report the created file rather than deleting a potentially referenced resource.

Preferences only select the target of subsequent operations. Changing a template or Save As/rename does not relocate old assets. Explicit Organize local images plans from all eligible references outside the new target, including resources still inside the publishing scope; keep the existing publishing/Git organization affordances as wrappers using that common executor, preserving their narrower candidate-selection semantics when appropriate. Rendering, exporting, publishing, and Git inspect the actual authored URLs rather than assuming one directory name.

Alternative rejected: globally changing `document_asset_dir` to take live UI preferences. That would accidentally alter DOCX transaction assumptions and hidden callers, and would make running jobs sensitive to a preferences change.

### 5. Preserve exact occurrences and reference-style semantics

Extend the action-time image scanner to return structured occurrences: semantic URL, source kind, full image source range, precise destination range where available, syntax/escaping kind, and metadata needed for an equivalent replacement. Reuse `body_text_and_offset`, pulldown-cmark events, and the raw-HTML attribute scanner to exclude front matter, fenced/inline code, and ordinary links. Cover nested blocks and Markdown/HTML tables.

Inline Markdown updates replace only a verified destination with valid escaping. Raw HTML updates replace `src` with attribute-context escaping and preserve all other attributes, including presentation. Resolved reference-style images become equivalent inline image uses when processed, leaving definitions and unrelated uses untouched. Preserve authored alt content and the resolved title; do not rewrite a shared definition as a shortcut. Report unresolved references, unsupported schemes (including browser-only `blob:`), `srcset`-only nodes, and ambiguous mappings before any transfer for that occurrence. One-image commands carry an occurrence ID, not just a URL.

Plans group duplicate inputs only for transfer; rewrite scope remains the captured occurrence set. A newly typed or pasted occurrence with the same URL after a batch begins is not implicitly added. A retry reuses known successful outputs for still-identical inputs/provider configuration, and reruns only selected failed/uncertain inputs after explicit user action.

Alternative rejected: regular-expression/global string replacement and the current URL-keyed rewrite helper alone. They can touch code, hyperlinks, or other occurrences and cannot safely handle shared reference definitions.

### 6. Track in-flight targets through canonical edits

Store pending jobs by `DocumentInstanceId`, with a captured document path/base, current tracked version, job generation, and source anchors. Anchor the entire image syntax (plus the resolved definition dependency for reference-style images), not only the URL, so changing an image's identity/metadata invalidates that target. Track non-overlapping canonical range edits by shifting affected offsets and versions; never relocate by searching for a matching URL. A zero-width insertion uses left affinity so later typing at that location follows the pending image. Changes crossing an anchor invalidate it. Undo/redo, whole-document reload/replacement, a missing mutation interval, rename/Save As, and tab closure conservatively invalidate automatic application. These invalidations do not discard successful outputs.

Integrate lightweight edit events at canonical mutation acceptance, including mutation paths currently handled directly by tab undo/redo or trusted transformations. Pending anchors do no parsing or I/O on that path; there is no work when no image operation is live. Bound live operation/anchor metadata and refuse excess work with a visible result rather than silently dropping targets. Immutable job source snapshots and per-image expected source bytes permit final validation; never replay an old full-document snapshot over current text.

At completion, locate the original tab by identity without activating it. Collect valid replacements in current-source order and build one checked edit against the current version, using the smallest encompassing span with intervening current text preserved. Finish that tab's current typing undo group first and create one separate undo snapshot. Rebase that tab's selection/caret through the accepted edit; do not move the caret to the upload result or touch another tab's selection. Notify through the ordinary document-change/cache path only after acceptance. Progress updates are UI state only and do not invalidate Markdown derivations.

Alternative rejected: applying against the current active tab, fuzzy matching, or accepting a stale `CheckedMutation`. A strict version-only rejection would be simpler but would make routine typing during uploads unusable; tracked non-overlapping edits preserve usability with a conservative invalidation fallback.

### 7. Define insertion, batch, cancellation, and recovery explicitly

- File/clipboard insertion captures its intended selection and stages input, shows pending progress outside document source, and inserts successful items in input order in one edit. A required first Save As is a preflight transition that binds the previously absent base exactly once before execution, and remains bound to the originating document; subsequent Save As/rename invalidates the job as described above. Canceling that initial save produces no resource-directory writes.
- Paste of textual Markdown/rich text inserts the fragment immediately as the existing atomic paste. Schedule only eligible image occurrences in that newly inserted region; local relative paths resolve from the current document's directory, not the clipboard source application's unknown directory. Bare URLs and retained data-URI destinations are left as authored. Automatic transformation is a later, separate undo edit, never retroactively merged across user typing. HTML data images discarded by the existing converter remain a documented converter limitation.
- Single-image actions execute directly using the selected provider/target. Full-document actions show a review with unique-input and occurrence counts plus skip reasons, then run the same executor. All image commands in the Format menu live in one Images submenu. A selected visual image exposes width, alignment, source editing, replacement, save-local, and upload commands in its right-click menu rather than an inline button strip; the existing block duplicate, move, and delete commands remain available there. Source image context actions expose the applicable single-image operations. Explicit file/URL insertion controls coexist with the existing syntax-only formatting action.
- On normal completion, apply all successful valid replacements once, leave failures/stale occurrences unchanged, and show succeeded/failed/skipped/unapplied counts with per-item reasons. Cancellation invalidates the job generation, stops scheduling, cancels owned work, and prevents the automatic completion commit; known outputs can be applied through a new explicit action after revalidation.
- Retain failed raw inputs and successful unapplied URLs/files in an app-private recovery area outside the notes repository, with an atomic minimal result manifest. Normal restart exposes these as recoverable inputs/results; it never resumes uploads or auto-edits reopened documents. Do not persist transferable anchors as authority across restarts. Applied jobs clean up only owned staging files; user Discard removes only the matching private recovery entries. Final document resources and remote objects are never automatically deleted.
- Retry is explicit, particularly for uncertain delivery. Save locally for failed clipboard uploads is an explicit override and may ask for a Markdown save location. Copy URL / Reveal saved file / Save recovered input provide recovery when the original target no longer exists. Undo restores links only; redo uses recorded destinations and performs no I/O.

Alternative rejected: writing fake `uploading://` Markdown or deleting a cloud image on undo. Pending state does not belong in canonical text, and external upload providers do not supply a safe, uniform object-ownership/delete contract.

### 8. Validate through observable seams

Pure tests cover policy classification, nested destinations/containment, dedupe/collisions, source syntax, shared references, and anchor transforms. Provider tests use a loopback HTTP fixture and fake/fixture processes, including malformed success, nonzero exit, hanging children, excessive output, and filenames with Unicode/metacharacters. Streaming tests reject oversized content before the full response is buffered.

GPUI tests cover all explicit entry points, default compatibility, save cancellation, paste undo boundaries, partial failure, target edits/deletion/undo, inactive tabs, close/Save As/reload, Git admission, recovery presentation, and unchanged derived-cache identity during progress. Manual smoke tests exercise real PicGo HTTP/Core and one custom uploader on supported operating systems, including packaged Windows execution without console flashes. Tests do not require cloud credentials in fixtures or CI.

## Risks / Trade-offs

- External PicGo plugins can show their own dialogs or mutate the clipboard -> Markion supplies files directly, never uses clipboard output, documents external-tool settings, and reports timeout/cancellation without claiming control of third-party UI.
- An upload can finish remotely after cancellation/timeout -> retain known URLs, mark unknown delivery as uncertain, and never retry automatically or promise remote rollback.
- Reference-style materialization changes syntax shape -> preserve image semantics and leave shared definitions/links intact; verify with parser-based fixtures.
- Complex edits invalidate anchors -> keep outputs recoverable and require explicit reapplication, rather than guessing a target. This is intentional even if the user later recreates identical source.
- Shared `assets/` directories make deletion ownership unclear -> use no-clobber creation and reuse; never clean historical document resources automatically.
- Private recovery storage can grow -> show retained items and total size with explicit discard controls; enforce input/job bounds without silently removing unresolved inputs.
- Signed URLs can expire -> preserve the uploader's exact result and document that durable public image URLs are the provider's responsibility.
- Existing unarchived changes touch resources/preferences -> implement against current code, keep this change's deltas focused, and reconcile requirement conflicts before later archive rather than modifying stable specs now.

## Migration Plan

1. Add optional image configuration with default copy/save/keep policies and no selected uploader. Existing config and existing documents require no migration.
2. Retain default resource helpers for unrelated import transactions; introduce explicit policy-aware callers incrementally and validate current clipboard/drop/organize tests throughout.
3. Ship provider settings and explicit actions only after source-target validation, recovery, and fake-provider failure tests are passing. No network/provider action occurs simply from upgrading.
4. Document template examples, PicGo OSS setup, CLI launch conventions, command output, supported source forms, size/time bounds, and recovery/undo semantics in a new image-handling guide linked from existing user documentation.
5. Rollback by disabling upload policies or reverting the feature. Already written relative/HTTP(S) image references remain ordinary Markdown/HTML understood by older versions; leave image files and external uploader settings intact. Recovered inputs remain on disk even when a reverted build cannot display their recovery UI.
