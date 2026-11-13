## Context

The current runbook's nested release-note template and AGENTS.md request for detailed notes encourage lengthy descriptions.

## Goals / Non-Goals

**Goals:** a brief bilingual summary with complete upgrade/download/verification coverage and links to detailed evidence.

**Non-Goals:** changing published v0.5.3 notes, application behavior or release acceptance gates.

## Decisions

- Use one summary sentence and normally 3–5 combined highlights/fixes per language; use fewer for small patches. Keep compatibility, downloads and verification to brief lines.
- Retain English-first/Chinese parity, the comparison link, accurate skips and essential upgrade/first-launch instructions. Link detailed implementation, benchmarks and test logs from the summary.
- Update the operator instructions and canonical template together, then sync the specification through archival.

## Risks / Trade-offs

- Brevity could hide upgrade requirements or unverified scope → explicitly preserve those disclosures; required instructions take precedence over the normal length guidance.
