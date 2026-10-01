## 1. Format metadata and dialog adapter

- [x] 1.1 Add `rfd` 0.17.2 to the root app dependencies and register an isolated `src/app/save_dialog.rs` module without changing the existing open-file/open-folder prompts.
- [x] 1.2 Define the Markdown and export save-target profiles, including localized title/filter message keys, accepted extension aliases, canonical extensions, and the plain-HTML compound suggestion.
- [x] 1.3 Implement a pure, case-insensitive path-normalization helper that preserves accepted aliases and replaces missing, empty, or incompatible final extensions with the profile's canonical extension.
- [x] 1.4 Implement the asynchronous native save-dialog adapter with parent window, starting directory, suggested filename, localized title, and the selected target's file-type filter.
- [x] 1.5 Add non-empty localized file-type labels for every supported Markion language through `src/i18n.rs`.

## 2. Save and export integration

- [x] 2.1 Route Save As through the Markdown target profile and normalized result while preserving successful save, recovery cleanup, workspace-root updates, failure status, and non-destructive cancellation.
- [x] 2.2 Pass the current GPUI window and selected format profile through every export action, then export the normalized path with the originally selected `ExportFormat`.
- [x] 2.3 Preserve current filename-stem suggestions, plain HTML's `.plain.html` suggestion, PDF/DOCX backend disclosure, and all existing export write/error status behavior.

## 3. Automated coverage

- [x] 3.1 Add table-driven tests for every save target's accepted extensions, canonical extension, localized label/title key, and suggested suffix.
- [x] 3.2 Add path-normalization tests for no extension, empty/incompatible extensions, case-insensitive accepted aliases, Unicode/space-containing names, and the compound plain-HTML filename.
- [x] 3.3 Add regression coverage showing that a mismatched typed extension cannot switch the selected exporter and that no document/cache mutation occurs before a dialog result is accepted.
- [x] 3.4 Run the existing Save As, workspace-root, all-format export, and i18n completeness tests to confirm the new adapter does not change persistence, backend selection, or derived Markdown caches.

## 4. Verification

- [x] 4.1 Run `cargo fmt --check`, `cargo test`, and `cargo build` for the root application package.
- [x] 4.2 Verify the Windows, macOS, and Linux release build configuration accepts the new native-dialog dependency and its default Linux XDG portal backend.
- [x] 4.3 Smoke-test Save As plus each export action on an available desktop platform, checking the dialog title/filter, extension normalization, overwrite handling, cancellation, and reported final path.
