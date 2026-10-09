# AI Agent acceptance evidence

Change: `add-configurable-ai-agent`. Tested on Windows on 2026-10-09, with synthetic notes and a loopback mock provider. No user API key, paid service request, or real note was used. Package versions remain 0.4.7.

## Reproduce

1. Run `python scripts/ai-mock-provider.py --port 18114`.
2. Create a temporary workspace containing `a.md` (`Alpha 中文😀` plus a second paragraph) and `b.md` (`Beta` plus a second paragraph).
3. Open Preferences → AI Agent. Enable AI, choose Custom, enter base URL `http://127.0.0.1:18114/v1`, protocol `chat`, and model `mock-agent`; leave the key blank. Test and Save.
4. Find models explicitly. `mock-text` passes text testing and reports that tools are unavailable. Restore `mock-agent` and test before using workspace tools.
5. Send a Chinese prompt, a follow-up, and a prompt containing `slow`. Stop the slow turn; its partial output remains incomplete and copyable.
6. Select a source-backed paragraph, choose Polish, inspect the exact before/after range, Replace, then Undo. No source changes before application.
7. In a fresh conversation, grant only the temporary workspace folder and enable workspace tools. Send `synthesize` to read the first four bytes of each synthetic note. Send `organize` to propose `organized/` and moving `a.md` to `organized/a.md`. Inspect and Apply the selected plan.
8. Inspect Restore File Actions. Restore validates post-state; its deterministic temporary-workspace fixtures also exercise restart, conflict, cancellation and interrupted recovery.

The provider fragments UTF-8 SSE bytes, implements explicit model discovery, rejects tools for `mock-text`, and emits deterministic read/proposal tool calls. It does not log request bodies or credentials.

## Native desktop observations

The built Markion executable used separate temporary APPDATA/LOCALAPPDATA directories and two synthetic notes. The native window was 1182 × 792 pixels, with Simplified Chinese interface and Paper theme.

- With no AI configuration, Ctrl+Shift+A opened the AI settings tab while the document remained unchanged.
- Local/Ollama setup with `mock-agent` and a blank key passed text and tool probing. Save persisted the profile; restarting preserved setup and did not restore session-only conversations or grants.
- Explicit discovery displayed `mock-agent` and `mock-text`. Testing `mock-text` displayed the localized text-only state; retesting `mock-agent` restored tool availability.
- Chinese composer input stayed in the AI input. Enter started a streamed Markdown answer. A slow follow-up exposed Stop and retained incomplete output after stopping. Manual message scroll remained at its existing position.
- During a second slow turn, Ctrl+Shift+A closed the panel while the status bar continued displaying localized running state. Clicking that indicator reopened the active conversation with Stop visible; Stop canceled the turn and left the current note unchanged.
- Drag-selecting the first Visual Edit paragraph captured the exact source range `0..12`. Polish left the source unchanged until Replace. Replacement affected that range while preserving the emoji and second paragraph outside it. A single editor Undo restored original text and selection.
- Writing actions, conversation tabs, review controls and recovery surfaces were given explicit bounded scroll heights after native inspection found a zero-height overflow container. The rebuilt native window displayed all twelve writing actions and reachable before/after/destination controls.
- After a fresh folder grant, `synthesize` completed two tool reads and displayed verified `a.md`/`b.md` byte ranges. Clicking `b.md` opened that exact tab and selected `Beta` (`0..4`).
- `organize` proposed folder `organized` and move `a.md → organized/a.md`, with dependency `[0]`, editable destination and before/after previews. Both files remained in place before Apply. Apply created the folder and moved the note without changing its bytes or the unrelated active `b.md`. The file tree refreshed; the durable native journal contained two `Applied` records. Recovery displayed the exact scope, paths, outcomes and Restore/Resume/retire controls. Actual Restore execution is covered by the GPUI and real-workspace fixtures below.

## Deterministic integration and scenario coverage

Tests live in `crates/ai/src/`, `src/ai_actions.rs`, `src/ai_credentials.rs`, `src/storage/preferences.rs`, and `src/app/ai_panel.rs`. Existing editor, mapping, shortcut and Git suites run with them.

