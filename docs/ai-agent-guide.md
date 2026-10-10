# AI Agent

AI starts disabled. Open **Preferences → AI Agent**, or press **Ctrl+Shift+A** (Windows/Linux) / **Cmd+Shift+A** (macOS). You supply your own service and API key; Markion has no bundled subscription.

## Setup

Choose OpenAI, Anthropic, DeepSeek, Local/Ollama, or Custom. Enter an API key if required. **Find models** works before entering a model ID; its bounded list can be filtered. You can also type the model ID directly. **Test connection** tests the complete draft using synthetic text and a harmless tool probe, without document context. **Save and enable** saves that draft before enabling AI. A failed save retains the draft and does not enable AI; a changed endpoint/protocol/model requires fresh tool verification.

Presets use the service's base URL and protocol. Custom and Local/Ollama show the base URL and protocol (`responses`, `chat`, `anthropic`) in basic setup; cloud presets keep them under Advanced. Advanced contains finite request limits, with KiB equivalents for byte limits. Writing preferences and History and privacy are separate expandable sections. Writing guidance saves independently and survives preset/profile switches. Save controls remain at the bottom while settings scroll. Use a base URL such as `https://api.example.com/v1`, without `/chat/completions`, `/responses`, or `/messages`. HTTP is allowed only for loopback servers. Redirects and URL credentials are rejected.

For Ollama, start your local server, install a model you intend to use, choose Local/Ollama, and enter its model ID. The default base URL is `http://localhost:11434/v1`. Local/custom servers can be keyless. Their capabilities vary; text chat remains available when tool support is absent. Workspace tools require a successful tool probe for the exact saved endpoint, protocol and model.

Keys use the operating system's secure credential store. A blank key field retains the existing key for that profile and endpoint. **Use key for this session only** is an explicit alternative. Keys never enter preferences or history. A registered secure credential reference is not proof that the store is currently accessible: Test connection confirms availability. History and privacy lists retained references, including those left after profile removal/reset; **Forget key** removes the credential. Changing endpoints does not reuse another endpoint's key.

## Conversation and writing

The panel has a fixed header/composer and a flexible message region. New conversation, history, settings, copy, close and send/stop use compact icon controls with tooltips. Click the active service/model to switch a saved profile; unsaved settings remain guarded. Unsent composer drafts survive conversation switches in memory. Input wraps at the panel width and has independent Undo. Enter sends; Shift+Enter inserts a newline; IME composition Enter does not send unfinished text. Empty input and unavailable workspace setup disable Send. Closing the panel leaves a running status indicator; reopen it to stop or review.

A blank chat has no document context. Attach **Selection**, **Document**, or an explicitly chosen **File**. Pending chips identify file/range/size and can be removed before sending. **Previously referenced** sources are separate from pending attachments. Attachments capture unsaved buffer text; oversized context blocks rather than silently truncates sending. Verified source links come only from actual attachments/tool reads, never model-invented paths.

Expand **Writing** to choose Draft, Continue, Polish, Correct grammar, Shorten, Expand, Change tone, Translate, Summarize, Outline, Extract tasks, or Custom instruction, then Generate. Only Translate displays target-language controls; only Change tone displays tone input. Missing parameters give dedicated feedback before contacting the service. Target language/tone are independent of interface language. Without a selection, choose Whole document or Text before caret explicitly. Read selections are analysis-only.

Generation does not edit the document. Review the captured target and bounded additions/removals summary, plus Before/After text, then Copy, Replace, Insert, or create a New unsaved note. The summary describes the changed span, not a minimal full-file diff. Incomplete output cannot be applied. Replacement validates the original document instance, version, path and source. Changed/closed targets never redirect to another tab. Applied writing is one Undo group, separate from earlier typing. Persistent file actions use Restore, not editor Undo.

## Attachment sizes and input budget

The default **Input bytes** budget is 65536 B (64 KiB), shared by attachments, the prompt, conversation history, system guidance and serialized message overhead. It counts UTF-8 bytes, not characters or tokens; most Chinese characters occupy three bytes. Local input-limit feedback identifies the rejected attachment and its size when known, shows the saved budget, and opens the corresponding Advanced setting. Generation/output limits remain separate.

