# AI Agent

AI starts disabled. Open **Preferences → AI Agent**, or press **Ctrl+Shift+A** (Windows/Linux) / **Cmd+Shift+A** (macOS). You supply your own service and API key; Markion has no bundled subscription.

## Setup

Choose OpenAI, Anthropic, DeepSeek, Local/Ollama, or Custom. Enter the model ID supported by your service and an API key if required. **Find models** is optional; you can always type a model ID. **Test connection** sends a synthetic text request and a harmless tool probe, without document context. Save the profile when it works.

Presets use the service's base URL and protocol. Custom and Local/Ollama presets show the base URL and protocol (`responses`, `chat`, `anthropic`) directly in basic setup; for cloud presets they sit under Advanced, which also holds the finite request limits and writing guidance. Writing guidance saves with its own button and survives preset and profile switches. Use a base URL such as `https://api.example.com/v1`, without a `/chat/completions`, `/responses`, or `/messages` suffix. HTTP is allowed only for loopback servers. Redirects and credentials embedded in URLs are rejected.

For Ollama, start your local server, install a model you intend to use, choose Local/Ollama, and enter its model ID. The default base URL is `http://localhost:11434/v1`. Local/custom servers can be keyless. Their capabilities vary; text chat remains available when tool support is absent. Workspace tools require a successful tool probe for the exact saved endpoint, protocol and model.

Keys use the operating system's secure credential store. A blank key field retains the existing key for that profile and endpoint. **Use key for this session only** is an explicit alternative if secure storage is unavailable. Keys are never saved in preferences or conversation history. Advanced settings list retained keys, including references left after deleting a profile or resetting preferences; **Forget key** removes the corresponding credential. Changing endpoints does not reuse another endpoint's key.

## Conversation and writing

The AI panel has its own multiline input and Undo. Enter sends; Shift+Enter inserts a line break. Chinese IME composition Enter does not send an unfinished composition. **Stop** cancels generation. Closing the panel leaves a running indicator in the status bar; reopen it to stop or review the result.

A blank chat has no document context. Attach **Selection**, **Document**, or an explicitly chosen **File**. Attachment chips show the material and size; remove a chip before sending if it should not be included. Attachments capture unsaved buffer text. Oversized context blocks sending, instead of silently clipping it. Verified source buttons come only from actual attachments and tool reads; model-generated paths do not become verified links.

Writing actions include Draft, Continue, Polish, Correct grammar, Shorten, Expand, Change tone, Translate, Summarize, Outline, Extract tasks, and Custom instruction. Enter target language/tone independently of interface language. Without a selection, choose Whole document or Text before caret explicitly. Read view selections are analysis-only.

Generation does not edit the document. Review Before/After, then Copy, Replace captured text, Insert at captured caret, or create a New unsaved note. Incomplete output can be copied or retried, but cannot be applied. Replacement validates the original document instance, version, path and source bytes. A changed or closed target cannot redirect the result to another tab. Applied writing is one Undo group, separate from preceding typing. Persistent file actions use Restore instead of editor Undo.

## Workspace operations

Choose **Allow workspace reads…** and select a folder inside the open workspace. The visible grant is local to the conversation and session. It is never restored from history. **Revoke access**, changing workspace, or disabling AI cancels outstanding work and invalidates proposals.

**Use workspace tools** enables bounded listing, filename/text search and exact range reads, plus proposals for text edits, notes, folders and individual file moves/renames. Supported content is `.md`, `.markdown`, `.mdown`, and `.txt`. Hidden/protected folders, likely secret files, binaries, traversal, unsafe Windows names, symlinks/junctions and hard-link ambiguity are rejected. The agent has no shell, external OS tool, publishing, Git mutation, or permanent-delete tool.

Review each operation, Before/After text, names/destinations and dependencies. Revise a destination and update the review before applying. A selected dependent operation requires its prerequisite. **Review link updates** analyzes only the granted scope and offers incoming-link edits as separately selectable proposals; moved-note link changes appear in its diff. References outside the grant are visibly unchecked. Dirty file moves require Save First. Dirty/open-buffer edits are checked mutations and use editor Undo; disk changes have durable recovery records.

File actions save durable preparation and completion records outside the workspace. Partial failure retains completed operations and labels the rest; retry never automatically replays a completed item. Turning AI off stops unstarted work while an admitted filesystem primitive settles safely.

## Recovery and history

**Restore file actions** is available in AI settings even when AI is off. Inspect each affected path before Restore, Resume reviewed actions, or retiring a resolved record. Restore refuses to overwrite later edits, replaced workspaces, collisions, or nonempty created folders. Recovery records survive preferences reset and conversation-history deletion. Interrupted or ambiguous records remain retained for review.

Conversations stay in memory by default. Optional local history stores visible messages, with a 20-conversation / 8 MiB cap, excluding raw attachments, tool-read payloads, keys, read grants and executable approvals. Restored chats start with fresh context. Disable persistence to return to session-only operation; existing saved history remains explicitly deletable.

## Troubleshooting and reproducible local testing

- Authentication failure: check the endpoint-specific key.
- Model/endpoint not found: type a valid model ID and verify the base URL/protocol.
- Local unavailable: start the model server and confirm its loopback port.
- Unsupported discovery/tools: enter the model ID manually or use text chat.
- Context limit: remove attachments, use a smaller scope, or start a new conversation.
- Stale result / file conflict: save current work and regenerate a fresh plan; Copy and New unsaved note remain available.

For a deterministic keyless test, run `python scripts/ai-mock-provider.py --port 18114`. Create a Custom profile with base URL `http://127.0.0.1:18114/v1`, protocol `chat`, model `mock-agent`. Use only synthetic fixtures. `mock-text` exercises text-only fallback; a prompt containing `slow` exercises Stop. Synthetic files `a.md` and `b.md` can exercise `synthesize` and `organize` prompts under an explicit temporary-folder grant.

See the [Simplified Chinese guide](ai-agent-guide.zh-CN.md) and [acceptance evidence](ai-agent-acceptance.md).
