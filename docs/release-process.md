# GitHub Release Process

This runbook is the canonical procedure for publishing a stable Markion release to GitHub. It records the process verified by the v0.1.5 release. A release is complete only after the tag workflow succeeds, all installers are attached, curated release notes are present, and the repository is synchronized with GitHub.

## 1. Defaults and prerequisites

- Publish from `main` with a clean worktree and a local branch synchronized with `origin/main`.
- Use the version explicitly requested by the maintainer. If no version is supplied, increment PATCH from the highest stable `vMAJOR.MINOR.PATCH` tag. Do not infer a major, minor, or prerelease version.
- Publish a stable, non-draft Release unless the maintainer explicitly requests a prerelease or draft.
- Write final release notes bilingually by default — English first, followed by the corresponding Simplified Chinese version — unless another language arrangement is requested.
- Preserve public tags. Never delete, force-move, or recreate a published tag without explicit authorization.
- Required tools: stable Rust and Cargo, Git, GitHub CLI (`gh`), OpenSpec CLI, and an authenticated GitHub account with permission to push and publish Releases.

Check the operating context before editing anything:

```bash
gh auth status
gh repo view --json nameWithOwner,defaultBranchRef,url,isPrivate
git fetch --tags origin
git status --short --branch
git tag --sort=-version:refname
git log --oneline --decorate -20
```

Confirm all of the following:

- The repository is `willmove/markion` and the default branch is `main`.
- The worktree has no staged, unstaged, or untracked release-related changes.
- `main` is neither behind nor unexpectedly ahead of `origin/main`. Fast-forward a clean branch when necessary; stop and investigate divergence.
- The intended version does not already exist as a local tag, remote tag, or GitHub Release.
- The latest `main` packaging workflow is successful, or any known failure has been understood before tagging.

Useful collision checks are:

```bash
git tag --list vX.Y.Z
git ls-remote --tags origin vX.Y.Z
gh release view vX.Y.Z
```

`gh release view` is expected to return a not-found error for a new version.

## 2. Build the change summary

Use the previous stable tag as the comparison base. Review the complete commit set and diff, including direct commits that GitHub's generated notes may omit:

```bash
git log --oneline <previous-tag>..HEAD
git diff --stat <previous-tag>..HEAD
git diff <previous-tag>..HEAD
openspec list
```

Read the relevant completed OpenSpec proposals, delta specs, designs, and tasks. Release notes must describe user-visible behavior rather than copying commit subjects or listing internal tooling changes.

Before publication, identify:

- Major user-visible features and improvements.
- Important bug fixes.
- Compatibility, migration, and known limitation information.
- Any platform support or installer changes.
- Verification evidence that can truthfully be reported.

## 3. Synchronize version metadata

Change all Markion-controlled version fields to `X.Y.Z` in one release change:

- `[workspace.package].version` in `Cargo.toml`.
- `[package].version` in `Cargo.toml`.
- Top-level `version` in `packager.toml`.
- The `export`, `markdown`, `markion`, and `markion-diagram` workspace package entries in `Cargo.lock`.

Let Cargo refresh the lockfile instead of blindly replacing every matching version string; third-party packages can legitimately use the old or new version number.

```bash
cargo check --workspace
cargo metadata --no-deps --format-version 1
```

Inspect the metadata output and confirm that every Markion workspace package resolves to `X.Y.Z`.

## 4. Validate before publication

Run the complete workspace suite and inspect the release-only diff:

```bash
cargo test --workspace
git diff --check
git status --short --branch
git diff -- Cargo.toml Cargo.lock packager.toml
```

Expected release-only changes are the synchronized version fields. Stop before tagging if tests fail, metadata versions disagree, the diff includes unintended edits, or the target tag/Release collides.

## 5. Commit, tag, and push

Create one dedicated release commit and an annotated tag:

```bash
git add -- Cargo.toml Cargo.lock packager.toml
git commit -m "Release Markion vX.Y.Z"
git tag -a vX.Y.Z -m "Release Markion vX.Y.Z"
git push origin main vX.Y.Z
```

