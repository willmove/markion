# chrome-platform

## Purpose

Covers the application chrome: view modes, menus, status bar, themes, focus/typewriter modes, find/replace, preferences, cross-platform behavior, performance characteristics, and error feedback. Interface internationalization is tracked separately under the `ui-i18n` capability. Font-family/size configuration, per-theme code-highlight themes, extension-syntax toggles, error logging to file, and crash-report prompts are **not** part of this capability — they are future candidates.
## Requirements
### Requirement: View modes and application chrome
The editor SHALL provide source, split, and preview view modes, a toggleable sidebar (file tree / outline), a visible in-window menu bar (File, Edit, View, Format, Export, Help) with click-outside-to-close behavior, and a status bar. When the sidebar is visible, its column SHALL begin directly below the menu bar and extend through both the document-tab band and main content region, while document-tab controls and document panes remain in the adjacent document-workspace column.

#### Scenario: View modes are switchable
- **WHEN** the user switches between source, split, and preview modes
- **THEN** the editor pane layout updates accordingly

#### Scenario: In-window menu bar and status bar
- **WHEN** the editor is running
- **THEN** a visible in-window menu bar and a status bar are present, and open menus close on outside click

#### Scenario: Visible sidebar occupies the workspace from its top edge
- **WHEN** the Files or Outline sidebar is visible
- **THEN** the sidebar begins immediately below the menu bar and its tab controls occupy the top of that column
- **AND** no empty document-tab-band spacer is rendered above the sidebar
- **AND** any visible document-tab controls begin in the adjacent document-workspace column and remain aligned when the sidebar is resized

#### Scenario: Hidden sidebar returns the full workspace width
- **WHEN** the sidebar is hidden
- **THEN** the document-tab controls and document panes use the full available workspace width below the menu bar

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
The editor SHALL provide a Preferences panel and a persisted preferences file covering: theme (and custom theme selection), focus mode, typewriter mode, code-line-numbers, sidebar visibility, sidebar tab, Heading menu depth (H1–H5 default, optional H1–H6), source-editor font size, rendered-document font size, and rendered paragraph spacing. The preferences file SHALL be TOML (`config.toml` in the Markion config directory) with every field optional and defaulted, and SHALL additionally carry an `[auto_save]` section (`enabled`, `delay_secs`) that is configurable only via the file, not the panel. On startup, if `config.toml` does not exist but a legacy `preferences.conf` (the retired `key=value` format) does, the editor SHALL migrate it to `config.toml` once and thereafter ignore the legacy file. The editor SHALL also offer a preference reset action and a preferences summary in the Help menu. Font family, code-highlight theme, extension-syntax toggles, and image-uploader credentials are **not** configurable.

#### Scenario: Supported preferences persist and restore
- **WHEN** the user changes a supported preference (theme, focus mode, typewriter mode, code line numbers, sidebar visibility, sidebar tab, Heading menu depth, source-editor font size, rendered-document font size, or rendered paragraph spacing)
- **THEN** the change is written to `config.toml` and restored on the next launch

#### Scenario: Legacy preferences file is migrated once
- **WHEN** the editor starts with no `config.toml` but a legacy `preferences.conf` present
- **THEN** the legacy values are loaded, written out as `config.toml`, and used; subsequent launches read only `config.toml`

#### Scenario: Partial or missing config falls back to defaults
- **WHEN** `config.toml` is missing, or present but omits fields
- **THEN** missing values take their documented defaults and the editor starts normally

#### Scenario: Preferences summary and reset
- **WHEN** the user opens the Help → preferences summary or triggers the reset action
- **THEN** a summary including supported typography values is shown, or all preferences including typography are reset to their defaults

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
In Read mode and Visual Edit mode, the editor SHALL constrain rendered content to a default maximum width of 860px and center that content within the available pane. The editor SHALL provide a persisted "Preview adaptive width" preference that is disabled by default; when enabled, Read mode and Visual Edit mode rendered content SHALL use the full available pane width. This width preference SHALL NOT affect Edit mode or Split Preview mode.

