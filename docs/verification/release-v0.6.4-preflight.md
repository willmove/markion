# Markion v0.6.4 release verification

**Status: PUBLISHED AND VERIFIED under disclosed exception V064-EX-01. Unrun native/manual checks remain BLOCKED/NOT_RUN, not PASS.**

The comparison base is tag `v0.6.3`, the latest published GitHub Release. The range `v0.6.3..a9ce8e4` contains 77 commits: 75 fixes and features, one merge and the v0.6.3 records commit. All were direct pushes to `main`.

## Release scope

- Visual Edit tables: stable click and drag selection in tall tables, tables inside block quotes rendered and edited in place, a table-specific grip menu, **Equalize Column Widths** and an Insert Below flyout that shares the paragraph Insert catalog.
- Rendering fidelity in Visual Edit, Split Preview and Read: block-quote children, GFM table header emphasis and column alignment, footnote superscripts, bounded inline HTML styles, extended inline formatting around standard Markdown, nested same-kind emphasis, closing-punctuation wrapping, inline-formula baselines, and Linux font fallback with synthetic italic.
- Visual Edit code folding becomes opt-in: **Allow code folding in Visual Edit** under Preferences → Appearance, default off.
- Import and export: Word import list hierarchy, numbering, manual breaks, paragraph-initial tabs, run overrides and edge whitespace; LaTeX export list nesting and start numbers, literal dollar pairs and rich headings/cells; Word export styled links and HTML row headers; HTML export bare-link parameters.
- Editing and workspace: Read-mode edit actions are inert, typed Find works in Read mode, Find navigation and regex replace fixes, CRLF-preserving Enter and word navigation, code-fence protection, shortcut reset collisions and the reserved Ctrl+Left, rename protection for tabs with unsaved edits, tab-identity autosave, session restore leaving Open Recent untouched, file-tree counts and names.
- Release preparation: a formatting-only repair of six drifted files, a `duplicate_block` repair found during this preflight, synchronized `0.6.4` version metadata, eight refreshed READMEs and linked guidance.

Existing documents, preferences and workspace formats require no migration. `visual_code_folding` is a new optional key that loads as off when absent.

## Source and version identity

- Candidate implementation commit: `a9ce8e4a83393f2e7d77a31b795ca709f716b44f`.
- The release tree adds only: rustfmt output for `src/app/editing.rs`, `src/app/mod.rs`, `src/app/preview.rs`, `src/app/tests.rs`, `src/visual.rs` and `tests/linux_font_styles.rs` (114 insertions, 51 deletions, no semantic change); the `duplicate_block` repair with two tests in `src/block_edit.rs`; synchronized versions; and documentation.
- CI packages from the candidate run carry the commit's inherited `0.6.3` metadata and predate the `duplicate_block` repair. They are candidate evidence, not final v0.6.4 assets.
- Cargo metadata reports all ten workspace packages at `0.6.4`; `cargo check --workspace` passed without warnings.

## Automated evidence

Environment: Windows 10.0.26300, 125% DPI; rustc and cargo 1.98.1, rustfmt 1.9.0-stable, PowerShell 7.7.0, OpenSpec CLI 1.13.0, Python 3.14.7.

- `scripts/check-quality.ps1`, run 1, on the formatting repair plus version metadata: PASS. Formatting; Clippy with pre-existing non-fatal warnings; the complete workspace suite at **2,382 passed / 0 failed / 5 ignored** across 52 test binaries (library 830 passed / 1 ignored, desktop 939 passed / 2 ignored); the pinned MarkNice bundle; strict OpenSpec validation at 106 passed / 0 failed. Log SHA-256 `f815d5aa995a8c69ff2770fb779b97a73b9be09e13f5f3f03e39b2b26cdff2dc`.
- `scripts/check-quality.ps1`, run 2, on the same tree plus the `duplicate_block` repair: PASS with the same gates at **2,384 passed / 0 failed / 5 ignored** across 52 test binaries and 106 passed / 0 failed OpenSpec items. Log SHA-256 `a6de4c072edc15923cead28f7ec776f22fd0a26540b7c54981c465e6fdc8db21`.
- `scripts/verify-examples.ps1`: PASS (18 Markdown examples, index coverage and local references).
- `python docs/testing/fixtures/prepare.py check`: PASS (62 cases, blank result rows and 14 documentation pages; explicitly no native tests).
- Isolated fixture preparation `target/tmp/release-qa/run-001`: PASS; 160 source/working files, manifest SHA-256 `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa`; `verify --working` PASS. Native acceptance remains NOT_RUN.
- The repository quality workflow on GitHub had been failing at its first gate, `cargo fmt --all -- --check`, since before this preflight, so its workspace tests and lints did not run there. A local `cargo fmt --all -- --check` reproduced the same 26 diffs in six files; the formatting repair restores that gate.

