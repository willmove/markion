## Context

The app captures autosave snapshots under a per-tab gate, and the persistence layer currently checks the destination and replaces it in separate steps. The recovery inventory reads file paths independently of the tab that created them. Source Select All already uses the canonical paragraph range for both selection steps.

## Decisions

- Serialize in-process document writes to the same destination from freshness check through atomic replacement and identity recording. An explicit save in the app also drains its tab's active autosave, adopts the identity of its own completed write when applicable, and retires the old work only after success. The existing Git admission boundary remains in effect. This prevents late workers and late completions from invalidating a successful explicit save.
- Use the snapshot path as the tab's recovery ownership. Publish the first snapshot at a unique path with an atomic create-only operation, then use atomic replacement for that tab's subsequent snapshots. Restoring a draft transfers ownership of its path to the restored tab. The reader still accepts the existing payload formats.
- Find the nearest paragraph separators in the source using logical LF and CRLF endings and keep ranges in original UTF-8 bytes. The selection action still consumes the range, and any subsequent edit follows the existing undo and cache invalidation path.

## Risks and Trade-offs

- An explicit save may wait for its already-running autosave. The autosave continues to perform disk I/O on the background executor.
- A prior internal write can be adopted only through its own active autosave gate. Later outside changes still pass through normal disk identity checks.
- Recovery files created by earlier versions require no migration. A retained file remains readable even when unrelated snapshots subsequently use different names.
