## ADDED Requirements

### Requirement: Visual Edit preserves outer formatting across nested same-kind delimiters
Visual Edit SHALL track nested Markdown `Strong`, `Emphasis`, and `Strikethrough` events using nesting depth counters rather than single boolean flags, matching Split Preview and Read mode. Closing an inner same-kind construct (such as `**outer **inner** tail**`, `*outer *inner* tail*`, or `~~outer ~~inner~~ tail~~`) SHALL NOT clear the outer formatting style for the trailing text inside the outer construct.

#### Scenario: Nested strong preserves trailing bold text
- **WHEN** a prose block contains `**outer **inner** tail**` and is rendered in Visual Edit
- **THEN** the trailing `" tail"` run retains bold styling, matching Split Preview and Read mode

#### Scenario: Nested emphasis and strikethrough preserve trailing styles
- **WHEN** a prose block contains `*outer *inner* tail*` or `~~outer ~~inner~~ tail~~` and is rendered in Visual Edit
- **THEN** the trailing `" tail"` run retains italic or strikethrough styling respectively, matching Split Preview and Read mode