The push starts two packaging workflows: one for `main` and one for the tag. The tag run is the authoritative publication run because only a `v*` ref executes the `Publish GitHub Release` job.

## 6. Monitor the tag workflow

Find the run whose branch is `vX.Y.Z`, then wait for it to finish:

```bash
gh run list --workflow release.yml --limit 10 --json databaseId,headBranch,headSha,event,status,conclusion,createdAt,displayTitle,url
gh run watch <tag-run-id> --exit-status --interval 15
```

All of these jobs must succeed:

- Build and package on `windows-latest`.
- Build and package on `macos-latest`.
- Build and package on `ubuntu-22.04`.
- Publish GitHub Release.

If the workflow fails, inspect it with `gh run view <tag-run-id> --log-failed`. Do not report the release as complete. If the public tag already exists, preserve it and either fix forward or ask the maintainer how to proceed.

## 7. Curate the release notes

The workflow creates a Release with generated notes. Treat those notes as a seed, not the final description. Replace or expand them using:

```bash
gh release edit vX.Y.Z --title "Markion vX.Y.Z" --notes-file <release-notes-file>
```

Use this structure unless the release content calls for a small adjustment:

```markdown
# Markion vX.Y.Z

One-sentence summary of the release.

## Highlights

### Feature or improvement area

- User-visible change and its practical effect.

### Fixes

- Important reliability or behavior fix.

## Compatibility

- State whether Markdown files, preferences, or workspace data require migration.
- State relevant platform limitations. Markion's unsigned Windows and macOS installers normally require SmartScreen or Gatekeeper bypass steps.

## Downloads

- Windows x64: NSIS installer.
- macOS Apple Silicon: DMG.
- Linux x86_64: DEB and AppImage.

## Verification

- Local workspace test result.
- Windows, macOS, and Linux native CI result.

**Full comparison**: https://github.com/willmove/markion/compare/<previous-tag>...vX.Y.Z

---

# Markion vX.Y.Z（中文说明）

一句话版本总结。

## 主要更新

### 功能或改进领域

- 用户可见的变化及其实际效果。

### 修复

- 重要的可靠性或行为修复。

## 兼容性

- 说明 Markdown 文件、偏好设置或工作区数据是否需要迁移。
- 说明相关的平台限制。Markion 的 Windows 与 macOS 安装包未签名，通常需要手动绕过 SmartScreen 或 Gatekeeper。

## 下载

- Windows x64：NSIS 安装程序。
- macOS Apple Silicon：DMG。
- Linux x86_64：DEB 与 AppImage。

## 验证

- 本地 workspace 测试结果。
- Windows、macOS、Linux 原生 CI 结果。

**完整变更对比**: https://github.com/willmove/markion/compare/<previous-tag>...vX.Y.Z
```

Release-note rules:

- Derive claims from the actual tag-to-tag diff, commits, and completed OpenSpec changes.
- Cover direct commits as well as merged pull requests.
- Prefer user outcomes over implementation details; include technical detail only when it explains compatibility, safety, performance, or fidelity.
- State "no migration required" only after checking persisted file, preference, and workspace formats.
- Do not claim a test, platform build, installer, or feature that was not verified.

## 8. Final verification

Inspect the published Release and final repository state:

```bash
gh release view vX.Y.Z --json name,tagName,isDraft,isPrerelease,publishedAt,url,body,assets
git status --short --branch
git log -1 --oneline --decorate
git ls-remote --heads --tags origin main vX.Y.Z
```

Confirm that:

- The title is `Markion vX.Y.Z` and the tag is `vX.Y.Z`.
- The Release is published, not a draft, and not an unintended prerelease.
- The curated notes and full comparison link are present.
- The assets include `markion_X.Y.Z_x64-setup.exe`, `Markion_X.Y.Z_aarch64.dmg`, `markion_X.Y.Z_amd64.deb`, and `markion_X.Y.Z_x86_64.AppImage`.
- The tag workflow succeeded on all three native platforms and in the publish job.
- Local `main`, `origin/main`, the release commit, and the annotated tag resolve to the intended release state.
- The local worktree is clean.

Only after every check passes should the release be reported as complete, with links to the Release and tag workflow.
