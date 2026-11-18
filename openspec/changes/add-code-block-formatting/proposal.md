## Why

Markion syntax-highlights fenced code blocks across many languages, but pasting unformatted code (JSON, JavaScript, TypeScript, Java, Rust, Python) leaves broken indentation and layout intact, and there is no way to manually format a fenced code block inside the editor. Users expect IDE-style on-demand formatting and opt-in format-on-paste without sacrificing Markdown fidelity, Undo/Redo, or UI responsiveness.

## What Changes

- Introduce an extensible `CodeFormatter` trait and `CodeFormatterRegistry` in `src/code_format.rs` with built-in JSON formatting (`serde_json` with key order preserved) and external formatter adapters for JavaScript/TypeScript (Biome or Prettier), Java (`google-java-format` executable or `.jar` via `java -jar`), Rust (`rustfmt`), and Python (`ruff format`).
- Add direct-argument subprocess execution with bounded stdin/stdout/stderr, configurable timeout with process-tree termination, Windows `.cmd`/`.ps1`/space-containing path handling, no shell concatenation, no silent tool downloads, and no execution of pasted code.
- Support optional **Format on Paste** (disabled by default) when pasting inside an authored fenced code block payload with a recognized language, plus manual **Format Code Block** (`Ctrl+Alt+L` / `Cmd+Option+L`, Visual Edit and Split Preview block header buttons, editable source/visual context menus, and the Format menu bar).
- Add a **Preferences -> Editor -> Code Blocks** tab with Format on Paste toggle, JS/TS backend selector, tool path pickers/resets, and asynchronous availability probing, backed by `[code_blocks]` persistence in `config.toml`.
- Guarantee Markdown fidelity (fence markers, info string, LF/CRLF line endings, list indentation preserved; single-step Undo/Redo) and stale/tab-switch guards on async formatting completions.
- Route all new labels, tooltips, and status messages across all eight supported UI languages in `src/code_format_i18n.rs`.
- **Non-goals:** formatting prose, inline code spans, indented code blocks, or untagged fences; bundled external runtimes or automatic installation of third-party formatters; redesigning per-version Markdown derivation or syntax-highlighting caches.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `code-and-math`: add extensible fenced code block formatter registry, manual formatting triggers, and opt-in format-on-paste with Markdown fidelity and async stale guards.
- `theme-preferences`: add `Preferences -> Editor -> Code Blocks` UI and safe `[code_blocks]` TOML persistence.
- `keyboard-shortcuts`: register customizable `format-code-block` shortcut (`Ctrl+Alt+L` / `Cmd+Option+L`).
- `ui-i18n`: localize all code block formatting UI chrome and status feedback across all eight supported interface languages.

## Impact

- `src/code_format.rs`, `src/code_format_i18n.rs`, `src/external_command.rs`, `src/model.rs`, `src/storage/preferences.rs`: formatter registry, fenced code block localization/patching helpers, timeout process-tree termination, `[code_blocks]` persistence, and 8-language catalog.
- `src/app/editing.rs`, `src/app/preview.rs`, `src/app/root_view.rs`, `src/app/code_format_prefs.rs`, `src/app/bootstrap.rs`, `src/app/mod.rs`: manual/paste triggers, async dispatch & stale validation, Preferences Editor tab, and menus/shortcuts.
- `src/app/tests.rs`: unit and GPUI integration tests covering JSON valid/invalid, external tools, missing-formatter UX, Format on Paste toggle and exclusions, manual triggers, Undo/Redo, CRLF/LF, concurrent typing, tab switching, and timeouts.
- Preserves architecture invariants: formatting runs asynchronously off the GPUI UI thread; per-version `Arc`-shared Markdown derivation, memoized syntax highlighting, and cached text handles remain untouched until a verified formatted payload commits as a single checked document transaction.
