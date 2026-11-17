## MODIFIED Requirements

### Requirement: Bilingual project overview
The repository SHALL provide a concise English root README and equivalent standalone editions in Simplified Chinese, Traditional Chinese, Japanese, French, German, Spanish, and Korean (`README.ko.md`). Each edition SHALL visibly link all other editions and mark its own language active. Each edition SHALL cover product value, current published release highlights, downloads and first-launch instructions, editing modes, measured performance, essential limitations, and contributor verification commands. Detailed workflows and configuration MAY live in linked guides, with English and Simplified Chinese reference coverage retained. Existing `#english`, `#simplified-chinese`, and `#install` root anchors SHALL remain usable; the Simplified Chinese anchor MAY link to the standalone edition instead of duplicating it. Every edition SHALL link the WYSIWYG coverage matrix and dated project status. Stable capability purposes and project context metadata SHALL describe the current implemented architecture and MUST NOT characterize an archived capability as only future work.

#### Scenario: English reader opens the repository
- **WHEN** a reader opens `README.md`
- **THEN** they see an English product overview, an authentic application screenshot, current release highlights, and actionable downloads
- **AND** visible language navigation includes all seven standalone editions
- **AND** the page links the WYSIWYG coverage matrix and project status

#### Scenario: Chinese reader selects the Chinese edition
- **WHEN** a reader follows Simplified Chinese navigation or the legacy `#simplified-chinese` anchor
- **THEN** they can reach the standalone Simplified Chinese edition
- **AND** it provides equivalent capability, installation, performance, and limitation coverage and links back to English

#### Scenario: Chinese reader uses the standalone entry point
- **WHEN** a reader opens `README.zh-CN.md` directly or follows an existing link to it
- **THEN** the standalone Simplified Chinese overview presents the same current product, installation, performance, and limitation coverage as the English overview
- **AND** it links to the English root, detailed Chinese user guide, and Visual Edit WYSIWYG coverage matrix

#### Scenario: Reader selects another language edition
- **WHEN** a reader opens any standalone language README, including Korean
- **THEN** it presents equivalent current product information in that language
- **AND** all other editions remain reachable through visible language navigation
- **AND** published application UI languages are distinguished from README translation availability

#### Scenario: README claims are checked against the project
- **WHEN** an edition describes a feature, package, configuration option, limitation, development command, or WYSIWYG coverage class
- **THEN** the claim matches current implementation and recorded release evidence
- **AND** detailed configuration and workflow explanations remain accessible through linked guides

#### Scenario: Capability metadata is reviewed after archival
- **WHEN** an archived change makes a previously future capability part of the stable system
- **THEN** affected capability purposes and project context describe the implemented state
- **AND** no stable metadata contradicts archived requirements

#### Scenario: Document identity is described on the title bar
- **WHEN** a README or its linked guide describes document identity
- **THEN** it identifies the native window title as showing the active file name after the Markion brand
- **AND** it does not identify the status bar as that surface

## ADDED Requirements

### Requirement: Visual and measured product evidence
README editions SHALL include authentic Markion screenshots with meaningful alternative text and a linked capture record identifying platform, build, and source material. Performance tables SHALL cite reproducible methodology and raw evidence, identify the measured operation, document size, platform, build/date, sample count, and excluded stages. Model processing SHALL be distinguished from input-to-paint and hardware-to-display latency. Historical results SHALL be labelled as historical. Cross-product speed rankings SHALL NOT be claimed without equivalent measured workloads and environments.

#### Scenario: Reader evaluates performance
- **WHEN** a reader opens the performance section
- **THEN** they see measurements across document sizes or documented before/after stages
- **AND** a nearby link explains dates, sample counts, hardware information available, boundaries, and reproduction commands
- **AND** historical measurements are not presented as a fresh latest-release benchmark

#### Scenario: Reader evaluates screenshots
- **WHEN** a reader opens a screenshot's provenance link
- **THEN** they can identify the captured platform, build, date, and sample document
- **AND** the image is an actual application capture rather than a generated mockup
