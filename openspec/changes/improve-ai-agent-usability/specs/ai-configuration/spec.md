## ADDED Requirements

### Requirement: Settings drafts SHALL be protected from silent loss
AI settings SHALL track whether the visible draft differs from the persisted profile. Switching provider presets, switching profiles, adding or removing a profile, or leaving the AI settings tab with unsaved changes SHALL present an explicit discard/save choice rather than silently replacing or resetting the draft. Writing guidance SHALL be saveable independently of profile commits and SHALL NOT be reset by preset or profile switching. Model discovery results SHALL survive preset-unswitched edits and SHALL be cleared only when the endpoint, protocol, or profile identity they were fetched for changes.

#### Scenario: Provider switch with unsaved edits
- **WHEN** the user has typed a name, key, or endpoint and clicks a different provider preset
- **THEN** the user is asked to keep editing, discard changes, or save first
- **AND** no typed value is replaced before the user chooses

#### Scenario: Writing guidance survives profile browsing
- **WHEN** the user edits writing guidance, switches profiles without saving the profile, and saves the guidance
- **THEN** the guidance persists independently of which profile is selected
- **AND** switching back does not restore an older guidance value

### Requirement: Settings feedback SHALL be specific and actionable
Validation failures SHALL name the offending field and the constraint violated; the generic invalid-settings message SHALL be a fallback, not the primary path. Save, connection test, model discovery, key forgetting, and stop outcomes SHALL each use dedicated status strings, never reused button labels. Settings feedback and conversation-panel feedback SHALL be separate surfaces so one cannot overwrite the other. Model discovery SHALL show a loading state while running, an explicit empty-or-unsupported result message, and discovered models SHALL offer a visible selected state that fills the model draft without saving it. The Advanced disclosure SHALL visibly indicate its expanded or collapsed state.

#### Scenario: Invalid limits field
- **WHEN** the user enters a non-numeric or out-of-range value in one limits field and saves
- **THEN** the feedback names that limits field and its accepted range
- **AND** the other field values remain intact in the draft

#### Scenario: Discovery returns no models
- **WHEN** Find models completes against a server that does not support listing
- **THEN** the settings show an explicit "no models discovered, enter one manually" message
- **AND** the previously entered model draft is preserved

## MODIFIED Requirements

### Requirement: Setup SHALL support cloud keys and local models without unnecessary fields
AI settings SHALL offer OpenAI, Anthropic, DeepSeek, Local/Ollama, and custom OpenAI-compatible presets. Basic setup SHALL expose enablement, provider, masked API key where required, model selection/manual entry, and an explicit Test Connection action. Presets SHALL fill a suitable editable endpoint. The endpoint field SHALL be visible in basic setup whenever the selected preset ships without a usable default endpoint (custom) or commonly requires editing (local), so first-time setup of those presets never requires opening Advanced. Protocol SHALL be chosen from the supported protocol choices constrained by the preset, never typed as free text. Timeout/output/context/tool limits SHALL be edited as individually labeled numeric fields, each validated independently. Advanced settings SHALL retain endpoint/protocol overrides for cloud presets, the limits fields, and user writing instructions. Local endpoints SHALL support an absent key. Model-list discovery SHALL occur only after explicit user action and SHALL fall back to manual model entry when unsupported.

#### Scenario: Cloud setup with a preset
- **WHEN** the user selects a cloud preset, enters a key and a model, and saves valid settings
- **THEN** the profile can be used without manually entering an endpoint or editing a configuration file
- **AND** opening settings or changing a field does not send document content or test requests

#### Scenario: Local setup without a key
- **WHEN** the user selects Local/Ollama with a running local endpoint and manually enters an installed model
- **THEN** the profile can be saved and tested without requiring a paid account or an API key

#### Scenario: Custom endpoint setup without Advanced
- **WHEN** a first-time user selects the custom OpenAI-compatible preset
- **THEN** the endpoint field is visible in basic setup and the save validation names it if left empty
- **AND** the protocol is selectable from the supported choices rather than typed

### Requirement: Connection tests SHALL report usable capabilities and actionable failures
An explicit connection test SHALL send only a bounded synthetic prompt and, where supported, a harmless tool-capability probe, never user document or workspace content. It SHALL report reachable/authenticated/model-usable states separately from verified tool support. Authentication, missing model, unsupported protocol/tool features, rate limits, timeout, and unreachable local server SHALL have distinct actionable feedback. Unknown or unsupported tool capability SHALL disable Agent mode while preserving available writing/chat. Provider-reported usage SHALL be displayed when present and identified as unavailable otherwise; tests SHALL not claim to be free. Test Connection and Find Models SHALL be usable against the current draft without requiring the AI master switch, because they contact only the configured endpoint with synthetic content; a successful test while AI is disabled SHALL offer an explicit Enable AI action rather than enabling silently.

#### Scenario: Text-only model passes
- **WHEN** a model responds to the text probe but rejects tool definitions
- **THEN** writing and conversation remain usable while Agent mode is unavailable with an explanation

#### Scenario: Connection test fails
- **WHEN** the server rejects the key or model, returns a rate limit, or fails to respond within the configured timeout
- **THEN** the settings show the corresponding repair action and do not claim a successful connection
- **AND** document text, selection, dirty state, and undo history are unchanged

#### Scenario: Verify before enabling
- **WHEN** AI is disabled and the user tests a complete draft configuration
- **THEN** the test runs against the draft and reports its real result
- **AND** on success the user can enable AI with one explicit action

## REMOVED Requirements

(none)
