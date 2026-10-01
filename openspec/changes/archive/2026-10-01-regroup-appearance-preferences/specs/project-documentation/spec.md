## ADDED Requirements

### Requirement: User docs SHALL name the Appearance preferences tab
English FAQ and bilingual READMEs that tell users where to pick a theme or change document typography SHALL name **Preferences → Appearance** (not Theme as a sibling tab, and not an undifferentiated Preferences panel for those controls).

#### Scenario: FAQ points at Appearance
- **WHEN** a reader opens the Themes section of `docs/faq.md`
- **THEN** it directs them to Preferences → Appearance

#### Scenario: README grouping matches the panel
- **WHEN** a reader opens `README.md` or `README.zh-CN.md`
- **THEN** theme and document typography are described as Appearance preferences rather than as Theme-tab-only or General-tab typography