#### Scenario: Read mode defaults to readable width
- **WHEN** the active view mode is Read and Preview adaptive width is disabled
- **THEN** rendered preview content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Read mode
- **WHEN** the active view mode is Read and Preview adaptive width is enabled
- **THEN** rendered preview content uses the full available preview pane width

#### Scenario: Visual Edit mode defaults to readable width
- **WHEN** the active view mode is Visual Edit and Preview adaptive width is disabled
- **THEN** rendered visual edit content is centered and constrained to a maximum width of 860px

#### Scenario: Adaptive width restores full-width Visual Edit mode
- **WHEN** the active view mode is Visual Edit and Preview adaptive width is enabled
- **THEN** rendered visual edit content uses the full available pane width

#### Scenario: Split Preview mode remains full pane width
- **WHEN** the active view mode is Split Preview
- **THEN** rendered preview content uses the full preview pane width regardless of the Preview adaptive width preference

#### Scenario: Edit mode remains full pane width
- **WHEN** the active view mode is Edit
- **THEN** the source editing surface uses the full pane width regardless of the Preview adaptive width preference

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
The application chrome SHALL provide visible, right-side vertical scrollbars for the source editor pane, Visual Edit surface, and rendered preview pane when their content exceeds the visible area. The Visual Edit scrollbar SHALL match the Read-mode preview overlay in placement and drag behavior. The Preferences panel SHALL provide the same draggable, right-side vertical scrollbars for each of its scrollable regions — the General tab body, the Shortcuts category sidebar, the Shortcuts action list, and the Export tab body — whenever a region's content exceeds its visible area; wheel and trackpad scrolling SHALL continue to work unchanged. The editor SHALL keep main pane gaps, outer padding, and visible separator chrome compact so the source and preview content occupy substantially more of the available window area than the prior spacious layout. Resize handles SHALL remain draggable even when their visible separator is compact.

#### Scenario: Large source document exposes editor scrollbar
- **WHEN** the active document has more source lines than fit in the editor pane
- **THEN** the editor pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible source text

#### Scenario: Large rendered document exposes preview scrollbar
- **WHEN** the active document renders more preview content than fits in the preview pane
- **THEN** the preview pane shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible rendered content

#### Scenario: Large Visual Edit document exposes a scrollbar
- **WHEN** the active view mode is Visual Edit
- **AND** the visual document renders more content than fits in the visible surface
- **THEN** the Visual Edit surface shows a right-side vertical scrollbar
- **AND** dragging that scrollbar changes the visible visual document content
- **AND** the thumb placement and drag behavior match the Read-mode preview scrollbar

#### Scenario: Short or empty Visual Edit documents hide the scrollbar
- **WHEN** the active view mode is Visual Edit
- **AND** the visual document fits in the visible surface or the document is empty
- **THEN** no vertical scrollbar thumb is shown

#### Scenario: Overflowing Preferences panel region exposes a scrollbar
- **WHEN** the Preferences panel is open
- **AND** a scrollable panel region (General tab body, Shortcuts category sidebar, Shortcuts action list, or Export tab body) contains more content than fits its visible area
- **THEN** that region shows a right-side vertical scrollbar thumb
- **AND** dragging the thumb with the left mouse button scrolls that region up and down
- **AND** the thumb position reflects the region's scroll offset

#### Scenario: Fitting Preferences panel content hides the scrollbar
- **WHEN** the Preferences panel is open
- **AND** a scrollable panel region's content fits within its visible area
- **THEN** no vertical scrollbar thumb is shown for that region

#### Scenario: Preferences panel wheel scrolling is preserved
- **WHEN** the Preferences panel is open
- **AND** the user scrolls a scrollable panel region with the mouse wheel or trackpad
- **THEN** the region scrolls exactly as before the draggable scrollbar was added
- **AND** the scrollbar thumb moves to reflect the new scroll offset

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
The editor SHALL provide a persisted "Sync scroll" preference, disabled by default, that when enabled and the active view mode is Split Preview SHALL couple the source editor and rendered preview by document location rather than by whole-document scroll percentage. Scrolling either pane by mouse wheel, trackpad, scrollbar drag, or an existing editor navigation action SHALL establish a source-backed viewport anchor at that pane's top content edge; the other pane SHALL align the corresponding source location at its own top content edge, except where clamping at the start or end of a scrollable range prevents exact top alignment. The mapping SHALL use rendered blocks' source ranges, SHALL interpolate relative progress within a source-backed block, and SHALL deterministically bridge source gaps that have no rendered content. The preference SHALL have no effect in Edit, Visual Edit, or Read mode, where both panes are not visible.

