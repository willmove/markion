## ADDED Requirements

### Requirement: Preferences panel SHALL expose an Editor tab for Code Blocks
The Preferences panel SHALL provide an **Editor** tab exposing a **Code Blocks** configuration section using active-theme colors and the active interface language. The section SHALL include an actionable **Format on Paste** toggle (default off), a **JS / TS Formatter** backend selector (`Auto`, `Biome`, `Prettier`), explicit path pickers and reset actions for external formatter executables or JARs (`Biome`, `Prettier`, `google-java-format`, `Java`, `rustfmt`, `ruff`), and an asynchronous off-UI-thread availability probe summarizing tool availability without spawning processes during render.

#### Scenario: Editor tab toggles Format on Paste and probes availability off-thread
- **WHEN** the user opens Preferences -> Editor -> Code Blocks
- **THEN** Format on Paste reflects the persisted boolean (off by default) and toggles immediately with persistence
- **AND** formatter availability is probed asynchronously off the UI thread and rendered with localized status labels

### Requirement: Code block formatting preferences SHALL persist safely
The editor SHALL persist code block formatting settings under the `[code_blocks]` table in `config.toml` (`format_on_paste`, `js_ts_backend`, `biome_path`, `prettier_path`, `google_java_format_path`, `java_path`, `rustfmt_path`, `ruff_path`). Missing, empty, or invalid-typed fields SHALL fall back safely to documented defaults (`format_on_paste = false`, `js_ts_backend = "auto"`, unset paths) without preventing startup or discarding unrelated settings. Preferences reset SHALL restore default code block settings.

#### Scenario: Code block preferences round-trip and tolerate malformed fields
- **WHEN** `[code_blocks]` settings including Windows paths with spaces or `.jar` paths are saved and reloaded, or when malformed field types appear in `config.toml`
- **THEN** valid settings round-trip accurately while malformed or missing fields fall back safely to defaults
- **AND** Reset Preferences restores Format on Paste to off, JS/TS backend to Auto, and clears explicit tool paths
