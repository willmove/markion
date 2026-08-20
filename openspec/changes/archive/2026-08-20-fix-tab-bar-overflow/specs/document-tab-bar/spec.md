## Purpose

Defines how the document tab strip presents open tabs when they exceed the available horizontal space, so every tab stays reachable and readable: overflow scrolling, bounded tab widths with label truncation, active-tab auto-reveal, pinned strip actions, and hover tooltips that restore truncated information.

## ADDED Requirements

### Requirement: Every open tab remains reachable

When the combined width of open tabs exceeds the tab strip's available width, the strip SHALL scroll horizontally instead of clipping tabs out of view. Scrolling the mouse wheel or trackpad over the strip SHALL move it horizontally, and no scroll position SHALL leave a tab unreachable. When tabs close or the window resizes so that the content no longer overflows, the strip SHALL not remain scrolled past its content.

#### Scenario: Overflowing tabs are scrollable
- **WHEN** more tabs are open than fit in the tab strip's width
- **THEN** tabs beyond the visible area can be brought into view and clicked by scrolling the strip horizontally
- **AND** every open tab, including the leftmost and rightmost, can be made visible, selected, and closed

#### Scenario: Wheel over the strip scrolls horizontally
- **WHEN** the pointer is over the tab strip and the user scrolls a standard vertical mouse wheel
- **THEN** the strip scrolls horizontally in the wheel's direction without scrolling any content behind the strip

#### Scenario: Scroll position clamps when overflow disappears
- **WHEN** tabs are closed or the window widens so the strip's content fits without overflow
- **THEN** the strip is not left scrolled past its content and all remaining tabs are reachable without scrolling

### Requirement: Tab width is bounded and long titles truncate

Each tab SHALL impose a maximum width beyond which its title is truncated with an ellipsis while remaining hover-inspectable. Tabs SHALL shrink toward a minimum width as more tabs are opened; at the minimum width the strip scrolls rather than shrinking tabs further. The close control of every visible tab SHALL remain fully visible and operable at every width, and a truncated title SHALL NOT hide or crowd it out.

#### Scenario: Long filename truncates with ellipsis
- **WHEN** a tab's title is longer than the tab's maximum width allows
- **THEN** the title is displayed truncated with a trailing ellipsis and the tab does not exceed its maximum width

#### Scenario: Tabs shrink as tab count grows
- **WHEN** additional tabs are opened while the strip still has room
- **THEN** existing tabs narrow (truncating their titles further as needed) instead of immediately overflowing

#### Scenario: Close control stays operable on narrow tabs
- **WHEN** a tab is displayed at or near its minimum width
- **THEN** the tab's close control remains fully visible and clickable alongside the truncated title

### Requirement: Dirty indicator survives title truncation

A document tab's unsaved-changes indicator SHALL be rendered as an element separate from the title text, so truncating the title can never remove or obscure the indicator. A tab whose title is truncated SHALL show its unsaved state exactly as visibly as an untruncated tab.

#### Scenario: Truncated dirty tab still shows it is unsaved
- **WHEN** a tab with unsaved changes has its title truncated
- **THEN** the unsaved indicator is fully visible on the tab and is not part of the truncated text

### Requirement: The active tab is scrolled into view

Whenever the active tab changes through selection, opening, closing, or focusing an already-open file, the strip SHALL scroll the minimal amount needed to make the active tab fully visible, without changing the strip's position when the active tab is already fully visible.

#### Scenario: Switching to an off-screen tab reveals it
- **WHEN** the user activates a tab that is scrolled out of view (by shortcut, menu, or clicking a tab revealed by scrolling)
- **THEN** the strip scrolls just enough to fully reveal the newly active tab

#### Scenario: Activating a visible tab does not disturb the strip
- **WHEN** the user activates a tab that is already fully visible
- **THEN** the strip's scroll position does not change

### Requirement: Strip actions stay pinned outside the scroll area

Controls appended to the tab strip (the new-tab "+" button) SHALL remain visible and clickable at every scroll position and at every tab count, and SHALL NOT scroll with the tab strip's content.

#### Scenario: New-tab button remains reachable under overflow
- **WHEN** the strip is overflowing and scrolled to any position
- **THEN** the "+" new-tab button is visible and clicking it opens a new tab

### Requirement: Hovering a tab shows its full title and path

Hovering a tab SHALL present a tooltip containing the tab's full, untruncated title and, when the tab is backed by a file on disk, the file's full path. The tooltip SHALL appear regardless of whether the title is currently truncated, restoring any information lost to truncation.

#### Scenario: Tooltip on a truncated tab
- **WHEN** the pointer hovers over a tab whose title is truncated and the tab is file-backed
- **THEN** a tooltip shows the complete file name and its full path

#### Scenario: Tooltip on an untitled tab
- **WHEN** the pointer hovers over a tab that has no file on disk
- **THEN** the tooltip shows the tab's title without a path
