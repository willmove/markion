# AI assistant workflow refinement / AI 助手体验优化验收

Date: 2026-10-10. Change: `refine-ai-assistant-workflows`.

## Delivered / 已实现

- Fixed header and composer, flexible messages, compact embedded Lucide actions with tooltips and keyboard activation. Writing controls are disclosed on demand; workspace permissions belong to Workspace mode. Empty/unavailable sending is visibly disabled and Stop remains available during work.
- Cached input-only soft wrapping uses GPUI text measurement. Source-byte navigation, mouse hit testing and UTF-16 IME geometry preserve CJK/emoji and explicit newlines without modifying document caches or undo.
- Discovery accepts a blank model. Model results are filterable and tied to profile/endpoint/protocol. Testing and saving still validate the complete draft. Explicit Save and enable persists the draft before enabling, keeps matching verification, and retains draft/disabled state on failure. In-flight key saving cannot silently overwrite newer draft edits.
- Service configuration, advanced limits, writing preferences and history/privacy are separated. The save footer remains outside the settings scroll. Byte limits display KiB equivalents. Credential labels distinguish session keys, registered secure references and absent keys, without claiming a reference proves OS-store access.
- Pending attachments show file/range/size separately from prior sources. The header shows actual history mode. Unsent conversation drafts remain in memory across switches. Single-conversation deletion and full history clearing require confirmation.
- Writing parameters appear only for the selected action; missing language/tone produces dedicated feedback without a provider request. Review shows the captured target and bounded added/removed span. Existing checked application, stale-target rejection and one-step Undo remain intact.
- Updated English and Simplified Chinese guides. No new dependencies, settings schema migration, release version change or stable-spec edit.

## Automated Acceptance / 自动化验收

GPUI tests cover draft discovery, persistence failure, save-and-enable identity, missing writing parameters, conversation drafts, deletion confirmation, flexible message height, fixed composer under scrolling, CJK/emoji wrapped input/IME, captured-range Undo and stale-target protection. The layout matrix covers seven interface languages, six themes and two simulated scale factors at 560 x 420 logical pixels, including fixed settings-footer visibility. The transport fixture explicitly discovers models with an empty model ID.

Temporary fixtures and synthetic probe content are used; no paid cloud request or real document upload is needed.

### Final Results / 最终结果

| Check | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Passed. |
| `git diff --check` | Passed; Git reports only informational Windows line-ending warnings. |
| `cargo build -q` | Passed. Local executable: `target/debug/markion.exe`; no installed application replacement or release publication. |
| `cargo test --bin markion app::ai_ -- --test-threads=4` | Passed: 42 AI UI/input tests, including the 84-combination layout matrix. |
| `cargo test --workspace -- --test-threads=4` with `CARGO_INCREMENTAL=0` | Passed, including doctests. The application binary suite reports 711 passed, 0 failed and 2 ignored; the AI core suite reports 33 passed, 0 failed. |
| `cargo clippy --workspace --message-format=json` | Passed with existing repository warnings. An additional `--all-targets -- -D warnings` probe fails on existing lint findings; that stricter probe is not claimed to pass. |
| `openspec validate refine-ai-assistant-workflows --strict` | Passed. |
| `openspec validate --all --strict --no-interactive` | Existing repository baseline fails: 58 valid, 35 invalid items. Unrelated specifications were not rewritten. |

The initial incremental workspace-test link failed with MSVC LNK2001/LNK2019/LNK1120 errors for generated internal LLVM symbols. Rebuilding without incremental reuse succeeded. No toolchain or Cargo configuration was changed. The specifically implicated `target/debug/incremental/markion-01r497gbh6y8v` cache was retained under the backup name `markion-01r497gbh6y8v-pre-rebuild-20261010`; a subsequent default incremental workspace run has not been revalidated. This is build-cache isolation, not a claim to have fixed the compiler's underlying cause.

## Native Smoke Check / 原生界面检查

After the final resume, Windows Computer Use became available. The exact local executable was selected using its returned process-backed window identity, not the installed release's window. Native screenshots and interactions confirmed:

- The compact header, Chat/Workspace modes, configured profile chip, fixed composer and disabled empty Send control render in the real Windows window.
- Settings scroll independently; the Save footer stays visible at the service/key fields and at the advanced/writing/history/recovery entries. No configuration or credential policy was changed or saved.
- Writing actions expand on demand. Selecting Translate reveals only the target-language controls and native-language quick choices.
- Synthetic Chinese, English and emoji input soft-wraps, grows to the bounded input viewport and keeps the final caret visible. Input Undo restores an empty composer while the document remains saved and unchanged.

