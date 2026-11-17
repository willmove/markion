## Why

Opening a folder currently renders every Markdown-bearing descendant expanded, which makes the left file tree noisy and difficult to scan in deep workspaces. The initial view should expose the selected folder's immediate contents while leaving deeper levels available on demand.

## What Changes

- Initialize a newly established workspace tree with every top-level directory row collapsed, so the first render shows only files and folders directly inside the opened root.
- Keep directory rows individually expandable and collapsible after the initial view is created.
- Preserve the user's expansion state across refreshes of the same workspace instead of resetting it to the one-level default.
- Keep filename filtering able to surface matching nested paths without permanently changing the saved collapse state.

Non-goals: persisting expansion state across application launches, changing which files are scanned, adding expand-all/collapse-all commands, or changing the bounded-row rendering limit.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workspace`: Define the file tree's initial one-level expansion behavior, subsequent directory toggling, refresh preservation, and filtering interaction.

## Impact

- `src/app/application.rs`: initialize collapse state when a scan for a newly established workspace completes while retaining state for same-root refreshes.
- `src/app/root_view.rs`: keep nested filtering and collapse visibility behavior consistent with the new default.
- `src/app/tests.rs`: cover one-level initialization, manual expansion, same-root refresh behavior, and filtering through collapsed branches.
- The existing Markdown-only scan and bounded number of rendered file-tree rows per frame remain unchanged; no new dependencies or persisted settings are introduced.
