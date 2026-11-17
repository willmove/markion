## ADDED Requirements

### Requirement: AI workflows SHALL be completely localized
All user-visible AI settings, setup, panel/composer, writing presets, context scopes, model capability states, tool activity, change review, cancellation/error, usage, history, and action-recovery strings SHALL use the localization catalog for English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish. Language changes SHALL relabel visible AI surfaces without modifying documents, conversation content, provider/model identifiers, authored instructions, or paths. Missing translations SHALL be caught by exhaustive compilation or an explicit all-language catalog completeness test. Ordinary labels SHALL describe writing and file actions in user language; raw protocol details SHALL remain in expandable diagnostics.

#### Scenario: Chinese AI workflow
- **WHEN** the interface is Simplified Chinese and the user opens AI settings, rewrites a selection, or reviews a file plan
- **THEN** labels, validation, progress, apply/reject controls, and outcome feedback are Simplified Chinese

#### Scenario: Language changes during a conversation
- **WHEN** the interface language changes with the AI panel visible
- **THEN** UI chrome follows the selected language while existing conversation/document content stays unchanged

#### Scenario: AI translation is missing
- **WHEN** an AI message is added without a non-empty translation in a supported language
- **THEN** compilation or the completeness test fails
