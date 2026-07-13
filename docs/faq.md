# Markion — Frequently Asked Questions

## What is Markion?

Markion is a native desktop Markdown editor built in Rust with the [GPUI](https://github.com/zed-industries/zed) GPU-accelerated UI framework. It offers three view modes — Source, Split, and Preview — plus an outline, a file-tree workspace panel, find-and-replace, focus and typewriter modes, and export to several formats.

- **License:** MIT
- **Repository:** <https://github.com/willmove/markion>
- **Issues:** <https://github.com/willmove/markion/issues>

## Supported platforms

| Platform | Target | Notes |
|---|---|---|
| Windows | `x86_64-pc-windows-msvc` | Windows 10 and later; NSIS `.exe` installer |
| macOS | `aarch64-apple-darwin` | Apple Silicon native; **min macOS 11.0**; Intel Macs run via Rosetta. A universal binary is a future task. |
| Linux | `x86_64-unknown-linux-gnu` | Built on Ubuntu 22.04; ships as `.deb` and `.AppImage` |

**All releases are currently unsigned.** On first launch you will see Gatekeeper (macOS) or SmartScreen (Windows) warnings; bypass them manually to run Markion. Linux users installing the `.deb` get the required runtime libraries (Wayland / X11 / Vulkan / fontconfig) pulled in automatically.

## Markdown support

Markion uses [`pulldown-cmark`](https://github.com/pulldown-cmark/pulldown-cmark) (CommonMark + GFM) with the following enabled:

- CommonMark baseline
- GitHub Flavored Markdown: tables, strikethrough, task lists, autolinks
- Footnotes
- Math formulas (`$inline$` and `$$block$$`)
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

Markion cycles through three modes with **Ctrl+Shift+V** (default is Split):

1. **Source** — raw Markdown text only.
2. **Split** — source on the left, rendered preview on the right (default).
3. **Preview** — rendered preview only.

## Math

Markion **does not** render math graphically (no KaTeX / MathJax). Formulas are validated for brace and environment balance, then displayed as a readable Unicode plain-text approximation: Greek letters (`\alpha` → α), common operators (`\sum` → ∑, `\sqrt` → √, `\times` → ×), and simple fractions (`\frac{a}{b}` → a⁄b). Nothing extra needs to be installed.

## Export

Markion exports to eight formats. Run an export from the **Export** menu or with a keyboard shortcut (see [Keyboard Shortcuts](keyboard-shortcuts.md)).

| Format | Backend |
|---|---|
| Markdown | built-in |
| HTML (styled) | built-in |
| Plain HTML | built-in |
| LaTeX | built-in (rich inline styling, table alignment, `lstlisting` code blocks) |
| PDF | pandoc engine first, falls back to built-in |
| DOCX | pandoc engine first, falls back to built-in |
| PNG | built-in (basic text snapshot) |
| JPEG | built-in (basic text snapshot) |

**PDF and DOCX** try the pandoc engine first (a subprocess invoking [`pandoc`](https://pandoc.org/)). If pandoc (or the PDF engine, default `xelatex`) is unavailable or fails, Markion **silently falls back** to a simpler built-in writer so export always succeeds. The status bar message after export discloses which backend produced the file — pandoc engine or built-in — and the built-in message hints that installing pandoc yields richer output.

You can change the PDF engine via the config file:

```toml
[export]
pdf_engine = "xelatex"   # alternatives: "pdfroff", "tectonic", ...
```

The built-in PDF/PNG/JPEG paths are deliberately basic (text snapshots). For high-quality output, install pandoc plus a LaTeX engine (e.g. `xelatex` from TeX Live / MiKTeX, or `pdfroff` from groff).

YAML front matter `title` / `author` / `date` feed into export metadata: HTML `<meta>` tags, DOCX document properties, and the LaTeX preamble.

## Auto-save and crash recovery

Markion auto-saves your document after a period of inactivity. Defaults: **enabled, 5 second delay**. Configure in `config.toml`:

```toml
[auto_save]
enabled = true
delay_secs = 5
```

For documents that have never been saved to a file, Markion writes a recovery copy to the recovery directory. If Markion exits unexpectedly, the next launch offers to restore the unsaved work from that copy.

The title bar shows a `*` suffix next to the file name when there are unsaved changes.

## Themes

Markion ships **fourteen built-in themes**: the original six (Paper, Ink, Solar, Forest, Rose, Graphite) plus GitHub Light/Dark, Solarized Light/Dark, One Light/Dark, and Tokyo Night/Light. Pick one in the **Preferences** panel (Ctrl+,) by swatch.

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
theme = "Paper"                # built-in or custom theme name
# custom_theme = "Midnight"   # a custom theme name (optional)
language = "en"                # "en" or "zh"
focus_mode = false
typewriter_mode = false
code_line_numbers = true
sidebar_visible = true
sidebar_tab = "files"          # "files" or "outline"

[auto_save]
enabled = true
delay_secs = 5

[export]
pdf_engine = "xelatex"
```

A legacy `preferences.conf` from a pre-TOML Markion build is migrated to `config.toml` on first launch and left in place.

> Markion does **not** support font configuration, custom keybindings, or a toggle for footnotes. If a guide mentions `[font]`, `[keybindings]`, or `enable_footnotes`, it does not apply to Markion.

## Large-document performance

Markion caches derived document state (preview blocks, outline, statistics, syntax highlighting) per document version and shares it via `Arc`, so typing in large documents does not re-derive everything on every keystroke. The syntect grammar registry is loaded off the main thread at startup so first render stays responsive. There is no separate incremental parser or rope text buffer; the whole document is re-parsed, but memoized per version, which is sufficient for realistic note sizes.

## Troubleshooting

- **macOS says Markion "can't be opened because it is from an unidentified developer."** This is Gatekeeper. Right-click the app and choose **Open**, or in *System Settings → Privacy & Security* click **Open Anyway**. Releases are unsigned.
- **Windows SmartScreen warns before running the installer.** Click **More info → Run anyway**. Releases are unsigned.
- **PDF export produced a tiny, plain-looking file.** The built-in PDF writer (a text snapshot) was used because pandoc or its PDF engine was not installed. Install [pandoc](https://pandoc.org/) and a LaTeX engine (e.g. `xelatex`), or set `[export] pdf_engine = "pdfroff"` and install groff, then re-export — the status bar will say "pandoc engine" when the richer path succeeds.
- **A custom theme is not appearing in Preferences.** Confirm the `.toml` file is in the themes directory (see [Configuration locations](#configuration-locations)), that its `name` field is set and non-empty, and that no built-in theme has the same name (built-ins take precedence).
- **Where are the logs?** See the Logs dir column in [Configuration locations](#configuration-locations). Set `RUST_LOG=debug` before launching to increase verbosity.

## Reporting bugs

Please file issues at <https://github.com/willmove/markion/issues>. Including the Markion version (shown in the first log line on startup) and the platform helps.
