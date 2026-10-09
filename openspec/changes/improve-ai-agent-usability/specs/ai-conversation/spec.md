## ADDED Requirements

### Requirement: Enabled but misconfigured AI SHALL guide repair
When AI is enabled but the selected profile cannot run (missing model, missing required key, invalid endpoint, or invalid limits), the panel SHALL state what is missing and offer an action that opens AI settings focused on the offending field, instead of failing sends with a generic error. The composer SHALL remain usable for drafting while configuration is incomplete, and the blocked send SHALL preserve the drafted prompt.

#### Scenario: Enabled profile without a model
- **WHEN** AI is enabled and the active profile has an empty model and the user sends a prompt
- **THEN** the panel explains the model is missing and offers to open settings
- **AND** the prompt text remains in the composer or retry snapshot

### Requirement: Panel interactive elements SHALL be keyboard operable
Profile and provider chips, discovered-model chips, conversation tabs, attachment chips, source references, and disclosure toggles in AI surfaces SHALL be reachable by keyboard focus and activatable with Enter or Space, matching the button behavior already present. Focus SHALL be visibly indicated. The panel width clamp SHALL be identical during drag and during render so a dragged width is never overridden on the next frame.

#### Scenario: Keyboard-only model selection
- **WHEN** a keyboard-only user tabs through AI settings after discovering models
- **THEN** every chip, tab, and toggle is reachable in a sensible order and activatable without a mouse

#### Scenario: Dragged width persists
- **WHEN** the user drags the panel below the render clamp width
- **THEN** the rendered panel honors the dragged width consistently instead of snapping to a different minimum
