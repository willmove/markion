## MODIFIED Requirements

### Requirement: Markdown Reference chrome and body are localized
The system SHALL route Markdown Reference menu labels, overlay title, close control, and status feedback through the i18n `Msg` / `t` layer for every supported interface language. The overlay body SHALL be produced by `markdown_reference(language)` as a fixed set of sections (title, syntax example, caption) covering the construct groups required by the chrome-platform Markdown Reference requirement. Every supported language SHALL return the same section set with non-empty text. The in-app shortcut reference SHALL list New Tab, Open Folder, Edit / source mode, Visual Edit mode, Split Preview, Read mode, inline code, and Markdown Reference using their effective bindings in the active language.

#### Scenario: Markdown Reference menu label follows the interface language
- **WHEN** the interface language changes
- **THEN** the Markdown Reference label re-renders in the new language in the in-window Help menu and in the reinstalled native Help menu

#### Scenario: Markdown Reference body follows the interface language
- **WHEN** the Markdown Reference overlay is opened after the interface language changes
- **THEN** every section title, example, and caption is non-empty in the active language

#### Scenario: Shortcut catalog reflects refined defaults
- **WHEN** the in-app shortcut reference is shown with no overrides
- **THEN** Open Folder, Visual Edit, Read, and Inline Code appear with their effective platform bindings (`Ctrl+Shift+O` / `Cmd+Shift+O`, `Ctrl+E` / `Cmd+E`, `Ctrl+R` / `Cmd+R`, `Ctrl+Shift+\`` / `Cmd+Shift+\``)
