## Why

On Pop!_OS 24.04 with COSMIC on Wayland, the source editor reports an empty IME cursor rectangle: the candidate window falls back to the window's upper-left corner in typewriter mode and drifts horizontally with the preedit text when typewriter mode is disabled. Windows 11 does not expose the issue because its native IME path does not consume the same Wayland cursor-rectangle geometry.

## What Changes

- Make source-editor IME geometry expose a non-empty caret rectangle at the requested composition anchor.
- Keep the candidate anchor stable across successive preedit replacements, including while typewriter mode changes the source scroll position.
- Add regression coverage for ordinary and typewriter source-editor composition geometry on the real GPUI input-handler seam.
- Non-goal: change IME text mutation, undo grouping, candidate contents, or Windows-specific input behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `markdown-editing`: require valid, stable platform candidate-window geometry for source-editor IME composition.

## Impact

- Affects the source `EntityInputHandler::bounds_for_range` geometry path and focused GPUI tests in `src/app/`.
- Preserves per-document derived Markdown caches and cached text handles; the fix is presentation geometry only and does not add typing-path parsing or mutation work.
- No public API, persistence, localization, or dependency changes.
