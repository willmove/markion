## Why

Today Markion always starts on the in-memory welcome document with an empty file tree, even when the user previously had a folder and Markdown files open. That forces a repeated Open / Open Folder dance every launch. Persisting the last session and exposing Open Recent in the File menu restores continuity without changing how documents are edited.

## What Changes

- Persist a session snapshot on meaningful workspace changes and on clean shutdown: last file-tree workspace root, open Markdown document paths (saved tabs only), and the active-tab path.
- On launch with no CLI open intent, restore that session: re-establish the workspace root and scan the file tree, reopen still-existing document tabs, and focus the last active document when possible.
- Maintain a bounded recent-files list (paths the user opened or saved) and expose it under File → Open Recent, including a Clear Recent Files action.
- Keep crash recovery and CLI startup paths as higher-priority overrides: a CLI file/folder intent skips session restore for the conflicting pieces; recovery prompts remain unchanged.
- Localize all new Open Recent and session-related status / empty-state strings.

Non-goals: restoring unsaved buffer text or dirty state (recovery already covers that); persisting cursor/selection/scroll/view-mode; multi-window sessions; syncing session state across machines; changing auto-save intervals or preferences reset semantics for unrelated settings.

Invariants preserved: per-document derived-state caches, syntax-highlight memoization, cached editor text handles, bounded file-tree rendering, and GUI-free workspace members.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Add session persistence and launch-time restore for the file-tree workspace root and open saved document tabs; clarify interaction with the current empty-state-on-startup rule and CLI open intents.
- `chrome-platform`: Add File → Open Recent (and Clear Recent Files) to the in-window File menu.
- `ui-i18n`: Route Open Recent menu labels, empty recent-list placeholder, clear action, and related status feedback through the i18n layer.

## Impact

- Persistence: new session/recent storage under the Markion config directory (separate from `config.toml` preferences), loaded at bootstrap and written on session/recent mutations.
- App bootstrap / application state: restore order relative to preferences load, CLI `StartupOpenIntent`, and recovery prompt.
- Document and workspace flows: record recent paths and update the session snapshot when opening/saving/closing tabs or changing the workspace root.
- Menus / actions / i18n: File menu submenu wiring and localized strings.
- Tests: session load/save, missing-path skipping, CLI override, and recent-list ordering/capping.
