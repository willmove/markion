## ADDED Requirements

### Requirement: Help menu external links

The Help menu SHALL offer a "Report an Issue" item and an "Online Documentation" item, positioned between the update check and the About action, in both menu surfaces the application renders: the in-window menu bar dropdown and the native OS menu bar. Invoking "Report an Issue" SHALL open `https://github.com/willmove/markion/issues/new` and invoking "Online Documentation" SHALL open `https://github.com/willmove/markion#readme` — each in the system default browser via the platform shell, never inside an embedded web view, and the application SHALL keep running normally afterwards. Both items SHALL be pointer-driven with no keyboard shortcuts and no shortcut-reference entries. Invoking either item from the in-window dropdown SHALL dismiss the open menu. Both item labels SHALL be routed through the localization layer and render in the active language like every other menu item.

#### Scenario: Report an Issue opens the issue tracker in the browser

- **WHEN** the user chooses "Report an Issue" from the Help menu (in-window dropdown or native menu bar)
- **THEN** the system default browser opens the project's new-issue page (`https://github.com/willmove/markion/issues/new`)
- **AND** the application renders no embedded web content and continues running normally

#### Scenario: Online Documentation opens the documentation home in the browser

- **WHEN** the user chooses "Online Documentation" from the Help menu (in-window dropdown or native menu bar)
- **THEN** the system default browser opens the project's documentation home (`https://github.com/willmove/markion#readme`)
- **AND** the application renders no embedded web content and continues running normally

#### Scenario: Invoking an external link closes the in-window dropdown

- **WHEN** the user clicks either external-link item in the open in-window Help dropdown
- **THEN** the dropdown closes and no menu item stays highlighted

#### Scenario: External link labels follow the active language

- **WHEN** the interface language is switched
- **THEN** both external-link item labels re-render in the new language in the in-window menu bar and in the reinstalled native menu bar
