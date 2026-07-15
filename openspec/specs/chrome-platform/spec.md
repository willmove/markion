# chrome-platform

## Purpose

Covers the application chrome: view modes, menus, status bar, themes, focus/typewriter modes, find/replace, preferences, cross-platform behavior, performance characteristics, and error feedback. Interface internationalization is tracked separately under the `ui-i18n` capability. Font-family/size configuration, per-theme code-highlight themes, extension-syntax toggles, error logging to file, and crash-report prompts are **not** part of this capability — they are future candidates.
## Requirements
### Requirement: View modes and application chrome
The editor SHALL provide source, split, and preview view modes, a toggleable sidebar (file tree / outline), a visible in-window menu bar (File, Edit, View, Format, Export, Help) with click-outside-to-close behavior, and a status bar.

#### Scenario: View modes are switchable
- **WHEN** the user switches between source, split, and preview modes
- **THEN** the editor pane layout updates accordingly

#### Scenario: In-window menu bar and status bar
- **WHEN** the editor is running
- **THEN** a visible in-window menu bar and a status bar are present, and open menus close on outside click

### Requirement: Built-in and custom themes
The editor SHALL ship a fixed catalog of built-in themes (the original six — Paper, Ink, Solar, Forest, Rose, Graphite — kept first and in order, plus popular editor palettes) and SHALL load user-defined `.theme` files (hex-color key=value format) from the local themes directory. Theme names are identity keys written to the preferences file. Customization is via the `.theme` color format; CSS-based theming is **not** supported.

#### Scenario: Built-in catalog order is preserved
- **WHEN** the built-in theme catalog is enumerated
- **THEN** the original six themes appear first and in order, so legacy theme cycling keeps working

#### Scenario: Custom themes extend the list
- **WHEN** `.theme` files exist in the themes directory
- **THEN** they extend the theme list, with built-ins winning on name collisions

#### Scenario: Theme application and persistence
- **WHEN** the user selects a theme
- **THEN** it is applied immediately and its name is persisted to the preferences file

### Requirement: Focus mode and typewriter mode
The editor SHALL provide a focus mode that dims text outside the current paragraph and a typewriter mode that keeps the current line near the vertical center while editing. Each mode SHALL be independently toggleable and persisted.

#### Scenario: Focus mode dims non-current paragraphs
- **WHEN** focus mode is enabled and the cursor is in a paragraph
- **THEN** text outside the current paragraph is rendered dimmed

#### Scenario: Typewriter mode recenters the cursor
- **WHEN** typewriter mode is enabled and the user types or moves between lines
- **THEN** the editor scrolls to keep the current line near the vertical center

#### Scenario: Both modes persist
- **WHEN** the user toggles focus or typewriter mode
- **THEN** the choice is applied and persists across launches

### Requirement: Find and replace
The editor SHALL provide a find/replace workflow supporting case-sensitive and regular-expression search, next/previous match navigation, current-match and total counts, replace current, and replace all. The Find / Replace controls SHALL render as a compact floating overlay near the upper-right of the editor workspace, above the editor/preview panes, without consuming layout height or shifting the main workspace. The overlay SHALL provide an explicit close control that hides the overlay, clears active match highlighting and search focus, and preserves the current query and replacement text for a later reopen. The overlay, fields, buttons, borders, hover states, and summary text SHALL use the active theme palette rather than hard-coded light colors.

#### Scenario: Search with options
- **WHEN** the user enters a query and toggles case-sensitive or regex
- **THEN** matches are highlighted and the current/total counts are shown

#### Scenario: Navigate, replace, and replace all
- **WHEN** the user steps to next/previous, replaces the current match, or replaces all
- **THEN** the editor navigates/replaces accordingly and updates the match state

#### Scenario: Find overlay does not shift workspace layout
- **WHEN** the user opens Find or Replace
- **THEN** the controls appear as a compact upper-right floating overlay above the editor/preview workspace
- **AND** the tab bar, editor pane, preview pane, and status bar keep their existing layout positions

#### Scenario: Closing the overlay clears active highlights
- **WHEN** the Find / Replace overlay is visible and the user activates its close control
- **THEN** the overlay is hidden
- **AND** active search focus is cleared
- **AND** active match highlighting is cleared
- **AND** the current find query and replacement text are preserved for the next time Find or Replace opens

#### Scenario: Find overlay follows active theme
- **WHEN** the active theme changes
- **THEN** the Find / Replace overlay surface, input fields, buttons, borders, hover states, and summary text render using the active theme palette
- **AND** the overlay does not use hard-coded light-only chrome colors

#### Scenario: Existing Find and Replace behavior is preserved
- **WHEN** the user invokes existing Find / Replace shortcuts or actions
- **THEN** query editing, regex and case-sensitive toggles, next/previous navigation, match counts, replace current, and replace all continue to behave as before

