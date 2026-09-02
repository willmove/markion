## ADDED Requirements

### Requirement: Markdown Reference overlay tutorial link

The Markdown Reference overlay SHALL present a Kenhuang Markdown tutorial link at the top of the overlay, immediately below the overlay title and above the scrollable syntax-reference body, so the link remains visible without scrolling. The destination SHALL be `https://kenhuang.com/markdown/` when the active interface language is Simplified Chinese or Traditional Chinese, and `https://kenhuang.com/en/markdown/` for every other supported interface language. The URL SHALL be visibly identifiable as an interactive link. Pointer activation SHALL open that exact HTTPS destination in the system default browser through the platform shell. Link activation SHALL NOT render embedded web content, fetch tutorial HTML into the overlay, stop the application, mutate document text, dirty state, undo history, or derived Markdown caches, or implicitly dismiss the overlay.

#### Scenario: Tutorial link sits above the syntax body

- **WHEN** the user opens Help → Markdown Reference
- **THEN** a Kenhuang Markdown tutorial link appears below the overlay title
- **AND** the link is above the scrollable syntax-reference sections
- **AND** the existing syntax examples remain present below the link

#### Scenario: Chinese interface opens the Chinese tutorial

- **WHEN** the active interface language is Simplified Chinese or Traditional Chinese and the user activates the tutorial link
- **THEN** the system default browser opens exactly `https://kenhuang.com/markdown/`
- **AND** Markion renders no embedded web content and continues running
- **AND** the Markdown Reference overlay remains open

#### Scenario: Non-Chinese interface opens the English tutorial

- **WHEN** the active interface language is English, Japanese, French, German, or Spanish and the user activates the tutorial link
- **THEN** the system default browser opens exactly `https://kenhuang.com/en/markdown/`
- **AND** Markion renders no embedded web content and continues running
- **AND** the Markdown Reference overlay remains open

#### Scenario: Tutorial link does not mutate documents

- **WHEN** the user activates the tutorial link and later dismisses Markdown Reference
- **THEN** the active tab's text, dirty flag, undo history, view mode, and derived Markdown caches are unchanged
