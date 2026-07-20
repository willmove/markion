# AGENTS.md — Markion

This project uses **OpenSpec** for spec-driven development. OpenSpec is the source of truth for what the system is and what we plan to change. Always work through it.

## Project context

Markion is a Rust + GPUI Markdown editor application.

- **Language:** Rust (stable). UI framework: GPUI (the Zed GPU-accelerated UI crate).
- **Markdown core:** `pulldown-cmark` (CommonMark + GFM).
- **Entry points:** `src/main.rs` (app bootstrap, window, menus), `src/lib.rs` (module root). Domain types live in `src/model.rs`; persistence in `src/storage/`; localization in `src/i18n.rs`.
- **Build/test:** `cargo build`, `cargo test`. Root-package Cargo workspace: the `markion` app crate lives at the repo root (`src/`), and library crates absorbed from Typune (see `docs/typune-integration-plan.md`) join as members under `crates/*`. Root cargo commands target the app crate as before; use `cargo test -p <member>` for a single member and `cargo test --workspace` to run every crate's suite (plain `cargo test` covers only the root package).
- **Workspace invariants:** member crates under `crates/*` must not depend on `gpui` (types implementing gpui traits stay in the root crate — orphan rule); `[profile.dev.package."*"]` does not cover workspace members, so compute-heavy member crates on the typing path need an explicit `[profile.dev.package.<name>]` override in the root `Cargo.toml`.
- **Architecture notes:** derived Markdown state (preview blocks, outline, stats) is cached per document version and shared via `Arc`; syntax highlighting is memoized; the editor reuses a cached text handle per version. Preserve these invariants when editing — don't recompute derived state on every keystroke.

## GitHub release workflow

The canonical release procedure is [`docs/release-process.md`](docs/release-process.md). Follow it from preflight through final GitHub verification whenever publishing a version.

- If the user requests a new release without specifying a version, increment PATCH from the highest stable `vMAJOR.MINOR.PATCH` tag. Major, minor, and prerelease versions require explicit direction.
- Keep the workspace package, root package, packager, and lockfile versions synchronized; run `cargo test --workspace` before tagging.
- Use a dedicated `Release Markion vX.Y.Z` commit and an annotated `vX.Y.Z` tag, then monitor the tag-triggered GitHub Actions run until every native build and the publish job succeed.
- Generated GitHub notes are not final. Unless the user requests another language, publish detailed English notes covering user-visible changes, fixes, compatibility or migration status, downloads, verification, and the full comparison link.
- Do not report completion until the Windows NSIS installer, macOS Apple Silicon DMG, Linux amd64 DEB, and Linux x86_64 AppImage are attached and the final Release metadata has been verified.
- Never delete, force-move, or recreate a public release tag without explicit user authorization.

## OpenSpec workflow — how to work in this repo

The OpenSpec CLI is installed globally (`openspec`, v1.5+). The spec root is `openspec/`. Structure:

```
openspec/
  specs/<capability>/spec.md   # source of truth: what the system IS (stable, per capability)
  changes/<change-name>/        # one folder per planned change: proposal.md, tasks.md, (optional design.md, specs/ deltas)
  config.yaml                   # schema + project context + per-artifact rules
```

**Two layers, never confuse them:**
- **Specs** describe the system as it currently is. They change only through archived changes.
- **Changes** are proposals for what to do next. A change is a planning artifact, not code.

### Default way of working

Prefer this loop over ad-hoc coding:

1. **Explore** (`/openspec:explore`) — think through a problem, investigate the codebase, clarify requirements. No code is written here.
2. **Propose** (`/openspec:propose`) — create a change folder and generate `proposal.md` / `tasks.md` (+ `design.md` if needed) via `openspec new change` + `openspec instructions`.
3. **Apply** (`/openspec:apply`) — implement the tasks one by one, ticking `- [ ]` → `- [x]` in `tasks.md` as you go. Keep each change minimal and scoped to its task.
4. **Archive** (`/openspec:archive`) — once tasks are complete, archive the change into `openspec/changes/archive/YYYY-MM-DD-<name>/` and let it sync delta specs into `openspec/specs/`.

Slash commands `/openspec:propose`, `/openspec:explore`, `/openspec:apply`, `/openspec:sync`, `/openspec:archive` encode the full step-by-step procedure for each phase — use them.

### Hard rules

- **Do not start implementing a feature before a change proposal exists.** For anything beyond a typo or one-line fix, create a change first. If unsure whether something warrants a change, ask.
- **Never edit `openspec/specs/` directly** as part of a feature. Specs are updated only by archiving a change whose delta specs get synced (`/openspec:sync` or during `/openspec:archive`).
- **Trivial fixes** (typos, obvious bugs, formatting) do not need a change — just fix and explain.
- **Always read the existing spec for a capability before proposing a change to it.** Run `openspec list --specs` to see what exists.
- **Validate before archiving:** `openspec validate <change-name>`. Run `openspec doctor` if anything looks off.

### Useful commands

```bash
openspec list                 # active changes
openspec list --specs         # existing capability specs
openspec status --change <name> --json   # artifact graph + paths for a change
openspec instructions <artifact> --change <name> --json  # build rules for an artifact
openspec validate <name>      # check change/spec consistency
openspec doctor               # overall OpenSpec health for this root
openspec context              # resolved working context
```

When a command prints a `--store <id>` hint, keep that flag on follow-ups; otherwise commands act on the local `openspec/` root.
