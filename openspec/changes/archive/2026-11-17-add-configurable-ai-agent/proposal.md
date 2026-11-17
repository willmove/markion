## Why

Markion currently has no AI assistance: users must leave the editor to draft or improve prose, ask questions about their notes, or organize related files. An optional, configurable agent integrated with the document and workspace can complete these workflows while preserving the editor's responsive editing and reliable persistence.

## What Changes

- Add an AI master switch, off by default, and an approachable AI Preferences tab. Setup uses provider presets, an API key where required, a selectable or manually entered model, and an explicit connection test; endpoint, limits, guidance and history remain under advanced settings, with directly accessible profile switching.
- Support user-supplied API keys for cloud services and keyless local servers, as confirmed by the user. Initial adapters cover OpenAI Responses, OpenAI-compatible Chat Completions (including DeepSeek, Ollama, and custom servers), and Anthropic Messages. Models without verified tool support retain writing and conversation capabilities.
- Add a resizable AI conversation panel with streaming Markdown responses, follow-up questions, source references, visible context attachments, stop/retry controls, and optional local conversation history. AI can be closed independently of disabling the feature.
- Provide practical writing actions: draft from instructions, continue, polish, correct spelling/grammar, shorten, expand, adjust tone, translate, summarize, extract an outline/tasks, and custom instructions. Preview generated changes before replacing a selection, inserting at a captured caret, or creating a new note; accepted edits remain undoable.
- Add a bounded agent loop with workspace listing/search/reading, multi-note synthesis, and proposals for creating/editing notes, creating folders, and renaming/moving files. One review surface selects and applies concrete changes; read access is scoped to explicitly attached files or an approved workspace subtree.
- Preserve dirty buffers, reject stale document/disk results, respect Git write admission, validate real filesystem containment, and keep file mutations recoverable through a separate operation journal and Restore action. No arbitrary commands or permanent deletion are exposed to the agent.
- Localize every new interface string for all seven existing languages and integrate AI entry points with the existing menu/shortcut system. Disabling AI stops active work and prevents AI network/tool activity without affecting ordinary editing.

Non-goals: bundled model weights or model installation, subscription billing or account sign-in, web/browser/OS automation, shell execution, runtime plugins/MCP, autonomous background agents, automatic publishing/Git writes, permanent deletion, and automatic whole-workspace indexing. These are outside this initial complete delivery, not hidden implementation tasks.

## Capabilities

### New Capabilities

- `ai-configuration`: Opt-in setup, provider profiles, secure credentials, connection testing, model capabilities, compatible preferences, and disable behavior.
- `ai-conversation`: Streaming conversations, explicit context and read scopes, source references, bounded tool execution, cancellation, history, and usable panel behavior.
- `ai-writing`: Writing presets and custom instructions, exact source snapshots, review/application destinations, undo, and stale-result handling.
- `ai-workspace-actions`: Workspace retrieval and multi-file proposals, change review, safe execution, Git/persistence coordination, and durable recovery.

### Modified Capabilities

- `theme-preferences`: Append the AI tab while retaining existing tab ordering and ownership; make setup available when AI is disabled.
- `keyboard-shortcuts`: Register the AI panel action with a customizable platform shortcut and correct input-focus behavior.
- `ui-i18n`: Require complete localized AI settings, conversation, writing, tool, review, error, and recovery chrome.
- `crate-architecture`: Introduce a GUI-free `markion-ai` member with GPUI integration and live document/filesystem coordination retained in the root application.

## Impact

- Add `crates/ai` for provider/protocol adapters, streaming parsing, context budgets, the agent state machine, and pure proposal/path-policy models; no GPUI dependency.
- Extend `src/model.rs`, `src/storage/preferences.rs`, and new credential/history/action-journal storage. API keys never enter `config.toml`, sessions, logs, or AI request content; use OS credential storage with a session-only fallback.
- Integrate dedicated AI application modules with `src/app/mod.rs`, `application.rs`, `root_view.rs`, `appearance.rs`, `editing.rs`, `workspace.rs`, `shortcuts.rs`, and `network.rs`; add an attributable AI mutation origin in `src/lib.rs` and strings/catalog entries in `src/i18n.rs`.
- Reuse checked mutations, source/selection mapping, undo/recovery, atomic writes, file-tree refresh/tab remapping, and `markion-git-sync` admission. The existing HTTP adapter buffers entire response bodies, so AI requires a dedicated cancellable streaming transport rather than passing through that adapter.
- Reuse GPUI's operating-system credential APIs and add bounded async-stream support only as needed after checking the locked dependency graph; update Cargo manifests/lockfile without changing release versions.
- Preserve per-document-version `Arc` caches, memoized highlighting, cached editor text handles, and bounded tree rendering. Token deltas and conversation updates must not mutate documents or trigger document re-derivation.
- Validate with mock-provider protocol tests, real temporary-workspace/recovery fixtures, existing mutation/persistence regressions, workspace tests, and desktop interaction checks. No user API key or paid network call is needed for automated tests.
