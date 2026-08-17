# Proposal: open-documents-in-current-tab

## Why

Today only File → Open replaces the active tab; the high-frequency gestures — file-tree click, drag-and-drop, Open Recent — each append a new tab, so browsing a folder litters the tab bar and retains a dormant tab per visited file (document text plus undo history per tab). Users coming from Typora or VS Code preview mode expect the current tab to be reused unless they explicitly ask for a new one, and Markion has no way to configure this.

## What Changes

- Introduce a persisted boolean preference **"Open documents in current tab"** (`open_in_current_tab` in `config.toml`), **default on**. This is a **BREAKING** default change: file-tree clicks, drag-drop, and Open Recent that previously appended tabs now replace the current tab when that is safe.
- All *non-explicit* open entries follow the preference: File → Open, file-tree click and its context-menu "Open", drag-and-drop of supported documents, Open Recent.
- Safety rule when the preference is on: an open may replace the active tab only if it is an image tab, a clean document tab, or the untitled/welcome tab. A **dirty** editable active tab is never silently replaced — gesture opens (tree/drop/recent) silently divert to a new tab; File → Open keeps its existing dirty-guard confirmation.
- Explicit new-tab affordances are unchanged and always append: File → Open in New Tab, tab-bar "+", Ctrl+T, and the file-tree context-menu "Open in New Tab".
- Add **Ctrl/Cmd+click** on a file-tree row as a per-click escape hatch that forces a new tab while the preference is on.
- Multi-file drag-drop under the preference: the first file follows the default rule; each subsequent file appends its own tab.
- Already-open files still dedupe to (focus) their existing tab regardless of the preference.
- Preferences panel gains a toggle in the General tab's "Other" section; preference persists safely (missing/invalid field falls back to the default) and participates in reset.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-editing`: the Multi-document tab model requirement gains the default open-target rule — which entries are non-explicit, when replacement is allowed (clean/image/untitled active tab), the dirty-tab diversion rule, the Ctrl/Cmd+click force-new-tab gesture, and multi-drop ordering; File→Open and tab-creation scenarios are reworded to follow the preference.
- `workspace`: file-tree opening scenarios ("Plain-text file opens from the tree", "Image file opens from the tree") no longer hard-code "in its own tab"; they defer to the default open-target rule and cover the Ctrl/Cmd+click override.
- `image-file-viewing`: the routing requirement's file-tree / Open Recent scenarios follow the default open-target rule instead of always appending an image tab.
- `theme-preferences`: adds the Preferences-panel toggle requirement and the safe-persistence requirement for `open_in_current_tab` (default on).

## Impact

- **Code**: `src/app/workspace.rs` (`OpenPathIntent` dispatch, `open_file_in_new_tab_from_path`, `handle_external_drop`), `src/app/application.rs` (`open_tree_file`, `open_recent_path`, preference plumbing in `new` / `current_preferences`), `src/app/documents.rs` (File→Open intent selection), `src/app/root_view.rs` (file-tree click modifier handling, Preferences "Other" row), `src/app/appearance.rs` (toggle handler), `src/model.rs` + `src/storage/preferences.rs` (field, default, round-trip), `src/i18n.rs` (label + status strings in all seven languages).
- **Invariants touched**: replacement must keep deleting the replaced tab's recovery snapshot only when that snapshot belongs to a *clean discard* — a dirty tab is never replaced by a preference-driven open, so no unsaved work or recovery snapshot is lost. Tab dormancy on switch/open (derived-cache eviction, image-claim release) is unchanged and still applies whenever a new tab *is* appended.
- **Spec-tree interaction**: the unarchived `add-drag-drop-open` change's delta hard-codes "open each dropped path … as a new tab". This change updates that delta's wording to the preference-driven rule so the two changes cannot archive into contradictory specs.
- **Memory**: modest — dormant tabs already evict derived caches (`evict-inactive-tab-caches`); what this removes per avoided tab is document text plus undo history (roughly a small multiple of the text size) and tab-handle overhead. The primary user-visible win is tab-bar hygiene while browsing.
- **Release notes**: default behavior change for existing users must be called out (how to restore prior behavior: toggle off).

## Non-goals

- No tab-count limits, LRU tab eviction, or undo-history trimming (dormancy already bounds the big caches).
- No "ask every time" mode (enum extension can come later if needed).
- No changes to session restore, crash recovery, CLI startup opening, or File → New.
- No splitting the window or preview-tab behavior changes.
