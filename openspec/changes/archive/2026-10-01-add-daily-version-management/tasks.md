## 1. Core history and comparison APIs

- [x] 1.1 Add typed bounded working-version comparison and single-path history queries to `markion-git-sync`, using literal paths and disabled external diff/text conversion.
- [x] 1.2 Add real-repository tests for file history pagination/rename following, text comparisons, binary results, unusual paths, and output truncation.

## 2. Checked local-version planning

- [x] 2.1 Add transient commit-draft models and helpers for nonblank messages, selectable committable changes, exact path sets, and stale HEAD/status detection.
- [x] 2.2 Route selected-path local commits through existing writer admission, selected-buffer checked saves, exact-path staging, journal recovery, and post-commit refresh without network access.
- [x] 2.3 Add core and root regression tests for selected/unselected paths, dirty selected/unselected buffers, external staging, stale drafts, empty input, hook failure, and successful refresh.

## 3. Versions sidebar

- [x] 3.1 Add a Create Local Version composer to Changes with selectable whole-file rows, authored message editing, validation, progress, retry-preserving failure state, and success clearing.
- [x] 3.2 Add repository/file history modes with bounded pagination, commit metadata, changed-path drilldown, request-generation guards, and current-file availability states.
- [x] 3.3 Add bounded Compare with Current presentation with text, binary, and truncated fallbacks while keeping Git work off the render/input path.

## 4. Safe restore

- [x] 4.1 Load and validate historical text against repository/path/tab/disk identity, then restore it through one ordinary undoable document replacement transaction without writing disk or Git metadata.
- [x] 4.2 Add dirty-buffer confirmation plus binary, oversized, invalid UTF-8, symlink, deleted-path, and stale-request fallbacks that retain guarded Save a Copy.
- [x] 4.3 Add document-lifecycle tests proving restore dirties only the target tab, preserves undo/recovery behavior and unrelated caches, and refuses stale or cancelled replacements.

## 5. Localization documentation and verification

- [x] 5.1 Add complete localized labels/status/errors for the composer, history modes, comparison, and restore across every supported language.
- [x] 5.2 Update Git user documentation and shortcut guidance for local versions, selection semantics, file history, comparison, restore, and the explicitly unsupported history-rewriting operations.
- [x] 5.3 Run formatting, focused Git/core/root tests, `cargo test --workspace`, and `openspec validate add-daily-version-management --strict`; record any unrelated pre-existing workspace failure instead of marking it verified.

## 6. Repository menu organization

- [x] 6.1 Add a localized Repository top-level entry to the native and in-window menu bars, including click, hover-switch, dropdown geometry, and localization completeness wiring.
- [x] 6.2 Move all existing Git and synchronization actions from File into Repository without changing action types, shortcut IDs, or handlers; add structural regression tests for both menu implementations.
- [x] 6.3 Update menu documentation and verification evidence, then run formatting, focused root tests, `cargo test --workspace`, and strict OpenSpec validation.
