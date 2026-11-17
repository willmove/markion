# v0.6.0 release preflight

Date: 2026-11-17 (Asia/Shanghai). Previous stable tag: `v0.5.3`. Requested release: `v0.6.0`. This record distinguishes completed source checks from publication and manual acceptance.

## Candidate and scope

- Candidate starts from `e6f38e510c1bf023556930fb6cf07a8f1b36e2aa` plus the reviewed pending block-context-menu and offline-example-library changes, OpenSpec reconciliation and documentation updates.
- Repository is public `willmove/markion`, default branch `main`; authenticated publisher is `willmove`. Fetch confirmed the base matches `origin/main`; local/remote `v0.6.0` tag and Release collision checks found no existing version.
- [OpenSpec archival review](openspec-archive-review-2026-11-17.md) records archived changes and deliberately retained work. Existing unchecked acceptance tasks are not silently completed.
- Markdown remains canonical; new outline/slash/menu settings use optional defaults. No manual document migration is needed. Existing AI credentials, document resources and recovery formats are not replaced by this release.

## Source checks

| Check | Result |
| --- | --- |
| `cargo test --workspace` | 2,216 passed, 0 failed, 5 existing ignored; 51 result groups, Windows source tree |
| `cargo fmt --all -- --check` | Passed |
| `cargo clippy --workspace` | Passed with existing warnings; warnings are not represented as clean lint output |
| Pinned MarkNice `verify-bundle` | Passed: 23 files, 3,140,387 bytes |
| `pwsh.exe -File scripts/verify-examples.ps1` | Passed: 18 Markdown files, index coverage and local references |
| OpenSpec strict validation | Passed: 92 items, 0 failures after the final 25 archives and retained-delta reconciliation |
| Version metadata / `cargo check --workspace` | Passed: all 10 workspace packages, root/workspace/packager and lockfile resolve to 0.6.0; no third-party lock versions changed |
| Release-version workspace regression | Passed after 0.6.0 synchronization: 2,216 passed, 0 failed, 5 existing ignored; 51 result groups |

## Native evidence and manual boundaries

The archived [context-menu tasks](../../openspec/changes/archive/2026-11-17-unify-compact-block-context-menu/tasks.md) preserve isolated Windows OS-input verification, source-byte comparisons, menu/tab lifecycle regressions and the intermittent code-background symptom's limits. The [example-library tasks](../../openspec/changes/archive/2026-11-17-bundle-example-document-library/tasks.md) record copied-payload rejection and a debug ZIP extraction fixture; that fixture is not a release installer.

The supported default-browser/WeChat/Word-viewer matrix, macOS/Linux native rich-clipboard and editing/input matrix, Linux CSD live resize, DOCX real-producer corpus and clean previous-version Windows/AppImage updater end-to-end runs have no fresh complete v0.6.0 evidence here. Previous release-specific skips do not establish a pass or automatically authorize skips for this version. Publication is held until the maintainer decides the unavailable scope; all automated native packaging, extracted examples/bundle verification, AppImage smoke, signing, manifests, hashes and OSS reachability remain required.

## Publication checklist

- [ ] Release-specific manual scope decision recorded.
- [ ] Reviewed implementation/documentation commit pushed; dedicated `Release Markion v0.6.0` commit and annotated tag created without changing an existing public tag.
- [ ] Four native build jobs, extracted-package verification, AppImage smoke, updater signing, GitHub publication and OSS mirror succeed.
- [ ] Seven packages, both signatures and `update.json` verified; GitHub/OSS manifest versions, host URLs, signature content and payload hashes match.
- [ ] Curated bilingual notes include “喜迎APEC，深圳欢迎您”; actual macOS signing/notarization status is recorded.
- [ ] Stable non-draft metadata and final Git synchronization verified.
