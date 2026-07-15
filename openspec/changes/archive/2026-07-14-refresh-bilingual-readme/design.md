## Context

`README.md` is the repository's main landing page, but several statements predate implemented changes such as Visual Edit, Open Folder, preview selection/context-menu behavior, and removal of mutation controls from ordinary preview tables. The project also exposes a Simplified Chinese interface but has no Chinese repository overview. The documentation must be checked against stable OpenSpec requirements, implemented source paths, package metadata, and the current release workflow; active change proposals are not authoritative unless their behavior is already present in code.

## Goals / Non-Goals

**Goals:**

- Make the English README an accurate overview of the current v0.1.2 application.
- Add a Simplified Chinese edition with equivalent structure and claims.
- Make language switching obvious near the top of both files.
- Distinguish Visual Edit's source-backed live-preview behavior from full WYSIWYG editing and document its conservative source islands.
- Keep installation, configuration, export, packaging, and development guidance concrete and verifiable.

**Non-Goals:**

- Changing application behavior or OpenSpec requirements for existing product capabilities.
- Documenting unimplemented active proposals as shipped features.
- Turning the README into a complete user manual or duplicating the keyboard shortcut and FAQ documents.

## Decisions

### Use parallel, structurally equivalent language editions

`README.md` remains English and `README.zh-CN.md` becomes the Simplified Chinese counterpart. Both use the same section order, feature coverage, code samples, and caveats, with reciprocal language links. This keeps GitHub's default landing page conventional while making the Chinese edition one click away.

### Treat implementation plus stable specifications as evidence

Stable OpenSpec requirements define intended current behavior; the Rust implementation, package manifests, and release workflow resolve stale or contradictory prose. Completed-but-unarchived changes may be mentioned only where the implementation verifies the feature. Planning-only behavior remains excluded.

### Describe capabilities by user workflow

The README will group information into installation, view/edit workflows, workspace, preview/Markdown, preferences, export, performance, limitations, and development. This is more scannable than mirroring internal module boundaries and lets both language editions stay aligned.

### State limitations next to the related feature

Visual Edit is described as source-backed rather than full WYSIWYG; tables, code, math, HTML, front matter, and ambiguous syntax may use source islands. Preview tables are read-only, while Visual Edit and source commands provide table mutations. Export fallback quality and unsigned installers are stated where users make decisions.

## Risks / Trade-offs

- [Risk] Parallel editions drift over time → Keep identical headings and ordering, reciprocal links, and review both files in the same change.
- [Risk] Active development makes claims stale quickly → Prefer durable capability language and avoid enumerating implementation details that are not user-facing.
- [Risk] Visual Edit can be mistaken for full WYSIWYG → Explicitly describe the canonical Markdown source and conservative source-island limitations.
- [Risk] README becomes too long → Link to existing FAQ and shortcut documentation instead of duplicating them.
