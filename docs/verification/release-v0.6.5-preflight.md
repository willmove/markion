# Markion v0.6.5 release verification

**Status: PUBLISHED AND VERIFIED under disclosed exception V065-EX-01. Unrun native/manual checks remain BLOCKED/NOT_RUN, not PASS.**

The comparison base is tag `v0.6.4`, the latest published GitHub Release. The range `v0.6.4..aebad6a` contains 8 commits: 7 fixes and features plus the v0.6.4 records commit. All were direct pushes to `main`.

## Release scope

- In-app examples and templates gallery: **Help → Examples and Templates** now opens a scrollable thumbnail gallery inside the Markion window; a selected sample opens as an editable tab with its own save destination and working relative links and images, while the current workspace stays available. The previous Windows failure to reveal the bundled folder in Explorer is fixed, with localized status-bar feedback when the file manager cannot launch.
- Word import fidelity for academic documents: heading bookmarks map to heading identifiers instead of exposed bookmark markup, emphasis survives Word runs split across a styled phrase and Markdown delimiter boundaries, Word math structures convert to renderable LaTeX with explicit diagnostics for unsupported equations, quote styling, thematic breaks and table alignment are retained, formula-heavy documents render viewport-scoped, and the outline can summarize heading formulas.
- Thirty-four defects reported in GitHub issues #107–#140, across editing (Replace All activation, ordered-list continuation, emoji shortcode completion, word selection, slash-command insertion, Unicode word navigation), navigation and restoration (source/visual page navigation, recent-workspace tabs, rename-aware history diffs), preview (reference links, non-breaking spaces, UTF-8 BOM, folding around matches), and built-in exports (HTML copy with reference definitions, PDF alt text and combining-mark offsets, DOCX ordered-list starts and nesting, lossless image URL/title serialization).
- Visual Edit rendering repairs: Mermaid diagrams no longer remain on the loading placeholder in Visual Edit, Split Preview or Read; checked and unchecked task markers share the same square geometry; standalone HTML text keeps authored alignment per rendered line (including line breaks and subscripts); rendered HTML in Visual Edit is presented directly without the source-editor card frame, keeping the collapsible source editor for raw HTML.
- macOS quick-menu reliability: the Visual Edit block-grip quick-settings menu opens reliably despite trackpad click jitter, mid-click reflow, zero/micro-delta scroll phases and synchronous IME commits; display math blocks now expose the block-leading grip.
- Arch/Omarchy stability: the pacman package declares the Lavapipe software Vulkan driver as a runtime dependency and its desktop entry and terminal command use a launcher that retries a failed renderer startup on Lavapipe, recovering installs whose system Vulkan stack segfaults at startup.

Existing documents, preferences and workspace formats require no migration.

## Source and version identity

- Candidate implementation commit: `aebad6a81e9d75778d47d289c30beec946a65c44`.
- The release tree adds only the synchronized version fields (`Cargo.toml` workspace and root package, `packager.toml`, ten `Cargo.lock` package versions) and this documentation.
- CI packages from the candidate main-branch run carry the commit's inherited `0.6.4` metadata. They are candidate evidence, not final v0.6.5 assets.
- `cargo metadata --no-deps` reports all ten workspace packages at `0.6.5`; `cargo check --workspace` passed.

## Automated evidence

Environment: Windows 10.0.26300, 125% DPI; rustc and cargo 1.98.1, PowerShell 7.7.0, OpenSpec CLI 1.13.0, Python 3.14.7.

- `cargo test --workspace`: PASS — exit code 0, **2,434 passed / 0 failed / 5 ignored** across 52 test binaries (up from 2,384 passed at v0.6.4; full log retained at `target/tmp/release-qa/test-run2.log` during the preflight).
- `cargo run -p wechat-workspace --bin verify-bundle -- assets/marknice-workspace`: PASS — 23 files, 3,140,387 bytes, MarkNice `c009c1ec7e7c92f89afa5a32edcb126b5296bda7`.
- `scripts/verify-examples.ps1`: PASS (18 Markdown examples, index coverage and local references).
- `python docs/testing/fixtures/prepare.py check`: PASS (62 cases, blank result rows and 14 documentation pages; explicitly no native tests).
- Isolated fixture preparation `target/tmp/release-qa/run-001`: PASS; 160 source/working files, manifest SHA-256 `f2ea48d67f62b383cdc92dce5772fa379cbb495f0dd93dc050ccae3b303723aa` (identical to the v0.6.4 record; fixtures unchanged); `verify --working` PASS. Native acceptance remains NOT_RUN.
- Release-only diff: `git diff --check` clean; `Cargo.lock` changes are exactly the ten Markion package version lines.

## Candidate and native acceptance

This host is a Windows desktop with no macOS, Linux desktop or Arch/Omarchy environment, no clean test account or virtual machine, and no OS-level input injection.