## Defect found and repaired during this preflight

**D064-01 (P1, repaired before tagging).** Commit `e381aa3` fixed duplicated paragraphs joining at the end of a document by adding a line ending whenever a block's source unit ended with exactly one. A list item in a tight list has the same shape, so Duplicate began inserting a blank line between an item and its copy. A temporary probe against the candidate source reproduced it:

| Input | Duplicate output before the repair |
| --- | --- |
| `- a⏎- b⏎`, first item | `- a⏎⏎- a⏎- b⏎` |
| `- a⏎- b⏎`, last item | `- a⏎- b⏎⏎- b⏎` |
| `- a⏎  - x⏎  - y⏎- b⏎`, nested item | `- a⏎  - x⏎⏎  - x⏎  - y⏎- b⏎` |

The blank line turns a tight list into a loose one in every Markdown renderer. The regression was introduced inside this release range and never shipped; `v0.6.3` kept tight lists tight.

The repair in `src/block_edit.rs` treats a list item outside a quote as bounded by its own marker: the copy keeps the item's existing boundary instead of a blank line. It also covers the pre-existing case of a list item at the end of a file with no trailing newline, which previously gained a blank line as well. Paragraph behaviour from `e381aa3` is unchanged. Regression tests: `duplicate_tight_list_item_keeps_the_sibling_boundary` (LF and CRLF, first, last and nested items) and `duplicate_single_newline_paragraph_before_another_block_stays_independent`. The complete workspace suite was rerun as run 2 above.

**Observation, not a defect.** The Windows CI runner checks out with `core.autocrlf=true`, so example files inside the Windows packages use CRLF while the repository stores LF. `scripts/verify-packaged-workspace.ps1 -Format zip` therefore stops at the first example when it is run against an LF working copy, because it compares raw SHA-256 values. The same verifier passes in the workflow that built the package.

## Candidate and native acceptance

This host is a Windows desktop with no macOS, Linux desktop or Arch/Omarchy environment, no clean test account or virtual machine, and no OS-level input injection. The only local WSL distribution belongs to an unrelated project and was not used.