Synchronization SHALL be a no-op in a direction whose driving pane has no scrollable range or whose current preview mapping cannot identify a valid source location. When the preview list contains blocks for an older document version, synchronization SHALL NOT use those stale source ranges or force a Markdown parse; it SHALL retain the latest driving-pane intent and reconcile once the normal debounced preview update supplies a current mapping. A source-to-preview jump whose target virtual row has not been measured SHALL first reveal that row and then refine the within-row offset after layout, without falling back to whole-document percentage coupling or entering a feedback loop. Synchronization SHALL NOT reset the preview list, mutate the document, force a Markdown reparse, or disturb per-version derived-state caches.

#### Scenario: Sync scroll defaults to off
- **WHEN** the editor starts with no `sync_scroll` value in the preferences file
- **THEN** Sync scroll is disabled and the source editor and preview panes scroll independently as before

#### Scenario: Scrolling the editor aligns the corresponding preview content
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the source editor pane
- **THEN** the rendered preview aligns the block and relative block position corresponding to the source location at the editor viewport anchor
- **AND** the result is independent of unrelated differences between the panes' total scrollable heights

#### Scenario: Scrolling the preview aligns the corresponding source content
- **WHEN** Sync scroll is enabled, the active view mode is Split Preview, and the user scrolls the rendered preview pane
- **THEN** the source editor aligns the source location and relative block position represented at the preview viewport anchor
- **AND** the follower movement does not become a new preview-driving scroll on the next frame

#### Scenario: Non-uniform rendered blocks do not accumulate drift
- **WHEN** a Split Preview document contains blocks whose rendered heights differ substantially from their source heights, such as images, tables, wrapped prose, or code fences
- **AND** the user scrolls through multiple such blocks with Sync scroll enabled
- **THEN** each pane continues to show the same source-backed block near its viewport anchor instead of drifting according to total document percentage

#### Scenario: Source positions without rendered content bridge deterministically
- **WHEN** the editor viewport anchor falls in blank lines, link definitions, or another source interval with no independently rendered preview block
- **THEN** the preview target is derived from the adjacent source-backed block anchors
- **AND** continued scrolling across that interval does not jump to an unrelated document region

#### Scenario: An unmeasured preview target is refined after layout
- **WHEN** an editor-driven scrollbar jump targets a virtualized preview row that has not yet been measured
- **THEN** the preview first reveals the source-matched row
- **AND** after that row is measured, the preview refines its within-row offset to the source-mapped position without oscillating back to the editor pane

#### Scenario: Stale preview blocks defer source-mapped reconciliation
- **WHEN** the document version is newer than the source ranges represented by the debounced preview list
- **AND** the user scrolls either pane with Sync scroll enabled
- **THEN** the editor does not use the stale ranges and does not synchronously reparse Markdown
- **AND** once current preview blocks arrive through the normal debounce path, the latest driving pane reconciles the other pane by source location

#### Scenario: Sync scroll is inactive outside Split Preview
- **WHEN** Sync scroll is enabled but the active view mode is Edit, Visual Edit, or Read
- **THEN** scrolling the visible pane does not affect any other pane and the preference persists without error

#### Scenario: A pane with no scrollable range does not drive the other
- **WHEN** Sync scroll is enabled, the view mode is Split Preview, and one pane's content fits within its viewport
- **THEN** that pane does not move the other pane, and the other pane may still scroll independently

#### Scenario: Document boundaries remain clamped
- **WHEN** a source-mapped target would place either pane before its start or beyond its maximum scroll offset
- **THEN** that pane is clamped to the corresponding document boundary
- **AND** reaching the document start or end in the driving pane reaches the same boundary in the follower pane

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

### Requirement: Square-corner primary document surfaces
The application chrome SHALL render the primary source editor, visual editor, and rendered preview surfaces as square-corner rectangles with zero corner radius in every view mode where those surfaces appear. The surfaces SHALL retain their active-theme background fill, border, padding, scrollbar behavior, and existing input interactions. Rounded styling on secondary controls and content elements is outside this requirement.

