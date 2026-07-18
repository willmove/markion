## MODIFIED Requirements

### Requirement: Absorbed-crate de-inventory policy
Absorbed library crates (under `crates/*`) SHALL NOT retain modules whose keep/delete verdict has been settled by an evaluation (e.g. `docs/typune-integration-audit.md` §3 P2a/P3, the `unify-pulldown-cmark` AST-adoption decision, or the source-mapped incremental-model decision) and which are verified to have no live reference from the application crate. Once the decision point resolves to "delete", the module SHALL be removed in that same change alongside its `pub mod`/`pub use` declarations and any test file that imports it directly, rather than persisting as permanent dead code. Modules serving as inventory for a recorded, gated future decision MAY be retained only while the gate and its re-open triggers remain documented in the corresponding change's design.

#### Scenario: Decided-unused module is removed, not retained
- **WHEN** an evaluation concludes an absorbed-crate module is unused and will not be adopted, including the spanless Typune incremental AST path after Markion adopts its source-ranged incremental model
- **THEN** the module file, its `pub mod` / `pub use` declarations in the crate `lib.rs`, and any test file importing it directly are all removed in one change

#### Scenario: Gated inventory is retained with a documented trigger
- **WHEN** a module is unused at present but still serves a recorded future decision whose acceptance conditions have not been resolved
- **THEN** it MAY remain in the crate only while a current `design.md` names the module and the conditions under which it will be adopted or deleted

#### Scenario: Live export-parse-path modules are not deleted
- **WHEN** a module is transitively reachable from a code path the application uses (e.g. `extended_inline` / `emoji` reached from `parser.rs` feeding the export engine, even though `src/` does not name them directly)
- **THEN** it SHALL NOT be removed without first verifying the reachability and recording the analysis in the change proposal
