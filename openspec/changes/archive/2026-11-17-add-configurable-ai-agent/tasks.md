## 1. Core boundaries and compatible configuration

- [x] 1.1 Add GUI-free `crates/ai` (`markion-ai`) with request/event/error, conversation identity, capability, and finite-limit models; wire the root dependency without version changes; verify `cargo test -p markion-ai` and inspect `cargo tree -p markion-ai` for no GUI dependencies.
- [x] 1.2 Add optional AI preferences/profile serialization in `src/model.rs` and `src/storage/preferences.rs`; verify old config defaults to AI disabled, valid values round-trip, malformed AI fields preserve unrelated settings, and reset preserves external credential/history/recovery data.
- [x] 1.3 Implement provider presets and endpoint/protocol normalization including loopback-only HTTP, invalid URL/credential rejection, limits, and manual model IDs; verify table-driven endpoint/default/validation fixtures including duplicate `/v1` prevention.
- [x] 1.4 Implement root credential-store abstraction using GPUI APIs, secret redaction, session-only fallback, endpoint-bound references, and a durable non-secret reference index; verify fake-store save/read/forget/unavailable/reset/endpoint-change cases and absence of keys in serialized settings or diagnostics.

## 2. Provider protocols and cancellable transport

- [x] 2.1 Implement incremental bounded SSE decoding independent of GUI state; verify fragmented UTF-8, CRLF, multiline data, keepalives, oversized frames, malformed input, and incomplete-stream fixtures.
- [x] 2.2 Implement the Responses adapter with client-managed replay, `store: false`, text/tool completion events, paired call IDs, usage/errors, and required opaque replay fields; verify mock fixtures for text, multiple tool calls, tool continuation, and missing terminal events.
- [x] 2.3 Implement the compatible Chat Completions adapter with optional-field capability handling, keyless local support, completed-response fallback, usage, and ordered tool replay; verify DeepSeek/Ollama-style mock fixtures and text-only-server behavior without external requests.
- [x] 2.4 Implement the Anthropic Messages adapter with content-block sequencing, tools/results, required replay metadata, usage and typed failures; verify text/tool/reasoning-block/error fixtures and correct continuation after a tool result.
- [x] 2.5 Add dedicated streaming HTTP transport on the existing runtime with finite connect/idle/turn/output limits, bounded channels, origin-bound credentials and redirect rejection; verify local mock-server streaming, mid-stream cancellation, timeout, oversized response, and redirect/header tests.
- [x] 2.6 Implement explicit synthetic connection/model discovery/tool probes and text-only fallback; verify success, no-tools, unsupported discovery, authentication, missing model, rate-limit and unavailable-local states, with disabled AI producing no probe requests.

## 3. Settings, lifecycle, and localized entry points

- [x] 3.1 Add exhaustive `AiMsg` translations for all seven UI languages and localized AI preset/action/reference catalog entries; verify the existing completeness mechanism catches a missing entry and all new messages return non-empty strings.
- [x] 3.2 Add the AI Preferences tab after Export while preserving Images and existing tab ownership, basic draft controls, masked key, model, explicit test/discovery, Save and advanced disclosure; verify disabled setup, successful setup, invalid drafts, keyboard navigation, and scroll/tab access at constrained sizes.
- [x] 3.3 Add multiple-profile management, advanced endpoint/limits/guidance/history controls, session-key state and explicit Forget Key/retained-key management; verify profile switching preserves draft/stored values and reset leaves retained data discoverable without exposing secrets.
- [x] 3.4 Implement the master enable/disable gate, configuration/workspace generations, cancellation and queue invalidation; verify zero AI network/tools while disabled, late-event rejection, grant revocation, preserved ordinary editing and safe settlement of an already admitted file operation.
- [x] 3.5 Register `toggle-ai-panel` and Ctrl/Cmd+Shift+A through actions, native/in-app menu entry, keymap, and customizable references; verify disabled action opens AI setup, conflict checks/live rebind work, and the existing fixed editor/file-tree keys remain functional.

