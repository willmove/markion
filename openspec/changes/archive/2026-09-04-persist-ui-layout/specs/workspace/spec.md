## ADDED Requirements

### Requirement: Session snapshot includes chrome layout
The session file (`session.toml` under the Markion config directory) SHALL accept an optional `[layout]` table that records chrome geometry: window origin (`x`, `y`), window size (`width`, `height`), maximized flag, sidebar width, and editor/preview split ratio. Every field SHALL be optional. A missing table or missing field SHALL leave that value at the built-in default. Unknown extra keys SHALL be ignored. Loading a session for a CLI file or folder open intent SHALL still load `[layout]` even when document and workspace-root restore is skipped for that launch. Saving layout SHALL reuse the existing atomic session write and MUST NOT require a second session file.

#### Scenario: Layout table round-trips with the session file
- **WHEN** the editor persists window bounds, sidebar width, and split ratio
- **THEN** those values are written under `[layout]` in `session.toml`
- **AND** a subsequent load returns the same numeric values

#### Scenario: Older session files without layout still load
- **WHEN** `session.toml` exists but has no `[layout]` table
- **THEN** workspace-root, open-files, and recent-files fields load as before
- **AND** chrome geometry falls back to built-in defaults

#### Scenario: CLI open still restores layout
- **WHEN** the app launches with a CLI file or folder open intent and `session.toml` contains a valid `[layout]` table
- **THEN** the recorded chrome geometry is still applied
- **AND** conflicting document or workspace-root restore remains skipped as today
