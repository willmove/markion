## 1. Limits and presets in crates/ai

- [x] 1.1 Update `Limits` defaults and validation ranges to the design table (512 KiB context, 512 KiB output bytes, 8192 output tokens, 120 second idle, ceilings 8 MiB / 4 MiB / 131072 tokens / 300 second idle) and verify `cargo test -p markion-ai` covers the new bounds and rejects the old ceilings
- [x] 1.2 Upgrade a loaded profile only when its full limit tuple equals the previous factory defaults, leaving any customized field unchanged, and verify a deserialize test for both cases
- [x] 1.3 Add the Gemini, xAI, Mistral, Moonshot, Qwen, Zhipu, and OpenRouter presets with the design endpoints, chat protocol, required keys, and starter model ids, plus starter ids for OpenAI, Anthropic, and DeepSeek, and verify preset URL validation and `key_required` in `cargo test -p markion-ai`

## 2. Context trimming

- [x] 2.1 Trim oldest complete messages when they, and not the current user turn, exceed the input budget, return a trim count, and leave an oversized current turn or attachment as `AiError::Limit` without shortening it; verify conversation tests for omit, keep-current, and attachment overflow
- [x] 2.2 When the encoded body plus system prompt still exceeds the budget, trim another older plain-text turn or fail; do not rewrite provider wire history, and fail a wire continuation that is still over budget; verify a protocol or app-level test that an over-budget wire body is not partially sent

## 3. Localization

- [x] 3.1 Add seven-language strings for context size, output size, approximate tokens, restore recommended limits, show/hide API key, localized protocol labels, updated limit ranges, and the omitted-turns notice, and verify the AI catalog completeness test passes

## 4. AI settings UI

- [x] 4.1 Edit context and output limits as whole KiB, show the approximate token hint, restore recommended limits into the dirty draft, and keep tool bytes and the other counters in their existing units; verify the settings parse/save tests accept 512 KiB and reject an out-of-range KiB value by field name
- [x] 4.2 Show the new provider chips and localized protocol labels while storing `responses`, `chat`, and `anthropic`; verify a provider-switch test fills the Moonshot or Gemini endpoint and starter model without sending a request
- [x] 4.3 Add show/hide for the draft API key only, allow copy when revealed, reset to masked when the draft is refilled, and do not read stored credentials; verify a test that toggling visibility does not mark the draft dirty and an empty saved-key field stays empty

## 5. Composer and panel feedback

- [x] 5.1 Give the composer the active profile's context budget, refuse a larger paste without changing the previous composer text or the document, and show the existing budget feedback; verify an input test that a paste above the budget leaves both the composer and the document unchanged
- [x] 5.2 Show the omitted-turns notice after a successful trim and keep the existing attachment repair path when the current turn does not fit; verify the panel feedback test distinguishes those two outcomes

## 6. Preferences tab strip

- [x] 6.1 Make the preferences tab strip a single non-wrapping, horizontally scrollable row without the 112px minimum width, and use an 800px panel width for every preferences tab; verify the preferences source test asserts the strip does not wrap and still wires General through AI Agent

## 7. Docs and check

- [x] 7.1 Update `docs/ai-agent-guide.md` and `docs/ai-agent-guide.zh-CN.md` for the 512 KiB context default, the new presets, and draft-only key reveal, and verify those guides no longer state 65536 as the default input budget
- [x] 7.2 Run `cargo test --workspace` and `openspec validate refine-ai-settings-and-preference-tabs --strict`, and verify both succeed without changing document-derived Markdown caches
