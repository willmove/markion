## ADDED Requirements

### Requirement: Maintained release acceptance suite
The repository SHALL provide a single release acceptance entry point under `docs/testing/` with separate human and coding-agent execution instructions. The suite SHALL identify cases by stable identifiers and specify prerequisites, owned fixtures, ordered actions, observable expected outcomes, variants, and evidence for persistence/recovery, editing/input, rendering/tables/resources, workspace, import/export, Git, AI, and platform/package/update behavior. It MUST distinguish implemented behavior from proposals and historical verification.

#### Scenario: Maintainer starts acceptance
- **WHEN** a maintainer opens the testing entry point
- **THEN** they can select the platform/package matrix, prepare synthetic material, follow detailed cases, and create an execution record without reconstructing historical reports

#### Scenario: Coding agent cannot perform an interaction
- **WHEN** an agent lacks an OS input gesture, live input method, package, browser, viewer, or platform required by a case
- **THEN** it records the affected variant as blocked or skipped with a reason
- **AND** callback replay, simulated layout, a screenshot, or a previous build's result does not become a native pass

### Requirement: Reproducible acceptance material and evidence
Acceptance material SHALL be repository-owned synthetic content or explicitly documented external producer samples with provenance. Preparation SHALL use a separate run directory and preserve existing runs and fixture originals. The suite SHALL provide byte-sensitive text variants, structured editing and resource samples, deterministic large-document material, and a manifest identifying inputs. Records SHALL identify candidate commit and dirty state, package and executable digests, environment and external tool versions, case variants, execution method, actual outcomes, evidence, failures, and exceptions. Editing acceptance SHALL compare complete persisted output and semantic undo outcomes where applicable.

#### Scenario: Fresh fixture preparation
- **WHEN** a tester prepares material in a new run directory
- **THEN** editable working copies and independent baselines are available with input identities
- **AND** preparation refuses to overwrite an existing run

#### Scenario: Candidate or executable changes
- **WHEN** a tested source revision, packaged resource, or executable changes
- **THEN** the record identifies affected results for rerun instead of silently carrying them forward

### Requirement: Release readiness references current native evidence
The canonical release process SHALL reference the acceptance suite before tagging and require final tagged-artifact identity and smoke checks before publication is reported complete. Applicable required case variants SHALL remain distinguishable as passed, failed, blocked, skipped, not run, or not applicable. A release-specific maintainer exception SHALL record its scope and reason without reclassifying an unverified case as passed or carrying the exception to another release. Existing mandatory browser/viewer and package/signature gates SHALL remain effective.

#### Scenario: Required manual row is pending
- **WHEN** a release record contains an applicable required case that has no passing evidence
- **THEN** ordinary readiness remains incomplete unless the maintainer explicitly records an exception for that release
- **AND** automated checks and historical release exceptions do not supply that authorization

#### Scenario: Final tag packages are available
- **WHEN** the tag workflow produces installers and authenticated update metadata
- **THEN** final artifact identities, actual installed application smoke results, and publication checks are linked to the release record
- **AND** preparatory development-binary results remain separately labeled