### Requirement: Narrow-scope preferences with persistence and reset
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, sidebar tab, and Heading menu depth (H1–H5 default, optional H1–H6). The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section (`enabled`, `delay_secs`) that is configurable only via the file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family/size, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable.

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab, Heading menu depth)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields
- **THEN** missing values take their documented defaults and the editor starts normally

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary is shown, or all preferences are reset to their defaults

### Requirement: Cross-platform desktop application
The editor SHALL run as a GPUI desktop application and SHALL build and run on Windows (the primary developed platform); the same source targets macOS and Linux via GPUI. On Windows the binary is built as a GUI-subsystem executable.

#### Scenario: Windows build and run
- **WHEN** the project is built on Windows
- **THEN** it produces a GUI-subsystem executable that can be launched directly or via `cargo run`

### Requirement: Derived-state caching for typing-path responsiveness
For each document version, the editor SHALL cache the derived Markdown state (preview blocks, outline, stats, line count) and share it via `Arc`, memoize syntax highlighting across edits, skip derived caches in undo snapshots, and reuse a cached text handle per version. Note this is full-reparse-plus-memoization, not incremental parsing; lazy offscreen rendering and memory-pressure degradation are **not** implemented.

#### Scenario: Derived state is cached per version
- **WHEN** the document is at a given text version
- **THEN** preview blocks, outline, stats, and line count are computed at most once for that version and shared without recomputation

#### Scenario: Highlighting is memoized and bounded
- **WHEN** the same `(language, code)` code block is encountered across edits
- **THEN** its highlighting result is reused, and the highlight cache evicts entries beyond a bounded size

### Requirement: In-app error feedback
The editor SHALL surface operation failures (file read/write, export steps, math validation, empty clipboard, no selection, etc.) as user-facing status messages. Error logging to a local file and a crash-report prompt on next launch are **not** implemented.

#### Scenario: Failures are surfaced as status
- **WHEN** an operation (file I/O, export, etc.) fails
- **THEN** the editor shows a user-facing status message describing the failure

### Requirement: Diagnostic file logging
The editor SHALL write diagnostic logs to a platform-appropriate Markion log directory (Linux `~/.cache/markion/logs`, macOS `~/Library/Logs/Markion`, Windows `%LOCALAPPDATA%\Markion\Logs`) using daily rotation and keeping at most the last 7 files. The default level SHALL be `info`, overridable via the `RUST_LOG` environment variable. Logging SHALL be initialized at startup and record at minimum: startup (with version), preference load/migration events, auto-save failures, and export-engine fallbacks. Logging failures SHALL never prevent the editor from starting.

#### Scenario: Logs rotate daily and are bounded
- **WHEN** the editor runs across multiple days
- **THEN** each day gets its own log file and no more than 7 files are retained

#### Scenario: Log level override
- **WHEN** the editor is launched with `RUST_LOG=debug`
- **THEN** debug-level events are recorded for that run

#### Scenario: Logging failure is non-fatal
- **WHEN** the log directory cannot be created or opened
- **THEN** the editor starts normally without file logging

### Requirement: Read mode preview width
In Read mode, the editor SHALL constrain rendered preview content to a default maximum width of 860px and center that content within the available preview pane. The editor SHALL provide a persisted "Preview adaptive width" preference that is disabled by default; when enabled, Read mode rendered preview content SHALL use the full available preview width. This width preference SHALL NOT affect Edit mode or Split Preview mode.

#### Scenario: Read mode defaults to readable width
- **WHEN** the active view mode is Read and Preview adaptive width is disabled
- **THEN** rendered preview content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Read mode
- **WHEN** the active view mode is Read and Preview adaptive width is enabled
- **THEN** rendered preview content uses the full available preview pane width

#### Scenario: Split Preview mode remains full pane width
- **WHEN** the active view mode is Split Preview
- **THEN** rendered preview content uses the full preview pane width regardless of the Preview adaptive width preference

### Requirement: Preview adaptive width preference persistence
The editor SHALL persist the Preview adaptive width preference in the existing preferences file as an optional boolean that defaults to disabled when missing or invalid. The preference SHALL be included in preferences reset behavior and restored on launch.

#### Scenario: Missing preference falls back to disabled
- **WHEN** the preferences file omits the Preview adaptive width setting
- **THEN** the editor starts with Preview adaptive width disabled

#### Scenario: Preference round-trips
- **WHEN** the user enables Preview adaptive width and restarts the editor
- **THEN** Preview adaptive width remains enabled

#### Scenario: Reset restores readable default
- **WHEN** the user resets preferences
- **THEN** Preview adaptive width is disabled

### Requirement: Dense pane chrome with draggable scrollbars
The application chrome SHALL provide visible, right-side vertical scrollbars for the source editor pane and rendered preview pane when their content exceeds the visible area. The editor SHALL keep main pane gaps, outer padding, and visible separator chrome compact so the source and preview content occupy substantially more of the available window area than the prior spacious layout. Resize handles SHALL remain draggable even when their visible separator is compact.

