## Design

### Backend preference semantics

`ExportBackendPreference` (in `src/model.rs`) has two variants with config tokens:

- `BuiltIn` (token `"builtin"`, serde default): `export_to_with` dispatches straight to the built-in PDF writer / built-in DOCX writer. No pandoc process is spawned, `ExportOutcome.engine_failure` is `None`, and the status message is the neutral built-in one (the DOCX "install pandoc for richer output" hint is dropped for this path because the user explicitly chose the built-in writer).
- `Pandoc` (token `"pandoc"`): today's engine-first flow unchanged — pandoc runs, and on binary-missing or conversion failure the built-in writer silently takes over with the existing failure-category disclosure. The hint that pandoc yields richer output stays meaningful here precisely because the user asked for pandoc.

Fallback is deliberately kept only in the `pandoc` direction: choosing `builtin` and then secretly trying pandoc would defeat the point of a dependency-free default, while choosing `pandoc` and failing hard would break the "export always succeeds" invariant.

`backend_status_msg` loses its `format` parameter: the only remaining built-in-without-failure case is the explicit `builtin` preference, which wants the neutral message for both formats. `Msg::StatusExportedDocxBuiltin` becomes unreachable and is removed.

### Preferences panel

A third `PreferencesTab::Export` joins General and Shortcuts (same 640px width, scrollable body, existing section idioms). Sections:

1. **Export engine** — two `preference_option_button` choices (Built-in / Pandoc) plus a muted availability line. Availability is probed by `pandoc_available()` (a `pandoc --version` subprocess) on a background executor when the tab is opened, when the backend switches to Pandoc, or when the pandoc path changes; the result is cached in app state (`Option<bool>`) so rendering never spawns processes.
2. **Pandoc options** (rendered only while backend = Pandoc):
   - pandoc binary path: shows the configured path or "Auto (system PATH)", with *Browse…* (native open-file dialog via `rfd`, same parenting as the save dialogs) and *Reset* actions;
   - DOCX reference template: shows the configured file or "Bundled template", with *Browse…* / *Reset*;
   - pandoc PDF engine: mutually exclusive buttons for `xelatex` (default), `tectonic`, `pdfroff`, `lualatex`.
3. **Word (DOCX)** — page size (A4/Letter/Legal), table of contents (engine path only; labeled as such), image policy (embed / text fallback).
4. **PDF** — page size, margin stepper (10–50 mm), table of contents, page-number footer.

Setters live on `MarkionApp`, mutate `export_preferences`, call `persist_preferences()`, and set a status message; the panel re-renders from app state like every other preference row.

### Dialog removal

`export_docx` collapses to `export_with_prompt(ExportFormat::Docx, …)`; the three `window.prompt` steps, the `page_size_label` helper, the `pandoc_available` call on the menu path, and the persist-after-success special case in `export_with_prompt` are deleted (options now persist when edited in the panel). The `DialogDocx*` message variants are removed from all seven language functions and from the exhaustive i18n test list.

### Non-goals

- Surfacing `pdf_mainfont` / `pdf_cjk_font` in the panel (stay `[export]`-file-only; font pickers are a heavier UI and a future candidate).
- Any change to HTML/LaTeX/image exports, to the built-in writers' fidelity, or to the pandoc invocation arguments themselves.
