## Why

On Ubuntu 26.04.1 (GNOME Wayland) every native application window is a rounded rectangle: GTK4/libadwaita clients paint their own corner radius because Mutter provides no server-side decorations and does not clip client surfaces. Markion now paints client-side decorations on that session (see `fix-linux-wayland-window-controls`), but its root view fills the whole surface with a square-cornered background, so the window reads as a sharp rectangle next to every other app on the desktop.

## What Changes

- Round the main window's corners on client-side-decorated sessions only, using the same `Decorations::Client` gate as the window-control cluster. Radius matches the GNOME/Adwaita window radius (12 logical px).
- Square the corners again when the window is maximized or fullscreen, and square only the tiled edges when the compositor reports tiling (`Decorations::Client { tiling }`), matching GNOME's behavior for half-tiled windows.
- Leave the corner pixels unpainted so the transparent Wayland surface shows the desktop through them; add a hairline rounded border overlay so the window edge stays defined on light wallpapers (libadwaita's outline; its drop shadow is a non-goal).
- Keep the menu bar and window-control cluster inside the rounded silhouette: the menu bar row carries the top corner radii, its left padding grows to the radius so menu hover backgrounds never spill into the corner zone, and the control cluster sits flush right with the close button carrying the top-right radius so its danger hover respects the corner.
- Non-goals: client-drawn drop shadows (would require a shadow margin around the whole layout), edge-resize handles, and any change on platforms that report server-side decorations (macOS, Windows, X11, Wayland compositors that grant them) or in test fixtures.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `chrome-platform`: the client-side-decorated window chrome gains a rounded silhouette with maximized/fullscreen/tiling-aware corners and a hairline outline.

## Impact

- Affected code: `src/app/window_controls.rs` (corner-radius decision + close-button radius), `src/app/root_view.rs` (root rounding, border overlay, menu bar row radii/padding), `src/app/tests.rs` (radius unit test + source-contract assertions).
- Configuration: none.
- Dependencies and APIs: no dependency or vendored-GPUI changes; uses already-public GPUI 0.2.2 APIs (`Decorations::Client { tiling }`, `is_maximized`, `is_fullscreen`, `rounded_*` style helpers). GPUI 0.2.2 content masks are rectangular, so rounding is achieved with per-row radii and padding rather than a rounded clip.
- Architecture invariants: rendering-only chrome change; derived-state caches, highlight memoization, and the cached text handle are untouched.