| OpenSpec area and scenarios | Evidence |
| --- | --- |
| Configuration: absent/malformed settings, local/cloud presets, invalid drafts, profile changes, reset | Preference migration tests; config preset/URL/finite-limit fixtures; disabled-shortcut and preserved-profile-draft GPUI tests; native setup and restart |
| Credentials: unavailable store, forget, endpoint change, redirects | Injected credential-store tests, retained reference index, corrupt index refusal with session fallback; origin header and redirect transport tests; secret Debug/serialization checks |
| Capabilities: text-only model, connection failures, discovery | Native mock-agent/mock-text probes; bounded local transport fixtures covering authentication, model/route missing, throttling, local unavailable, unsupported features and discovery |
| Conversation: follow-ups, Stop, delayed events, incomplete output, no default context | Identity-gated conversation tests; 10,000-delta root test; native Chinese chat/follow-up/Stop; bounded explicit-context fixtures |
| Composer: IME Enter, multiline paste/undo, shortcut isolation | GPUI EntityInputHandler tests inject Chinese/emoji marked UTF-16 ranges; contextual key tests exercise Shift+Enter, selection/undo and blocked document formatting; native Unicode input is additional evidence, not an OS IME automation claim |
| Context: selection/document/file, budget overflow, dirty buffers, subtree grant/revocation | Immutable snapshot/range tests; context limits reject rather than clip; scoped workspace/read tests; generation cancellation and grant revocation tests |
| Sources: multiple notes, fabricated/missing/moved sources | Actual attachment/read provenance only; native scoped synthesis; unsaved-source document identity test; missing/replaced paths fail identity checks; model Markdown has no automatic path-link authority |
| Agent: complete arguments, unknown tools, budgets, embedded instructions | Mock native paired tool results; malformed/incomplete/unknown tools and repeated errors stop; embedded file instructions cannot execute; canceled host calls and exhausted budgets stop without mutation |
| Writing: all presets, explicit language/tone, empty scope, Visual/Read mapping | Twelve pure prompt fixtures; explicit Whole document/Before caret scope controls; existing CJK/emoji/CRLF canonical mapping tests; analysis-only unmapped/Read results; native selected Visual Edit rewrite |
| Writing review: discard, new note, captured caret, undo, stale/inactive/closed target | Original-target GPUI checked mutation/Undo/Redo tests; separate prior typing group; stale and closed target tests; native review and one-step Undo; incomplete results have no Apply |
| Retrieval/path policy: dirty reads, pagination, junctions, collisions, unsupported work | Large listing/dense search continuation tests; actual Windows junction and hard-link fixtures; traversal, sibling root, ADS/device/case/trailing-dot and protected-file tests; unknown tools fail closed; existing directory moves excluded |
| Plans: organization, optional relative link consequences, revised names, prerequisites | Pure proposal/no-mutation tests; fresh edited destination/dependency/baseline test; inline/image/reference links with fragments/titles and code exclusion; journal linked-move round trip; outside-scope references remain unchecked |
| Writer coordination: dirty moves, disk changes, autosave, Git admissions | Root dirty-move/Save First, in-flight autosave and Git path-claim refusals; checked buffer batch preserves unrelated document/disk; affected-path save/reload locks; existing interactive workspace/Git regression suite; canceled resumed batch retains the applied prefix and refuses replay |
| Durable application: preparation failure, collision/partial failure, no replay | Six real temporary-workspace journal tests: budget before mutation, duplicate destinations, completed-prefix retention, no overwrite, prepared/post-state reconciliation and no replay; explicit buffer Undo vs persistent Restore |
| Recovery: unchanged move, newer edit conflict, startup interruption/reset/history deletion | Reverse-order create/edit/move and linked-move fixtures; interrupted Restore reconciliation; unknown version retention; AI-disabled root restore; queued history clear preserves recovery files |
| History: session-only restart, saved reopen, clear, caps/corrupt entries | Native session-only restart; bounded 20-conversation/8-MiB history tests, corrupt-entry isolation, fresh restored context without grants/wire/approvals; serialized history excludes raw attachments/tool payloads/keys; atomic queued-clear race test |
| Shortcuts/preferences/architecture | Default AI registry/menu binding plus generic live override/conflict tests; disabled shortcut opens setup; existing Images/Appearance/General ownership tests; AI crate dependency graph has no GPUI/toolkit dependency |
| Localization and layout | Exhaustive seven-column catalog and nonempty translation test; 84 GPUI layout combinations: 7 languages × 6 themes × scale factors 1.25/1.5, at 560 × 420 logical pixels; settings/panel/writing review/recovery and positive scroll-container height assertions |

The DPI checks simulate Windows GPUI device scale and resize callbacks; they do not change the user's global Windows display setting. Languages: English, Simplified/Traditional Chinese, Japanese, French, German, Spanish. Themes: Paper, Ink, Solar, Forest, Rose, Graphite. Existing document/input focus and undo tests run alongside the layout matrix.

## Verification and limits

The full workspace suite passed with four test threads (1933 tests passed, 5 existing tests ignored). An earlier unlimited-thread run encountered a transient existing Git subprocess initialization failure; the four-thread complete run passed without changing or skipping that test. Targeted AI core tests passed (33), as did journal recovery tests (6) and root AI integration tests. Formatting, final build and strict OpenSpec validation are recorded with the final verification below.

Final verification on 2026-10-09:

- `cargo fmt --all -- --check` — passed.
- `rustfmt --check --edition 2024 vendor/zed/crates/gpui/src/app/test_context.rs vendor/zed/crates/gpui/src/platform/test/window.rs` — passed.
- `cargo test -p markion-ai -- --test-threads=4` — 33 passed.
- `cargo test --lib ai_credentials -- --test-threads=4` — 3 passed; `cargo test --lib ai_profiles -- --test-threads=4` — 1 passed.
- `cargo test --lib ai_actions::tests -- --test-threads=4` — 6 passed.
- `cargo test --bin markion ai_panel::tests -- --test-threads=4` — 13 passed.
- `cargo test --workspace -- --test-threads=4` — 1933 passed, 5 ignored.
- `cargo build` — passed.
- `cargo tree -p markion-ai` — no `gpui`, `gtk`, or `winit` dependency.
- `openspec validate add-configurable-ai-agent --strict` — passed.

Protocol tests use mocks, including native Responses/Anthropic replay metadata. They do not certify current cloud model availability or every third-party compatible server. Local model installation and account/key provisioning remain user-owned. Workspace operations require supported text files, verified tools and explicit read scope; shell, permanent deletion, arbitrary OS/browser actions and background indexing are outside the approved feature. Restore is conditional on unchanged recorded post-state; no cross-file transaction is promised.
