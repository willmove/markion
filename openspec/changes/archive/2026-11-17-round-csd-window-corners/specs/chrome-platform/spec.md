## ADDED Requirements

### Requirement: Rounded window silhouette on client-side-decorated sessions
Where the platform reports client-side decorations, the main window SHALL paint a rounded-rectangle silhouette with the desktop visible through the corner pixels, and SHALL outline that silhouette with a hairline border. The corner radius SHALL be zero when the window is maximized or fullscreen, and each edge the compositor reports as tiled SHALL have its two corners squared. Platforms reporting server-side decorations SHALL render the window chrome unchanged. No painted background or hover surface SHALL extend outside the rounded silhouette.

#### Scenario: Floating window on GNOME Wayland is rounded
- **WHEN** the main window is floating on a Wayland compositor that requires client-side decorations
- **THEN** all four corners are rounded at the desktop's window radius and the desktop shows through the corner pixels
- **AND** a hairline outline follows the rounded silhouette

#### Scenario: Maximized and fullscreen windows are square
- **WHEN** the window is maximized or fullscreen
- **THEN** all four corners are square

#### Scenario: Tiled edges are square
- **WHEN** the compositor reports the window as tiled on one or more edges
- **THEN** the corners on each tiled edge are square while the remaining corners stay rounded

#### Scenario: Server-side-decorated platforms are unchanged
- **WHEN** the platform reports server-side decorations (macOS, Windows, an X11 session, or a Wayland compositor that granted them)
- **THEN** the window chrome renders exactly as before, with square client-painted backgrounds and no outline overlay

#### Scenario: Corner-adjacent controls respect the silhouette
- **WHEN** the pointer hovers the client-side close button or the first menu button on a rounded window
- **THEN** the hover background stays inside the rounded silhouette