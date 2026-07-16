## ADDED Requirements

### Requirement: Open in-window menus track top-level pointer movement
The in-window menu bar SHALL switch the visible dropdown to a different top-level menu when the pointer moves over that menu title while any top-level dropdown is already open. Pointer movement over top-level titles MUST NOT open a dropdown when no menu session is active. Existing click toggle and click-outside dismissal behavior SHALL remain available.

#### Scenario: Pointer switches an open menu
- **WHEN** the user clicks Format to open its in-window dropdown
- **AND** moves the pointer over the View menu title
- **THEN** the Format dropdown closes and the View dropdown opens without another click

#### Scenario: Pointer can continue across menus
- **WHEN** an in-window dropdown is open and the user moves the pointer across multiple different top-level menu titles
- **THEN** the visible dropdown follows the currently hovered title throughout the active menu session

#### Scenario: Idle menu bar does not open on hover
- **WHEN** no in-window dropdown is open and the pointer moves over a top-level menu title
- **THEN** no dropdown opens

#### Scenario: Existing dismissal behavior ends hover tracking
- **WHEN** the user dismisses an open dropdown by clicking its active title or outside the menu
- **THEN** the menu session closes
- **AND** subsequent pointer movement over top-level titles does not open a dropdown until the user clicks a title again
