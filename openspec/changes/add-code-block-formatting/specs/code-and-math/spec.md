## ADDED Requirements

### Requirement: Extensible fenced code block formatter registry
The editor SHALL provide an extensible `CodeFormatter` and `CodeFormatterRegistry` architecture for formatting fenced code block payloads without modifying core editor dispatch for each new language. Built-in JSON formatting SHALL use `serde_json` while preserving key order; JavaScript/TypeScript SHALL support Biome and Prettier adapters with explicit or automatic backend selection; Java SHALL support `google-java-format` executables and `.jar` archives invoked via `java -jar`; Rust SHALL support `rustfmt`; and Python SHALL support `ruff format`. External formatters SHALL be invoked asynchronously off the GPUI UI thread via direct process argument lists without shell command concatenation, SHALL enforce bounded input/output sizes and execution timeouts with child process-tree termination, SHALL NOT silently download or install missing tools, and SHALL NOT execute pasted user code.

#### Scenario: Built-in JSON formatter formats valid JSON without external tools
- **WHEN** a fenced code block tagged `json` contains valid unformatted JSON and formatting is invoked
- **THEN** the registry formats the JSON payload in-process via `serde_json` without spawning an external process
- **AND** invalid JSON leaves the original code block content untouched and reports a localized failure status

#### Scenario: External formatter runs with direct arguments and timeout protection
- **WHEN** formatting runs for a supported external language (JS, TS, Java, Rust, or Python)
- **THEN** the configured or resolved executable (including `.jar` via `java -jar` or paths containing spaces) receives stdin via direct argument arrays with no shell interpolation
- **AND** a hanging formatter process is terminated on timeout without freezing the UI or modifying the document

#### Scenario: Missing formatter fails safely with clear feedback
- **WHEN** formatting is requested for a language whose external formatter binary or archive cannot be resolved
- **THEN** no silent download or install is attempted
- **AND** the document remains unchanged while localized status guidance identifies the unavailable formatter

### Requirement: Manual and paste-triggered code block formatting
The editor SHALL expose a manual **Format Code Block** command via keyboard shortcut (`Ctrl+Alt+L` on Windows/Linux, `Cmd+Option+L` on macOS), Format menu bar entry, editable Visual Edit and Split Preview code block header buttons, and editable source/visual context menus, even when automatic format-on-paste is disabled. When **Format on Paste** is enabled in preferences and the user pastes inside an editable fenced code block payload with a recognized formatter, the editor SHALL insert the clipboard plain text and schedule asynchronous formatting. Automatic formatting SHALL NOT trigger in Read mode, ordinary prose paragraphs, inline code spans, indented code blocks, unfenced or untagged fences, or normal non-paste typing. Every committed format SHALL replace only the code block payload while preserving opening/closing fences, info strings, LF/CRLF line endings, and list-item indentation as a single undoable transaction, and SHALL reject stale async completions if the user edits further, switches tabs, or enters Read mode before completion.

#### Scenario: Format on paste respects setting and excludes non-fence contexts
- **WHEN** Format on Paste is disabled, or when the user pastes into prose, inline code, an indented block, an untagged fence, or Read mode
- **THEN** automatic code formatting does not run
- **AND** enabling Format on Paste and pasting into a tagged `json` fence formats the pasted payload asynchronously

#### Scenario: Manual triggers format without enabling format-on-paste
- **WHEN** Format on Paste is off and the user activates the code block Format button, context menu item, or keyboard shortcut in Source, Split Preview, or Visual Edit
- **THEN** only that targeted fenced block payload is formatted while opening/closing fences, info string, and surrounding Markdown remain intact
- **AND** Undo and Redo restore or reapply the formatted payload in one step, preserving LF or CRLF endings

#### Scenario: Concurrent typing or tab switch discards stale formatting results
- **WHEN** an asynchronous formatter task is in flight and the user types into the document or switches to another tab before it completes
- **THEN** the late formatting completion is rejected by version, generation, and slice checks
- **AND** neither the edited document nor the active tab is overwritten
