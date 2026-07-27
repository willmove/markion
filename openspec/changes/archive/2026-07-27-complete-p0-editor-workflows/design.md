## Context

`MarkdownDocument.text` is Markion's only canonical state. Every editing surface maps back to exact UTF-8 source ranges and uses the tab's semantic undo/autosave path. Images are already parsed and rendered, including explicit failed preview states, but an image block is currently a conservative source island. GPUI 0.2.2 exposes clipboard image bytes and OS drops as `ExternalPaths`. File persistence currently calls `fs::write`; documents do not remember an on-disk identity, so neither autosave nor an explicit save can detect an intervening external write.

The P0 work crosses the pure document model, filesystem helpers, tab state, GPUI event routing, overlays, and session/recovery persistence. The design therefore keeps filesystem and Markdown transformations as independently tested pure or GPUI-free modules, with thin UI orchestration.

## Goals / Non-Goals

**Goals:**

- Make pasted/dropped local images portable document resources with deterministic safe relative links.
- Edit image/link metadata and visual formatting through atomic canonical-source mutations.
- Prevent torn saves and silent overwrites after external changes.
- Restore useful dirty work after interruption without overriding a newer source file.
- Preserve exact source mapping, undo/redo, IME, tab isolation, and per-version cache identity.

**Non-Goals:**

- Remote upload, credential management, binary asset synchronization, or garbage collection of unreferenced assets.
- A mutable rendered DOM/rich-text tree or inferred edits against rendered output.
- Three-way merge or OS-specific native file-watcher dependencies.

## Decisions

### Resource destination and Markdown form

Named documents store imported images in a sibling `<document-stem>.assets/` directory. Untitled documents cannot produce a stable portable relative link, so image insertion first asks the user to save the Markdown document. Asset filenames use a sanitized source stem plus a short content hash and retain a recognized extension; an existing byte-identical target is reused and a true collision receives a numeric suffix. Links always use forward slashes, percent-encode characters that would make the Markdown destination ambiguous, and never contain `..` traversal.

The canonical source stays ordinary `![alt](relative/path "title")`. Presentation controls use an optional, Markion-recognized title suffix (`{width=25|50|75|100 align=left|center|right}`) while retaining any authored title before the suffix. This remains valid CommonMark, degrades to harmless title text elsewhere, and avoids an HTML-only representation. The parser strips the suffix only for presentation/caption purposes; source/export round-trips remain exact.

Alternative considered: a shared workspace-level `assets/` directory. A document-sibling directory is chosen because Save As, moving one document, and similarly named notes are less collision-prone and the link remains locally understandable.

### One atomic source mutation per command

Resource insertion, image replacement, presentation changes, and link-editor submission each produce one replacement string and apply it through the existing tab snapshot/commit path. Link/image parsing returns an exact source range plus decoded fields only when the construct is unambiguous. Ambiguous reference-style or malformed constructs retain their source-island fallback.

The contextual toolbar is presentation-only state derived from a non-empty Visual Edit selection or an exact link/image focus range. Opening, hovering, or canceling it does not change document version or derived caches. Existing format actions remain the mutation implementation for bold/italic/code; the link editor uses a dedicated exact replacement so label, destination, and title change together.

### Atomic write and disk identity

A GPUI-free storage helper writes to a uniquely named temporary file in the target directory, flushes it, atomically replaces the destination, and best-effort syncs the parent directory where supported. Cleanup removes the temporary file on every failure path. Save As uses the same primitive.

Each named document stores a `DiskIdentity` captured after open/save: modified time when available, length, and a content digest. Cheap metadata equality avoids rereading unchanged files; a metadata difference is confirmed by digest so timestamp-only touches do not create conflicts. Before writing, save compares the current disk identity with the stored identity. A mismatch returns a typed conflict and does not mutate path, dirty state, recovery state, or history.

Alternative considered: timestamp-only comparison. It is rejected because timestamp resolution and metadata-only tools create both missed changes and false conflicts.

### External-change observation and conflict state

A low-frequency GPUI timer checks named tabs without adding a platform watcher dependency. Clean tabs reload changed disk text into the same tab and reset source-derived interaction state; dirty tabs record a conflict banner/dialog state and stop autosave from overwriting the file. The user can reload from disk (discarding the local dirty buffer after confirmation), overwrite explicitly (atomic forced save), or Save As a copy. Deleted files are treated as conflicts for dirty tabs and as a retained in-memory document for clean tabs.

### Recovery lifecycle

Autosave always writes a recovery snapshot for dirty tabs first. For named documents it then attempts a conflict-checked file save only when autosave is enabled; the recovery snapshot is retired only after that save succeeds. Recovery records gain a stable tab identifier, original path, captured disk identity, and update timestamp. Startup restores recovery when no original exists or the recovery is newer/dirty relative to the recorded identity; when disk diverged, it opens the recovered content as an explicitly dirty recovery tab rather than replacing disk state.

## Data flow and cache boundaries

```text
clipboard/drop bytes -> resource store -> relative Markdown
                                      -> one tab mutation -> document version++
selection toolbar/link editor --------------------------^

disk poll -> DiskIdentity compare -> clean reload OR conflict state
explicit/autosave -> recovery snapshot -> conflict check -> atomic replace
```

Resource I/O and disk polling never derive preview/outline/stats. Only a successful source mutation or reload changes `MarkdownDocument.text` and invalidates versioned caches. Toolbar/dialog/conflict presentation does not change the document version. Undo snapshots continue to omit derived caches.

## Risks / Trade-offs

- [A crash between asset copy and Markdown insertion can leave an unreferenced file] → Copy first because a dangling extra asset is safer than a broken reference; no automatic asset deletion is attempted.
- [The title-suffix presentation convention may be visible in other Markdown renderers] → Keep it optional, compact, and valid CommonMark; do not alter existing authored images until a control is used.
- [Polling detects changes after a short delay] → Always repeat the identity check synchronously before every save, which is the data-loss boundary.
- [Atomic replacement semantics differ across platforms] → Keep the helper platform-aware, same-directory, and covered by replacement/failure tests; never clear dirty state before success.
- [Recovery volume can grow after repeated crashes] → Use stable per-tab snapshot paths and replace them atomically rather than creating one timestamped file per edit.

## Migration Plan

No Markdown migration is required. Existing documents and legacy recovery-v1 files remain readable. New recovery records use v2 metadata and coexist with v1 during transition. Rollback leaves ordinary Markdown and asset directories usable; v2 recovery files may be ignored by older binaries but do not affect source files.

## Open Questions

None blocking. Native watcher integration and unreferenced-asset cleanup can be evaluated after the polling-based safety contract is established.
