# v0.6.0 release preflight

Date: 2026-11-17 (Asia/Shanghai). Previous stable tag: `v0.5.3`. Requested release: `v0.6.0`. Publication completed. This record distinguishes source checks, final artifact verification and explicitly skipped manual acceptance.

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
| OpenSpec strict validation | Local working-tree check: 92 passed / 0 failed, including the concurrently prepared acceptance-doc change; committed candidate CI: 91 passed / 0 failed. The concurrent documentation change was excluded from the tagged application tree. |
| Version metadata / `cargo check --workspace` | Passed: all 10 workspace packages, root/workspace/packager and lockfile resolve to 0.6.0; no third-party lock versions changed |
| Release-version workspace regression | Passed after 0.6.0 synchronization: 2,216 passed, 0 failed, 5 existing ignored; 51 result groups |

## Native evidence and manual boundaries

The archived [context-menu tasks](../../openspec/changes/archive/2026-11-17-unify-compact-block-context-menu/tasks.md) preserve isolated Windows OS-input verification, source-byte comparisons, menu/tab lifecycle regressions and the intermittent code-background symptom's limits. The [example-library tasks](../../openspec/changes/archive/2026-11-17-bundle-example-document-library/tasks.md) record copied-payload rejection and a debug ZIP extraction fixture; that fixture is not a release installer.

The supported default-browser/WeChat/Word-viewer matrix, macOS/Linux native rich-clipboard and editing/input matrix, Linux CSD live resize, DOCX real-producer corpus and clean previous-version Windows/AppImage updater end-to-end runs have no fresh complete v0.6.0 evidence here. Previous release-specific skips do not establish a pass or automatically authorize skips for this version. On 2026-11-17 the maintainer explicitly authorized recording the unavailable cross-platform manual acceptance as unverified/skipped for v0.6.0 and continuing publication after all automated tests, native packaging, signatures and mirror verification pass. This exception applies only to this release: pending matrix rows remain unverified, not passed. Automated native packaging, extracted examples/bundle verification, AppImage smoke, signing, manifests, hashes and OSS reachability remain required.

## Publication checklist

- [x] Release-specific manual scope decision recorded: unavailable manual environments explicitly skipped with maintainer authorization; not claimed as passed.
- [x] Reviewed implementation/documentation commits pushed; dedicated `Release Markion v0.6.0` commit `402d16e5242bb819ee3a08bbe156262aa96b0570` and annotated tag object `0f3b711237d208fc4412c7560534af62c948cf61` created and pushed without changing an existing public tag.
- [x] Four native build jobs, extracted-package verification, AppImage smoke, updater signing, GitHub publication and OSS mirror succeed.
- [x] Seven packages, both signatures and `update.json` verified; GitHub/OSS manifest versions, host URLs, signature content and payload hashes match.
- [x] Curated bilingual notes include “喜迎APEC，深圳欢迎您”; actual macOS signing/notarization status is recorded.
- [x] Stable non-draft metadata and final Git synchronization verified.

## CI and publication tracking

- Candidate main packaging: [44041639382](https://github.com/willmove/markion/actions/runs/44041639382), succeeded on all four native builds and AppImage smoke.
- Candidate repository quality: [44041639336](https://github.com/willmove/markion/actions/runs/44041639336), succeeded.
- Authoritative tag publication: [44050240423](https://github.com/willmove/markion/actions/runs/44050240423), succeeded on all eight jobs: four native builds, no-GPU AppImage smoke, updater signing/metadata, GitHub publication and OSS mirroring.

## Final publication evidence

- [Markion v0.6.0](https://github.com/willmove/markion/releases/tag/v0.6.0) is the latest stable, published non-draft Release, titled `Markion v0.6.0`, with curated English-first/Simplified-Chinese notes and the requested “喜迎APEC，深圳欢迎您” message.
- [Tag workflow](https://github.com/willmove/markion/actions/runs/44050240423), attempt 1: all eight jobs passed. Both macOS jobs log `macOS signing mode: adhoc; notarize: false`; essential Gatekeeper instructions remain in the notes.
- [Machine-readable verification](release-v0.6.0-assets.json): seven platform packages, both updater signatures and `update.json` attached. Both manifests declare 0.6.0, name their own distribution host, and match in every field apart from the platform URLs. Installer signature content matches both attached signature files and mirrored copies.
- All 13 public mirror objects returned HTTP 200. All seven mirror payloads were streamed and hashed; SHA-256 and byte size match GitHub asset metadata and `sha256sums.txt`. The mirrored `manifest.json` and `packager.toml` also declare 0.6.0.
- Candidate repository quality independently reports 51 successful test groups, 2,216 passed / 0 failed / 5 existing ignored, plus formatting, Clippy, pinned bundle and 91 committed OpenSpec items passed. The earlier 92-item local check included an additional concurrently prepared documentation change and is not mislabeled as the tag's inventory.
- The immutable annotated tag object `0f3b711237d208fc4412c7560534af62c948cf61` points to dedicated release commit `402d16e5242bb819ee3a08bbe156262aa96b0570`. Subsequent commits record acceptance scope and completed verification without modifying the tagged application or public tag.
- Publication used an isolated clean `main` checkout. The original workspace's parallel acceptance-suite documentation edits were preserved and excluded from this release; synchronization and clean-tree checks refer to the publication checkout.