#### Scenario: Large source document exposes editor scrollbar
- **WHEN** the active document has more source lines than fit in the editor pane
- **THEN** the editor pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible source text

#### Scenario: Large rendered document exposes preview scrollbar
- **WHEN** the active document renders more preview content than fits in the preview pane
- **THEN** the preview pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible rendered content

#### Scenario: Main pane chrome is compact
- **WHEN** the editor renders the main content area
- **THEN** the visual gaps between the sidebar, editor pane, split divider, and preview pane are reduced to approximately 15% of the previous spacious padding
- **AND** source and preview content occupy the reclaimed space

#### Scenario: Resize handles remain usable
- **WHEN** the visible sidebar or editor/preview separator is compact
- **THEN** the user can still drag the separator handle to resize the corresponding panes

#### Scenario: Single-pane modes remain full-width
- **WHEN** the active view mode is Edit or Read
- **THEN** the visible editor or preview pane fills the remaining main workspace instead of retaining split-mode width

### Requirement: Sync scroll preference
The editor SHALL provide a persisted "Sync scroll" preference, disabled by default, that when enabled and the active view mode is Split Preview SHALL couple the source editor and rendered preview scroll positions proportionally: scrolling either pane (mouse wheel, trackpad, or dragging that pane's scrollbar) SHALL move the other pane to the same fraction of its scrollable range, clamped to its bounds. The preference SHALL have no effect in Edit or Read mode, where only one pane is visible. The proportional coupling SHALL be based on each pane's scroll offset divided by its maximum scrollable offset, and SHALL be a no-op for a direction whose pane has no scrollable range. The coupling SHALL NOT force a Markdown reparse, reset the preview list, or disturb the per-version derived-state caches.

#### Scenario: Sync scroll defaults to off
- **WHEN** the editor starts with no `sync_scroll` value in the preferences file
- **THEN** Sync scroll is disabled and the source editor and preview panes scroll independently as before

#### Scenario: Scrolling the editor moves the preview proportionally
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the source editor pane
- **THEN** the rendered preview pane scrolls to the same fraction of its scrollable range as the source editor's current fraction

#### Scenario: Scrolling the preview moves the editor proportionally
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the rendered preview pane
- **THEN** the source editor pane scrolls to the same fraction of its scrollable range as the preview's current fraction

#### Scenario: Sync scroll is inactive outside Split Preview
- **WHEN** Sync scroll is enabled but the active view mode is Edit or Read
- **THEN** scrolling the visible pane does not affect any other pane and the preference persists without error

#### Scenario: A pane with no scrollable range does not drive the other
- **WHEN** Sync scroll is enabled, the view mode is Split Preview, and one pane's content fits within its viewport (no scrollable range)
- **THEN** scrolling that pane does not move the other pane, and the other pane may still scroll independently

### Requirement: Sync scroll preference persistence
The editor SHALL persist the Sync scroll preference in the existing preferences file as an optional boolean that defaults to disabled when missing or invalid. The preference SHALL be included in preferences reset behavior, restored on launch, and migrated from a legacy `preferences.conf` file that contains a `sync_scroll` line.

#### Scenario: Missing preference falls back to disabled
- **WHEN** the preferences file omits the `sync_scroll` setting
- **THEN** the editor starts with Sync scroll disabled

#### Scenario: Invalid value falls back to disabled
- **WHEN** the preferences file contains a `sync_scroll` value that is not a valid boolean
- **THEN** the editor starts with Sync scroll disabled rather than failing

#### Scenario: Preference round-trips
- **WHEN** the user enables Sync scroll and restarts the editor
- **THEN** Sync scroll remains enabled

#### Scenario: Reset restores disabled default
- **WHEN** the user resets preferences
- **THEN** Sync scroll is disabled

#### Scenario: Legacy preferences file migrates the setting
- **WHEN** the editor starts with a legacy `preferences.conf` containing `sync_scroll=true` and no `config.toml`
- **THEN** the value is migrated into `config.toml` and Sync scroll starts enabled

### Requirement: In-window menus SHALL follow the active theme
The in-window menu bar and dropdown menus SHALL derive their backgrounds, text colors, borders, separators, and active states from the active theme palette so both light and dark themes remain readable and visually consistent with the editor chrome.

#### Scenario: Menu bar adapts to a dark theme
- **WHEN** the active theme is a dark theme such as One Dark or GitHub Dark
- **THEN** the in-window menu bar and dropdown menus render with dark-compatible backgrounds and readable text

#### Scenario: Menu bar adapts to a light theme
- **WHEN** the active theme is a light theme such as Paper or GitHub Light
- **THEN** the in-window menu bar and dropdown menus render with light-compatible backgrounds and readable text

#### Scenario: Changing theme updates menus
- **WHEN** the user selects a different theme from Preferences
- **THEN** the in-window menu bar and any subsequently opened dropdown use the newly active theme palette