The [main packaging run](https://github.com/willmove/markion/actions/runs/46024017302) at `aebad6a`: **PASS**. Windows x64, macOS arm64 and x64, and Ubuntu x86_64 packaging passed for the portable ZIP, NSIS installer, both DMGs, DEB, RPM, AppImage and the pacman package, as did the no-GPU AppImage smoke test. Tag-gated signing, notarization, installed-app smoke and publication jobs are skipped on branch runs by design.

Windows candidate artifact from that run:

- `markion_0.6.4_x64-portable.zip`, SHA-256 `65d2dc8c1221e046c0ba82bda2ef4db2fc44d674324a04aa06c75e84720e0946`.
- `markion_0.6.4_x64-setup.exe`, SHA-256 `290fd3b2efa15e033cc974f4d5bb49a9edcab38ecef89caa837bea1d27dd700e`.
- Packaged `markion.exe`, SHA-256 `e4ca7e20857199f12d043e78252d8c5e8152e528376c6a568aaf0b10621436bf` (69,439,488 bytes).

- **Packaged resources (CLI_CHECK): PASS.** All 95 packaged `assets/` files have a checkout counterpart: 60 are byte-identical and 35 differ only by CRLF versus LF. No file differs in content, none is missing and none is unpackaged. The packaged MarkNice `bundle-manifest.json` SHA-256 is `e45d0d19db043a43f05e02a5a7577211cc2e5fa3e977a90d2b1349806d156f42`, equal to the v0.6.4 record.
- **Portable launch (P01, isolated profile): PASS.** The packaged executable was started with redirected `APPDATA` and `LOCALAPPDATA`, update checks and autosave disabled, and one synthetic tables document as its argument. It opened a visible, non-minimized window titled `Markion - tables.md` (1180×760 logical client). A `PrintWindow` capture of that window alone is non-blank (18,421 samples, 58 colours). Standard error is empty, the opened document's bytes are unchanged, and the maintainer's real `session.toml` hash is unchanged (`93f03d07…` before and after). Capture SHA-256 `8ccd5e0d4f8ee2850f45432705d63001950a933d9bd37b842814acfeca37fe1c`.
- **Native visual smoke (CALLBACK_REPLAY): PASS.** The packaged executable ran the in-app replay through a real native window, renderer and input handlers with `MARKION_NATIVE_SMOKE_DIR` and `MARKION_PROFILE_VISUAL_INPUT`. Report `passed: true`, 9 observations, exit code 0, peak working set 296.1 MiB, reported version `0.6.4` (candidate metadata, as expected before tagging). Report SHA-256 `fa099c4d156a57d7e0a49840ee0d8e733a6a1d77d74cecba50285c1772de8783`. This is application-callback evidence; it injects no OS mouse, keyboard or IME input.

### Development-binary precheck (labeled precheck, not package acceptance)

- Local `cargo build --release --bin markion` on the release tree: PASS.
- **Native visual smoke (CALLBACK_REPLAY), development binary:** PASS — exit code 0, `passed: true`, 9 observations (4 fixture paint-sample rows for 100/300/600/1024 KiB plus block-selection/clipboard, table-rectangle, matrix-paste, code-folding and IME-callback cases), peak working set 297.7 MiB, reported version `0.6.5`. Report SHA-256 `c4c90bb18387f48cbf3e5f9086dbcb39ee4d1ab9c3a9c7bc88c5053f2661ccea` (retained at `target/tmp/release-qa/devbinary-smoke2/native-smoke-report.json` during the preflight). Application-callback evidence; no OS mouse, keyboard or IME injection.

### Not run or blocked

- **BLOCKED, no environment:** macOS Apple Silicon and Intel (DMG install, browser download and first launch with Gatekeeper, real IME, Cmd/Option keys, `public.html`); Linux X11 and Wayland (DEB, RPM, AppImage and pacman installation and interaction, IBus/Fcitx, clipboard, GPU and software Vulkan, window corners); Arch and Omarchy desktops. CI packaging, the CI AppImage smoke and the CI pacman installation are automated evidence only.
- **NOT_RUN, needs a clean test account or virtual machine:** installing the NSIS package, and the signed update from v0.6.4 on Windows and from a running v0.6.4 AppImage on Linux, including the dirty-document refusal and failure fallbacks. Nothing was installed on the maintainer's account.
- **NOT_RUN, needs real OS input:** Microsoft Pinyin composition, CF_HTML pasted from a real browser, system dialogs, drag and resize, and real display-scaling changes.
- **NOT_RUN:** the MarkNice browser, WeChat and Word-viewer matrix; real-producer DOCX qualification; the theme × language × scaling sweep; interactive long-session observation.
- **PENDING until the tag run:** final tagged packages, updater signatures, `update.json`, the GitHub Release, the OSS mirror, and both macOS installed-app smoke jobs.

## Release-specific exception V065-EX-01

On 2026-11-27, before tagging, the maintainer directed:

> 发布新版本到Github。 如果有未执行的原生验收范围（macOS/Linux 实机、签名升级链、真实 IME/浏览器/Word 互操作等），有可能的就进行测试验收，如果没有条件就跳过即可。

This authorizes proceeding with publication while skipping the disclosed unrun items for v0.6.5: native macOS and Linux desktop environments, previous-version signed upgrade chains, real OS input (IME, browser clipboard, system dialogs), the MarkNice/WeChat/Word-viewer matrix, real-producer DOCX qualification, and the theme × language × scaling sweep. The exception does not relabel any result as PASS and does not waive final asset, signature, manifest, mirror or workflow verification after tagging. It is not reusable by another release and does not inherit V064-EX-01.

## Final tagged artifact and publication evidence

- Release commits: documentation `489c053`, dedicated release commit `5c6ee78e422da84c2b92adfca53cb4be0223d50b` with only the synchronized version fields; annotated tag `v0.6.5` (`afac8713`) resolves to it. Local `main`, `origin/main` and the tag were synchronized at push time; the worktree is clean; the tag was never moved.
- [Tag workflow 46035164162](https://github.com/willmove/markion/actions/runs/46035164162): **PASS**. All ten jobs succeeded: four native build/package jobs, the no-GPU AppImage smoke, updater signing, both macOS installed-app launches, GitHub publication and OSS mirroring.
- [GitHub Release v0.6.5](https://github.com/willmove/markion/releases/tag/v0.6.5): published stable, non-draft, title `Markion v0.6.5`, curated English/Chinese notes with the full comparison link. Exactly 11 assets: eight packages, two Minisign signatures and `update.json`.
- GitHub `update.json`: version `0.6.5`; only `windows-x86_64`/NSIS and `linux-x86_64`/AppImage entries; each installer URL names its GitHub asset, and each embedded signature equals the corresponding published `.sig` asset byte-for-byte (the `.sig` assets carry the base64-encoded Minisign file, as in earlier releases).
- OSS `latest/`: all 14 public objects return HTTP 200. `manifest.json` reports version `0.6.5`, tag `v0.6.5` and all eight packages including `linux-arch-x86_64`; every digest in `sha256sums.txt` equals the GitHub asset digest for the same package; the mirrored `update.json` names OSS installer URLs on both platforms and its signatures are identical to GitHub's.
- Apple Silicon DMG SHA-256 `41a3724affa78667e31e460b3b4ee4e55260bf4527e083483dd764eb47bad40b` and Intel DMG SHA-256 `3848676ec8eaae2354866ea54accec4143fb63a030eb06f523ab1407c8ddf160` match the public GitHub assets. Both launch jobs verified Developer ID signatures (`Developer ID Application: Weimu Ma (MY2Y7T53XG)`) and stapled notarization (`source=Notarized Developer ID`, `Notarization Ticket=stapled`) and then opened a visible installed-app window on the matching architecture.
- Final Windows portable package check: the released `markion_0.6.5_x64-portable.zip` was downloaded from the OSS mirror, SHA-256 `eb0ef4688130d1a6142f48321e8ca4d4d059a837000680fcf670ec4dae33cba5`, equal to the GitHub asset and `sha256sums.txt`; packaged `markion.exe` SHA-256 `ce424aa9fc671838dfb0a61358afc65a64ec775a61b007a66a29e2c3c6362b35` (70,892,544 bytes), differing from the candidate `e4ca7e20…` as expected from the `0.6.5` metadata.
- Final package smoke addendum (Windows, isolated profile): launch PASS — visible window `Markion - tables.md` (1180×760 logical), non-blank `PrintWindow` capture (SHA-256 `7612c9759ba066ad6c455be7fba36a2c01142b367fe63ba270b381b014ce7805`), unchanged document bytes, untouched real `session.toml`; native visual smoke (CALLBACK_REPLAY) PASS — `passed: true`, exit code 0, 9 observations, peak working set 295.7 MiB, reported version `0.6.5`; report SHA-256 `df1745265ab36358eb3370884e1311abd6ae8f4fca4ae7a38b64e58bf44a7d14`. Real OS input, installation and signed-update flows remain in the disclosed NOT_RUN/BLOCKED scope.
- Previous-version signed upgrades remain NOT_RUN under V065-EX-01; nothing was installed on the maintainer's account.

## Final publication checklist

- [x] Complete workspace and release-only diff validation; all ten workspace packages resolve to 0.6.5 and no tag/Release collision existed.
- [x] Windows candidate and final package resource, launch and native-smoke checks under V065-EX-01; unrun native scope disclosed and not relabelled.
- [x] Successful tag workflow, including both notarized macOS installed-app launches, AppImage smoke, updater signing and publication.
- [x] Eight GitHub packages, NSIS/AppImage signatures, `update.json`, and curated bilingual notes verified.
- [x] Eight OSS package mirrors, both signatures, both manifests, `packager.toml` and `sha256sums.txt` verified with matching versions, URLs and signatures.