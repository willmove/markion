## ADDED Requirements

### Requirement: Official plugin file types SHALL participate in workspace handling
The Files panel SHALL include file types declared by the signed official plugin catalog, subject to the existing background scan, filename filtering, folder hierarchy, ignore/hidden rules, bounded-row rendering, and context actions. An installed and enabled compatible provider SHALL open the path through its declared host capability. An absent, disabled, unhealthy, or incompatible provider SHALL route to the localized missing-provider flow rather than interpreting the bytes as Markdown or silently omitting the file. Conflicting provider claims SHALL fail closed using deterministic first-party catalog validation.

#### Scenario: Known plugin type appears without installation
- **WHEN** the workspace contains a `.pdf` file recognized by the signed official catalog and the PDF plugin is absent
- **THEN** the PDF remains visible with its plugin-provided file-kind identity
- **AND** opening it shows the explicit install flow without reading it as UTF-8 text

#### Scenario: Enabled provider opens a workspace file
- **WHEN** the user opens a plugin-supported file and one compatible enabled provider owns that file type
- **THEN** Markion opens or focuses its generic plugin-document tab under the existing open-target rule
- **AND** current-file marking, recent paths, and workspace-root behavior remain generic and path-based

#### Scenario: Catalog contains conflicting file claims
- **WHEN** catalog validation encounters multiple active first-party providers claiming the same extension and capability priority
- **THEN** Markion rejects the conflicting catalog update or providers before activation
- **AND** does not nondeterministically choose one for workspace files

#### Scenario: Plugin-backed path is renamed or deleted
- **WHEN** a clean open plugin-backed file is renamed, moved, or deleted through existing workspace actions
- **THEN** its path and tab follow the generic remap/deletion pipeline without a dirty-document prompt
- **AND** the provider session is closed or reopened only as required by that path operation

