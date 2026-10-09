## 1. Settings draft refactor

- [x] 1.1 Replace the positional `inputs[]` array in `src/app/ai_panel.rs` with a named `SettingsDraft` struct (name, model, key, endpoint, protocol, limits as eight fields, guidance) plus a `dirty()` comparison against the persisted profile; verify `cargo build` passes and existing AI settings tests still pass
- [x] 1.2 Split the shared feedback slot into `settings_feedback` and `panel_feedback` and route existing call sites accordingly; verify a settings result no longer appears in the panel and vice versa (unit test or manual check)
- [x] 1.3 Add the new `AiMsg` variants (dedicated save/test/discovery/forget outcomes, per-field validation with field name and range, draft-loss confirmation, misconfiguration guidance, empty-discovery result) for all seven languages in `src/ai_i18n.rs`; verify the catalog completeness test passes

## 2. Basic setup improvements

- [x] 2.1 Show the Base URL input in basic setup when the selected preset is `custom` or `local` (cloud presets keep it under Advanced); verify by selecting each preset and observing field visibility
- [x] 2.2 Replace the protocol free-text input with protocol choice chips constrained per preset (cloud presets locked to native protocol with Advanced override); verify invalid protocol strings can no longer be entered and `cargo test` protocol tests pass
- [x] 2.3 Replace the comma-separated limits string with eight individually labeled numeric fields, each validated independently with field-specific error messages; verify entering an out-of-range value names that field and preserves the other seven values
- [x] 2.4 Remove reused-label feedback (Save/Model ID/Forget key/Stopped) and emit the dedicated outcome messages from 1.3; verify each action displays its own meaningful status string in every language

## 3. Verify-before-enable flow

- [x] 3.1 Allow `ai_probe` (Test Connection and Find Models) to run against a validatable draft while AI is disabled, using an ephemeral profile and existing key precedence without persisting anything; verify with the mock provider that a disabled installation can test successfully
- [x] 3.2 Show an explicit "Enable AI" action after a successful test while disabled; verify enabling remains a separate user action and no document content is ever sent by testing
- [x] 3.3 Add model-discovery loading and explicit empty/unsupported result states, and give discovered-model chips a visible selected state that fills the model draft without saving; verify discovery results clear only when endpoint, protocol, or profile identity changes

## 4. Draft-loss protection and profile management

- [x] 4.1 Add the inline unsaved-changes confirmation row (Save / Discard / Keep editing) intercepted on provider chip, profile chip, add/remove profile, and preferences-tab switch away from AI; verify a dirty draft is never silently replaced and a clean draft switches without prompts
- [x] 4.2 Persist writing guidance independently of profile commits and stop resetting it on preset/profile switches; verify editing guidance, switching profiles, and saving guidance keeps the new value
- [x] 4.3 Make Add Profile preset the provider currently being viewed, and add a Remove Profile confirmation offering to also forget the retained key; verify both behaviors manually and the orphaned key no longer lingers when forgotten

## 5. Panel guidance and keyboard operability

- [x] 5.1 When AI is enabled but the active profile fails validation, show the specific gap (missing model/key/endpoint/limits) in the panel with an "Open AI settings" action that focuses the offending input, preserving the drafted prompt; verify with an empty-model profile that sending shows guidance instead of a generic error
- [x] 5.2 Convert profile/provider/model chips, conversation tabs, attachment chips, source references, and disclosure toggles to the focusable button pattern with visible focus indication; verify Tab reaches every element and Enter/Space activates it
- [x] 5.3 Unify the panel width clamp to 320–650 in both render and drag paths, and add the expanded/collapsed indicator to the Advanced disclosure; verify a dragged width survives re-render
- [x] 5.4 Unify the AI product name as "AI Agent" across all seven languages (tab, status bar, menu labels) and update `docs/ai-agent-guide.md` and `docs/ai-agent-guide.zh-CN.md` to match; verify no stale divergent labels remain via `grep`

## 6. Validation

- [x] 6.1 Add/extend tests for per-field validation messages, pre-enable probing with the mock provider, draft-loss guard triggers, discovery empty state, and feedback slot separation; verify `cargo test` passes
- [x] 6.2 Run `cargo test --workspace` and the quality script (`scripts/check-quality.ps1` via pwsh) and fix any regressions
- [ ] 6.3 Walk the acceptance flow manually (fresh profile → custom preset → enter endpoint → test → enable → chat) and confirm every step is reachable without Advanced for custom/local setups
- [x] 6.4 Run `openspec validate improve-ai-agent-usability --strict` and confirm the change is valid
- [ ] 6.5 Before archiving this change, archive the completed `add-configurable-ai-agent` change first so the `ai-configuration` base spec and the `ui-i18n` AI requirement exist (this change's MODIFIED deltas target them); verify with `openspec list --specs` that `ai-configuration` is present