For a larger document, open **Preferences → AI Agent → Advanced**, set **Input bytes** to `262144` (256 KiB) or `524288` (512 KiB), and **Save** the profile. The maximum is `1048576` (1 MiB); defaults and saved profiles are not raised automatically. Leave room for the other request content. Increasing Output bytes or Output tokens does not fix input overflow. Provider context limits still apply, and larger requests can increase cost and latency.

Whole-file disk reads also have a 1 MiB ceiling. For a larger file, open it in the editor, select the relevant section and add **Selection**; process sections explicitly rather than uploading the entire file. Starting a new conversation reduces history, but cannot make an individually oversized attachment fit. Markion never silently truncates attachments or sends them automatically.

## Workspace operations

Select **Workspace** mode, verify tool support in AI settings, then choose **Allow workspace reads…** for a folder inside the open workspace. Missing verification or authorization is shown before sending. Grants belong to the conversation/session and are never restored from history. **Revoke access**, changing workspace or disabling AI cancels work and invalidates proposals.

Workspace mode enables bounded listing, filename/text search and exact range reads, plus proposals for text edits, notes, folders and individual file moves/renames. Supported content is `.md`, `.markdown`, `.mdown`, and `.txt`. Hidden/protected folders, likely secret files, binaries, traversal, unsafe Windows names, symlinks/junctions and hard-link ambiguity are rejected. The agent has no shell, external OS tool, publishing, Git mutation, or permanent-delete tool.

Review each operation, Before/After text, names/destinations and dependencies. Revise a destination and update the review before applying. A selected dependent operation requires its prerequisite. **Review link updates** analyzes only the granted scope and offers incoming-link edits as separately selectable proposals; moved-note link changes appear in its diff. References outside the grant are visibly unchecked. Dirty file moves require Save First. Dirty/open-buffer edits are checked mutations and use editor Undo; disk changes have durable recovery records.

File actions save durable preparation and completion records outside the workspace. Partial failure retains completed operations and labels the rest; retry never automatically replays a completed item. Turning AI off stops unstarted work while an admitted filesystem primitive settles safely.

## Recovery and history

**Restore file actions** is available in AI settings even when AI is off. Inspect each affected path before Restore, Resume reviewed actions, or retiring a resolved record. Restore refuses to overwrite later edits, replaced workspaces, collisions, or nonempty created folders. Recovery records survive preferences reset and conversation-history deletion. Interrupted or ambiguous records remain retained for review.

Conversations stay in memory by default; the header displays the actual persistence mode. Optional local history stores visible messages with a 20-conversation / 8 MiB cap, excluding raw attachments, tool payloads, keys, grants and approvals. Restored chats need fresh context. Disable persistence to return to session-only mode. Deleting a conversation and clearing all history require confirmation; recovery records remain untouched.

## Troubleshooting and reproducible local testing

- Authentication failure: check the endpoint-specific key.
- Model/endpoint not found: type a valid model ID and verify the base URL/protocol.
- Local unavailable: start the model server and confirm its loopback port.
- Unsupported discovery/tools: enter the model ID manually or use text chat.
- Input limit: raise the saved Input bytes budget within its finite range, remove attachments, use a smaller selection, or start a new conversation to reduce history. See Attachment sizes and input budget above.
- Stale result / file conflict: save current work and regenerate a fresh plan; Copy and New unsaved note remain available.

For a deterministic keyless test, run `python scripts/ai-mock-provider.py --port 18114`. Create a Custom profile with base URL `http://127.0.0.1:18114/v1`, protocol `chat`, model `mock-agent`. Use only synthetic fixtures. `mock-text` exercises text-only fallback; a prompt containing `slow` exercises Stop. Synthetic files `a.md` and `b.md` can exercise `synthesize` and `organize` prompts under an explicit temporary-folder grant.

See the [Simplified Chinese guide](ai-agent-guide.zh-CN.md) and [acceptance evidence](ai-agent-acceptance.md).
