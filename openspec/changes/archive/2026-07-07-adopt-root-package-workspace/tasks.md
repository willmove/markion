# Implementation Plan: Adopt root-package workspace (Typune integration Phase 0)

## Overview

Phase 0 of `docs/typune-integration-plan.md`: give future Typune library crates a home without touching `src/` or the release pipeline. Two files carry the implementation (`Cargo.toml`, `AGENTS.md`); the rest of Phase 0 is decisions recorded in the proposal (product name stays Markion; Typune code arrives by copy with source commit noted; MIT licensing already settled). No Typune code is imported in this change.

## Tasks

- [x] 1. Workspace table (`Cargo.toml`)
  - [x] 1.1 Add `[workspace]` with `members = ["crates/*"]` to the existing root manifest (root-package form; `src/` does not move).
  - [x] 1.2 Extend the profile comments: `[profile.dev.package."*"]` does not cover workspace members — future compute-heavy member crates need explicit `[profile.dev.package.<name>]` overrides.
  - [x] 1.3 Verify `cargo metadata` reports the workspace with the root package as sole member and `cargo check` still passes.
  - [x] _Requirements: crate-architecture (root-package layout; dev-profile coverage rule documented)_

- [x] 2. Layout note (`AGENTS.md`)
  - [x] 2.1 Replace "There is no separate workspace — single crate in `Cargo.toml`" with the root-package workspace description: app crate at root, absorbed library crates under `crates/*`, `cargo test -p <member>` for member-only runs.
  - [x] 2.2 Record the two structural invariants for future phases: member crates never depend on gpui (gpui-trait types stay in the root crate), and compute-heavy members get dev-profile overrides.
  - [x] _Requirements: crate-architecture (member crates free of GUI coupling)_

- [x] 3. Plan doc status (`docs/typune-integration-plan.md`)
  - [x] 3.1 Mark Phase 0 as implemented (workspace table + AGENTS.md updated; naming/import/licensing decisions recorded in this change's proposal).

- [x] 4. Verify and finalize
  - [x] 4.1 `cargo check` passes at the repo root (root package unaffected).
  - [x] 4.2 `cargo test` passes at the repo root (102 + 6 tests, 0 failed — behavior identical to pre-workspace).
  - [x] 4.3 Confirm the empty `members` glob is accepted by cargo before any `crates/*` member exists.
