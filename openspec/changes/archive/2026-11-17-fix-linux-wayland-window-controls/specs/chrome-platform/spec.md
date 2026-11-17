## ADDED Requirements

### Requirement: Wayland windows request the decorations they can actually get
On a Wayland session the main window SHALL request client-side decorations, so the decoration mode the platform reports reflects what the compositor applies instead of an unnegotiated default that no compositor honors. On every other backend the window SHALL keep the platform's default decoration request.

#### Scenario: A compositor without the decoration protocol reports client-side decorations
- **WHEN** the main window opens on a Wayland compositor that does not implement `zxdg_decoration_manager_v1` (GNOME/Mutter, as on Ubuntu 26.04)
- **THEN** the window reports client-side decorations and the in-window window-control cluster renders

#### Scenario: A compositor that grants server-side decorations keeps its native title bar
- **WHEN** a Wayland compositor answers the client-side request by granting server-side decorations
- **THEN** the window reports server-side decorations and the menu bar renders without the control cluster

#### Scenario: Non-Wayland backends keep their default request
- **WHEN** the main window opens on macOS, Windows, or an X11 session
- **THEN** the decoration request is the platform default and the native title bar is unchanged

### Requirement: Client-side window controls when the platform requires them
The in-window menu bar SHALL render minimize, maximize/restore, and close controls at its right end, and SHALL expose its free space as a window drag region, whenever the platform reports client-side decorations. Platforms reporting server-side decorations (macOS, Windows, X11 sessions, Wayland compositors that grant server-side decorations) SHALL render the menu bar unchanged. Each control SHALL be rendered only when the platform reports it as supported, defaulting to rendered when the platform is silent. Minimize SHALL call the platform minimize API and maximize/restore SHALL toggle the platform maximize state; close SHALL follow the same unsaved-changes guard as the native window close. Window dragging SHALL begin only after the pointer moves past a small threshold from the press origin so clicks still complete, and a double-click on the drag region SHALL toggle maximize. All control tooltips SHALL be routed through the localization layer.

#### Scenario: Controls appear when the compositor requires client-side decorations
- **WHEN** the main window opens on a Wayland compositor that does not provide server-side decorations (GNOME/Mutter, as on Ubuntu 26.04)
- **THEN** minimize, maximize/restore, and close buttons are visible at the right end of the in-window menu bar
- **AND** the window can be moved by dragging the menu bar's free space and maximized/restored by double-clicking that space

#### Scenario: Server-side-decorated platforms are unchanged
- **WHEN** the platform reports server-side decorations (macOS, Windows, an X11 session, or a Wayland compositor that granted them)
- **THEN** the in-window menu bar renders exactly as before, with no window-control cluster and no drag region

#### Scenario: Unsupported controls are hidden
- **WHEN** the platform reports a window control as unsupported
- **THEN** that control is not rendered

#### Scenario: Close honors unsaved changes
- **WHEN** the user clicks the client-side close button while documents are dirty or a Git operation is running
- **THEN** the existing unsaved-changes confirmation (or Git cancellation) flow runs, and the window closes only after the user saves or discards
- **WHEN** no documents are dirty or closing was already allowed
- **THEN** the window closes immediately with its layout persisted

#### Scenario: Tooltips follow the interface language
- **WHEN** the pointer hovers a window-control button
- **THEN** its tooltip renders in the active interface language like every other chrome string
