## Context

See proposal.md for why this change exists. Limits, presets, and credential references live in GPUI-free `crates/ai`. The preferences UI and composer live in the root app. Document-derived Markdown caches are not on the AI request path.

```
composer + attachments + complete turns
        │
        ▼
context assembly (crates/ai)
        │  current turn over budget → existing limit error, no send
        │  older turns over budget → drop oldest complete turns
        ▼
JSON body + system prompt check
        │  still over → drop another older turn, or fail if only the current turn remains
        ▼
existing transport
```

## Goals / Non-Goals

**Goals:**

- Make the factory context budget large enough for a normal document plus several chat turns, without removing the finite ceiling.
- Keep attachment and current-turn overflow as a hard, visible refusal.
- Reveal only the key characters the user has just typed.
- Add the seven cloud presets through the existing chat protocol.

**Non-Goals:**

- See proposal.md. This design also does not rewrite provider wire history, add per-provider request headers, or change credential identity when an endpoint changes.

## Decisions

### 1. Factory limits and exact-tuple upgrade

Previous factory tuple, used only to recognize untouched profiles:

| Field | Previous |
| --- | --- |
| input bytes | 65536 |
| output bytes | 131072 |
| tool bytes | 65536 |
| max tools | 12 |
| max operations | 20 |
| timeout seconds | 300 |
| idle seconds | 30 |
| output tokens | 4096 |

New factory defaults and validation ranges:

| Field | Default | Range |
| --- | --- | --- |
| input bytes | 524288 (512 KiB) | 1024–8388608 (1–8192 KiB) |
| output bytes | 524288 (512 KiB) | 1024–4194304 (1–4096 KiB) |
| tool bytes | 65536 | 256–262144, unchanged |
| max tools / operations / timeout | unchanged | unchanged |
| idle seconds | 120 | 1–300, and still no greater than timeout |
| output tokens | 8192 | 128–131072 |

On load, if every limit field equals the previous tuple, replace that profile's limits with the new defaults. Any single customized field keeps the whole saved tuple. Missing limit objects already deserialize through `Default`, so they receive the new defaults without a special case.

Alternative considered: always rewrite every saved profile to 512 KiB. Rejected because a user who chose a smaller budget would lose that choice.

### 2. Edit context and output size in KiB

The settings draft stores the same byte fields. The context and output inputs display and parse whole KiB (`bytes / 1024` on fill, `KiB * 1024` on save). Tool bytes, counts, and seconds stay in their current units because the tool minimum is 256 bytes, which is not a whole KiB. Each byte field still shows an approximate token hint of `bytes / 4`, labeled as approximate. Restore recommended limits writes the new factory tuple into the draft and marks it dirty; it does not persist until the existing Save action.

### 3. Trim older turns, never the current turn

`Conversation::context` gains a trim count. It keeps the current user message, including attachment text, intact. While the assembled messages exceed the budget and an older complete message remains, it drops the oldest message. A request that still does not fit returns the existing limit error.

The app then builds the system prompt and encoded body. If that byte check still fails and an older message remains, it repeats the trim. Provider `wire_history` is not edited. When a continuation still carries wire history and the encoded body exceeds the budget, the send fails with the existing budget feedback instead of dropping an unpaired tool result.

The panel shows a localized notice with the number of omitted turns. That notice is conversation feedback, not a settings error.

Alternative considered: always fail, as today. Rejected because a long chat then becomes unusable even after the user raises the budget only modestly. Alternative considered: clip attachment text. Rejected by the existing attachment rule.

### 4. Composer capacity follows the profile budget

`AiInput` takes a per-field byte ceiling. The composer uses the active profile's `input_bytes` (512 KiB when no profile is selected). Key, endpoint, model, and name fields stay at 64 KiB. Guidance stays within its existing 8192-character persistence cap. When an insertion would exceed the composer ceiling, the previous composer text stays and the panel uses the existing input-budget feedback. Document text and undo history are untouched.

### 5. Draft-only API key visibility

The key field starts masked. Show/Hide toggles that field's `masked` flag. Copy and cut follow the existing rule that masked fields do not reach the clipboard. `fill`, provider switch, and profile switch set the flag back to masked and continue to clear the field. The toggle does not mark the draft dirty and does not call the credential store. An empty field stays empty, so a saved key cannot appear.

### 6. Preset table

Append these chat-protocol, key-required presets after DeepSeek and before Local. `key_required` includes their ids. Basic setup keeps their endpoints under Advanced, same as the other cloud presets. Switching provider still replaces the draft through `Profile::preset`, so the starter model replaces an unsaved model only after the existing unsaved-draft confirmation.

| Id | Label | Endpoint | Starter model |
| --- | --- | --- | --- |
| `gemini` | Gemini | `https://generativelanguage.googleapis.com/v1beta/openai` | `gemini-2.5-flash` |
| `xai` | xAI | `https://api.x.ai/v1` | `grok-4` |
| `mistral` | Mistral | `https://api.mistral.ai/v1` | `mistral-large-latest` |
| `moonshot` | Moonshot | `https://api.moonshot.cn/v1` | `kimi-k2.5` |
| `qwen` | Qwen | `https://dashscope.aliyuncs.com/compatible-mode/v1` | `qwen-plus` |
| `zhipu` | Zhipu | `https://open.bigmodel.cn/api/paas/v4` | `glm-4.7` |
| `openrouter` | OpenRouter | `https://openrouter.ai/api/v1` | `openrouter/auto` |

OpenAI, Anthropic, and DeepSeek also receive starter ids on newly built presets: `gpt-5.2`, `claude-sonnet-4-5`, and `deepseek-chat`. Saved profiles keep their stored model until the user switches provider and saves. Local and custom stay empty. Starter ids are editable starting points; discovery remains the way to pick a live model. No network call is added to tests.

Protocol chips display localized names and write the existing codes.

### 7. One-row preferences tabs

Remove wrapping and the 112px minimum width from the preferences tab strip only. Tabs use their label width, the row does not wrap, and the strip scrolls horizontally when needed. Set the preferences panel width to 800px for every tab so the six current labels fit at normal width in the supported languages; Shortcuts no longer needs a separate 720px width. Language pills and provider chips inside scrolling bodies may still wrap.

## Risks / Trade-offs

- [Starter model ids go stale] → They are editable, discovery still overrides them, and tests do not contact providers.
- [512 KiB default sends more text and costs more] → It stays a per-profile ceiling, and untouched custom budgets are preserved.
- [Idle default of 120 seconds waits longer on a dead stream] → The overall timeout stays 300 seconds, and customized idle values are preserved.
- [KiB editing rounds a hand-edited byte value that is not a multiple of 1024] → Fill uses integer division only for display; save writes `KiB * 1024`. The migration path only rewrites the exact previous factory tuple, whose values are already multiples of 1024.
- [Horizontal tab scroll is easy to miss] → 800px fits the current labels, so scrolling is the narrow-window fallback.
- [Moonshot's China endpoint is wrong for some accounts] → The endpoint remains editable under Advanced. The international host is not a second preset.

## Migration Plan

1. Ship the new defaults and the exact-tuple upgrade in preferences loading.
2. No credential or document migration.
3. Rollback is a code revert. Profiles already upgraded to the new factory tuple would remain on those larger limits until the user restores or edits them; customized profiles are unchanged either way.

## Open Questions

None. Starter model ids are fixed in the table above and can be replaced later without a behavior change beyond the prefilled text.
