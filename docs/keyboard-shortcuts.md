# Keyboard Shortcuts

Markion's keyboard shortcuts are registered with platform-aware modifiers. On Windows and Linux the primary modifier is **Ctrl**; on macOS it is **Cmd** (⌘). In the tables below, `Ctrl` stands for that platform modifier. Bindings that include `Alt` use **Option** (⌥) on macOS.

> Every action below (except raw navigation/editing keys like arrows, Backspace, and Tab) can be rebound in **Preferences → Shortcuts**, or by editing the `[shortcuts]` table in `config.toml` (action id → GPUI keystroke string). The bindings shown here are the defaults.

## Notation

| Symbol | Meaning |
|---|---|
| **Ctrl** | `Ctrl` on Windows/Linux, `Cmd` (⌘) on macOS |
| **Alt** | `Alt` on Windows/Linux, `Option` (⌥) on macOS |
| **Shift** | `Shift` |
| `+` | hold the keys together |

## File

| Action | Shortcut |
|---|---|
| New document | Ctrl+N |
| Open document | Ctrl+O |
| Save | Ctrl+S |
| Save As | Ctrl+Shift+S |
| Preferences | Ctrl+, |
| Quit | Ctrl+Q |

## Tabs

| Action | Shortcut |
|---|---|
| New tab (empty) | File → New Tab, or the **+** button on the tab bar |
| Open file in new tab | Ctrl+T |
| Close tab | Ctrl+W |
| Next / previous tab | Ctrl+Tab / Ctrl+Shift+Tab |

## Text formatting

| Action | Shortcut |
|---|---|
| Bold | Ctrl+B |
| Italic | Ctrl+I |
| Inline code | Ctrl+E |
| Insert link | Ctrl+K |
| Insert image | Ctrl+Shift+I |
| Heading 1–6 | Ctrl+1 through Ctrl+6 |
| Indent | Tab |
| Outdent | Shift+Tab |

## Editing

| Action | Shortcut |
|---|---|
| Undo | Ctrl+Z |
| Redo | Ctrl+Y |
| Cut / Copy / Paste | Ctrl+X / Ctrl+C / Ctrl+V |
| Select all | Ctrl+A |
| Line start / end | Home / End |

## View

| Action | Shortcut |
|---|---|
| Cycle view mode (Edit → Visual Edit → Split → Read) | Ctrl+Shift+V |
| Edit mode (source only) | Ctrl+Alt+1 |
| Visual Edit mode (WYSIWYG-first, source-backed) | Ctrl+Alt+4 |
| Split mode (source + preview) | Ctrl+Alt+2 |
| Read mode (preview only) | Ctrl+Alt+3 |
| Toggle sidebar | Ctrl+Shift+B |
| Toggle focus mode | F7 |
| Toggle typewriter mode | F8 |
| Toggle code line numbers | Ctrl+Shift+4 |
| Cycle theme | Ctrl+Shift+T |

## Sidebar — Files panel

| Action | Shortcut |
|---|---|
| Toggle file tree | Ctrl+Shift+F |
| Focus file-tree search | Ctrl+Alt+F |
| Clear file-tree search | Esc |
| Refresh file tree | F5 |
| New file in tree | Ctrl+Alt+N |
| New folder in tree | Ctrl+Alt+Shift+N |
| Rename tree entry | F2 |
| Delete tree entry | Ctrl+Delete |
| Open clicked file in a new tab | Ctrl+click (Cmd+click on macOS) |

## Sidebar — Outline panel

| Action | Shortcut |
|---|---|
| Toggle outline | F6 |

## Find & replace

| Action | Shortcut |
|---|---|
| Find | Ctrl+F |
| Replace | Ctrl+H |
| Find next / previous | F3 / Shift+F3 |
| Next / previous while a search field is focused | Enter / Shift+Enter |
| Move between available Find/Replace fields | Tab / Shift+Tab |
| Close Find/Replace | Esc |

Find searches authored Markdown in Edit, Visual Edit, and Split modes. In Read mode it searches only the rendered visible text (for example, a link label but not its hidden destination). Invoking Replace in Read mode keeps the query, replacement value, options, and requested Replace form, but presents Find-only controls because rendered content cannot be mutated; the Replace row returns when an editable mode is restored.

## Tables

| Action | Shortcut |
|---|---|
| Format table (align columns) | Ctrl+Shift+M |
| Add row | Ctrl+Alt+Enter |
| Delete row | Ctrl+Alt+Backspace |
| Move row up / down | Ctrl+Alt+Up / Ctrl+Alt+Down |
| Add column | Ctrl+Alt+Right |
| Delete column | Ctrl+Alt+Left |

## Export

| Action | Shortcut |
|---|---|
| Export to HTML (styled) | Ctrl+Shift+H |
| Export to plain HTML | Ctrl+Alt+Shift+H |
| Export to PDF | Ctrl+Shift+P |
| Export to LaTeX | Ctrl+Shift+L |
| Export to DOCX | Ctrl+Shift+D |
| Export to PNG | Ctrl+Shift+G |
| Export to JPEG | Ctrl+Alt+Shift+G |

## Help

| Action | Shortcut |
|---|---|
| Show this shortcut reference | F1 |

---

## Notes

- The default view mode is **Split** (source + preview side by side). Cycle through Edit → Visual Edit → Split → Read with Ctrl+Shift+V, or jump straight to a mode with Ctrl+Alt+1/4/2/3. Preview width in Read mode follows the "adaptive preview width" preference (File → Preferences).
- Opening a file — from the file tree, drag-and-drop, Open Recent, or File → Open — targets the current tab when that is safe (an image tab, the untitled document, or an unmodified document) and otherwise opens a new tab, following the "Open documents in current tab" preference (File → Preferences, on by default). A dirty document is never silently replaced, and Ctrl+click in the file tree always opens a new tab.
- Focus mode dims paragraphs outside the current one; typewriter mode keeps the cursor line vertically centered.
- A few bindings combine `Ctrl` with an `Alt` modifier (for example Ctrl+Alt+N for "new file in tree"); on macOS these are **Cmd+Option**.
- If a binding does not behave as expected, check whether your window manager or OS has reserved it.