#### Scenario: Source editor uses square corners
- **WHEN** the active view mode is Edit or Split Preview
- **THEN** the source editor surface is rendered with square, zero-radius corners

#### Scenario: Visual editor uses square corners
- **WHEN** the active view mode is Visual Edit
- **THEN** the visual editor surface is rendered with square, zero-radius corners

#### Scenario: Preview uses square corners
- **WHEN** the active view mode is Split Preview or Read
- **THEN** the rendered preview surface is rendered with square, zero-radius corners

#### Scenario: Existing surface chrome and behavior are preserved
- **WHEN** a square-corner primary document surface is rendered
- **THEN** its active-theme background fill, border, padding, scrolling, resizing, drag-and-drop handling, and mode-specific visibility behave as before

### Requirement: Persistent document context in the status bar
The status bar SHALL retain its existing document identity, save-state, and transient operation feedback while also presenting a compact persistent context for the active tab. The persistent context SHALL include the active document's character count and word count, the one-based line and column of the active caret whenever an editing surface is present, and the current named Git branch when a repository can be resolved from the active document or workspace. Character count SHALL count Unicode scalar values, including whitespace and line breaks, and word count SHALL count contiguous non-whitespace sequences. Every new user-visible label SHALL use the active interface language. Document metrics SHALL reuse per-document-version derived state, and Git discovery or refresh SHALL NOT perform filesystem or process work on the render or typing path.

#### Scenario: Active document metrics are always visible
- **WHEN** an active tab displays a document in any view mode
- **THEN** the persistent status context shows that document's character count and word count
- **AND** editing the document updates both values from the new document version

#### Scenario: Counts have defined Unicode and whitespace semantics
- **WHEN** a document contains non-ASCII text, emoji, whitespace, and line breaks
- **THEN** the character count equals the number of Unicode scalar values in the complete source
- **AND** the word count equals the number of contiguous non-whitespace source sequences

#### Scenario: Editing modes show the active caret position
- **WHEN** the active view mode is Edit, Visual Edit, or Split Preview
- **THEN** the persistent status context shows the active caret's one-based logical line and Unicode-scalar column
- **AND** a non-empty selection reports the position of its active caret end rather than always reporting the selection's lower offset

#### Scenario: Read mode omits caret position
- **WHEN** the active view mode is Read and no editing caret is presented
- **THEN** the persistent status context omits the line-and-column item
- **AND** the character count, word count, and any available Git branch remain visible

#### Scenario: Named Git branch is shown for repository-backed context
- **WHEN** the active saved document belongs to a Git working tree with a named branch
- **THEN** the persistent status context shows that branch name
- **AND** the active document's nearest repository takes precedence over a broader workspace repository

#### Scenario: Unsaved document uses an established workspace
- **WHEN** the active document has no filesystem path but the user has established a workspace inside a Git working tree with a named branch
- **THEN** the persistent status context shows the workspace repository's branch name

#### Scenario: Unavailable branch is omitted without replacing feedback
- **WHEN** neither the active document nor established workspace belongs to a Git repository, Git HEAD is detached, or repository metadata cannot be read
- **THEN** the Git branch item is omitted
- **AND** the status bar continues to show document metrics and existing transient operation feedback without surfacing the lookup failure as an operation error

#### Scenario: Branch context follows repository changes
- **WHEN** the active document or workspace changes, or the repository switches to another named branch while Markion remains open
- **THEN** the persistent status context eventually refreshes to the branch for the current context
- **AND** a stale lookup result from an earlier document or workspace is not displayed

#### Scenario: Switching tabs updates all document context
- **WHEN** the user activates a different tab
- **THEN** counts and caret position immediately describe the newly active tab
- **AND** the Git branch item resolves from the newly active tab or its workspace fallback

#### Scenario: Persistent context coexists with transient feedback
- **WHEN** a save, export, search, formatting, error, or other existing operation updates the transient status message
- **THEN** that feedback remains visible in the status bar alongside the persistent document context
- **AND** the status bar remains a single compact row without overlapping or wrapping its items

