## ADDED Requirements

### Requirement: Native Word import documentation SHALL state workflow and fidelity boundaries
The English and Simplified Chinese READMEs SHALL provide equivalent native DOCX import descriptions and link to a maintained Word-import guide. The guide SHALL document the local native menu/report/save/new-tab flow; accepted-revision policy; supported and unsupported content/formats; `.md` plus `.assets` portability; source preservation; create-only destinations; conversion limits; explicit loss continuation; cancellation/failure and possible crash remnants; and the absence of external tool requirements. It SHALL distinguish durable native import from the existing session-only MarkNice browser import and SHALL NOT promise page-layout fidelity, reversible DOCX round trips, legacy `.doc`, macro/encrypted input, or automatic browser write-back.

#### Scenario: A user follows native Word import instructions
- **WHEN** a reader opens either README and follows its import guide
- **THEN** the reader can identify where the saved Markdown and images will live, which content may be simplified or omitted, and how the report controls continuation
- **AND** both language editions describe equivalent behavior

#### Scenario: A user compares native and browser import
- **WHEN** the guide explains Word import in Markion and MarkNice
- **THEN** it identifies native import as creation of a saved Markdown document and browser import as a session-local replacement requiring explicit recovery into Markion
- **AND** it does not imply that browser edits are synchronized back

#### Scenario: Compatibility claims are checked
- **WHEN** the guide or compatibility report names a supported producer, platform, equation, or table structure
- **THEN** the claim matches the recorded corpus and verification evidence
- **AND** unresolved coverage is explicitly identified rather than presented as verified support
