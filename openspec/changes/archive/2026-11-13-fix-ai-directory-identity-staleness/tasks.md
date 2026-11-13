## 1. Repair directory validation

- [x] 1.1 Add a directory-only volume/object-id comparison and use it for read-grant and journal roots/scopes, preserving file checks and journal schema.
- [x] 1.2 Cover child-directory changes and persisted-journal recovery; verify replacement, symlink and hard-link denial remains enforced.

## 2. Verify

- [x] 2.1 Run the AI crate, root journal regressions, formatting, strict OpenSpec validation and the complete workspace suite; document the release-blocking fix.

Verification (2026-11-13): AI crate 40 passed; full workspace 2136 passed / 5 existing ignored / 0 failed; cargo check, cargo clippy, Rustfmt and strict OpenSpec validation passed. The SVG spike fixture now explicitly loads bundled Libertinus fonts instead of requiring host Georgia; application font behavior is unchanged.
