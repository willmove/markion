## Context

Markion caches parsed `PreviewBlock`s per document version via `Arc` and exposes both Source/Split Preview and Visual Edit (`visual_direct_code_editor`) surfaces. External process execution (`src/external_command.rs`) already exists for export/diagram paths, and `Preferences` (`src/storage/preferences.rs`, `src/app/root_view.rs`) uses TOML sections with lenient deserialization and off-thread availability probing.

## Goals / Non-Goals

**Goals:**
- Provide an open-ended `CodeFormatterRegistry` so adding a new language requires only implementing `CodeFormatter` and registering it without rewriting editor dispatch.
- Keep all external formatter execution strictly asynchronous, bounded in input/output size, shell-free (argument lists), and timeout-enforced with process-tree cleanup.
- Ensure formatted output patches only the fenced code payload, preserving fences, info strings, list-item prefixes, and LF/CRLF line endings as a single Undo/Redo step while rejecting stale completions after subsequent typing or tab switches.

**Non-Goals:**
- Silently downloading or executing `npx` for missing formatters.
- Formatting outside editable fenced code block payloads (Read mode, prose, inline code, indented blocks, or diagram fences).

## Decisions

### Extensible `CodeFormatter` trait and `CodeFormatterRegistry`

Define `CodeFormatter` (`id`, `supports_language`, `format`, `probe_availability`, `missing_tool_hint`) and register `JsonFormatter`, `JsTsFormatter`, `JavaFormatter`, `RustFormatter`, and `PythonFormatter` inside `CodeFormatterRegistry`. Language normalization maps fence info-string aliases (`json`, `jsonc`, `js`, `javascript`, `jsx`, `ts`, `typescript`, `tsx`, `java`, `rs`, `rust`, `py`, `python`) case-insensitively on the first info-string token.

### Async pipeline and stale-state validation

Data flow for manual and paste-triggered formatting:

`Editable fenced block range + cursor -> capture (tab_id, doc_id, doc_version, code_format_generation, payload_range, payload_snapshot, line_ending) -> background thread CodeFormatter::format -> foreground revalidation (matching tab_id, doc_id, doc_version, generation, and exact payload slice) -> checked single-step source edit (payload_range only) + caret clamp -> single Undo/Redo entry`

If the user types further, switches/closes the tab, switches to Read mode, or the formatter errors/times out, the captured generation/version/slice check fails or cancels cleanly and the user's buffer remains untouched.

### Safe external invocation on Windows and Unix

`src/external_command.rs` extends `run_external_command_with_input` to terminate child process trees on timeout (`taskkill /F /T` on Windows; signal on Unix) and supports direct execution of executables with spaces, Windows `.ps1` scripts via `powershell.exe -NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File`, and `google-java-format` `.jar` archives via `<java_path> -jar <jar> -`.

### Isolated Preferences tab (`Editor -> Code Blocks`)

`PreferencesTab::Editor` hosts the `Code Blocks` section (`src/app/code_format_prefs.rs`), keeping `General` uncluttered while reusing existing button/toggle/file-picker/probe patterns and storing settings under `[code_blocks]` in `config.toml`.

## Risks / Trade-offs

- [User pastes HTML-formatted code from an IDE into a fenced block] -> When inside an editable fenced code block payload, paste bypasses HTML-to-Markdown/table conversion and inserts literal clipboard plain text atomically before dispatching async formatting.
- [Background formatter finishes after user keeps typing] -> Generation counter plus `document_id`, `document_version`, and exact payload slice comparison prevent stale writes.
- [Missing external formatter causes UI stall or noise] -> Probes run asynchronously when the Editor tab opens or paths change; missing formatters during editing leave text intact and show localized non-intrusive status guidance.

## Migration Plan

Existing `config.toml` files without `[code_blocks]` load with `format_on_paste = false`, `js_ts_backend = "auto"`, and empty custom tool paths. Preferences Reset restores those defaults cleanly.
