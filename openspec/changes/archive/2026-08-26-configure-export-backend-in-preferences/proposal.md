## Proposal

### Why

PDF/DOCX export currently always tries the pandoc engine first and silently falls back to the built-in writers, and the only way to influence that choice is by installing (or not installing) pandoc. The DOCX flow additionally interrupts every export with a three-step options dialog (page size, TOC, image policy) whose answers are really durable preferences, not per-export decisions. Users who prefer the deterministic, dependency-free built-in writers — or who want pandoc with a custom binary path and a custom Word reference template — have no first-class way to say so.

### What Changes

- Add an `[export] backend` preference (`builtin` | `pandoc`, default `builtin`) that selects which implementation produces PDF/DOCX exports:
  - `builtin` (default): the built-in PDF writer and built-in DOCX writer produce the file directly; no pandoc subprocess is spawned. This inverts today's engine-first default.
  - `pandoc`: the Typune pandoc engine runs first; the existing silent built-in fallback and status-bar failure-category disclosure are preserved so export still always succeeds.
- Add an **Export** tab to the in-app Preferences panel exposing:
  - the backend choice, with a pandoc-availability status line (probed in the background);
  - pandoc-only options, shown when `pandoc` is selected: custom pandoc binary path (with a system file picker and a reset-to-PATH action), custom DOCX reference template (picker + reset-to-bundled), and the pandoc PDF engine (xelatex / tectonic / pdfroff / lualatex);
  - Word (DOCX) options: page size, table of contents, image policy;
  - PDF options: page size, margin, table of contents, page numbers.
  All changes apply immediately and persist through the existing preferences file.
- Remove the DOCX pre-export options dialogs; DOCX export now goes straight to the save-path prompt like every other format, reading its options from the preferences.
- Adjust backend disclosure: a built-in export under the explicit `builtin` preference reports neutrally (no "install pandoc" hint); the hint remains only when the `pandoc` preference actually fell back.
- Localize every new panel string in all seven UI languages.

### Impact

- Affected specs: `export` (backend selection requirement, DOCX/PDF options requirements, pandoc path requirement), `theme-preferences` (new Preferences-panel Export tab requirement), `ui-i18n` (new localized surface).
- Code: `src/model.rs` (new `ExportBackendPreference` + `ExportPreferences.backend`), `src/storage/preferences.rs` (`[export] backend` token), `src/lib.rs` (`export_to_with` backend branch), `src/export.rs` (status-message mapping simplification), `src/app/documents.rs` (dialog removal), `src/app/mod.rs` / `application.rs` / `root_view.rs` / `appearance.rs` (new tab, setters, availability probe), `src/app/save_dialog.rs` (open-file picker helper), `src/i18n.rs` (new messages; removal of the now-dead DOCX-dialog messages).
- Compatibility: existing config files without `backend` keep parsing and default to `builtin`; no persisted format is removed. Behavior change: pandoc no longer runs unless the user opts in.
