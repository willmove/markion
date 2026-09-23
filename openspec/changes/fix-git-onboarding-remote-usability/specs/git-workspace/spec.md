## ADDED Requirements

### Requirement: An existing sync address SHALL be replaceable through explicit confirmation
Setup SHALL honor a user-entered sync address on every route, including connecting the current folder. When the entered address differs from the repository's existing `origin` URL, setup SHALL stop before touching the repository, display both the current and the new address, and require a second explicit confirmation to reset the sync address. Only after that confirmation SHALL the origin URL be replaced and setup continued; without confirmation nothing is modified and the previous error-only dead end does not occur. An entered address identical to the existing origin SHALL proceed without prompting. Replacement SHALL remain impossible without the explicit confirmation step — the existing rule that remote configuration is never replaced implicitly is preserved.

#### Scenario: Wrong existing address is corrected
- **WHEN** the repository's origin points at a wrong address and the user submits a different, valid address
- **THEN** setup shows the current address and the new address side by side and asks for explicit confirmation
- **AND** after confirmation the origin URL is replaced and setup continues with the new address
- **AND** cancelling the confirmation leaves the repository's remote configuration unchanged

#### Scenario: Entered address matches the existing origin
- **WHEN** the user submits an address identical to the existing origin URL
- **THEN** setup proceeds without any replacement prompt

#### Scenario: Address is honored on the connect-current-folder route
- **WHEN** the current folder's origin already resolves a sync target and the user enters a different address
- **THEN** the replacement confirmation is offered instead of silently ignoring the entered address

#### Scenario: Replacement never happens implicitly
- **WHEN** any setup route would change the existing origin URL without the explicit confirmation step
- **THEN** the change is refused and the user is asked to confirm first

#### Scenario: Replacement onto a remote with existing work completes
- **WHEN** the confirmed reset points at a remote whose branch already contains work with a common history
- **THEN** setup binds the upstream without attempting a rejected non-fast-forward push
- **AND** the first synchronization fetches, integrates, and pushes both sides
- **WHEN** the remote's history shares nothing with the local repository
- **THEN** setup refuses with localized clone-and-copy guidance instead of an automatic unrelated-history merge

### Requirement: Sync addresses SHALL be validated before setup applies them
Every setup route SHALL validate the entered sync address before using it. Addresses that cannot be parsed, are not HTTPS or SSH transports, embed credentials, or are local paths SHALL be rejected inline with a localized message without touching the repository. For an address that passes syntax validation but is not yet applied, setup SHALL run a bounded, cancellable reachability probe; when the address cannot be contacted — because it is mistyped, the repository is missing, access is denied, or the machine is offline — setup SHALL present a localized warning and require explicit confirmation to continue, and SHALL proceed without warning when the probe succeeds.

#### Scenario: Invalid address is rejected inline
- **WHEN** the user submits an unparseable address, a non-HTTPS/SSH transport, a credential-embedding URL, or a local path on any route including connecting the current folder
- **THEN** setup rejects it with a localized message and no repository configuration changes

#### Scenario: Unreachable address warns and asks for confirmation
- **WHEN** a syntactically valid address cannot be contacted by the reachability probe
- **THEN** setup warns that the address may be wrong or the machine may be offline and asks whether to use it anyway
- **AND** continuing requires explicit confirmation, while cancelling keeps the repository unchanged

#### Scenario: Reachable address proceeds without warning
- **WHEN** the reachability probe contacts the address successfully
- **THEN** setup continues without an unreachability warning

#### Scenario: Probe is bounded and cancellable
- **WHEN** the reachability probe hangs or the user closes setup
- **THEN** the probe is bounded by a timeout and cancelled with the setup operation
