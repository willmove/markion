## Why

Markion's `docs/` directory contains only engineering notes (`typune-integration-*.md`). It has no end-user documentation. Typune shipped user-facing docs under `docs/`; two of them (`faq.md`, `keyboard-shortcuts.md`) describe features Markion genuinely has and are the highest-value ports. The other five Typune docs (`architecture.md`, `building-linux.md`, `building-macos.md`, `packaging.md`, `theme-customization.md`) describe Typune's winit/rfd/GTK toolchain and 6-theme TOML+CSS model, neither of which matches Markion's GPUI stack and 14-theme model — rewriting them from scratch would cost more than it saves, so they are not ported in this change.

Every fact in the two ported docs was verified against Markion's source (paths.rs, model.rs, parse.rs, main.rs, packager.toml, release.yml). Typune-specific inaccuracies corrected: 8 s → 5 s auto-save; MIT/Apache → MIT; KaTeX → Unicode fallback; `Ctrl+Shift+M` view toggle → `Ctrl+Shift+V`; macOS `~/Library/...` config paths → XDG (`~/.config/markion`); GTK3 Linux deps → Wayland/X11/Vulkan; Intel+ARM macOS → arm64-only + Rosetta; 6 Typune themes → 14 Markion themes; non-existent config keys (`[font]`, `[keybindings]`, `enable_footnotes`) removed; `.theme` → `.toml` theme format.

## What Changes

- **New `docs/faq.md`** — end-user FAQ: what Markion is, supported platforms (with the unsigned-build caveat), Markdown feature coverage (including Markion's `==highlight==` / `^sup^` / `~sub~` / `:emoji:` extended inline syntax), view modes, math (Unicode fallback, not KaTeX), all 8 export formats with the pandoc-engine-with-fallback disclosure and configurable `pdf_engine`, auto-save/recovery (5 s default), themes (14 built-ins + TOML custom themes with legacy `.theme` migration), config locations (platform-accurate), the full `config.toml` schema, large-document performance note, and troubleshooting.
- **New `docs/keyboard-shortcuts.md`** — every binding verified against `src/main.rs` `cx.bind_keys(...)` (line 5783-5856). Includes Markion-only bindings the Typune doc lacked (Save As, heading 1/2/3, indent/outdent, all 8 export shortcuts, file-tree actions, outline toggle, cycle theme, find next/prev, preferences, show shortcuts, quit). Removes Typune's `[keybindings]` custom-config section (Markion has no such feature) and the `Ctrl+W` close / `Ctrl+D` strikethrough / `Ctrl+G` goto-line bindings (not registered in Markion).

## Capabilities

This is a docs-only change; it adds no system capability and does not modify any spec. Archive with `--skip-specs`.

## Impact

- New files only: `docs/faq.md`, `docs/keyboard-shortcuts.md`. No code changes.
- The Typune docs at `C:\Coding\EditorProjects\typune\docs\` are the research source; they are not copied verbatim — each claim was verified against Markion's source and corrected.
