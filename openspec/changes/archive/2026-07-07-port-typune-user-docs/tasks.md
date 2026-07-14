# Implementation Plan: Port Typune user docs to Markion

## Overview

Adapt the two highest-value Typune user docs (faq.md, keyboard-shortcuts.md) into Markion's `docs/`, with every fact verified against Markion's source. Docs-only change; archive with `--skip-specs`.

## Tasks

- [x] 1. Fact-mapping: verify every Typune doc claim against Markion source (paths.rs, model.rs, parse.rs, main.rs, packager.toml, release.yml).
  - [x] 1.1 Corrected: 8 s → 5 s auto-save; MIT/Apache → MIT; KaTeX → Unicode fallback; view-toggle key; macOS paths; GTK3 → Wayland/X11/Vulkan; Intel+ARM → arm64-only; 6 themes → 14; non-existent config keys removed; `.theme` → `.toml`.
  - [x] 1.2 Confirmed Markion DOES support `==highlight==` / `^sup^` / `~sub~` / `:emoji:` extended inline syntax (src/parse.rs:146-192) — keep these in the FAQ.

- [x] 2. `docs/keyboard-shortcuts.md`
  - [x] 2.1 Every shortcut verified against `cx.bind_keys` (main.rs:5783-5856).
  - [x] 2.2 Removed Typune-only bindings (Ctrl+W, Ctrl+D, Ctrl+G, `[keybindings]` section); added Markion-only bindings (Save As, headings, indent, exports, file-tree, outline, cycle theme, find next/prev, preferences, shortcuts, quit).

- [x] 3. `docs/faq.md`
  - [x] 3.1 Sections: overview/license, platforms (unsigned caveat, arm64+Rosetta, Linux Wayland stack), Markdown support (incl. extended inline), view modes, math (Unicode fallback), export (8 formats + pandoc/fallback + pdf_engine), auto-save/recovery (5 s), themes (14 + TOML custom + `.theme` migration), config locations (platform-accurate), config.toml schema, large-doc performance, troubleshooting.

- [x] 4. Verification
  - [x] 4.1 No Typune brand residue; no claims contradict Markion source.
  - [x] 4.2 `openspec validate port-typune-user-docs` passes (docs-only; `--skip-specs` on archive).