## 4. Conversation panel, composer, context, and history

- [x] 4.1 Implement conversation/request state with one active turn per conversation, app-wide concurrency bounds, explicit retries, incomplete output and identity-scoped events; verify switching/clearing/retrying conversations cannot mix results or replay an applied operation.
- [x] 4.2 Implement an independent focusable multiline composer with selection, paste, undo and IME routing; verify Chinese composition Enter does not submit, Shift+Enter inserts a newline, and composer editing never changes the active document or its undo state.
- [x] 4.3 Build the resizable panel, safe cached Markdown messages, copy/new/follow-up/Stop controls, virtualized history and narrow-window fallback; verify incremental output, preserved manual scroll, restored focus, and visible running/Stop status after closing the panel.
- [x] 4.4 Implement explicit Selection/Document/File attachments and immutable dirty-buffer snapshots with size/scope presentation and input budgets; verify blank chat sends no document, selection-only sends only that range, missing files are actionable, and oversized attachments block sending without silent clipping.
- [x] 4.5 Implement conversation/workspace read-grant selection, visible scope and revocation; verify denied out-of-scope reads, no restoration of grants after restart, and fresh context after revocation/workspace/conversation changes.
- [x] 4.6 Add verified source references backed only by actual attachments/tool reads; verify clicking opens the supplied file/range, invented paths are unverified, and missing/moved files cannot broaden scope or open a guessed substitute.
- [x] 4.7 Add session-only default and opt-in bounded atomic local history with open/delete/clear actions; verify restart behavior, caps, corrupt-entry isolation, absence of raw attachments/tool-read payloads/keys/grants/approvals, and history deletion preserving action recovery.
- [x] 4.8 Verify event coalescing and AI message cache isolation under a long mocked stream; record that source versions, document `Arc` caches, syntax memoization, editor text handles and undo remain unchanged until application, and that typing stays responsive.

## 5. Writing actions and checked application

- [x] 5.1 Implement draft/continue/polish/correct/shorten/expand/tone/translate/summarize/outline/tasks/custom templates and target language/tone/guidance controls; verify prompt scopes and Markdown preservation instructions, plus UI-independent translation target behavior.
- [x] 5.2 Wire writing actions into the AI panel and source/Visual Edit/Read selection context, with explicit empty-selection/preceding-context choice and analysis-only unmapped ranges; verify CJK/emoji/CRLF source mapping and disabled actions on read-only/image surfaces.
- [x] 5.3 Add completed result review with exact source target, before/after preview, copy/replace/insert/new-note/discard/regenerate destinations; verify generation/discard does not modify the document and incomplete output has Copy but no Apply.
- [x] 5.4 Add checked `AiWriting` mutation application and atomic undo grouping using original document identity/version/source bytes and captured caret; verify exact undo/redo/selection restoration and a separate preceding typing group.
- [x] 5.5 Handle edited/closed/reopened/remapped/locked targets and inactive original tabs; verify stale application preserves current content, no result applies to another tab/current caret, and Regenerate/Copy/New Note remain useful.

## 6. Retrieval, Agent loop, and concrete change review

