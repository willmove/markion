## ADDED Requirements

### Requirement: Automatic slash completion SHALL respect its preference

The existing Visual Edit slash-command palette and slash-trigger recognition requirements SHALL apply while the slash-command popup preference is enabled. While disabled, `/` and the equivalent `、` marker SHALL remain ordinary input and SHALL NOT open the palette, including after editing, caret movement, or rendering. Disabling the setting SHALL immediately dismiss an open slash palette and prevent pending palette actions from applying. Enabling it SHALL resume existing recognition for the current eligible query. Other formatting entry points and emoji completion SHALL retain their behavior.

#### Scenario: Literal typing while disabled
- **WHEN** the preference is disabled and the user types `/hea` or `、hea` in an otherwise eligible Visual Edit block
- **THEN** the authored text is retained and no slash-command popup opens

#### Scenario: Disable an existing palette
- **WHEN** the preference is disabled while a slash palette is open
- **THEN** the palette closes immediately and a pending palette command cannot mutate the document
- **AND** source, version, selection, dirty state, history, and derived caches remain unchanged

#### Scenario: Re-enable existing recognition
- **WHEN** the preference is enabled while the caret follows an eligible slash query
- **THEN** the existing filtered palette can appear with the same keyboard and pointer behavior

#### Scenario: Other command surfaces remain available
- **WHEN** slash completion is disabled
- **THEN** existing Format and contextual block commands and emoji completion remain available
