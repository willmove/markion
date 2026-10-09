## Context

See `proposal.md` for motivation and scope. The user confirmed user-supplied API keys with support for both cloud services and local models.

Observed integration points:

- `src/model.rs::AppPreferences` and `src/storage/preferences.rs::PreferencesFile` separate application preferences from lenient TOML serialization. `config.toml` already stores optional provider settings for images; AI can use the same compatibility pattern.
- `src/app/mod.rs` owns actions, the customizable menu-shortcut registry, `PreferencesTab`, and `MarkionApp` state. `root_view.rs` renders menus/preferences; `appearance.rs` switches and persists settings. Existing tabs are General, Appearance, Images, Shortcuts, Export.
- The stable Appearance spec currently lists only four tabs. The Images extension exists in the active `add-image-upload-and-resource-policies` delta and in code. This change preserves Images and modifies the stable requirement to permit capability-owned tabs while appending AI. Do not archive or alter that other change. Some stable preference prose still describes retired `key=value` persistence; the implementation and this design use TOML and retain legacy compatibility rather than changing unrelated requirements.
- `MarkdownDocument` has a session-local instance identity, a text version, checked source mutations, and content-free mutation journaling. `application.rs::apply_document_mutation` checks the active document and Git state. Per-tab state owns source/visual selections, IME and undo. This is the only document-mutation boundary AI should extend.
- `workspace.rs` already coordinates file creates/renames/moves with Git write admission, refuses dirty moves, remaps clean tabs, and refreshes the bounded tree. Its interactive prompts are not directly suitable for executing reviewed batches; shared execution helpers can be extracted without changing ordinary commands.
- `network.rs` owns a lazy Tokio HTTP runtime, but its GPUI HTTP adapter calls `response.bytes()` and buffers the entire body. AI needs its own streaming response path on this runtime, not a reuse of the buffering `HttpClient::send` implementation.
- Vendored GPUI exposes `App::{read_credentials,write_credentials,delete_credentials}` with platform adapters. Reuse these behind root-app credential storage instead of adding a second keychain library. Tests use an injected fake store, and unavailable system stores use session-only keys.
- `src/i18n.rs` supports English, Simplified Chinese, Traditional Chinese, Japanese, French, German, and Spanish. New `AiMsg` catalog functions can follow the existing `ImageMsg`/`GitMsg` organization while preserving completeness checks.

## Goals / Non-Goals

**Goals:** Keep document mutation and filesystem authority inside the host app; make provider differences and failure/cancellation explicit; allow useful writing even on models without tools; make reviewed persistent actions recoverable; keep all generation/retrieval off input/render paths.

**Non-Goals:** The proposal lists product exclusions. At the design level, no conversation event owns a live `MarkdownDocument`, no provider can directly access the filesystem, no native provider SDK/runtime is required, and no token event uses the document's undo or derived cache.

## Decisions

### 1. A pure AI member plus small root integration modules

Add `crates/ai` (`markion-ai`) for protocol-neutral messages/events, provider adapters, SSE parsing, context budgets, a bounded agent state machine, and proposal/scope validation models. Keep its dependencies GUI-free. The root app uses cohesive `ai_preferences`, `ai_panel`, `ai_session`, `ai_writing`, and `ai_workspace` modules, with credential/history/journal storage behind dedicated root modules.

The member sends typed host-tool requests through an injected executor interface. The root host supplies snapshots and returns results/proposals; it retains all live selection/version checks and filesystem execution. Add `MutationOrigin::AiWriting` with the same checked range admission as other atomic commands. Represent a whole-document rewrite as a checked full source range rather than granting arbitrary whole-document replacement to the agent.

Alternative considered: add everything to `MarkionApp`/`root_view.rs`. Dedicated modules and a headless core reduce UI coupling and make provider/policy tests practical. No new optimization override is needed unless implementation puts compute-heavy AI logic on a live typing path; this design does not.

### 2. Three explicit protocols with provider presets

