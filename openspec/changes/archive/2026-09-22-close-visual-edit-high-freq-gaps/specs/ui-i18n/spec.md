## ADDED Requirements

### Requirement: YAML Visual Edit header chrome is localized
Every user-visible string introduced for the Visual Edit YAML front-matter header SHALL go through the i18n layer (`t` / `tf` and exhaustive per-language `Msg` arms), including the collapsed YAML label. Hard-coded English literals SHALL NOT remain on that surface. Changing the interface language SHALL relabel the header without mutating the document.

#### Scenario: Simplified Chinese YAML label
- **WHEN** the interface language is Simplified Chinese and a document with YAML front matter is open in Visual Edit
- **THEN** the YAML header label is the Simplified Chinese string from the i18n module
- **AND** the document text and version are unchanged
