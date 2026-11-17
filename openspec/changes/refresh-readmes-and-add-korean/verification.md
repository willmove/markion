# Verification — 2026-11-17

## Delivered

- Eight equivalent README editions: English, Simplified Chinese, Traditional Chinese, Japanese, Korean, French, German and Spanish. The English root is approximately 70% shorter by line count than the former combined English/Chinese root, including retained compatibility anchors.
- Two English and three Korean unretouched Windows development-build captures, their sample workspace, and [capture provenance](../../../docs/images/README.md). Delivered images were visually inspected for legibility and personal content; the Korean native File menu has an additional verification capture.
- A [bilingual detailed guide](../../../docs/user-guide.md) preserving the former README reference material and a [performance methodology page](../../../docs/performance.md) separating model, callback-replay and system-event measurements.
- Korean README plus complete application localization: menus, preferences, editor controls, recovery, import/export, Backup and Sync, shortcuts, Markdown reference and AI. The user confirmed both scopes. Current source has eight UI languages; already-published v0.6.0 binaries retain seven, explicitly distinguished in every README.
- A restart-only English Ready status discovered in the native check now uses the persisted language. The final Korean startup displays `준비됨`.

## Checks

| Check | Result |
| --- | --- |
| Eight-language reciprocal navigation and active-language labels | Passed |
| Markdown/HTML local file and anchor links in all READMEs and added guides/assets | 331 checked, passed |
| Shared model timing tables versus `fluent-editing-windows-2026-11-11.json` | All four sizes and P50/P95 values match to two decimal places |
| Latest release and download assets | Verified via GitHub release API: published v0.6.0, 2026-11-17; filenames and displayed MiB sizes match asset metadata |
| OS-input figures | 26.63/33.14 ms match the 1080-event final record; boundaries and historical date disclosed |
| Desktop Markdown rendering | All eight editions rendered in headless Microsoft Edge, 1280 px viewport; local images loaded and no page overflow |
| Narrow viewport | English, Simplified Chinese and Korean checked at 390 px; no page overflow |
| Visual review | English hero, Chinese mobile view and Korean performance/table rendering inspected |
| `git diff --check` | Passed |
| `openspec validate refresh-readmes-and-add-korean --strict` | Passed |
| Locale aliases and preferences | `ko`, `ko-kr`, `kor`, `korean`, `한국어`, and uppercase/whitespace aliases tested; persisted code is `ko` |
| Korean message completeness and placeholder compatibility | Main, auxiliary, recovery, Git and AI tests passed; one-off audit verifies all 174 Git and 194 AI rows contain eight values and preserves the original seven translations exactly |
| `cargo test --workspace -- --quiet` (final source) | **2,221 passed, 0 failed, 5 existing ignored**, across 51 reported suites; the new cases are included, not added again to the total |
| Focused localization tests | `cargo test --lib i18n -- --quiet`: 23 passed; these overlap the workspace suite |
| `cargo build --bin markion` | Passed; isolated native verification used the resulting executable |
| `cargo clippy --workspace` | Passed with existing warnings (99 in the app binary); no diagnostics in the added localization catalogs |
| `cargo fmt --all -- --check` | Passed |
| Pinned MarkNice bundle | 23 files, 3,140,387 bytes verified |
| Strict full OpenSpec validation | **92 passed, 0 failed** |
| Windows native language switch | Actual Preferences click changed English to 한국어 and relabeled native menus, General preferences and AI settings |
| Windows restart | Saved `language = "ko"` survives restart; final startup Ready status is Korean |
| Document invariants | GPUI test retains dirty text, version, non-empty undo history and shared preview identity through language change; native demo's full-file SHA-256 is unchanged through switching and restarts |

The local rendering uses Markdown-it with HTML enabled and GitHub-like table/image constraints; it is a local preview, not a claim of browser testing on GitHub or application testing on macOS/Linux. Preview captures and one-off audit/render helpers remain under ignored `target/tmp/readme-refresh/`.

The final native executable SHA-256 is `454B74DBB1C8B07A1F6FD92E4784D3D036375DEA879BC0AD1FDC6D1A657A7DEB`. Its app data was isolated under `target/tmp/readme-refresh/korean-native`, with a temporary copy of the demo workspace outside the repository, startup update checks and auto-save disabled. AI remained off; no credentials were entered and no AI requests were sent. The test app was closed after capture. The demo file SHA-256 stayed `EAF59227D9DE23980A0D1448399AAE618512E007A8ACA7C650D911D543EE38DC`.

No Cargo dependencies changed. Existing release acceptance documentation/spec edits and their untracked artifacts were left intact. This change remains complete and ready for OpenSpec archival; no release or Git push was performed. Native macOS/Linux interaction was not part of this Windows check.