| Preset | Protocol | Key | Model selection |
| --- | --- | --- | --- |
| OpenAI | Responses | Required | Explicit discovery or manual ID |
| Anthropic | Messages | Required | Explicit discovery or manual ID |
| DeepSeek | Compatible Chat Completions | Required | Explicit discovery or manual ID |
| Local/Ollama | Compatible Chat Completions | Optional | Explicit local discovery or manual installed ID |
| Custom compatible | Compatible Chat Completions | Optional according to server | Explicit discovery where available or manual ID |

Store endpoint root and protocol separately, normalize the preset's route once, and avoid accidental duplicate `/v1` segments. Presets populate endpoints, not permanently pinned model IDs or promises about model availability. Advanced profiles preserve manually configured values when switching presets.

Adapters map messages, text deltas, tool argument completion, tool results, usage, and terminal/errors to one event model. OpenAI Responses uses client-managed conversation replay and `store: false`; compatible chat uses ordered assistant/tool messages and tool-call IDs; Anthropic retains required opaque protocol blocks/signatures for valid replay without rendering hidden reasoning as chat. Provider metadata is bounded and retained only where needed to replay that conversation. Do not assume every compatible endpoint implements every optional field; capability tests distinguish text, streaming, model-list, and tools. If streaming is unsupported, a bounded completed response can still serve writing/chat with an appropriate generating state.

Connection testing is explicit and uses a small synthetic text prompt followed by a harmless `connection_probe` tool where appropriate. Unknown tool support leaves Agent mode unavailable. Model/provider changes invalidate verified capabilities and require a new test before Agent use. Tool capability failure affects Agent only, not available writing/chat.

Alternative considered: one generic OpenAI request for all providers. Native Messages and Responses differ in conversation/tool replay, event framing, and errors; explicit adapters are more reliable while keeping compatible services convenient.

Official protocol references consulted on 2026-10-09:

