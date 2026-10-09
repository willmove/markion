## Context

See proposal.md — Why. The relevant current implementation (all in the root crate unless noted):

- `src/app/ai_panel.rs` holds both the settings view (`settings_view`, ~line 2569) and the panel view. Settings draft state is a positional `inputs: Vec<TextInput>` array (`inputs[0]`=name, `[1]`=model, `[2]`=key, `[3]`=endpoint, `[4]`=protocol, `[5]`=guidance, `[6]`=limits-string), refilled by `fill_inputs` from the persisted profile on every provider/profile switch. Probe entry is `ai_probe(test: bool)` (~1278); commits go through `ai_commit_profile` (~1251). One shared `feedback: Option<AiMsg>` slot serves settings and panel (~2723 vs ~3004).
- Chips (profile/provider/model), conversation tabs, attachment chips, and source links are plain mouse-only `div`s; the existing `button()` helper (~2159) already implements focus + Enter/Space activation.
- `crates/ai/src/config.rs` owns `Protocol` (3 variants), `Limits` (8 validated numeric fields), and the 5 presets; endpoint/protocol/limits validation already exists and can report which value failed — the UI just collapses everything into `AiMsg::Invalid`.
- `src/ai_i18n.rs` is the `AiMsg` catalog, ~115 messages × 7 languages with a completeness test.
- Panel width: render clamps at ≥260px (~2856), drag handler clamps 320–650 (~3186).

Constraints: `crates/ai` must stay GPUI-free; per-version document caches are untouched because settings/panel state never flows through document versioning.

## Goals / Non-Goals

**Goals:**
- Make first-time setup of every preset completable from the basic section alone, with per-field validation.
- Make verification (Test Connection / Find Models) possible before enabling AI.
- Eliminate silent draft loss and label-reuse feedback.
- Keyboard-operable AI surfaces; consistent clamps and naming.

**Non-Goals:**
- No new `AiPreferences`/`Profile` persisted fields beyond what exists (session-only UI state is fine); no changes to credential storage, transport, agent loop, or journal.
- No modal dialog framework; confirmations reuse inline UI patterns.

## Decisions

1. **Structured `SettingsDraft` replaces positional `inputs[]`.** A named struct (`name`, `model`, `key`, `endpoint`, `protocol`, `limits: [String; 8]`, `guidance`) with a `dirty()` comparison against the persisted profile. Rationale: named fields are what per-field validation, dirty tracking, and conditional visibility all need; the positional array is the root of the current fragility. Alternative considered: keep `inputs[]` and add side tables — rejected, it compounds the indexing problem.

2. **Per-field validation surfaced from existing config validation.** `crates/ai` validation already knows which limit/endpoint/protocol is wrong; the UI maps the failing field to a dedicated `AiMsg` variant with the field name and range. The limits string parser at `ai_panel.rs` ~256 is deleted; eight labeled numeric inputs write directly into `Limits` fields. Limits labels use short names (Timeout, Output tokens, Context tokens, Tool steps, …) from `AiMsg`.

3. **Conditional endpoint visibility by preset, protocol as chips.** `Profile` presets already declare their defaults: `custom` (empty endpoint) and `local` show Base URL in basic setup; cloud presets keep it under Advanced. Protocol becomes a chip row of the `Protocol` variants compatible with the preset (custom/local get all three; cloud presets lock to their native protocol with an Advanced override). Alternative considered: always show endpoint — rejected, it re-clutters the cloud happy path the original design deliberately simplified.

4. **Pre-enable probing without persisting.** `ai_probe` drops the `enabled` gate and instead requires a *validatable draft*; it builds an ephemeral profile from the draft plus the typed/session/keychain key (existing precedence) and never writes preferences. On success with `enabled == false`, settings show an explicit "Enable AI" button; enabling remains a separate user action per the opt-in requirement. Document content is never involved — the probe is synthetic, so the disable-invariant (no AI requests while disabled) is re-scoped by the spec delta to exempt the explicit synthetic test.

5. **Split feedback slots and dedicated messages.** `AiUi` gains `settings_feedback` and `panel_feedback`. New `AiMsg` variants: `Saved`, `TestSucceeded`, `TestTextOnly`, `DiscoveryEmpty`, `DiscoveryRunning`, `KeyForgotten`, `DraftDiscardPrompt`, per-field validation variants, `MissingModel`/`MissingKey` guidance, etc. Reused-label feedback (`AiMsg::Save`, `AiMsg::Model` as status) is removed.

6. **Inline discard confirmation, no modal.** Dirty-guard prompts render as an inline confirmation row ("Unsaved changes — Save / Discard / Keep editing") at the top of settings, consistent with the app's existing inline patterns; intercept points: provider chip click, profile chip click, add/remove profile, preferences-tab switch away from AI.

7. **Keyboard operability via the existing `button()` helper.** Chips/tabs/source links/attachment chips convert to the focusable helper (or gain equivalent `focus_handle` wiring); focus ring uses the existing accent styling. No new dependency.

8. **Panel width clamp unified at 320–650** (the drag handler's values), applied in render too; the 260px render-only minimum was unreachable and inconsistent.

9. **Consistent product naming: "AI Agent" in all seven languages** (proper noun, not translated), replacing "AI 助手"/"AI アシスタント"-style variants for the tab and status-bar labels; docs (`docs/ai-agent-guide*.md`) updated to match. Alternative considered: translating per language — rejected, the divergence itself was the defect.

10. **Misconfiguration guidance in the panel.** When `enabled` but the selected profile fails validation, the panel shows the specific gap (model/key/endpoint/limits) with an "Open AI settings" action that focuses the offending input (settings view accepts an optional focus target).

## Risks / Trade-offs

- [Pre-enable probing weakens the "disabled = zero network" invariant] → Spec delta re-scopes the invariant: disabled still means no *feature* requests; the synthetic, user-initiated test against the configured endpoint is explicitly allowed. Document content remains unsent.
- [Eight limits inputs lengthen the advanced section] → They live under Advanced only; basic setup unchanged for cloud presets.
- [Dirty-guard intercepts add click friction to chip switching] → Only fires when the draft actually differs; clean drafts switch instantly as today.
- [`SettingsDraft` refactor touches most of `settings_view`] → Done as one mechanical commit before behavior changes; covered by existing settings tests plus new per-field validation tests.
- [Renaming zh labels to "AI Agent" may read less native] → Proper-noun treatment is common in zh product UI; consistency and doc alignment outweigh it.

## Migration Plan

Pure UI/UX change; existing `config.toml` AI tables load unchanged (no persisted schema change). Users who previously saved settings see the same values in the new structured fields. Rollback is a normal revert; no data migration or format version bump.

## Open Questions

(none)
