## Context

GNOME Wayland gives clients a transparent, undecorated surface: the Wayland backend creates its Blade surface with `transparent: true`, and `Window::new`'s `set_background_appearance(Opaque)` call runs `update_window`, which with `Decorations::Client` keeps the surface alpha-capable (`update_transparency(true)`, no opaque region). Whatever the client leaves unpainted shows the desktop. GTK4/libadwaita apps exploit this to paint a 12 px rounded rectangle plus outline and shadow; Mutter clips nothing.

Two GPUI 0.2.2 constraints shape the design:

1. `ContentMask` carries only rectangular bounds, so `overflow_hidden` cannot clip children to a rounded rectangle. Rounding must come from the painted backgrounds themselves.
2. Only three surfaces touch the window's outer corners: the root background, the menu bar row background (top), and — because the status bar paints no background of its own — nothing at the bottom. Everything else (workspace, dropdowns, dialogs, toasts) is interior.

## Goals / Non-Goals

**Goals:**

- A rounded window silhouette on CSD sessions that matches GNOME's radius and its maximized/fullscreen/tiling behavior, with zero change elsewhere.
- No hover/background pixel ever paints outside the rounded silhouette (the corner zone is a 12×12 px square minus the quarter circle at each corner).
- Keep the window edge readable on light wallpapers without a shadow margin.

**Non-Goals:**

- Client-drawn drop shadow: it needs transparent margin around the entire layout (root insets, drag region, hit areas), a much larger change than the requested corner style.
- Edge-resize handles, per-compositor radius tuning, animated radius transitions.
- Any change on server-side-decorated platforms or test windows (the gate stays false there).

## Decisions

### 1. One radius decision, shared by every painted surface

`window_controls::csd_corner_radii(decorations, maximized, fullscreen)` returns `Corners<Pixels>`: all zero unless `Decorations::Client`; otherwise 12 px per corner, zeroed for maximized/fullscreen and for each edge the compositor reports tiled (`Tiling { top, left, right, bottom }` zeroes the two corners on that edge). The root, the menu bar row, and the close button all read from this single value so the silhouette can never disagree with itself. A pure function, unit-tested for the normal, maximized, fullscreen, and tiled cases.

### 2. Round the backgrounds, not a clip mask

The root div keeps `.bg(palette.app_bg)` but gains the four radii, so its background quad is the rounded silhouette and the corner pixels stay alpha 0. The menu bar row gains the two top radii (its `panel_bg` would otherwise square the top corners). The status bar needs nothing: it paints no background, and its 16 px horizontal padding keeps its content outside the bottom corner zone.

### 3. Padding discipline replaces rounded clipping

Because children cannot be clipped to a curve, interactive backgrounds must stay out of the corner zone:

- Menu bar left padding becomes 12 px (the radius) on CSD sessions, so the first menu button's always-painted background starts exactly where the top-left curve ends.
- The control cluster moves flush to the right edge (right padding 0 on CSD), like a GNOME headerbar, and the close button carries `rounded_tr` equal to the top-right radius. Its 36×28 hover rectangle then coincides with the window corner instead of spilling a red nib outside it. Minimize/maximize are interior and need nothing.

### 4. Hairline outline as a top overlay

A final root child — `div().absolute().inset_0()` with the same radii, `border_1`, `palette.border`, no id and no listeners — paints a 1 px rounded outline above the chrome. Without handlers it registers no hitbox, so it is input-transparent. It stands in for libadwaita's window outline; the shadow is deliberately omitted (non-goal).

### 5. Gate reuse, not a new platform branch

The radius helper returns zero corners for `Decorations::Server`, so every existing platform and test fixture renders byte-identical chrome. No `#[cfg]` branches are added; the only platform-specific input is the `Tiling` payload GPUI already reports.

## Risks / Trade-offs

- 12 px is libadwaita's window radius at 1x; GPUI's logical pixels scale it with the display scale factor, matching GTK's behavior. Compositors with a different house radius (KDE CSD) still get a consistent, if not identical, curve.
- The flush-right cluster changes the menu bar's right padding on CSD sessions only; the drag region absorbs the difference, so menu layout on other platforms is untouched.
- Without a shadow the window relies on the hairline outline for edge definition; on wallpapers close to `app_bg` the edge is subtler than a native GTK window's shadowed edge. Acceptable for this change's scope.