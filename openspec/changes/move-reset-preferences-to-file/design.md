## Design

The change is a menu-placement update only. Markion already has a `ResetPreferences` action, localized `Msg::ItemResetPreferences` label, prompt flow, and installed action listener. Both in-window menu rendering (`active_menu_dropdown`) and native menu construction (`install_menus`) should reuse that existing action and label.

## Implementation Notes

- In the File menu, render `ResetPreferences` immediately after `ShowPreferences`.
- In the Help menu, remove the `ResetPreferences` entry so the command appears in only one menu.
- Mirror the same ordering in the native OS File and Help menus.
- Do not rename action structs, message keys, or status strings.

## Risks

The primary risk is changing only one menu surface and leaving native and in-window menus inconsistent. Verification should inspect both builder paths and run the full test suite.
