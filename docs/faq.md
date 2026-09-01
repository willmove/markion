# Markion — Frequently Asked Questions

## What is Markion?

Markion is a native desktop Markdown editor built in Rust with the [GPUI](https://github.com/zed-industries/zed) GPU-accelerated UI framework. It offers four view modes — Edit, Visual Edit, Split, and Read — plus an outline, a file-tree workspace panel, find-and-replace, focus and typewriter modes, and export to several formats.

- **License:** MIT
- **Repository:** <https://github.com/willmove/markion>
- **Issues:** <https://github.com/willmove/markion/issues>

## Supported platforms

| Platform | Target | Notes |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | Windows 10 and later; NSIS `.exe` installer |
| macOS | `aarch64-apple-darwin` | Apple Silicon native; **min macOS 11.0**; Intel Macs run via Rosetta. A universal binary is a future task. |
| Linux | `x86_64-unknown-linux-gnu` | Built on Ubuntu 22.04; ships as `.deb` and `.AppImage` |

**Releases are not platform code-signed.** On first launch you can still see Gatekeeper (macOS) or SmartScreen (Windows) warnings; bypass them manually to run Markion. The Windows x86_64 in-app updater separately verifies its NSIS payload with a Minisign key, which does not suppress SmartScreen. macOS and Linux update actions open the release download in the system browser. Linux users installing the `.deb` get the required runtime libraries (Wayland / X11 / Vulkan / fontconfig) pulled in automatically.

## Markdown support

Markion uses [`pulldown-cmark`](https://github.com/pulldown-cmark/pulldown-cmark) (CommonMark + GFM) with the following enabled:

- CommonMark baseline
- GitHub Flavored Markdown: tables, strikethrough, task lists, autolinks
- Footnotes
- Math formulas (`$inline$` and `$$block$$`)
- Mermaid diagrams (` ```mermaid ` fenced code blocks — flowcharts, sequence diagrams, and more)
- Smart punctuation (smart quotes, dashes)
- Heading attributes
- YAML front matter (`---` delimited, with `title` / `author` / `date` used by exports)
- Extended inline syntax (Markion-specific, layered on top of pulldown-cmark text runs):
  - `==highlight==`
  - `^superscript^`
  - `~subscript~`
  - emoji shortcodes such as `:smile:`, `:heart:`
  - bare autolinks

## View modes

Markion cycles through four modes with **Ctrl+Shift+V** (default is Split), or jumps straight to one with the mode shortcuts:

1. **Edit** (Ctrl+/) — raw Markdown source text only.
2. **Visual Edit** (Ctrl+E) — a WYSIWYG-first, source-backed editing surface; see [README: Editing modes](../README.md#editing-modes) for what it covers.
3. **Split** (Ctrl+P) — source on the left, rendered preview on the right (default).
4. **Read** (Ctrl+R) — rendered preview only, non-editing.

Switching modes preserves the active document, cursor/selection, undo history, and per-tab scroll state.

## Math

Markion typesets math on screen (Split/Read preview and Visual Edit) with an embedded RaTeX engine (KaTeX-compatible, bundled fonts) that renders `$inline$` and `$$block$$` formulas into cached SVG — no network access and no external LaTeX install required. LaTeX export keeps the native `$...$`/`$$...$$` source for the reader's own toolchain to typeset. The built-in DOCX export fallback (used only when pandoc is unavailable) still degrades formulas to a readable Unicode plain-text approximation rather than embedding typeset glyphs.

## Export

Markion exports to eight formats. Run an export from the **Export** menu or with a keyboard shortcut (see [Keyboard Shortcuts](keyboard-shortcuts.md)).

| Format | Backend |
|---|---|
| Markdown | built-in |
| HTML (styled) | built-in |
| Plain HTML | built-in |
| LaTeX | built-in (rich inline styling, table alignment, `lstlisting` code blocks) |
| PDF | pandoc engine first, falls back to the built-in `markion-pdf` writer |
| DOCX | pandoc engine first, falls back to built-in |
| PNG | built-in (basic text snapshot) |
| JPEG | built-in (basic text snapshot) |

**PDF and DOCX** try the pandoc engine first (a subprocess invoking [`pandoc`](https://pandoc.org/)). If pandoc (or the PDF engine, default `xelatex`) is unavailable or fails, Markion **silently falls back** to the built-in writer so export always succeeds. The status bar message after export discloses which backend produced the file. For PDF, the built-in fallback is now a full layout engine (krilla + cosmic-text) that supports CJK and Latin typography, embedded images, syntax-highlighted code blocks, tables, and footnotes, so the built-in message is shown neutrally. For DOCX, the built-in writer is still simpler, and its status message continues to hint that installing pandoc yields richer output.

You can change the PDF engine and tune the built-in PDF output via the config file:

```toml
[export]
pdf_engine = "xelatex"   # alternatives: "pdfroff", "tectonic", ...

[export.pdf]
page_size = "a4"      # "a4" | "letter" | "legal"
margin_mm = 25        # page margin in millimetres
toc = false           # table of contents
page_numbers = true   # footer page numbers
```

**DOCX export options.** File → Export → DOCX opens a short options step before the save dialog: page size (A4 / Letter / Legal), table of contents (offered only when the pandoc engine is available — the built-in writer ignores it), and image policy (embed local images into the document, or export them as `alt: url` text). Both backends honor the page size and image policy; the table of contents is emitted only by the pandoc engine (`--toc`). After a successful export the choices are remembered as the defaults for the next run, persisted in the config file:

```toml
[export.docx]
page_size = "a4"              # "a4" | "letter" | "legal"
toc = false                   # table of contents (pandoc engine only)
image_policy = "embed"        # "embed" | "text-fallback"
```

Two more DOCX-related keys live in the `[export]` section: `pandoc_path` (explicit pandoc binary; unset = system `PATH`) and `reference_doc` (a pandoc `--reference-doc` styling template; unset or missing files fall back to the bundled template):

```toml
[export]
pandoc_path = "C:/tools/pandoc.exe"       # optional
reference_doc = "templates/my-style.docx" # optional
```

The built-in PNG/JPEG paths are deliberately basic text snapshots. The built-in PDF writer is now a rich fallback; for DOCX-specific pandoc customization or advanced PDF templating, install pandoc plus a LaTeX engine (e.g. `xelatex` from TeX Live / MiKTeX, or `pdfroff` from groff).

YAML front matter `title` / `author` / `date` feed into export metadata: HTML `<meta>` tags, DOCX document properties, and the LaTeX preamble.

## Auto-save and crash recovery

Markion writes a crash-recovery snapshot after a period of inactivity (default **5 seconds**). For documents that already have a file path, it can also **silently save back to that file**. Both behaviors share the same idle timer.

Configure from **Preferences → General → Auto-save**, or in `config.toml`:

```toml
[auto_save]
enabled = true        # master switch (file only): false disables timer, recovery, and write-back
silent_save = true    # write named documents back to their path (Preferences toggle)
delay_secs = 5        # idle interval in seconds, 1–300 (Preferences stepper)
```

- **`silent_save = true` (default):** after recovery is written, named files are also saved to disk and the dirty marker clears when nothing raced the write.
- **`silent_save = false`:** Markion still writes/updates a recovery snapshot, but **does not** overwrite the original file. The tab stays dirty (`*` in the title) until you save manually. Use this when you want crash protection without silent overwrites.
- **`enabled = false`:** nothing automatic runs (no recovery, no write-back). Edit `config.toml` for this; it is not shown in Preferences. Manual Save is unaffected.

Untitled documents always use recovery copies only. If Markion exits unexpectedly, the next launch offers to restore work from the recovery directory.

The title bar shows a `*` suffix next to the file name when there are unsaved changes.

## Themes

Markion ships **fourteen built-in themes**: the original six (Paper, Ink, Solar, Forest, Rose, Graphite) plus GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light. Pick one in **Preferences → Theme** (Ctrl+,) by swatch.

### Custom themes

Author custom themes as TOML files in the themes directory (see [Configuration locations](#configuration-locations) below). Each file needs a `name`, an `is_dark` flag, and a `[colors]` sub-table with eight color keys:

```toml
name = "Midnight"
is_dark = true

[colors]
app_bg      = "#10131a"
panel_bg    = "#171b24"
surface_bg  = "#0f1720"
text        = "#e5edf5"
muted       = "#91a4b7"
border      = "#2b3544"
active_bg   = "#23304a"
active_text = "#9ec5ff"
```

Color values accept either `"#rrggbb"` or bare `"rrggbb"`. Any color you omit falls back to the default palette, so a partial file still loads. To activate a custom theme, set `theme = "Midnight"` in `config.toml` (using the theme's `name`, not the file stem).

**Legacy `.theme` files migrate automatically:** if a `.theme` file from an older Markion build exists with no `.toml` of the same stem, Markion parses it once, writes the equivalent `.toml` next to it, and leaves the original `.theme` in place (ignored thereafter).

## Configuration locations

Markion stores its configuration under platform-standard directories.

| Platform | Config dir (`config.toml`, themes/) | Recovery dir | Logs dir |
|---|---|---|---|
| **Windows** | `%APPDATA%\Markion\` | `%LOCALAPPDATA%\Markion\Recovery\` | `%LOCALAPPDATA%\Markion\Logs\` |
| **macOS** | `~/.config/markion/` (XDG) | `~/.cache/markion/recovery/` | `~/Library/Logs/Markion/` |
| **Linux** | `~/.config/markion/` (XDG) | `~/.cache/markion/recovery/` | `~/.cache/markion/logs/` |

> Note: on macOS, only the logs use `~/Library/...`; config and recovery follow XDG conventions (`~/.config/markion`, `~/.cache/markion`).

### `config.toml`

The complete supported schema (all fields optional, defaults shown):

```toml
theme = "Paper"                   # built-in or custom theme name
# custom_theme = "Midnight"      # a custom theme name (optional)
language = "en"                   # en, zh-hans, zh-hant, ja, fr, de, es
focus_mode = false
typewriter_mode = false
code_line_numbers = true
preview_adaptive_width = false
heading_menu_max_level = 5        # 5 or 6
sync_scroll = false
sidebar_visible = true
sidebar_tab = "files"             # "files" or "outline"
show_hidden_files = false

# Optional font families per plane; absent = follow the theme, then the
# built-in default (system UI font for source/reading, JetBrains Mono for code).
# editor_font_family = "Cascadia Code"
# rendered_font_family = "Georgia"
# code_font_family = "JetBrains Mono"

[auto_save]
enabled = true
silent_save = true
delay_secs = 5

[export]
pdf_engine = "xelatex"

# Menu-action shortcut overrides: action id -> GPUI keystroke string. Actions
# without an entry keep their default binding. See Keyboard Shortcuts.
# [shortcuts]
# "toggle-sidebar" = "ctrl-alt-b"
```

A legacy `preferences.conf` from a pre-TOML Markion build is migrated to `config.toml` on first launch and left in place.

> Font families (per source/reading/code plane) and menu-action keybindings (the `[shortcuts]` table, also editable from the Preferences panel) **are** configurable. Markion does **not** support a toggle for footnotes — footnotes are always enabled. If a guide mentions `enable_footnotes`, it does not apply to Markion.

## Large-document performance

Markion caches derived document state (preview blocks, outline, statistics, syntax highlighting) per document version and shares it via `Arc`, so typing in large documents does not re-derive everything on every keystroke. The syntect grammar registry is loaded off the main thread at startup so first render stays responsive. Source-mapped Visual Edit incrementally reuses independently parseable regions after localized edits, falling back to a full derivation whenever Markdown context or byte ranges are uncertain; Split/Read preview derivation stays debounced and cached rather than incremental. Markion still uses a `String` buffer rather than a rope, and some semantic reads intentionally require a full parse.

## Troubleshooting

- **macOS says Markion "can't be opened because it is from an unidentified developer."** This is Gatekeeper. Right-click the app and choose **Open**, or in *System Settings → Privacy & Security* click **Open Anyway**. Releases are unsigned.
- **Windows SmartScreen warns before running the installer.** Click **More info → Run anyway**. Releases are unsigned.
- **PDF export produced a tiny, plain-looking file.** The built-in PDF writer (a text snapshot) was used because pandoc or its PDF engine was not installed. Install [pandoc](https://pandoc.org/) and a LaTeX engine (e.g. `xelatex`), or set `[export] pdf_engine = "pdfroff"` and install groff, then re-export — the status bar will say "pandoc engine" when the richer path succeeds.
- **A custom theme is not appearing in Preferences.** Confirm the `.toml` file is in the themes directory (see [Configuration locations](#configuration-locations)), that its `name` field is set and non-empty, and that no built-in theme has the same name (built-ins take precedence).
- **Where are the logs?** See the Logs dir column in [Configuration locations](#configuration-locations). Set `RUST_LOG=debug` before launching to increase verbosity.

## Reporting bugs

Please file issues at <https://github.com/willmove/markion/issues>. Including the Markion version (shown in the first log line on startup) and the platform helps.
