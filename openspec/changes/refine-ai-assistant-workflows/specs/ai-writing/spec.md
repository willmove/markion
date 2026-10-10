## ADDED Requirements

### Requirement: Writing actions SHALL request their own parameters and show review changes
Translation and tone actions SHALL expose their respective target controls and report missing parameters as writing-input errors rather than invalid service configuration. The selected writing scope SHALL be visible before or during generation. Review SHALL show a bounded addition/removal summary together with the complete before/after text and explicit application destination. Existing incomplete/stale/checked-undo safeguards SHALL continue to apply.

#### Scenario: Translation without a target language
- **WHEN** the user starts translation with no target language
- **THEN** the language field is exposed and a target-language message is shown without contacting the provider or directing the user to repair service configuration

#### Scenario: Review a rewrite
- **WHEN** a completed rewrite is available
- **THEN** review identifies its source document/range and shows changed content before the user explicitly applies it
