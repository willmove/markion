# v0.5.3 release preflight

Date: 2026-11-13 (Asia/Shanghai). Candidate base: `15fcdf88ea9bc4d72be42492cc41b86442a3b6be`; previous public stable tag: `v0.5.2`. This is a preparation record, not a publication-success report.

## Repository and metadata

- Initial `main` worktree was clean and matched `origin/main`; GitHub repository `willmove/markion` is public and defaults to `main`.
- Public tag/latest-Release queries confirmed `v0.5.2` as the latest stable version during preflight; the next patch is `v0.5.3`. Recheck collisions immediately before tagging.
- Cargo metadata resolves all **10 workspace packages** to `0.5.3`. Cargo regenerated the lockfile's ten workspace version entries; root/workspace and packager versions agree, with no third-party version changes.
- The connected GitHub app can read/write this repository, but its available tools do not create annotated tags or edit Releases. The maintainer subsequently configured GitHub CLI login as `willmove`; authenticated HTTPS Git operations and Release access are available. SSH configuration is not required for publication.

## Checks

| Check | Result |
| --- | --- |
| Original full workspace test attempt | Failed: root `ai_actions::tests::durable_roundtrip_and_newer_content_conflict`, 740 passed / 1 failed / 1 ignored. Investigated as directory link-count identity drift after folder creation |
| AI directory-identity correction | Implemented under [dedicated change](../../openspec/changes/archive/2026-11-13-fix-ai-directory-identity-staleness/); AI crate focused suite: 40 passed / 0 failed |
| Current full workspace tests at 0.5.3 | **2136 passed / 5 existing ignored / 0 failed**, `cargo test --workspace --offline --no-fail-fast` |
| `cargo check --workspace` | Passed (`--offline`) |
| `cargo clippy --workspace` | Passed (`--offline`), with existing lint suggestions retained |
| Pinned workspace bundle verification | Passed: 23 files / 3,140,387 bytes, pinned MarkNice `c009c1ec7e7c92f89afa5a32edcb126b5296bda7` |
| Rustfmt / diff whitespace | Passed |
| Strict OpenSpec validation | 108 passed / 0 failed before archiving; 106 passed / 0 failed after the AI fix and outline change were archived and synced |
| README parity and local links | Passed: exact Chinese content parity except navigation; local Markdown links checked in 15 documentation files |
| GitHub quality workflow for original base | [Failed before tests on Rustfmt](https://github.com/willmove/markion/actions/runs/43408726698); the single assertion-format mismatch has been corrected locally |
| Candidate branch packaging | [All four native builds and AppImage smoke passed](https://github.com/willmove/markion/actions/runs/43413836691) at `2d71c07`; tag signing/publication remains to run |
| Candidate Windows quality gate | [Windows tests reached the new AI regression](https://github.com/willmove/markion/actions/runs/43413836715); its temporary-path assertion compared a short path with a canonical long path. Corrected to use the canonical grant scope; full local suite rerun: 2136 passed / 5 ignored / 0 failed |
| Current browser checks | PASS: 15 themes plus formatting, exports, locales, skin, Word import and themed print checks; actual Copy Markdown/HTML/DOCX downloads verified |

The test failure was not ignored or excluded. The directory-only repair keeps filesystem volume/object checks, regular-file hard-link rejection and unchanged version-1 journal serialization. The final workspace run passes without skipped failures. The first all-target pass also exposed missing container Git templates, fixed for the verification process through `GIT_TEMPLATE_DIR`; the PDF SVG spike depended on proprietary Georgia, so that fixture now explicitly loads the project’s bundled Libertinus font and retains its empty/populated fontdb assertions. Application font selection is unchanged. The documentation contract test retains its `VisualBlockEditor::Table` marker. All resulting regressions pass in the final full suite.

The original base’s [native packaging workflow](https://github.com/willmove/markion/actions/runs/43408726646) subsequently completed all four builds and the no-GPU AppImage smoke test successfully. This is branch-build evidence, not a tag publication or updater-signing result.

## Additional browser evidence

The local `preview-workspace` harness ran in the Codex in-app browser on this Linux X11 session (engine/version is not exposed by the interface). The visible result was **PASS (15 themes + formatting + exports + skin + word + pdf)**. [Raw self-test results](marknice-browser-self-test-v0.5.3.json), [screenshot](marknice-browser-self-test-v0.5.3.png), and [download checks](marknice-browser-downloads-v0.5.3.json) are retained. Copy Markdown matched the textarea byte-for-byte; themed HTML contained the heading/table without session/blob/loopback URLs; the downloaded browser DOCX was a valid ZIP with its expected Word/altChunk entries. These checks do not establish Microsoft Word rendering fidelity.

## Required manual evidence and release-specific skips

- [MarkNice browser/platform/viewer matrix](../marknice-workspace-release-evidence.md): incomplete. The canonical runbook explicitly requires completed manual browser/WeChat evidence before readiness.
- Linux CSD resize cursor/drag acceptance: task 3.4 remains pending in [resize-border tasks](../../openspec/changes/add-csd-resize-borders/tasks.md).
- Clean previous-version Windows NSIS signed update, dirty-document refusal and manual fallback: no current candidate result recorded.
- Previous-version running Linux AppImage signed replacement/relaunch, dirty-document refusal, and DEB browser fallback: no current candidate result recorded.
- macOS notarization is determined by tag build logs, not by prior releases or credential assumptions.

The maintainer explicitly authorized performing available checks and skipping checks when the required environment is genuinely unavailable, then publishing. For **v0.5.3 only**, the remaining cross-platform/default-browser/WeChat/Word-viewer, native resize interaction and previous-version updater-installation matrix is recorded as **skipped, not passed**. No native app control surface or Windows/macOS installation was available; Xvfb/firejail and a previous-version running AppImage were unavailable here. The WeChat-copy local-image confirmation could not be controlled by the browser interface (CDP focus/input timed out); its cancellation/omission path is also skipped. This does not change the default release runbook or waive automated tests, signing, package/mirror integrity or final Release verification.

## Publication checklist

- [x] Required local checks complete; unavailable manual checks explicitly skipped for this release by the maintainer; authenticated publisher available.
- [ ] Documentation committed, dedicated `Release Markion v0.5.3` commit created, annotated tag checked and pushed without changing any public tag.
- [ ] Windows, macOS arm64/x64 and Linux builds, extracted-resource verification, no-GPU AppImage smoke, updater signing, GitHub publication and OSS mirror all succeed.
- [ ] Seven packages, both updater signatures and `update.json` attached; GitHub and OSS manifests, URLs, signature content, public objects and versions verified.
- [ ] Curated English-first/Chinese notes, title/tag, stable non-draft metadata, repository synchronization and clean worktree verified.
