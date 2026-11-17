## Why

AI setup still assumes a small local byte budget, a short provider list, and a preferences tab strip that wraps once Images and AI Agent are included. The default 64 KiB context rejects ordinary documents and chat history, the API key field can only show asterisks, and the sixth preferences tab sits alone on a second row.

## What Changes

- Raise the factory context, output, and idle limits, and upgrade saved profiles that still match the previous factory tuple. Customized limits stay as saved.
- When older complete conversation turns are what overflow the context budget, omit those turns and show a trim notice. The current turn and its attachments are never clipped; an oversized attachment or current turn still blocks sending with the existing repair feedback.
- Let the composer accept text up to the active profile's context budget. Pasting more than that budget keeps the previous composer text and shows the existing budget feedback.
- Edit the context and output byte budgets as whole kibibytes, with an approximate token hint and a control that restores the recommended limits into the draft.
- Raise the output-token ceiling so long-form replies are not capped at 32768, and raise the idle-timeout ceiling so reasoning models are not limited to 120 seconds.
- Add Gemini, xAI, Mistral, Moonshot, Qwen, Zhipu, and OpenRouter as OpenAI-compatible cloud presets, each with an editable starter model id. Existing OpenAI, Anthropic, DeepSeek, Local, and Custom presets remain.
- Add a show/hide control that reveals only the API key currently typed into the draft. Saved credentials stay in the OS store or session memory and are not loaded back into the field.
- Show protocol choices with localized labels while persisting the existing `responses`, `chat`, and `anthropic` codes.
- Keep the preferences tab strip on one non-wrapping row. If the panel is too narrow, that strip scrolls horizontally.

**Non-goals**: reading a saved API key back for display; sampling parameters such as temperature; new wire protocols; Groq, SiliconFlow, Together, or Azure as their own presets; changing how a credential is addressed when the endpoint changes; per-field steppers on all eight limit fields; silently clipping attachments or the current user turn.

## Capabilities

### New Capabilities

- None.

### Modified Capabilities

- `ai-configuration`: Larger factory limits, kibibyte context/output editing, recommended-limit reset, additional cloud presets with starter model ids, and draft-only API key reveal. Base requirements currently live in the unarchived `add-configurable-ai-agent` and `improve-ai-agent-usability` changes; this delta uses those requirement headers.
- `ai-conversation`: Older complete turns may be omitted with a visible trim notice. Attachments and the current turn stay unclipped, and the composer follows the context budget. Base requirements currently live in the unarchived `add-configurable-ai-agent` and `refine-ai-assistant-workflows` changes.
- `theme-preferences`: The preferences tab strip stays on one row and scrolls horizontally instead of wrapping.
- `ui-i18n`: New AI settings labels, protocol display names, limit ranges, and the trim notice are localized in all seven languages.

## Impact

- `crates/ai` limit defaults, validation ceilings, preset table, and conversation context assembly. The crate stays free of GPUI. Document-derived Markdown caches, syntax highlighting, and per-version text handles are unchanged.
- Preferences AI settings in `src/app/ai_panel.rs`, the masked AI input in `src/app/ai_input.rs`, and the preferences tab strip in `src/app/root_view.rs`.
- Localization in `src/ai_i18n.rs`, plus `docs/ai-agent-guide.md` and `docs/ai-agent-guide.zh-CN.md`.
- Archive this change only after `add-configurable-ai-agent`, `improve-ai-agent-usability`, and `refine-ai-assistant-workflows`, so their base requirements exist before these modified headers are synced.
