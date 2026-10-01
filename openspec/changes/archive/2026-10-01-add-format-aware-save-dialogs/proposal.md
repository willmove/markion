## Why

Markion currently opens generic save dialogs for Save As and every export action, so the operating system cannot show or enforce the document type being written. Users can therefore omit an extension or choose a misleading extension that does not match the Markdown or exported file contents.

## What Changes

- Make Save As identify the target as a Markdown document, advertise the supported Markdown extensions, and produce a Markdown filename when the user omits or supplies an incompatible extension.
- Make each export dialog identify the format selected by the export action (styled/plain HTML, PDF, LaTeX, DOCX, PNG, or JPEG), advertise that format's accepted extensions, and keep the final filename consistent with the encoder that writes it.
- Centralize format labels, canonical extensions, accepted aliases, filename suggestions, and post-dialog path normalization so all platforms and export entry points follow the same rules.
- Preserve cancellation, error reporting, workspace-root updates after Save As, export backend selection, and existing filename-stem suggestions.

Non-goals: this change does not turn Save As into an export-format chooser, infer a different exporter from a manually typed extension, add export formats, change export fidelity, or alter Markdown parsing and per-document derived-state caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Save As becomes format-aware for Markion's canonical Markdown document format while retaining its existing workspace and persistence behavior.
- `export`: Export save dialogs declare and enforce the output type selected by each export action.

## Impact

- Affected application code: `src/app/documents.rs`, plus a small format/path helper in `src/app/` or the document model and focused tests in `src/app/tests.rs` / `src/lib.rs`.
- User-facing file-type labels and any format/path validation feedback must be routed through `src/i18n.rs`.
- The current GPUI 0.2.2 `prompt_for_new_path` API accepts only a directory and suggested name, so implementation requires a format-aware cross-platform save-dialog adapter or dependency while keeping async dialog results integrated with the existing GPUI task flow.
- No persisted configuration, document contents, public export encoders, or derived Markdown caches change.