No request was submitted, no document attachment was added, no workspace permission was granted, and no credential or privacy setting was changed. The synthetic input was undone. The local build was left open with the compact panel for inspection. The extra hidden test instance was stopped. Screenshots were inspected inline; no additional screenshot file is claimed as a saved artifact.

## Reported Input Regressions / 输入问题回归验收

The user's follow-up exposed a coverage gap in the earlier native smoke check: Unicode text injection bypasses native keydown dispatch and therefore did not certify physical English-key input. A new direct-key GPUI test first failed with an empty input instead of `hello World.1`.

The AI input's blanket keydown propagation stop prevented Windows from translating ordinary keys into `WM_CHAR`. Printable keys now reach the platform input handler; Windows' named Space key is explicitly recognized even without `key_char`, and layout-generated character input is respected. Modified document shortcuts remain isolated. Tests cover the composer and AI settings model field, independent Undo and existing IME semantics.

Local attachment and prepared-request budget failures now have dedicated input feedback, the known attachment label/UTF-8 byte size, the saved input budget and an Advanced settings action. The same attachment gate applies to source/read-mode selections and whole-document/file attachments. Aggregate context and serialized-message/system overhead failures preserve the draft and pending attachments without starting a provider request. Other protocol/profile errors are no longer misreported as input overflow. Default and saved limits are not automatically increased.

Synthetic acceptance uses 30,000 Chinese characters (90,000 UTF-8 bytes): the default 65,536 B rejects the attachment; a saved 262,144 B budget accepts the complete text and serialized request without truncation. Core file-read tests verify the same larger budget and retain the 1 MiB disk-read ceiling, including range reads. Both guides document manual budget increases, shared history/envelope overhead and explicit editor-selection processing for larger files.

Final follow-up checks on 2026-10-10:

- `cargo test --workspace -- --test-threads=1` with `CARGO_INCREMENTAL=0`: passed, including doctests. Application binary: 715 passed, 2 ignored; AI core: 34 passed. The 46 AI UI/input tests are included; a separate targeted run also passed all 46 before the final native-Space guard refinement.
- `cargo fmt --all -- --check`, `git diff --check` and `openspec validate refine-ai-assistant-workflows --strict`: passed.
- `cargo clippy --workspace --message-format=json` with `CARGO_INCREMENTAL=0`: passed, retaining the existing 133 warnings. The new feedback renderer's style warning was corrected without unrelated lint cleanup.
- `cargo build --bin markion` with `CARGO_INCREMENTAL=0`: passed on retry. The first artifact replacement reported Windows access denied; no running Markion process was found, and no user process was terminated or permissions changed.
- Native Windows physical-key input, using the exact freshly built `target/debug/markion.exe`: toggling the current IME to English and pressing `a`, Shift+B, Space, period and `1` produced `aB .1` without an IME candidate popup. Individual Ctrl+Z steps restored the empty composer; the document remained saved and unchanged. The initial Chinese preedit/candidate popup was canceled with Escape, and the original Chinese input mode was restored afterwards.

No cloud request, attachment upload, workspace grant, credential/persistence change, profile save, document edit, installed-application replacement or release publication occurred. The local build remains open for inspection. Full native Chinese candidate selection/commit remains outside this follow-up smoke check; its composition semantics are covered by the automated tests.

## Deferred / 建议后续优化

1. A full history manager: search, rename, pin, export, and non-destructive merging of saved and in-memory conversations. The current bounded session picker is not a searchable archive.
2. Richer message rendering: inline Markdown styles, navigable tables, code-block copy and per-turn retry/edit actions. Existing bounded Markdown block rendering remains the base.
3. A true structured diff review: minimal line/word differences, side-by-side comparison, per-hunk acceptance, and richer newline-only change presentation. This change deliberately uses a bounded contiguous-span summary and atomic captured-range application.
4. Capability and service diagnostics: last verified time, explicit provider-returned model metadata versus configured model, protocol-specific endpoint hints and service-specific limits. The displayed model is the saved configuration, not a certified server identity.
5. Broader native visual/usability acceptance on real Windows IMEs and device DPI, particularly very narrow panels and long service/file names; use findings to decide whether a dedicated resizable preferences window is warranted.
6. Explicit large-document chunking, model-aware token estimates, and user-reviewed history compaction. The current input limit remains finite; larger files use explicit selections rather than automatic truncation or background uploads.

## Limits / 验证边界

The native smoke check covers one Windows desktop configuration, not every device DPI or language/theme combination. IME semantics are covered by GPUI tests; an actual Windows IME candidate-window workflow and paid-service compatibility remain unverified. Earlier native observations in `docs/ai-agent-acceptance.md` describe the older UI; this refinement's native observations are recorded above.

The two prerequisite AI changes remain unarchived. This change is not archived ahead of them and does not write directly to `openspec/specs/`. Known repository lint warnings are not addressed by unrelated refactoring.
