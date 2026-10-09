## ADDED Requirements

### Requirement: AI panel entry SHALL use the customizable shortcut registry
The editor SHALL register a stable `toggle-ai-panel` action using Ctrl+Shift+A on Windows/Linux and Cmd+Shift+A on macOS. Its effective binding SHALL appear in the localized menu/reference and participate in conflict validation and live rebinding. When AI is disabled or not configured, invoking the explicit action SHALL open AI setup without enabling AI or sending a request. With a configured profile it SHALL toggle the panel and transfer focus deliberately between composer and editor. Composer-local editing bindings SHALL not invoke document editing commands.

#### Scenario: Shortcut opens AI setup
- **WHEN** the user invokes the AI panel action with AI disabled
- **THEN** Preferences opens on AI and AI remains disabled until the user explicitly enables it

#### Scenario: Override applies consistently
- **WHEN** the user assigns a valid override for `toggle-ai-panel`
- **THEN** dispatch, menu labels, and the shortcut reference immediately use that override

#### Scenario: Composer has focus
- **WHEN** the user invokes editing keys while the AI composer has focus
- **THEN** only the composer is edited and the active document's text/selection are unchanged
