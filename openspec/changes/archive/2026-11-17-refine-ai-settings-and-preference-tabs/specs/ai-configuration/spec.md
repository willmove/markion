## MODIFIED Requirements

### Requirement: Setup SHALL support cloud keys and local models without unnecessary fields
AI settings SHALL offer OpenAI, Anthropic, DeepSeek, Gemini, xAI, Mistral, Moonshot, Qwen, Zhipu, OpenRouter, Local/Ollama, and custom OpenAI-compatible presets. Basic setup SHALL expose enablement, provider, a masked API key where required, model selection or manual entry, and an explicit Test Connection action. Each cloud preset SHALL fill an editable endpoint and an editable starter model id. Local and custom presets SHALL leave the model empty. Presets SHALL fill a suitable editable endpoint. The endpoint field SHALL be visible in basic setup whenever the selected preset ships without a usable default endpoint (custom) or commonly requires editing (local), so first-time setup of those presets never requires opening Advanced. Protocol SHALL be chosen from the supported protocol choices constrained by the preset, never typed as free text. The visible protocol labels SHALL be localized, while the stored protocol values remain `responses`, `chat`, and `anthropic`. Timeout, output, context, and tool limits SHALL be edited as individually labeled numeric fields, each validated independently. The context-size and output-size fields SHALL be edited as whole kibibytes and SHALL show an approximate token estimate derived from the byte value. A restore control SHALL put the recommended factory limits into the draft without saving them. Advanced settings SHALL retain endpoint and protocol overrides for cloud presets, the limits fields, and user writing instructions. Local endpoints SHALL support an absent key. Model-list discovery SHALL occur only after explicit user action and SHALL fall back to manual model entry when unsupported. Provider choices MAY wrap inside the scrolling settings body.

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

#### Scenario: New cloud preset supplies a starter model
- **WHEN** the user selects Gemini, xAI, Mistral, Moonshot, Qwen, Zhipu, or OpenRouter on a new draft
- **THEN** the draft receives that preset's HTTPS endpoint, chat protocol, and starter model id
- **AND** the user can replace the model id before saving

#### Scenario: Reveal only the typed API key
- **WHEN** the user types an API key and chooses to show it
- **THEN** the field shows the typed characters and allows them to be copied
- **AND** choosing hide returns the field to asterisks
- **AND** a previously saved key is not loaded into the field

### Requirement: Profiles and AI preferences SHALL persist compatibly
Non-secret AI preferences SHALL persist in optional AI configuration tables with stable profile identifiers and one selected profile. Missing fields SHALL use documented defaults; malformed AI fields SHALL not discard unrelated preferences or prevent startup. A profile whose complete limit tuple still equals the previous factory defaults SHALL load with the current factory defaults. A profile with any customized limit SHALL keep that saved tuple. Invalid endpoints, protocols, models, or limits SHALL leave the affected profile unavailable with repair feedback. Profile changes SHALL cancel that profile's active request and invalidate pending proposals before later requests use the new profile. Resetting preferences SHALL disable AI and reset non-secret settings without deleting credential-store entries, conversation files, action journals, or user documents; explicit management actions SHALL remove retained data.

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

#### Scenario: Untouched factory limits move forward
- **WHEN** a saved profile still has the previous factory limit tuple and preferences load
- **THEN** that profile uses the current factory context, output, and idle limits
- **AND** a profile whose context size or any other limit was customized keeps the saved values

### Requirement: Credentials and endpoint handling SHALL protect secrets
Persistent API keys SHALL use the operating system credential store and SHALL never be serialized into preferences, sessions, conversations, action journals, logs, or exported diagnostics. If secure storage is unavailable, the UI SHALL offer clearly labeled session-only key use without a plaintext fallback. The UI SHALL support replacing and explicitly forgetting a key. Showing or hiding the draft key SHALL affect only characters present in that draft field and SHALL NOT read the credential store or session secret back into the field. Requests SHALL use HTTPS except for literal loopback addresses or localhost; non-loopback insecure endpoints, embedded URL credentials, and credential forwarding across redirects SHALL be rejected. A changed endpoint SHALL require explicit key reattachment before a saved credential can be sent there.

#### Scenario: Credential service is unavailable
- **WHEN** the operating system cannot securely persist a key
- **THEN** the user can continue with an explicitly labeled session-only key
- **AND** restarting requires re-entering it and no plaintext key is written

#### Scenario: Endpoint changes or redirects
- **WHEN** a configured endpoint changes authority or returns a redirect
- **THEN** the old credential is not forwarded to a new authority
- **AND** the user receives actionable endpoint or key feedback

#### Scenario: Saved key stays concealed
- **WHEN** settings open for a profile that already has a stored or session key and the draft key field is empty
- **THEN** the show control does not display that stored or session key
- **AND** the field remains empty until the user types a replacement
