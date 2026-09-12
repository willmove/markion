## ADDED Requirements

### Requirement: Typing-loop chrome is localized
Every user-visible string introduced for Markdown auto-pair and emoji shortcode completion SHALL go through the i18n layer (`t` / `tf` and exhaustive per-language `Msg` arms): the Preferences → General auto-pair label and any helper copy, the emoji palette title, empty-match copy, and row labels that are not the raw `:shortcode:` token. Hard-coded English literals SHALL NOT remain on those surfaces. Changing the interface language SHALL relabel an open emoji palette and the auto-pair control without mutating the document.

#### Scenario: Auto-pair label follows the interface language
- **WHEN** the active interface language is Simplified Chinese and Preferences → General is open
- **THEN** the Markdown auto-pair control label is the Simplified Chinese string from the i18n module

#### Scenario: Emoji palette chrome follows the interface language
- **WHEN** the emoji shortcode palette is open and the interface language is Japanese
- **THEN** the palette title and empty-match copy are Japanese via the i18n module
- **AND** shortcode names remain the ASCII `:name:` tokens

#### Scenario: Language switch does not edit the document
- **WHEN** the user changes the interface language while the emoji palette is open
- **THEN** palette chrome relabels
- **AND** document version, dirty state, and undo history are unchanged
