## Context

`src/app/documents.rs` currently calls GPUI 0.2.2's `prompt_for_new_path(directory, suggested_name)` for both Save As and format-specific export actions. That API has no file-type filter, dialog title, or default-extension parameter. The app already knows the output format before opening the dialog, but only communicates it through the suggested filename; a path returned without the expected extension is passed unchanged to `save_as` or `export_to_with`.

The implementation must work on Windows, macOS, and Linux, remain asynchronous from the GPUI application's perspective, preserve the current write/export completion flow, and keep all display strings localized. It does not touch editor state, Markdown parsing, document versioning, or derived-state caches.

## Goals / Non-Goals

**Goals:**

- Show a native save dialog whose title and file-type filter match Save As or the selected export action.
- Use one authoritative format profile for labels, filename suggestions, accepted extension aliases, and canonical extension normalization.
- Guarantee that the final path extension agrees with the writer/encoder selected before the dialog opened.
- Preserve native overwrite handling, dialog cancellation, Save As workspace updates, and existing export success/failure reporting.
- Keep format and path rules independently unit-testable without opening a native dialog.

**Non-Goals:**

- Letting Save As choose an export format or switching encoders based on a typed filename.
- Changing export output, adding formats, or changing how PDF/DOCX choose between pandoc and built-in backends.
- Replacing the existing GPUI open-file or open-folder dialogs.
- Changing persisted settings or any Markdown/editor cache.

## Decisions

### 1. Add a narrow `rfd`-backed save-dialog adapter

Add `rfd` 0.17.2 and use `AsyncFileDialog` only for Save As and export destinations. Its builder supplies the missing `set_title`, `set_directory`, `set_file_name`, `add_filter`, `set_parent`, and asynchronous `save_file` operations on all three desktop platforms. Keep GPUI's current path prompts for open-file and open-folder flows.

The adapter will be isolated in `src/app/save_dialog.rs` (registered from `src/app/mod.rs`) so `documents.rs` deals only with a returned `Option<PathBuf>`. Construct the dialog on the GPUI action path, attach the current `gpui::Window` as its parent, and await the returned future through the existing `cx.spawn` flow. A returned `None` follows the existing cancellation path.

On Windows, GPUI 0.2.2 implements the window handle but leaves its display-handle method unimplemented. The adapter therefore borrows GPUI's Win32 window handle through a narrow `raw-window-handle` wrapper and supplies the stateless Windows display handle before passing the parent to `rfd`. macOS and Linux pass the GPUI window directly.

This is preferred over platform-specific Windows/macOS/Linux implementations, which would duplicate native APIs and packaging concerns, and over retaining `prompt_for_new_path`, which cannot express the requested file type.

### 2. Represent the selected writer separately from its dialog profile

Introduce a small app-layer output descriptor such as `SaveTarget::Markdown` and `SaveTarget::Export(ExportFormat)`. It resolves to a profile containing:

| Target | Dialog extensions | Canonical extension | Suggested suffix |
| --- | --- | --- | --- |
| Markdown Save As | `md`, `markdown`, `mdown` | `md` | existing filename or `Untitled.md` |
| Styled HTML | `html`, `htm` | `html` | `html` |
| Plain HTML | `html`, `htm` | `html` | `plain.html` |
| PDF | `pdf` | `pdf` | `pdf` |
| LaTeX | `tex`, `latex` | `tex` | `tex` |
| DOCX | `docx` | `docx` | `docx` |
| PNG | `png` | `png` | `png` |
| JPEG | `jpg`, `jpeg` | `jpg` | `jpg` |

The descriptor also supplies an i18n message key for the filter label and reuses the existing localized Save As / export action label for the dialog title. Extension strings passed to `rfd` omit the leading dot.

The `ExportFormat` argument captured before opening the dialog remains authoritative. Filename text never selects a different exporter. This prevents a PDF action from unexpectedly producing DOCX merely because the user typed `.docx`.

### 3. Normalize the returned path in application code

Native save-dialog filter behavior differs by platform, so a pure helper will normalize every returned path before writing:

1. Read the final extension and compare it case-insensitively with the target profile's accepted extensions.
2. Preserve the path unchanged when the extension is accepted, including aliases such as `.markdown`, `.htm`, `.latex`, and `.jpeg`.
3. If the extension is absent, empty, or incompatible, replace it with the profile's canonical extension using `PathBuf::set_extension`.

Windows may append the selected filter's extension after an incompatible typed extension (for example, `report.docx.html`). When the penultimate extension is a known Markion output extension from another target, collapse that native-dialog suffix to the canonical selected extension (`report.html`). Ordinary dotted names such as `report.v1.html` remain unchanged.

The status bar displays the normalized path because the existing save/export completion logic receives that path. This makes the actual destination visible without adding another confirmation prompt. Plain HTML's compound suggestion remains `name.plain.html`; its final extension is the accepted `.html`.

Post-dialog normalization is required even with native filters: it makes GTK behavior deterministic and protects against unsupported extensions typed manually. It is preferred over rejecting and reopening the dialog, which would add a second interaction and complicate cancellation state.

### 4. Keep mutation after successful path selection

Opening or cancelling the dialog does not mutate the document. After a normalized path is returned, Save As continues to call `MarkdownDocument::save_as`, discard the recovery file, and update the workspace root only on success. Export continues to call `export_to_with` using the originally selected `ExportFormat` and keeps PDF/DOCX backend disclosure unchanged.

Tests will cover every profile, case-insensitive accepted aliases, missing and incompatible extensions, compound plain-HTML suggestions, and the invariant that format selection is independent of the filename. Existing document save/export tests continue to cover filesystem output.

## Risks / Trade-offs

- [Risk] `rfd` adds another native-dialog dependency and Linux portal/Zenity runtime assumptions. → Mitigation: use its default XDG portal backend, exercise release builds on all three platform CI jobs, and keep the dependency behind one app module so it can be replaced without changing document/export code.
- [Risk] Native filter presentation and automatic extension behavior vary by platform. → Mitigation: treat the filter as user guidance and enforce the same accepted/canonical extension rules after the dialog returns.
- [Risk] `rfd` reports cancellation as `None` and does not expose a distinct structured dialog error through `AsyncFileDialog`. → Mitigation: retain detailed reporting for all filesystem and export errors; log adapter failures if the selected backend exposes them in a future version, and keep cancellation non-destructive.
- [Risk] Replacing an explicitly typed incompatible extension may surprise a user. → Mitigation: the chosen Save As/export action and visible filter establish the authoritative type, and the completion status reports the normalized destination.
- [Trade-off] Save and open dialogs come from different adapters. → This keeps the change focused; open dialogs do not need output-format metadata and can remain on GPUI.

## Migration Plan

No data or configuration migration is required. Add the dependency and adapter, route Save As and export prompts through it, then verify unit tests plus Windows/macOS/Linux build jobs and smoke-test the native filters. Rollback consists of restoring the two call sites to GPUI's `prompt_for_new_path` and removing the isolated adapter/dependency; saved documents and exports remain compatible.

## Open Questions

None.