#### Scenario: Context labels follow the active language
- **WHEN** the active interface language changes
- **THEN** the character, word, branch, line, and column labels are rendered through the localization catalog in the selected language
- **AND** document text and the branch name are displayed verbatim rather than translated

#### Scenario: Status rendering preserves typing-path caches
- **WHEN** the document is rendered repeatedly without a text-version change, or only the caret moves
- **THEN** the status bar reuses the cached document metrics for that version
- **AND** caret-only changes do not invalidate Markdown-derived state
- **AND** Git discovery and refresh do not run synchronously during rendering or text input

### Requirement: Help menu external links

The Help menu SHALL offer a "Report an Issue" item and an "Online Documentation" item, positioned between the update check and the About action, in both menu surfaces the application renders: the in-window menu bar dropdown and the native OS menu bar. Invoking "Report an Issue" SHALL open `https://github.com/willmove/markion/issues/new` and invoking "Online Documentation" SHALL open `https://github.com/willmove/markion#readme` — each in the system default browser via the platform shell, never inside an embedded web view, and the application SHALL keep running normally afterwards. Both items SHALL be pointer-driven with no keyboard shortcuts and no shortcut-reference entries. Invoking either item from the in-window dropdown SHALL dismiss the open menu. Both item labels SHALL be routed through the localization layer and render in the active language like every other menu item.

#### Scenario: Report an Issue opens the issue tracker in the browser

- **WHEN** the user chooses "Report an Issue" from the Help menu (in-window dropdown or native menu bar)
- **THEN** the system default browser opens the project's new-issue page (`https://github.com/willmove/markion/issues/new`)
- **AND** the application renders no embedded web content and continues running normally

#### Scenario: Online Documentation opens the documentation home in the browser

- **WHEN** the user chooses "Online Documentation" from the Help menu (in-window dropdown or native menu bar)
- **THEN** the system default browser opens the project's documentation home (`https://github.com/willmove/markion#readme`)
- **AND** the application renders no embedded web content and continues running normally

#### Scenario: Invoking an external link closes the in-window dropdown

- **WHEN** the user clicks either external-link item in the open in-window Help dropdown
- **THEN** the dropdown closes and no menu item stays highlighted

#### Scenario: External link labels follow the active language

- **WHEN** the interface language is switched
- **THEN** both external-link item labels re-render in the new language in the in-window menu bar and in the reinstalled native menu bar

### Requirement: File paths SHALL be presented and persisted in platform-normal form

