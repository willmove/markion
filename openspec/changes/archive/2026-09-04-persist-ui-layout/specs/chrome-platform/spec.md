## ADDED Requirements

### Requirement: Chrome layout persists across launches
The editor SHALL remember the last desktop chrome geometry and restore it on the next launch: the windowed size and screen position, whether the window was maximized, the sidebar column width, and the Split Preview editor/preview split ratio. Restoration of window bounds SHALL occur when the window is created, so the first frame uses the saved rectangle rather than the built-in centered default (1180×760). Fullscreen SHALL NOT be persisted; while the window is fullscreen the last windowed or maximized snapshot SHALL be kept. Preferences reset SHALL NOT clear this chrome geometry. Missing, non-numeric, or out-of-range values SHALL fall back to the built-in defaults (centered 1180×760 window, sidebar width 230, split ratio 0.5) without preventing startup. A saved window rectangle that does not intersect any current display SHALL be replaced by a windowed window of the clamped saved size (or the default size) centered on the primary display. Sidebar width SHALL clamp to the existing 150–480 logical-pixel range and the split ratio SHALL clamp to the existing 0.15–0.85 range. Writes during live resize or divider drag SHALL be deferred until the gesture settles; a clean window close SHALL persist the last geometry. Applying or persisting chrome geometry MUST NOT mutate document text, increment a document version, or recompute per-version derived Markdown caches.

#### Scenario: Window size and position restore on launch
- **WHEN** the previous run recorded a windowed origin and size that still intersects a connected display
- **THEN** the next launch opens the window at that origin and size on the first frame

#### Scenario: Maximized state restores
- **WHEN** the previous run left the window maximized and recorded restore bounds
- **THEN** the next launch opens the window maximized
- **AND** those restore bounds remain available if the user later restores the window

#### Scenario: Sidebar width and split ratio restore
- **WHEN** the previous run recorded a sidebar width and a Split Preview split ratio
- **THEN** the next launch applies those values to the sidebar column and the editor/preview divider

#### Scenario: Off-screen bounds are recentered
- **WHEN** the recorded window rectangle does not intersect any current display
- **THEN** the editor opens a windowed window centered on the primary display
- **AND** startup still succeeds

#### Scenario: Missing or invalid layout uses defaults
- **WHEN** no chrome geometry has been recorded, or a recorded field is missing or invalid
- **THEN** the editor uses the built-in default window (centered 1180×760), sidebar width 230, and split ratio 0.5 for each missing or invalid field
- **AND** the editor starts normally

#### Scenario: Preferences reset leaves chrome geometry in place
- **WHEN** the user confirms Reset Preferences
- **THEN** theme, language, and other `config.toml` preferences reset as today
- **AND** the persisted window bounds, sidebar width, and split ratio are left unchanged

#### Scenario: Layout persistence does not touch document caches
- **WHEN** the editor writes or restores chrome geometry
- **THEN** no open document’s text, version, or per-version derived Markdown caches change
