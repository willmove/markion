## Purpose

Provide optional AI service configuration with convenient setup, secure user-supplied credentials, cloud and local endpoint support, and predictable enable/disable behavior.

## ADDED Requirements

### Requirement: AI SHALL be opt-in and fully disableable
AI SHALL default to disabled when its configuration is absent. An explicit master switch SHALL enable or disable AI independently of panel visibility. Disabled AI SHALL make no AI requests, discover no models, and execute no agent tools. Disabling SHALL cancel active requests and unstarted operations, revoke session read grants, and invalidate pending proposals and asynchronous completions. An already admitted file operation SHALL settle at a recoverable local boundary. Ordinary editing, saving, search, export, and existing preferences SHALL remain available.

#### Scenario: Existing installation starts without AI configuration
- **WHEN** an installation without AI settings launches
- **THEN** AI is disabled, ordinary AI toolbar/context actions are hidden, and no AI service is contacted
- **AND** AI settings remain reachable without enabling the feature

#### Scenario: Disable while a response is streaming
- **WHEN** the user disables AI during a response or tool loop
- **THEN** the active work is canceled and subsequent deltas, tool calls, and generated edits are ignored
- **AND** already authored document content is preserved

#### Scenario: Disable during an admitted file operation
- **WHEN** the user disables AI after one reviewed file operation has begun
- **THEN** that operation completes or fails at its journaled boundary and no subsequent operation starts
- **AND** the UI reports the actual completed, failed, and canceled operations

### Requirement: Setup SHALL support cloud keys and local models without unnecessary fields
AI settings SHALL offer OpenAI, Anthropic, DeepSeek, Local/Ollama, and custom OpenAI-compatible presets. Basic setup SHALL expose enablement, provider, masked API key where required, model selection/manual entry, and an explicit Test Connection action. Presets SHALL fill a suitable editable endpoint. Advanced settings SHALL expose profiles, endpoint/protocol overrides, timeout/output/context/tool limits, and user writing instructions. Local endpoints SHALL support an absent key. Model-list discovery SHALL occur only after explicit user action and SHALL fall back to manual model entry when unsupported.

#### Scenario: Cloud setup with a preset
- **WHEN** the user selects a cloud preset, enters a key and a model, and saves valid settings
- **THEN** the profile can be used without manually entering an endpoint or editing a configuration file
- **AND** opening settings or changing a field does not send document content or test requests

#### Scenario: Local setup without a key
- **WHEN** the user selects Local/Ollama with a running local endpoint and manually enters an installed model
- **THEN** the profile can be saved and tested without requiring a paid account or an API key

### Requirement: Profiles and AI preferences SHALL persist compatibly
Non-secret AI preferences SHALL persist in optional AI configuration tables with stable profile identifiers and one selected profile. Missing fields SHALL use documented defaults; malformed AI fields SHALL not discard unrelated preferences or prevent startup. Invalid endpoints, protocols, models, or limits SHALL leave the affected profile unavailable with repair feedback. Profile changes SHALL cancel that profile's active request and invalidate pending proposals before later requests use the new profile. Resetting preferences SHALL disable AI and reset non-secret settings without deleting credential-store entries, conversation files, action journals, or user documents; explicit management actions SHALL remove retained data.

#### Scenario: Malformed AI field is isolated
- **WHEN** a hand-edited AI profile contains an invalid endpoint or incompatible value
- **THEN** the editor loads unrelated valid preferences and identifies the unavailable profile
- **AND** it does not silently send requests using a substituted endpoint

#### Scenario: Profile change during generation
- **WHEN** the user changes the selected profile or its request-affecting settings during generation
- **THEN** that generation is canceled, its unapplied proposals become invalid, and the next turn uses the newly selected settings

#### Scenario: Reset is non-destructive
- **WHEN** general preferences are reset after AI has been configured
- **THEN** AI returns to disabled defaults while retained AI data remains explicitly manageable
- **AND** no user file or external service configuration is changed

### Requirement: Credentials and endpoint handling SHALL protect secrets
Persistent API keys SHALL use the operating system credential store and SHALL never be serialized into preferences, sessions, conversations, action journals, logs, or exported diagnostics. If secure storage is unavailable, the UI SHALL offer clearly labeled session-only key use without a plaintext fallback. The UI SHALL support replacing and explicitly forgetting a key. Requests SHALL use HTTPS except for literal loopback addresses or localhost; non-loopback insecure endpoints, embedded URL credentials, and credential forwarding across redirects SHALL be rejected. A changed endpoint SHALL require explicit key reattachment before a saved credential can be sent there.

#### Scenario: Credential service is unavailable
- **WHEN** the operating system cannot securely persist a key
- **THEN** the user can continue with an explicitly labeled session-only key
- **AND** restarting requires re-entering it and no plaintext key is written

#### Scenario: Endpoint changes or redirects
- **WHEN** a configured endpoint changes authority or returns a redirect
- **THEN** the old credential is not forwarded to a new authority
- **AND** the user receives actionable endpoint/key feedback

### Requirement: Connection tests SHALL report usable capabilities and actionable failures
An explicit connection test SHALL send only a bounded synthetic prompt and, where supported, a harmless tool-capability probe, never user document or workspace content. It SHALL report reachable/authenticated/model-usable states separately from verified tool support. Authentication, missing model, unsupported protocol/tool features, rate limits, timeout, and unreachable local server SHALL have distinct actionable feedback. Unknown or unsupported tool capability SHALL disable Agent mode while preserving available writing/chat. Provider-reported usage SHALL be displayed when present and identified as unavailable otherwise; tests SHALL not claim to be free.

#### Scenario: Text-only model passes
- **WHEN** a model responds to the text probe but rejects tool definitions
- **THEN** writing and conversation remain usable while Agent mode is unavailable with an explanation

#### Scenario: Connection test fails
- **WHEN** the server rejects the key or model, returns a rate limit, or fails to respond within the configured timeout
- **THEN** the settings show the corresponding repair action and do not claim a successful connection
- **AND** document text, selection, dirty state, and undo history are unchanged