File paths shown to the user, written to the clipboard, or persisted in session state SHALL be in platform-normal form — on Windows, drive-rooted paths SHALL NOT carry the extended-length verbatim prefix (`\\?\`), and network paths SHALL NOT carry the `\\?\UNC\` form (they use `\\server\share\...`). Paths attached to opened content (document tabs, image tabs), the workspace root, file-tree entries, and recent-files entries SHALL all be in normal form, so that any surface reading them — copy actions, status feedback, reveal-in-file-manager, session persistence — presents the normal form. A path that genuinely requires the extended-length syntax (e.g. longer than the classic Windows path limit) MAY retain the verbatim form, since correct file access takes precedence over cosmetic presentation.

#### Scenario: Copy File Path omits the verbatim prefix

- **WHEN** the user copies the path of a file that was opened from the file tree, restored from a saved session, or opened from the recent-files list on Windows
- **THEN** the clipboard receives the drive-rooted path without the `\\?\` prefix (e.g. `C:\Workspace\Vaults\articles\AGENTS.md`)
- **AND** the status feedback following the copy shows the same normal-form path

#### Scenario: Reveal in File Manager uses a normal-form path

- **WHEN** the user reveals an opened file in the system file manager on Windows
- **THEN** the path handed to the file manager and shown in the feedback message is in normal form, without the `\\?\` prefix

#### Scenario: Paths attached to opened content stay in normal form

- **WHEN** a file is opened from the file tree, restored from a saved session, or opened from the recent-files list
- **THEN** the tab's stored path and the workspace root are in normal form
- **AND** tab dedupe and workspace-containment checks continue to treat the same file as the same file (case and symlink differences on Windows resolve to one identity)

#### Scenario: Legacy verbatim session entries are healed on load

- **WHEN** a session persisted by an earlier version contains `\\?\`-prefixed open-file, active-file, workspace-root, or recent-file entries
- **THEN** loading that session converts the entries to normal form wherever the shortened path is equivalent, before any tab, workspace, or recent list is built from them
- **AND** the next session save persists the healed normal-form paths

#### Scenario: Paths requiring extended-length syntax keep working

- **WHEN** a path exceeds the classic Windows path limit or otherwise requires verbatim syntax, so that removing the prefix would not yield an equivalent path
- **THEN** the verbatim form is retained for file access rather than stripped unconditionally
- **AND** open, save, and reveal operations on such a path continue to succeed

### Requirement: Local WeChat publishing workspace action
The application SHALL expose a localized action in its Export menu for opening the active document in the local WeChat publishing workspace. The action SHALL use the platform default browser, SHALL report successful launch and launch failures through localized in-app status feedback, and SHALL remain available for untitled and empty documents. Invoking it SHALL take a snapshot only and SHALL NOT save or mutate the document, change its active tab or view mode, or disturb document selection and versioned derived-state cache identity.

#### Scenario: Export menu launches the local workspace
- **WHEN** the user activates the WeChat publishing workspace item in the Export menu
- **THEN** Markion creates a publishing snapshot and asks the operating system to open its local session URL in the default browser
- **AND** shows localized in-app feedback that the publishing workspace was opened

#### Scenario: Launch preserves editor state
- **WHEN** the workspace action is invoked for an active document
- **THEN** the active tab, view mode, selection, text, dirty state, document version, and already-derived cache identities remain unchanged

#### Scenario: Browser launch failure is visible
- **WHEN** a publishing session is created but the operating system cannot open its URL
- **THEN** Markion revokes that unused session
- **AND** shows a localized status explaining that the browser could not be opened

#### Scenario: Session setup failure is visible
- **WHEN** the local workspace assets are missing or the loopback service cannot start securely
- **THEN** Markion does not open a partial or unauthenticated workspace
- **AND** shows a localized actionable error while the editor remains usable

### Requirement: Markdown Reference overlay tutorial link

The Markdown Reference overlay SHALL present a Kenhuang Markdown tutorial link at the top of the overlay, immediately below the overlay title and above the scrollable syntax-reference body, so the link remains visible without scrolling. The destination SHALL be `https://kenhuang.com/markdown/` when the active interface language is Simplified Chinese or Traditional Chinese, and `https://kenhuang.com/en/markdown/` for every other supported interface language. The URL SHALL be visibly identifiable as an interactive link. Pointer activation SHALL open that exact HTTPS destination in the system default browser through the platform shell. Link activation SHALL NOT render embedded web content, fetch tutorial HTML into the overlay, stop the application, mutate document text, dirty state, undo history, or derived Markdown caches, or implicitly dismiss the overlay.

#### Scenario: Tutorial link sits above the syntax body

- **WHEN** the user opens Help → Markdown Reference
- **THEN** a Kenhuang Markdown tutorial link appears below the overlay title
- **AND** the link is above the scrollable syntax-reference sections
- **AND** the existing syntax examples remain present below the link

#### Scenario: Chinese interface opens the Chinese tutorial

- **WHEN** the active interface language is Simplified Chinese or Traditional Chinese and the user activates the tutorial link
- **THEN** the system default browser opens exactly `https://kenhuang.com/markdown/`
- **AND** Markion renders no embedded web content and continues running
- **AND** the Markdown Reference overlay remains open

#### Scenario: Non-Chinese interface opens the English tutorial

- **WHEN** the active interface language is English, Japanese, French, German, or Spanish and the user activates the tutorial link
- **THEN** the system default browser opens exactly `https://kenhuang.com/en/markdown/`
- **AND** Markion renders no embedded web content and continues running
- **AND** the Markdown Reference overlay remains open

#### Scenario: Tutorial link does not mutate documents

- **WHEN** the user activates the tutorial link and later dismisses Markdown Reference
- **THEN** the active tab's text, dirty flag, undo history, view mode, and derived Markdown caches are unchanged