- [OpenAI function-call streaming](https://developers.openai.com/api/docs/guides/function-calling#streaming): tool arguments arrive in fragments and require complete assembly.
- [Anthropic streaming messages](https://platform.claude.com/docs/en/build-with-claude/streaming): content blocks and message termination require provider-specific event handling.
- [Ollama compatibility](https://docs.ollama.com/api/openai-compatibility): local compatible Chat Completions exposes streaming and tools; supported features remain model dependent.
- [DeepSeek API introduction](https://api-docs.deepseek.com/): a compatible service can share the Chat Completions adapter with its own endpoint/model configuration.

### 3. Safe, simple settings and credential ownership

Add optional `[ai]` and AI profile records to TOML, using stable profile IDs. Basic UI: master switch, provider, masked key, model, Test Connection, Save. Profile switching/add/remove are directly accessible; advanced disclosure contains endpoint/protocol, bounds, writing guidance, and history. AI tab remains available while disabled. Changes are edited in a draft, validated, persisted atomically, then installed; explicit enablement is separate from a successful test. Test and model discovery require the master switch and show Enable AI guidance while disabled; neither action implicitly enables it.

Credential storage keys combine application ID, stable profile ID, and normalized endpoint identity. A changed endpoint cannot inherit an old key automatically. Persistent credential-store entries are read only for explicit setup/use, with no network activity from reading a key. API keys are opaque bytes, masked in the editor, redacted from `Debug` and errors, and scoped to the exact request origin. Disable cross-origin redirects; reject URL userinfo/query credential patterns and non-loopback HTTP. Honor ordinary proxy behavior while documenting that loopback bypass behavior is tested.

Changing provider/model/request settings cancels active generation and invalidates its proposal. Reset disables AI and clears ordinary preferences but preserves separately managed credentials/history/recovery. Removing a profile presents an explicit Forget Key action. A separate non-secret index of app-owned credential references survives preference reset, so retained keys remain discoverable for explicit removal without relying on a keychain enumeration API. Forget treats an absent credential as success.

Alternative considered: store the key in `config.toml`. Existing user files and synchronized folders should never become a credential store; session-only fallback handles Linux desktops without an unlocked credential service without silently weakening storage.

### 4. Explicit snapshots and finite budgets

Use immutable snapshots: request ID, conversation ID, configuration generation, workspace identity/epoch, captured source path/identity/version, exact source bytes/ranges, attachment labels, and read scope. Ordinary chat starts with no document attachment. Quick actions add a visible Selection or Current Document chip only after choosing scope. Continue previews the captured preceding text. Attachment changes affect the next turn, not an in-flight request.

An approved workspace subtree grants bounded reads for that conversation/session, not permanent trust. Disable, revoke, workspace change, conversation switch, and restart remove grants. Previously sent content can remain in displayed history, so scope revocation starts a fresh context session before further requests instead of claiming it can retract earlier disclosure. Protected secrets are denied even when explicitly attached: `.env` variants, private keys, credential/config stores, `.git`, cache/build/dependency directories, and binary files. Hidden files are excluded by default independently of the tree's Show Hidden preference.

Initial engineering defaults: 64 KiB combined attached/tool input, 128 KiB output text, 1 MiB maximum supported source file, 64 KiB per tool result/argument, 100 entries per listing/search page, 12 tool calls, 20 mutation operations per plan, 30 seconds of stream inactivity, and 5 minutes per Agent turn. Settings can lower these or raise them within explicit finite ceilings. Validate bytes and UTF-8 boundaries; display approximate token usage as an estimate, not exact model accounting. Report overflow rather than silently clipping attachments. Older complete chat turns can be omitted only with a visible context-trim indicator; tool calls/results stay paired.

Alternative considered: automatic workspace embeddings/indexing. Explicit retrieval uses the existing bounded tree and current buffers, avoids startup work, and makes data sent to a service inspectable.

### 5. A cancellable event flow isolated from document caches

```text
AI input + chosen attachments
  -> immutable request snapshot + scope/budget checks
  -> credential lookup + provider transport on HTTP runtime
  -> bounded incremental SSE/protocol decoder
  -> bounded event channel (request/config/workspace identities)
  -> AI session state -> cached AI Markdown presentation -> panel repaint
                         |
                         +-> validated host-tool call -> retrieval/proposal
                                                        |
User review --------------------------------------------+
  -> fresh target checks + application admission
  -> checked document mutation OR journaled persistent action
  -> affected document version/cache update OR tree/tab reconciliation
```

Use an abortable request/task plus a cancellation token shared by transport/tool work. Dropping a GPUI task alone is insufficient: cancel the Tokio task, close/drop the response body, and reject late events by request/configuration/workspace generation. At most one turn per conversation and one workspace mutation batch run at once; impose a small app-wide request cap. Expose activity/Stop when the panel is closed.

SSE decoding handles arbitrary byte fragmentation, CRLF, multiline data, UTF-8 splits, keepalives, usage-only events, provider errors, and missing completion. Accumulate tool arguments by their provider IDs; dispatch only complete JSON after a terminal tool marker, with schema/size/tool-name checks. Unknown optional events can be ignored; malformed/truncated required events mark the turn incomplete. Never infer success from connection close.

Coalesce UI text updates at most about 30 times per second. Re-derive only the changed AI message's bounded presentation, use virtualized conversation rows, and preserve user scroll position while reviewing older output; follow the tail only when already at the end. Chat Markdown disables automatic remote resources and uses safe presentation. No event reparses or edits the active document. Document caches change only at accepted mutations.

Alternative considered: retry a failed stream automatically. Replay can duplicate output/tool planning; expose user-initiated retry and never repeat journaled writes.

### 6. Writing review and exact application

Preset prompts live as pure templates with explicit operation, language/tone, preserved Markdown intent, and user guidance. All modes supply canonical source mapping where proven; unmappable rendered ranges become analysis-only context. Read mode/image tabs expose Copy/New Note rather than inline apply.

Review captures destination choices and shows a before/after preview. Replacing a selected/full range or inserting at a captured caret validates the original identity/version/bytes, finishes the prior undo group, applies one checked `AiWriting` range mutation, sets the resulting selection, and starts ordinary recovery/autosave. Applying to an inactive target explicitly activates the original still-open document, rather than interpreting its offset against the currently active tab. A stale/closed/remapped target offers Regenerate, Copy, and New Note; no automatic fuzzy rebase.

New Note creates a dirty untitled tab and leaves saving to the ordinary workflow. A stopped response is copyable but never applicable as a complete rewrite.

Alternative considered: replace a selection continuously during streaming. A review boundary avoids source corruption, undo noise, and conflict with ongoing user typing.

### 7. Tool proposals and host-side authority

Supported tools are `list_files`, `search_files`, `read_text`, `propose_text_edit`, `propose_create_note`, `propose_create_folder`, and `propose_move_or_rename`. Tools never provide shell, Git, publishing, permanent delete, or arbitrary HTTP access. Mutation tools return a typed proposal, not an execution result. Read failures and unsupported tools return typed results to the bounded loop; retrieved content cannot grant permissions.

Text edits carry source identity/version/hash and exact before bytes. Closed-file edits may use a reviewed whole-file replacement under the size bound. Plans contain operation IDs, dependencies, affected paths, checked baselines, and proposed postconditions. Combine non-overlapping edits to the same file into one reviewed post-image against its captured baseline, reject overlapping/conflicting edits, and apply one checked mutation/undo entry per affected buffer. A proposed dirty-buffer edit followed by moving that buffer remains blocked by Save First unless the user explicitly saves between separately reviewed plans. Review supports row selection and destination edits; changed plans get new identities and fresh checks. Rename/move includes proposed relative Markdown-link updates in permitted source files, or explicitly shows unchecked references outside scope. Never scan beyond scope for link repair.

For folder organization, create destination folders and move individual listed files. Existing folder moves are supported only when every affected descendant is permitted, supported, and included in review; otherwise expand into individual file proposals. This prevents moving hidden secrets or unsupported files through an apparently safe parent operation.

Alternative considered: grant model tools direct filesystem access after one global enable. Concrete plan approval plus host validation avoids interpreting a language-model response as permission.

### 8. Filesystem identity, Git coordination, and recoverable batches

Create a root-app executor with shared admission/helpers extracted from existing workspace operations. Before applying, preflight the complete selected dependency graph, capture/compare dirty-buffer versions and disk identities, acquire source/destination Git write admissions, and stop conflicting app saves/reloads for affected paths until mutation/reconciliation settles. Dirty text edits apply to buffers and ordinary Undo. Dirty moves/renames require Save First; no implicit save. Open clean paths remap through existing helpers.

Do not rely on string-prefix path checks. Normalize paths, reject unsafe lexical components/Windows devices/ADS/trailing dot-space aliases, resolve the root and nearest existing destination ancestor, inspect reparse/symlink/hard-link aliases, and compare actual identities. Reject reparse-containing mutation paths rather than follow an ambiguous chain. Revalidate immediately before I/O and use handle/identity checks where the platform offers them; fail if containment cannot be proven. No overwrite of an existing destination, and no directory cycles. Serialize application-owned writes and retain identity checks for external processes; arbitrary external writers cannot be made transactional by an app lock.

Persist an AI action journal outside the workspace before any persistent mutation. Each record has schema version, workspace identity, operation ID/order, original paths/bytes/identities, expected post-state identities and exact bytes, and prepared/applied/reconciled/restored state. Use atomic storage and unique destination creation. For each operation: durable preparation -> filesystem mutation -> durable completion -> checked tab/tree reconciliation. Stop on failure or cancellation, retaining accurate partial results. Keep a persisted-operation prefix before in-memory edits when dependencies allow; rollback never quietly reverses a newer buffer edit.

No cross-file atomicity is promised. File permissions are preserved by atomic replacement and move primitives rather than serialized as platform-specific permission objects. Text writes use existing atomic replacement/permission preservation; directory creation and rename/move have checked postconditions. Preparation/backups must fit a finite journal storage budget (initially 50 MiB); unresolved recovery is never evicted to meet a cap. A batch exceeding it is rejected before changes. An incomplete operation at startup is reconciled against actual source/destination identities before Resume/Restore is offered.

Restore checks recorded postconditions, takes fresh write admissions, and reverses persistent operations in dependency order. Subsequent edits block automatic restore and remain available with conflict feedback. New notes/files are removed only if their identities and bytes still match the generated state; created folders are removed only if empty and still match their record. Buffer edits use Undo, and the plan UI clearly distinguishes that from Restore File Actions. Disabling AI cancels the queue, not an in-progress filesystem primitive; recovery UI remains available even with AI disabled.

Alternative considered: rely on document Undo or a Git commit for moves. Neither protects users without Git, closed-file replacements, or interrupted filesystem operations; a separate bounded journal provides appropriate recovery.

### 9. Optional history and predictable keyboard behavior

Default conversations stay in memory. Opt-in local history stores visible messages atomically in the app data directory, without keys/raw attachment snapshots/tool-read payloads/read grants/approvals. Opaque provider replay metadata stays in the live session; restored conversations build a fresh visible-message context rather than replay an incomplete native tool exchange. Start with a 20-conversation/8-MiB cap; evict only ordinary saved conversation history with an explicit retention description, never action recovery. Delete/Clear operations remain usable when AI/history is disabled. Restored proposals are descriptive only; regeneration or fresh review constructs current executable plans.

Use an independent focusable composer with its own source buffer, selection, undo, and IME marked range. Adapt the existing search/text-input routing carefully; do not reuse document mutations. Enter submits only outside composition, Shift+Enter inserts a newline, and document shortcuts do not capture composer editing. `toggle-ai-panel` uses the currently unused Ctrl/Cmd+Shift+A default via the existing registry. When disabled it opens setup; configured AI switches focus deliberately. The tab strip/panel supports scrolling/wrapping at 125%/150% scale.

Alternative considered: persist full workspace context by default. Session-only history and explicit opt-in avoid copying note content into another storage surface without user choice.

## Risks / Trade-offs

- [Compatible endpoints differ in optional features and model limits] -> Use explicit capability probes, manual model entry, protocol fixture tests, and text-only fallback; do not promise Agent support for every model.
- [An approved plan becomes stale while the user edits or Git reconciles] -> Check request/workspace epochs, source versions/bytes, and disk identities at review and execution; return useful Regenerate/Save First feedback.
- [Filesystem mutations fail partway or the app crashes] -> Prepare durable journals first, record partial outcomes, reconcile startup state, and restore only when post-state still matches.
- [Credential stores may be locked or unavailable] -> Reuse platform APIs with an injected abstraction and explicit session-only fallback; never write plaintext secrets.
- [Long answers/retrieval would consume memory or interrupt typing] -> Finite transport/context/plan/history budgets, background work, bounded channels, coalesced repaint, and cached/virtualized AI presentation.
- [Relative links cannot be comprehensively checked under narrow scope] -> Review scoped link edits and list unchecked scope; obtain a wider read grant only when the user chooses it.
- [Conversation text can itself include user-provided secrets] -> Display the selected context before sending, never inject saved API keys, avoid content logs, and make optional history retention/deletion clear; do not claim the UI can detect all authored secrets.
- [Multiple existing OpenSpec changes touch preferences and workspace operations] -> Preserve their code and deltas, append AI in place, and resolve overlapping artifacts explicitly during implementation/archive rather than silently folding unrelated work into this change.

## Migration Plan

1. Implement/test the GUI-free core and additive preferences with AI disabled by default; keep package versions unchanged.
2. Integrate secure setup/transport, then conversation/writing, then bounded retrieval/review and journaled actions. Each implementation task includes relevant targeted evidence before its checkbox is marked complete.
3. Run workspace tests/build, protocol and recovery fixtures, and desktop scenarios listed in `tasks.md`; review all delta requirements and validate the change before archive.
4. Existing configs gain no active request behavior. Do not migrate legacy keys automatically, scan workspaces on startup, or call cloud APIs in tests. Keys are entered explicitly through setup.
5. Rollback is disabling AI or reverting the additive implementation. User source files stay in ordinary formats. Preserve action journals/backups until explicitly resolved; document that downgrades cannot execute the new recovery flow and retain the newer build for restoration if needed. The planning change remains unarchived until implementation and acceptance are complete.