The [main packaging run](https://github.com/willmove/markion/actions/runs/45938546191) at `a9ce8e4`: **PASS**. Windows x64, macOS arm64 and x64, and Ubuntu x86_64 packaging passed for the portable ZIP, NSIS installer, both DMGs, DEB, RPM, AppImage and the pacman package, as did the no-GPU AppImage smoke test. Tag-gated signing, notarization, installed-app smoke and publication jobs are skipped on branch runs by design.

Windows candidate artifact from that run:

- `markion_0.6.3_x64-portable.zip`, SHA-256 `44153d60016bd31b8958878e49e44509270816cf3cb536554157559cdf3fd07a`.
- `markion_0.6.3_x64-setup.exe`, SHA-256 `cebcd023bbfb324072df05bd2319d6c12abf9c4ecd3c06521a9b087e38d1cd7b`.
- Packaged `markion.exe`, SHA-256 `6abdaa68a21b22d2d1b6e2e0e84ba004da669d9f43e16c185b0e254716caf0b5` (69,248,512 bytes).

- **Packaged resources (CLI_CHECK): PASS.** All 95 packaged `assets/` files have a checkout counterpart: 59 are byte-identical and 36 differ only by CRLF versus LF. No file differs in content, none is missing and none is unpackaged. The packaged MarkNice `bundle-manifest.json` SHA-256 is `e45d0d19db043a43f05e02a5a7577211cc2e5fa3e977a90d2b1349806d156f42`, equal to the v0.6.3 record.
- **Portable launch (P01, isolated profile): PASS.** The packaged executable was started with redirected `APPDATA` and `LOCALAPPDATA`, update checks and autosave disabled, and one synthetic tables document as its argument. It opened a visible, non-minimized window titled `Markion - tables.md` (1180×760 logical client at 120 DPI). A `PrintWindow` capture of that window alone is non-blank (6,004 samples, 28 colours) and shows the document rendered in Visual Edit with emphasized table headers. Standard error is empty, the opened document's bytes are unchanged, and the maintainer's real `session.toml` hash is unchanged. Capture SHA-256 `0eb7a097a016c7db829704a7f47ab41f4b6f4e3c4476167e56f969299082f4b6`.
- **Native visual smoke (CALLBACK_REPLAY): PASS.** The packaged executable ran the in-app replay through a real native window, renderer and input handlers with `MARKION_NATIVE_SMOKE_DIR` and `MARKION_PROFILE_VISUAL_INPUT`. Report `passed: true`, 9 observations, exit code 0, 5.5 s, peak working set 254.2 MiB. Report SHA-256 `221471667d64fa9a0a7b36f2297ec9a54bb6256c5706ff118485db8f5a509cff`. This is application-callback evidence; it injects no OS mouse, keyboard or IME input.

| Fixture | Open | Input P50 | Input P95 | Resident |
| --- | ---: | ---: | ---: | ---: |
| 100 KiB | 61.06 ms | 14.16 ms | 18.95 ms | 146.2 MiB |
| 300 KiB | 36.65 ms | 16.88 ms | 20.45 ms | 170.0 MiB |
| 600 KiB | 71.51 ms | 21.90 ms | 27.68 ms | 205.4 MiB |
| 1024 KiB | 120.68 ms | 24.53 ms | 33.25 ms | 250.1 MiB |

The replay fixtures were generated locally with a debug build of the `bench_large_doc` example, whereas the documented procedure generates them with a release build; the generator's output does not depend on the build profile. `native-p3-smoke.md` was produced exactly as `scripts/Invoke-FluentEditingSmoke.ps1` produces it.

### Not run or blocked

- **BLOCKED, no environment:** macOS Apple Silicon and Intel (DMG install, browser download and first launch with Gatekeeper, real IME, Cmd/Option keys, `public.html`); Linux X11 and Wayland (DEB, RPM, AppImage and pacman installation and interaction, IBus/Fcitx, clipboard, GPU and software Vulkan, window corners); Arch and Omarchy desktops. CI packaging, the CI AppImage smoke and the CI pacman installation are automated evidence only.
- **NOT_RUN, needs a clean test account or virtual machine:** installing the NSIS package, and the signed update from v0.6.3 on Windows and from a running v0.6.3 AppImage on Linux, including the dirty-document refusal and failure fallbacks. Nothing was installed on the maintainer's account.
- **NOT_RUN, needs real OS input:** Microsoft Pinyin composition, CF_HTML pasted from a real browser, system dialogs, drag and resize, and real display-scaling changes.
- **NOT_RUN:** the MarkNice browser, WeChat and Word-viewer matrix; real-producer DOCX qualification; the theme × language × scaling sweep; interactive long-session observation beyond the replay samples above.
- **PENDING until the tag run:** final tagged packages, updater signatures, `update.json`, the GitHub Release, the OSS mirror, and both macOS installed-app smoke jobs.

## Release-specific exception V064-EX-01

On 2026-11-27, after the candidate evidence, the D064-01 disclosure and the explicit BLOCKED/NOT_RUN list above were presented, the maintainer authorized a one-time exception for v0.6.4 and chose to proceed with publication:

> 授权并继续发布：授权 v0.6.4 对上述未执行验收项使用一次性例外并继续发布；提交与推送按所列 4 提交 + annotated tag v0.6.4 计划执行。

The exception covers the disclosed unrun items: native macOS and Linux desktop environments, previous-version signed upgrade chains, real OS input (IME, browser clipboard, system dialogs), the MarkNice/WeChat/Word-viewer matrix, real-producer DOCX qualification, and the theme × language × scaling sweep. It does not relabel any result as PASS and does not waive final asset, signature, manifest, mirror or workflow verification after tagging. It is not reusable by another release and does not inherit V063-EX-01.

## Final tagged artifact and publication evidence

- Release commits: formatting `0fdf15c`, list-duplicate repair `dae08fb`, documentation `080c861`, dedicated release commit `4df26771b0337a402e5508ab719d44db9dacf7f7` with only the synchronized version fields; annotated tag `v0.6.4` (`14d6f5d4`) resolves to it. Local `main`, `origin/main` and the tag were synchronized at push time; the worktree is clean; the tag was never moved.
- [Tag workflow 45948581120](https://github.com/willmove/markion/actions/runs/45948581120): **PASS**. All ten jobs succeeded: four native build/package jobs, the no-GPU AppImage smoke, updater signing, both macOS installed-app launches, GitHub publication and OSS mirroring.
- [GitHub Release v0.6.4](https://github.com/willmove/markion/releases/tag/v0.6.4): published stable, non-draft, title `Markion v0.6.4`, curated English/Chinese notes with the full comparison link. Exactly 11 assets: eight packages, two Minisign signatures and `update.json`.
- GitHub `update.json`: version `0.6.4`; only `windows-x86_64`/NSIS and `linux-x86_64`/AppImage entries; each installer URL names its GitHub asset, and each embedded signature equals the corresponding published `.sig` asset byte-for-byte (the `.sig` assets carry the base64-encoded Minisign file, as in earlier releases).
- OSS `latest/`: all 14 public objects return HTTP 200. `manifest.json` reports version `0.6.4`, tag `v0.6.4` and all eight packages including `linux-arch-x86_64`; every digest in `sha256sums.txt` equals the GitHub asset digest for the same package; the mirrored `update.json` names OSS installer URLs on both platforms and its signatures are identical to GitHub's.
- Apple Silicon DMG SHA-256 `af4fd08c6f185e227a5ab10a3171de7a2f2745d5008135d9bc80a56b1222ef87` and Intel DMG SHA-256 `23d9838041496c89720e95c5b47ea2e75b97afc79c9b75848ab45f36b1474adb` match the public GitHub assets. Both launch jobs verified Developer ID signatures and stapled notarization (`source=Notarized Developer ID` on the app and the DMG) and then opened a visible installed-app window on the matching architecture.
- Final Windows portable package technical check: the released `markion_0.6.4_x64-portable.zip` (SHA-256 `46a45a63dd6c76dc66d57613fba337572495ccf98ec4d8b53873d40cc0ce36f1`, equal to GitHub and `sha256sums.txt`) was downloaded, its packaged `markion.exe` hashed, and the isolated-profile launch and native visual smoke were re-run against this final binary; see the addendum below.
- Previous-version signed upgrades remain NOT_RUN under V064-EX-01; nothing was installed on the maintainer's account.

### Final package smoke addendum (Windows, isolated profile)

- Final packaged `markion.exe`: SHA-256 `f44ba90d9a09ae282de37e68e48537175662f8ede2619d9834dcda43ef840fbc` (70,692,352 bytes); differs from the candidate `6abdaa68…`, as expected from the release-range repair and the `0.6.4` metadata.
- Isolated launch on the synthetic tables document: PASS — visible window `Markion - tables.md` (1180×760 logical at 120 DPI), non-blank `PrintWindow` capture (SHA-256 `5531f4e128a7c0897016bec843a264fb8cf10d0b5456e838c233dd161b2a5a49`), unchanged document bytes, untouched real `session.toml`.
- Native visual smoke (CALLBACK_REPLAY) on the final binary: `passed: true`, exit code 0, 5.6 s, peak working set 251.2 MiB, reported version `0.6.4`; input P50/P95 across 100/300/600/1024 KiB fixtures 12.46–27.76 ms / 20.72–36.97 ms; no OS input injection.
- Real OS input, installation and signed-update flows remain in the disclosed NOT_RUN/BLOCKED scope.

## Final publication checklist

- [x] Complete workspace and release-only diff validation; all ten workspace packages resolve to 0.6.4 and no tag/Release collision existed.
- [x] Formatting-gate repair verified formatting-only; D064-01 repaired with regression tests before tagging, never shipped.
- [x] Windows candidate and final package resource, launch and native-smoke checks under V064-EX-01; unrun native scope disclosed and not relabelled.
- [x] Successful tag workflow, including both notarized macOS installed-app launches, AppImage smoke, updater signing and publication.
- [x] Eight GitHub packages, NSIS/AppImage signatures, `update.json`, and curated bilingual notes verified.
- [x] Eight OSS package mirrors, both signatures, both manifests, `packager.toml` and `sha256sums.txt` verified with matching versions, URLs and signatures.
