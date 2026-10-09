## Why

The AI agent delivered by `add-configurable-ai-agent` is functionally complete but has first-run and everyday usability friction: first-time Custom/Local users cannot see the Base URL field without discovering an unmarked Advanced toggle, Test Connection is blocked until the user enables an unverified configuration, limits are edited as one comma-separated string of eight raw numbers, provider switching silently discards unsaved edits, and status feedback reuses button labels ("Save", "Model ID") instead of meaningful messages. These issues make setup error-prone and the panel harder to trust.

## What Changes

- **Guided basic setup.** Show the Base URL field in basic setup whenever the selected preset requires or commonly edits it (Local/Custom), present Protocol as a preset-aware dropdown instead of free text, and show a per-field validation message naming the failing field instead of the generic invalid-settings error.
- **Verify before enabling.** Allow Test Connection and Find Models against the current draft without requiring the master switch; a successful test on an unconfigured installation offers a one-click "Enable AI" affordance. Enabling AI never sends document content by itself.
- **Structured limits editing.** Replace the single comma-separated limits string with individually labeled numeric fields (or keep the string only as a hidden fallback); each field validates independently with its own error.
- **Draft-loss protection.** Warn before switching provider presets, profiles, or leaving the AI tab when the draft has unsaved changes; writing guidance edits are saved independently of profile commits instead of being silently reset.
- **Meaningful feedback.** Introduce dedicated status strings for save/test/discover/forget/stop outcomes (replacing reused button labels), separate settings feedback from panel feedback, add a loading indicator and an explicit empty-result message for model discovery, and give discovered-model chips a selected state that persists into the draft.
- **Enabled-but-misconfigured guidance.** When AI is enabled but the active profile is unusable (e.g. empty model), the panel and settings show what is missing with an action that jumps to the offending field, instead of failing sends with a generic error.
- **Profile management polish.** Add Profile presets the provider currently being viewed, Remove Profile asks for confirmation and offers to forget the retained key, and the Advanced disclosure shows its expanded/collapsed state.
- **Keyboard and consistency fixes.** Make profile/provider/model chips, conversation tabs, attachment chips, and source links keyboard-focusable and activatable; align the panel width clamp between render and drag; unify the AI tab naming across languages.

Non-goals: no new providers, protocols, tools, or writing actions; no changes to credential storage, the action journal, tool-loop bounds, or network transport; no automatic whole-workspace indexing. This change is presentation, validation, and workflow polish on the existing AI feature — the per-version `Arc` caches, memoized highlighting, and other editing invariants are untouched.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `ai-configuration`: Basic setup field visibility, pre-enable connection testing, structured limits/protocol editing, validation and feedback messaging, draft-loss protection, profile add/remove behavior. (The base spec currently exists as delta specs in the completed, not-yet-archived change `add-configurable-ai-agent`; this change's deltas target the same capability path.)
- `ai-conversation`: Misconfiguration guidance in the panel, dedicated feedback strings, keyboard-focusable interactive elements, panel width clamp consistency.
- `ui-i18n`: New dedicated feedback/validation/confirmation strings for all seven languages; consistent AI tab naming.

## Impact

- `src/app/ai_panel.rs`: settings view restructuring (conditional Base URL, protocol dropdown, labeled limits fields, draft-dirty tracking, confirmation prompts), feedback plumbing split, keyboard focus for chip/tab/link controls, panel width clamp fix.
- `src/app/ai_input.rs`: possible reuse for new labeled numeric inputs; no behavioral changes to the composer.
- `crates/ai/src/config.rs`: no schema change expected; draft validation may surface per-field errors already representable by existing config validation.
- `src/ai_i18n.rs` and `src/i18n.rs`: new `AiMsg` entries for all seven languages (en, zh-Hans, zh-Hant, ja, fr, de, es), including validation, confirmation, discovery, and guidance strings; completeness tests extended.
- `openspec/changes/add-configurable-ai-agent/specs/`: this change's delta specs are written against those capability paths (`ai-configuration`, `ai-conversation`); archiving order must archive `add-configurable-ai-agent` first.
- Tests: extend existing AI settings/panel tests (mock provider, no real API keys) for validation messages, pre-enable testing, draft-loss guards, and keyboard activation.
