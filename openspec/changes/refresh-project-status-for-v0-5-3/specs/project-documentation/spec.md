## ADDED Requirements

### Requirement: Evidence-linked project progress
The repository SHALL provide a dated project-status document linked from the English and Simplified Chinese READMEs. It SHALL distinguish implemented behavior, recorded verification, and pending verification or proposed work; link to the applicable OpenSpec artifacts and detailed evidence; and identify the platform and measurement boundaries for performance claims. Historical progress reports SHALL link to superseding results when later work closes their stated gaps. Release preparation SHALL NOT be described as a completed publication before the canonical release checks succeed.

#### Scenario: Reader checks current editing progress
- **WHEN** a reader follows the project-status link from either primary README edition
- **THEN** the document identifies the current implementation and its supporting verification reports
- **AND** completed structured editing and local derivation are not described as future implementation work

#### Scenario: Verification remains incomplete
- **WHEN** a platform interaction, browser matrix, or release-publication check has no successful recorded result
- **THEN** the status document names that limitation or pending check without inventing a passing result

#### Scenario: Reader opens a historical report
- **WHEN** an earlier P0 or P1 report describes a gap closed by later work
- **THEN** it links to the later implementation and verification report while preserving its original measurements and scope
