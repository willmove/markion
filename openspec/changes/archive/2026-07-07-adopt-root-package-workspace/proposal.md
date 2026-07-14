## Why

Markion is about to absorb library crates from the Typune project (see `docs/typune-integration-plan.md`, Phase 0). Typune's value is concentrated in UI-free library crates (`markdown`, `export`, `filesystem`, `editor`) that come with ~1935 tests and their own dependency versions (e.g. pulldown-cmark 0.11 vs Markion's 0.13). To bring those crates in **unmodified** — keeping their test suites intact as the safety net and letting incompatible dependency versions coexist — Markion needs to be able to host multiple Cargo packages.

The minimal form that achieves this is a **root-package workspace**: add a `[workspace]` table to the existing `Cargo.toml` and keep the `markion` package at the repo root with `src/` untouched. This was chosen over (a) relocating the app to `crates/markion-app` (a virtual workspace — pure churn: big rename commit, spec-path updates, release-pipeline reconfiguration, no added benefit) and (b) merging Typune code as modules into the single crate (would force rewriting Typune's imports and tests). Full analysis in `docs/typune-integration-plan.md` §4.

## What Changes

- **`Cargo.toml`.** Add a `[workspace]` table with `members = ["crates/*"]` to the existing manifest. The `markion` package stays at the repo root; no source files move. Extend the profile comments to document that `[profile.dev.package."*"]` does **not** cover workspace members, so future compute-heavy member crates need explicit `[profile.dev.package.<name>]` overrides to keep the typing path responsive in dev builds.
- **`AGENTS.md`.** Replace the "There is no separate workspace — single crate" build note with a description of the root-package workspace layout and the two structural invariants (dev-profile overrides for members; gpui-trait-implementing types stay in the root crate).
- **Phase 0 decisions recorded (no code):** the merged product keeps the **Markion** name; Typune code is brought in by **copying crates into `crates/`** with the source commit (`0b9e313`) noted in commit messages (no `git subtree` — 8 commits of history are not worth the tooling overhead); Typune code is used under its MIT license option (Markion is MIT; both repos share the same author).
- **Non-goals:** no Typune code is imported yet (that starts with Phase 1 / the `export` crate); the `markion` app crate is not split; no behavior, dependency, or release-pipeline change.

## Capabilities

### New Capabilities
- `crate-architecture`: The repository is a root-package Cargo workspace — the `markion` app crate at the root, absorbed library crates under `crates/*` — with the structural invariants future integrations must respect (dev-profile coverage for member crates, gpui-facing types confined to the root crate, member crates buildable and testable without GUI system libraries).

### Modified Capabilities
<!-- None — this change alters project structure only; no existing editor capability changes. -->

## Impact

- **Edited files:** `Cargo.toml` (add `[workspace]` + profile comment), `AGENTS.md` (build/layout note), `docs/typune-integration-plan.md` (mark Phase 0 done). **New file:** `crates/.gitkeep` (cargo rejects a `members` glob whose parent directory does not exist — verified empirically; with `crates/` present, a glob matching zero packages is accepted).
- **No source-code changes:** `src/` is untouched; `cargo build`/`cargo run`/`cargo test` at the repo root behave exactly as before (the root package is the default target of a root-package workspace).
- **Release pipeline:** unaffected by construction — `cargo packager` and `.github/workflows/release.yml` operate on the root package, which does not move. Verified `cargo check` still passes after the manifest change.
- **Invariants touched:** none of the runtime invariants (cached derived Markdown state, memoized highlighting) are on this path. The change *adds* two structural invariants for future phases, captured in the `crate-architecture` spec.
- **Risk:** near zero. `members = ["crates/*"]` matching zero packages is valid as long as `crates/` exists (hence the `.gitkeep`); the first real member arrives in Phase 1.
