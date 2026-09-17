# First-party plugin platform architecture

The platform separates reusable lifecycle infrastructure from narrow,
versioned capability families. The editor never loads a plugin dynamic library
and plugins never construct GPUI elements or mutate editor state directly.

## Trust and package model

- A small signed bootstrap catalog is compiled into every core build. It keeps
  known file types discoverable offline without embedding plugin payloads.
- Explicit refresh checks the OSS endpoint and then GitHub. Only a canonical,
  locale-complete, monotonically newer catalog signed by the embedded key can
  replace the atomic cache; invalid data leaves the last verified snapshot in
  place.
- A `.markion-plugin` is a deterministically ordered Deflate ZIP containing
  canonical `plugin.json`, `plugin.json.minisig`, one target worker, target
  resources, and plugin-specific notices. The signed manifest closes every
  member by normalized path, byte length, SHA-256 digest, and executable bit.
- Archive inspection rejects traversal, absolute paths, links, duplicates,
  case collisions, undeclared files, target/protocol/host mismatches, excess
  members, and compressed or expanded size violations before extraction.
- Production plugin signing uses a dedicated key, never the Windows updater
  key. Repository keys are development fixtures only; tagged builds fail
  closed when the production trust root is unavailable.

## Runtime modules

`PluginCatalog` resolves signed identity, compatibility, artifacts, declared
permissions, capability versions, and file handlers. An immutable registry
snapshot combines those declarations with core Markdown, text, and image
handlers. Workspace scans consume one snapshot, so no row performs network or
process work. Known-but-uninstalled types remain visible.

`PluginStore` owns per-user staging, verified versions, atomic active pointers,
one rollback version, disabled/quarantine state, recovery, and exact storage
accounting. Activation happens only after package verification and a worker
handshake/health check.

`PluginSupervisor` starts the exact re-verified executable directly from its
immutable version directory with a sanitized environment and controlled
working directory. It owns framed I/O, bounded stderr, request limits,
deadlines, cancellation, process generations, stale-response rejection,
crash-loop quarantine, graceful shutdown, and forced process-tree cleanup.

The transport uses a versioned, length-framed protocol with canonical JSON
headers and bounded raw bodies. Raster pages remain raw RGBA rather than
Base64. Unknown protocol majors and malformed, truncated, oversized, or
unsolicited frames fail closed.

## Paged documents

`paged-document/v1` is the first capability adapter. Generic plugin-document
tabs retain provider ID, capability ID, path identity, immutable page geometry,
scroll/zoom/page state, generation, and host raster claims. The reusable host
viewport owns continuous scrolling, page jumps, zoom anchoring, visible-page
prefetch, width buckets, a 32 MiB per-raster limit, and a 64 MiB bounded cache.

The official PDF worker owns PDFium discovery and every native document handle.
The root package depends on neither `markion-pdf-viewer` nor a PDFium resource;
the PDF export crate remains a separate built-in document-export concern.

## Release topology

Native jobs build and test one plugin archive per supported target, populate
exact lengths/digests/storage values into the catalog, sign it, then compile
the core against that exact catalog and public key. Core closure checks reject
plugin workers, archives, installed-version trees, PDFium libraries, and
plugin-only notices in NSIS, DMG, DEB, and AppImage outputs. Plugin and core
artifacts have independent checksums, notices, provenance, and size reports,
and tagged releases publish them to GitHub and the OSS mirror.

A later publishing change should introduce a typed capability such as
`publisher-job/v1` with preflight, submit, progress/status, cancel, and stable
error categories. It should reuse the package/store/supervisor layers rather
than add a universal JSON execution escape hatch.