- [x] 6.1 Implement supported-file and protected-secret policies plus filesystem identity/containment validation for existing paths and new ancestors; verify traversal, sibling-prefix roots, symlinks, Windows junctions/reparse points, case aliases, hard-link ambiguity, ADS/device/trailing-dot-space names, and destination collisions with platform-appropriate fixtures.
- [x] 6.2 Implement asynchronous scoped listing, filename/text search, and bounded range reads with pagination and current-buffer preference; verify large workspace pagination, denied secret/binary/ignored paths, dirty-buffer content and no document-cache invalidation.
- [x] 6.3 Implement the bounded Agent loop, validated complete tool arguments, typed host-tool results, tool-support gating, limits and cancellation; verify malformed/unknown tools, partial JSON, repeated failures, prompt instructions embedded in files, revoked scope and exhausted budgets cannot trigger unauthorized work.
- [x] 6.4 Implement typed proposals for note/folder creation, text edits and file move/rename with captured baselines, dependencies, postconditions and merged non-overlapping edits per file; verify proposals alone perform no mutation and overlapping/unsafe plans are rejected.
- [x] 6.5 Add scoped relative Markdown-link consequence analysis and optional link-edit proposals for reviewed moves/renames; verify relative links/images/reference definitions on permitted sources and visible unchecked references without scanning outside approved scope.
- [x] 6.6 Build batch review with per-operation selection, destination/name editing, before/after text diffs, dependencies and explicit Apply/Reject; verify changed plans require fresh validation, prerequisites cannot be silently deselected, and folder moves cannot conceal unsupported/unapproved descendants.

## 7. Writer admission, execution, and durable recovery

- [x] 7.1 Extract/reuse host workspace mutation helpers and integrate AI preflight with current disk/buffer/workspace identities and Git source/destination admissions; verify dirty moves need Save First and existing interactive create/rename/move behavior remains unchanged.
- [x] 7.2 Coordinate affected saves/autosave/external reads during an AI batch and apply reviewed open-buffer edits through checked mutations; verify stale disk/versions, Git locks, in-flight autosaves and external reloads cannot overwrite AI/user content.
- [x] 7.3 Implement versioned atomic action journals/backups outside the workspace with prepared/applied/reconciled states and finite storage limits; verify failed preparation makes no changes, keys are absent, unresolved journals are retained, and oversized plans fail before mutation.
- [x] 7.4 Execute selected persistent note/folder creates, atomic text writes and checked moves/renames with per-operation durable completion, tab remapping and bounded tree refresh; verify expected bytes/permissions, no collision overwrite, and only affected tabs/caches change.
- [x] 7.5 Implement failure/cancellation boundaries, accurate applied/failed/skipped/unapplied outcomes and non-replaying retries; verify injected failure after a successful operation, cancel/disable mid-batch, and dependent-operation stop behavior in real temporary workspaces.
- [x] 7.6 Implement reviewed Restore File Actions with fresh post-state/buffer/admission checks and reverse dependency order; verify round-trip creates/edits/moves, empty-folder rules, newer-content conflict refusal and clear distinction from buffer Undo.
- [x] 7.7 Inventory/reconcile interrupted journals on startup and provide Resume/Restore/retire recovery management even with AI disabled; verify crashes before/after mutation/completion recording, moved/replaced workspace identity, unknown journal versions, and reset/history deletion preserving unresolved recovery.

## 8. Integration acceptance and documentation

- [x] 8.1 Deliver a user guide in English and Simplified Chinese covering cloud/local setup, keys/model capability, disabled behavior, writing, explicit context, file review/restore, history, and troubleshooting; verify links from both READMEs and complete reproducible setup examples without real credentials or pinned availability claims.
- [x] 8.2 Run a deterministic local mock-provider desktop acceptance covering first setup, streaming, follow-ups, Stop, Chinese IME, selection rewrite/undo, text-only fallback, scoped multi-note synthesis and file organization/restore; record evidence and any actual limitations in an acceptance document.
- [x] 8.3 Verify all seven languages, supported themes, narrow windows and Windows 125%/150% display scale across settings/panel/review/recovery; record that labels/controls remain reachable and document/composer focus is preserved.
- [x] 8.4 Run `cargo fmt --all -- --check`, targeted AI/root mutation/persistence/recovery checks, `cargo test --workspace`, and `cargo build`; record results and investigate regressions without silently narrowing any specified behavior.
- [x] 8.5 Reconcile proposal/design/deltas with the finished behavior, verify every scenario's evidence, and run `openspec validate add-configurable-ai-agent --strict`; mark all tasks complete only when implemented and accepted, then leave the change ready for the separately requested archive workflow.
