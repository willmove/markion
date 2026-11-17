## MODIFIED Requirements

### Requirement: AI workflows SHALL be completely localized
All user-visible AI settings, setup, panel and composer, writing presets, context scopes, model capability states, tool activity, change review, cancellation and error, usage, history, and action-recovery strings SHALL use the localization catalog for English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish. Per-field validation messages, dedicated save, test, discovery, and forget outcomes, draft-loss confirmations, misconfiguration guidance, and empty-result states SHALL likewise be localized in all seven languages. Context-size and output-size labels, their accepted ranges, the approximate token hint, the restore-recommended-limits action, the show and hide API key actions, localized protocol choice labels, and the notice that earlier conversation turns were omitted SHALL be localized in all seven languages. Provider and model identifiers MAY stay proper nouns, and the stored protocol codes SHALL remain `responses`, `chat`, and `anthropic`. The AI feature SHALL use one consistent product name across languages instead of divergent per-language coinages. Language changes SHALL relabel visible AI surfaces without modifying documents, conversation content, provider or model identifiers, authored instructions, or paths. Missing translations SHALL be caught by exhaustive compilation or an explicit all-language catalog completeness test. Ordinary labels SHALL describe writing and file actions in user language; raw protocol details SHALL remain in expandable diagnostics.

#### Scenario: Chinese AI workflow
- **WHEN** the interface is Simplified Chinese and the user opens AI settings, rewrites a selection, or reviews a file plan
- **THEN** labels, validation, progress, apply and reject controls, and outcome feedback are Simplified Chinese

#### Scenario: Language changes during a conversation
- **WHEN** the interface language changes with the AI panel visible
- **THEN** UI chrome follows the selected language while existing conversation and document content stays unchanged

#### Scenario: AI translation is missing
- **WHEN** an AI message is added without a non-empty translation in a supported language
- **THEN** compilation or the completeness test fails

#### Scenario: New settings labels follow the interface language
- **WHEN** the interface is German or Japanese and the user opens AI settings
- **THEN** the context-size label, show API key action, protocol choice labels, and limit-range messages are in that language
- **AND** the stored protocol code and model id are unchanged
