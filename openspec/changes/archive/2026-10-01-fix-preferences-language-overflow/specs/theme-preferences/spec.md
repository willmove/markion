## ADDED Requirements

### Requirement: Preferences language picker SHALL contain variable-width labels
The Preferences panel SHALL render every entry from `Language::all()` as a complete interactive pill whose active check mark and native-language name remain inside that pill's border across supported display scales and variable system UI-font metrics. Language pills MUST NOT shrink below their marker-and-label content width; when the row lacks sufficient width, it SHALL wrap complete pills instead of clipping text, painting into adjacent controls, or compressing labels below their content width.

#### Scenario: Wide UI font keeps labels inside their pills
- **WHEN** the Preferences panel is rendered with a wide or monospaced system UI font
- **THEN** every native-language name and the active check mark remain fully contained by their own pill border without overlapping a neighbor

#### Scenario: Constrained width wraps complete pills
- **WHEN** the available language-row width is less than the combined intrinsic width of all language pills
- **THEN** complete pills wrap onto an additional row and every language remains visible and interactive
- **AND** no pill shrinks its marker or native-language name beyond its content width

#### Scenario: Active marker uses compact stable spacing
- **WHEN** the user selects any supported interface language
- **THEN** exactly that language pill shows a check mark in a dedicated leading slot separated from its native-language name by a compact fixed visual gap
- **AND** inactive pills reserve the same leading slot without inserting whitespace characters into their labels, so language names do not shift when selection changes

#### Scenario: Sufficient width retains a single comfortable row
- **WHEN** the General Preferences panel has sufficient logical width for all language pills, including at 125% Windows display scaling
- **THEN** all language pills appear on one row with their configured minimum width and complete marker-and-label content
