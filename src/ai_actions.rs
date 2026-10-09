//! Durable file action execution and conservative recovery, independent of the AI enable switch.
use markion_ai::{
    AiError,
    proposals::{Operation, Plan},
    workspace::{FileIdentity, ReadGrant, identity},
};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};
static NEXT: AtomicU64 = AtomicU64::new(1);
pub const JOURNAL_LIMIT: u64 = 50 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Outcome {
    Prepared,
    Applied,
    Failed,
    Skipped,
    Unapplied,
    Ambiguous,
    Restored,
    Restoring,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: usize,
    pub operation: Operation,
    pub dependencies: Vec<usize>,
    pub state: Outcome,
    pub post_identity: Option<FileIdentity>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Journal {
    pub version: u32,
    pub id: String,
    pub root: PathBuf,
    pub scope: PathBuf,
    pub root_identity: FileIdentity,
    pub scope_identity: FileIdentity,
    pub records: Vec<Record>,
    #[serde(skip)]
    pub path: PathBuf,
}
fn err(_: impl std::fmt::Debug) -> AiError {
    AiError::Unavailable
}
impl Journal {
    pub fn grant(&self) -> Result<ReadGrant, AiError> {
        let grant = ReadGrant::new(&self.root, &self.scope)?;
        if grant.root_identity != self.root_identity || grant.scope_identity != self.scope_identity
        {
            return Err(AiError::Stale);
        }
        Ok(grant)
    }
    pub fn persist(&self) -> Result<(), AiError> {
        let bytes = serde_json::to_vec(self).map_err(err)?;
        if bytes.len() as u64 > JOURNAL_LIMIT {
            return Err(AiError::Limit);
        }
        crate::storage::atomic_write(&self.path, bytes).map_err(err)
    }
    pub fn prepare(directory: &Path, grant: &ReadGrant, plan: &Plan) -> Result<Self, AiError> {
        grant.validate()?;
        plan.validate_selection()?;
        fs::create_dir_all(directory).map_err(err)?;
        let entries = fs::read_dir(directory).map_err(err)?;
        let mut used = 0;
        for e in entries {
            let e = e.map_err(err)?;
            let metadata = e.metadata().map_err(err)?;
            if metadata.is_file() {
                used += metadata.len();
            }
        }
        let records = plan
            .operations
            .iter()
            .filter(|p| p.selected)
            .map(|p| Record {
                id: p.id,
                operation: p.operation.clone(),
                dependencies: p.dependencies.clone(),
                state: Outcome::Prepared,
                post_identity: None,
            })
            .collect::<Vec<_>>();
        if records.is_empty() {
            return Err(AiError::Scope);
        }
        let id = format!(
            "{}-{}-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos(),
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let mut journal = Self {
            version: 1,
            path: directory.join(format!("{id}.json")),
            id,
            root: grant.root.clone(),
            scope: grant.scope.clone(),
            root_identity: grant.root_identity.clone(),
            scope_identity: grant.scope_identity.clone(),
            records,
        };
        let bytes = serde_json::to_vec(&journal).map_err(err)?;
        if used + bytes.len() as u64 > JOURNAL_LIMIT {
            return Err(AiError::Limit);
        }
        journal.preflight()?;
        journal.persist()?;
        for record in &mut journal.records {
            record.state = Outcome::Prepared;
        }
        Ok(journal)
    }
    pub fn preflight(&self) -> Result<(), AiError> {
        let grant = self.grant()?;
        let mut created = Vec::new();
        let mut destinations = std::collections::HashSet::new();
        let sources = self
            .records
            .iter()
            .filter(|r| {
                matches!(
                    r.operation,
                    Operation::EditText { .. } | Operation::Move { .. }
                )
            })
            .map(|r| r.operation.path().to_lowercase())
            .collect::<std::collections::HashSet<_>>();
        for record in &self.records {
            if record.state != Outcome::Prepared && record.state != Outcome::Unapplied {
                continue;
            }
            if matches!(
                record.operation,
                Operation::CreateNote { .. }
                    | Operation::CreateFolder { .. }
                    | Operation::Move { .. }
            ) {
                let destination = record
                    .operation
                    .destination()
                    .unwrap_or(record.operation.path())
                    .to_lowercase();
                if !destinations.insert(destination.clone()) || sources.contains(&destination) {
                    return Err(AiError::Stale);
                }
            }
            match &record.operation {
                Operation::CreateFolder { path } => {
                    validate_new(&grant, path, false, &created)?;
                    created.push(path.clone());
                }
                Operation::CreateNote { path, .. } => {
                    validate_new(&grant, path, true, &created)?;
                }
                Operation::EditText { path, baseline, .. }
                | Operation::Move { path, baseline, .. } => {
                    let p = grant.resolve(path, true, true)?;
                    if identity(&p)? != baseline.identity
                        || fs::read_to_string(&p).map_err(err)? != baseline.disk
                    {
                        return Err(AiError::Stale);
                    }
                    if baseline.buffer.is_some() && baseline.text != baseline.disk {
                        return Err(AiError::Stale);
                    }
                    if let Operation::Move { destination, .. } = &record.operation {
                        validate_new(&grant, destination, true, &created)?;
                    }
                }
            }
        }
        Ok(())
    }
    pub fn execute_one(&mut self, index: usize) -> Result<(), AiError> {
        if self.version != 1 {
            return Err(AiError::Protocol);
        }
        let grant = self.grant()?;
        let record = self.records.get(index).ok_or(AiError::Scope)?;
        if record.state != Outcome::Prepared && record.state != Outcome::Unapplied {
            return Err(AiError::Stale);
        }
        if record.dependencies.iter().any(|id| {
            self.records
                .iter()
                .find(|r| r.id == *id)
                .is_none_or(|r| r.state != Outcome::Applied)
        }) {
            self.records[index].state = Outcome::Skipped;
            self.persist()?;
            return Err(AiError::Stale);
        }
        let operation = record.operation.clone();
        self.records[index].state = Outcome::Prepared;
        self.persist()?;
        let mut mutation_started = false;
        let result = (|| match &operation {
            Operation::CreateFolder { path } => {
                let target = grant.resolve(path, false, false)?;
                mutation_started = true;
                fs::create_dir(&target).map_err(err)?;
                identity(&target)
            }
            Operation::CreateNote { path, text } => {
                let target = grant.resolve(path, false, true)?;
                mutation_started = true;
                create_complete(&target, text.as_bytes()).map_err(err)?;
                identity(&target)
            }
            Operation::EditText { path, baseline, .. } => {
                let target = grant.resolve(path, true, true)?;
                if identity(&target)? != baseline.identity
                    || fs::read_to_string(&target).map_err(err)? != baseline.disk
                {
                    return Err(AiError::Stale);
                }
                let after = operation.after()?.ok_or(AiError::Protocol)?;
                mutation_started = true;
                crate::storage::atomic_write(&target, after.as_bytes()).map_err(err)?;
                identity(&target)
            }
            Operation::Move {
                path,
                destination,
                baseline,
                link_edits,
            } => {
                let source = grant.resolve(path, true, true)?;
                let target = grant.resolve(destination, false, true)?;
                if identity(&source)? != baseline.identity
                    || fs::read_to_string(&source).map_err(err)? != baseline.disk
                {
                    return Err(AiError::Stale);
                }
                mutation_started = true;
                if link_edits.is_empty() {
                    move_exclusive(&source, &target).map_err(err)?;
                } else {
                    let after = operation.after()?.ok_or(AiError::Protocol)?;
                    let permissions = fs::metadata(&source).map_err(err)?.permissions();
                    create_complete_with_permissions(&target, after.as_bytes(), Some(permissions))
                        .map_err(err)?;
                    if identity(&source)? != baseline.identity
                        || fs::read_to_string(&source).map_err(err)? != baseline.disk
                    {
                        return Err(AiError::Stale);
                    }
                    fs::remove_file(&source).map_err(err)?;
                }
                identity(&target)
            }
        })();
        match result {
            Ok(post) => {
                self.records[index].post_identity = Some(post);
                self.records[index].state = Outcome::Applied;
                self.persist()?;
                Ok(())
            }
            Err(e) => {
                self.records[index].state = if mutation_started {
                    Outcome::Ambiguous
                } else {
                    Outcome::Failed
                };
                self.persist()?;
                Err(e)
            }
        }
    }
    pub fn stop_remaining(&mut self) -> Result<(), AiError> {
        for record in &mut self.records {
            if record.state == Outcome::Prepared {
                record.state = Outcome::Unapplied;
            }
        }
        self.persist()
    }
    fn post_matches(&self, record: &Record) -> Result<bool, AiError> {
        let grant = self.grant()?;
        let path = record
            .operation
            .destination()
            .unwrap_or_else(|| record.operation.path());
        let folder = matches!(record.operation, Operation::CreateFolder { .. });
        let target = match grant.resolve(path, true, !folder) {
            Ok(p) => p,
            Err(_) => return Ok(false),
        };
        if record
            .post_identity
            .as_ref()
            .is_some_and(|id| identity(&target).as_ref() != Ok(id))
        {
            return Ok(false);
        }
        if let Some(after) = record.operation.after()? {
            if fs::read_to_string(&target).map_err(err)? != after {
                return Ok(false);
            }
        } else if !target.is_dir() {
            return Ok(false);
        }
        if let Operation::Move { path, .. } = &record.operation {
            if grant.scope.join(path).exists() {
                return Ok(false);
            }
        }
        Ok(true)
    }
    pub fn restore_one(&mut self, index: usize) -> Result<(), AiError> {
        let grant = self.grant()?;
        let record = self.records.get(index).ok_or(AiError::Scope)?;
        if record.state != Outcome::Applied || !self.post_matches(record)? {
            return Err(AiError::Stale);
        }
        if self
            .records
            .iter()
            .any(|r| r.dependencies.contains(&record.id) && r.state == Outcome::Applied)
        {
            return Err(AiError::Stale);
        }
        let operation = record.operation.clone();
        self.records[index].state = Outcome::Restoring;
        self.persist()?;
        let result = (|| {
            match operation {
                Operation::CreateNote { path, .. } => {
                    let target = grant.resolve(&path, true, true)?;
                    fs::remove_file(&target).map_err(err)?;
                }
                Operation::CreateFolder { path } => {
                    let target = grant.resolve(&path, true, false)?;
                    if fs::read_dir(&target).map_err(err)?.next().is_some() {
                        return Err(AiError::Stale);
                    }
                    fs::remove_dir(&target).map_err(err)?;
                }
                Operation::EditText { path, baseline, .. } => {
                    let target = grant.resolve(&path, true, true)?;
                    crate::storage::atomic_write(&target, baseline.disk.as_bytes()).map_err(err)?;
                }
                Operation::Move {
                    path, destination, ..
                } => {
                    let source = grant.resolve(&destination, true, true)?;
                    let target = grant.resolve(&path, false, true)?;
                    if let Operation::Move {
                        baseline,
                        link_edits,
                        ..
                    } = &self.records[index].operation
                    {
                        if link_edits.is_empty() {
                            move_exclusive(&source, &target).map_err(err)?;
                        } else {
                            let permissions = fs::metadata(&source).map_err(err)?.permissions();
                            create_complete_with_permissions(
                                &target,
                                baseline.disk.as_bytes(),
                                Some(permissions),
                            )
                            .map_err(err)?;
                            fs::remove_file(&source).map_err(err)?;
                        }
                    }
                }
            }
            Ok::<(), AiError>(())
        })();
        if let Err(e) = result {
            self.records[index].state = if self.post_matches(&self.records[index]).unwrap_or(false)
            {
                Outcome::Applied
            } else {
                Outcome::Ambiguous
            };
            self.persist()?;
            return Err(e);
        }
        self.records[index].state = Outcome::Restored;
        self.persist()?;
        Ok(())
    }
    pub fn reconcile(&mut self) -> Result<(), AiError> {
        if self.version != 1 {
            return Err(AiError::Protocol);
        }
        let grant = self.grant()?;
        for index in 0..self.records.len() {
            let record = &self.records[index];
            if record.state == Outcome::Restoring {
                let restored = match &record.operation {
                    Operation::CreateFolder { path } | Operation::CreateNote { path, .. } => {
                        !grant.scope.join(path).exists()
                    }
                    Operation::EditText { path, baseline, .. } => {
                        fs::read_to_string(grant.scope.join(path)).is_ok_and(|s| s == baseline.disk)
                    }
                    Operation::Move {
                        path,
                        destination,
                        baseline,
                        ..
                    } => {
                        !grant.scope.join(destination).exists()
                            && fs::read_to_string(grant.scope.join(path))
                                .is_ok_and(|s| s == baseline.disk)
                    }
                };
                self.records[index].state = if restored {
                    Outcome::Restored
                } else if self.post_matches(record)? {
                    Outcome::Applied
                } else {
                    Outcome::Ambiguous
                };
                continue;
            }
            if record.state != Outcome::Prepared {
                continue;
            }
            if self.post_matches(record)? {
                let path = record
                    .operation
                    .destination()
                    .unwrap_or_else(|| record.operation.path());
                let id = identity(&grant.scope.join(path))?;
                self.records[index].post_identity = Some(id);
                self.records[index].state = Outcome::Applied;
            } else {
                let unchanged = match &record.operation {
                    Operation::CreateFolder { path } | Operation::CreateNote { path, .. } => {
                        !grant.scope.join(path).exists()
                    }
                    Operation::EditText { path, baseline, .. }
                    | Operation::Move { path, baseline, .. } => {
                        let destination_absent = record
                            .operation
                            .destination()
                            .is_none_or(|d| !grant.scope.join(d).exists());
                        destination_absent
                            && identity(&grant.scope.join(path))
                                .is_ok_and(|id| id == baseline.identity)
                            && fs::read_to_string(grant.scope.join(path))
                                .is_ok_and(|text| text == baseline.disk)
                    }
                };
                self.records[index].state = if unchanged {
                    Outcome::Unapplied
                } else {
                    Outcome::Ambiguous
                };
            }
        }
        self.persist()
    }
    pub fn retire(&self) -> Result<(), AiError> {
        if self.records.iter().any(|r| {
            matches!(
                r.state,
                Outcome::Applied | Outcome::Prepared | Outcome::Ambiguous | Outcome::Restoring
            )
        }) {
            return Err(AiError::Stale);
        }
        fs::remove_file(&self.path).map_err(err)
    }
}
fn validate_new(
    grant: &ReadGrant,
    path: &str,
    file: bool,
    created: &[String],
) -> Result<PathBuf, AiError> {
    match grant.resolve(path, false, file) {
        Ok(p) => Ok(p),
        Err(error) => {
            let candidate = Path::new(path);
            if crate_path_safe(candidate, file)
                && candidate
                    .parent()
                    .is_some_and(|parent| created.iter().any(|p| parent == Path::new(p)))
            {
                Ok(grant.scope.join(candidate))
            } else {
                Err(error)
            }
        }
    }
}
fn crate_path_safe(path: &Path, file: bool) -> bool {
    !markion_ai::workspace::protected(path)
        && path.components().all(|c| {
            matches!(c, std::path::Component::Normal(_))
                && c.as_os_str()
                    .to_str()
                    .is_some_and(markion_ai::workspace::safe_name)
        })
        && (!file || markion_ai::workspace::supported(path))
}
pub fn inventory(directory: &Path) -> Vec<Result<Journal, AiError>> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    let mut results = Vec::new();
    for entry in entries.flatten().take(200) {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "json") {
            continue;
        }
        let result = (|| {
            let meta = fs::symlink_metadata(&path).map_err(err)?;
            if !meta.is_file() || meta.file_type().is_symlink() || meta.len() > JOURNAL_LIMIT {
                return Err(AiError::Limit);
            }
            let bytes = fs::read(&path).map_err(err)?;
            let mut journal: Journal =
                serde_json::from_slice(&bytes).map_err(|_| AiError::Protocol)?;
            journal.path = path;
            if journal.version != 1 {
                return Err(AiError::Protocol);
            }
            journal.reconcile()?;
            Ok(journal)
        })();
        results.push(result);
    }
    results
}
fn create_complete(path: &Path, bytes: &[u8]) -> io::Result<()> {
    create_complete_with_permissions(path, bytes, None)
}
fn create_complete_with_permissions(
    path: &Path,
    bytes: &[u8],
    permissions: Option<fs::Permissions>,
) -> io::Result<()> {
    let id = NEXT.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_file_name(format!(".markion-ai-{}-{id}.tmp", std::process::id()));
    let result = (|| {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temp)?;
        file.write_all(bytes)?;
        if let Some(permissions) = permissions {
            file.set_permissions(permissions)?;
        }
        file.sync_all()?;
        drop(file);
        move_exclusive(&temp, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}
#[cfg(windows)]
fn move_exclusive(source: &Path, target: &Path) -> io::Result<()> {
    use std::os::windows::ffi::OsStrExt;
    use windows::{
        Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
        core::PCWSTR,
    };
    let from = source
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let to = target
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    unsafe {
        MoveFileExW(
            PCWSTR(from.as_ptr()),
            PCWSTR(to.as_ptr()),
            MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(io::Error::other)
}
#[cfg(unix)]
fn move_exclusive(source: &Path, target: &Path) -> io::Result<()> {
    // link is an atomic no-overwrite admission; supported operations are regular files on one volume.
    fs::hard_link(source, target)?;
    fs::remove_file(source)
}
#[cfg(not(any(unix, windows)))]
fn move_exclusive(_: &Path, _: &Path) -> io::Result<()> {
    Err(io::Error::new(
        io::ErrorKind::Unsupported,
        "Unsupported platform",
    ))
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::BTreeMap;
    fn propose(plan: &mut Plan, name: &str, args: serde_json::Value, grant: &ReadGrant) {
        plan.host_call(name, args, grant, &BTreeMap::new(), 65536, 20)
            .unwrap();
    }
    #[test]
    fn durable_roundtrip_and_newer_content_conflict() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        fs::write(work.path().join("n.md"), "before").unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_text_edit",
            json!({"path":"n.md","start":0,"end":6,"replacement":"after"}),
            &grant,
        );
        propose(
            &mut plan,
            "propose_create_folder",
            json!({"path":"notes"}),
            &grant,
        );
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"notes/new.md","text":"new"}),
            &grant,
        );
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        for i in 0..j.records.len() {
            j.execute_one(i).unwrap();
        }
        assert_eq!(
            fs::read_to_string(work.path().join("n.md")).unwrap(),
            "after"
        );
        fs::write(work.path().join("n.md"), "later user edit").unwrap();
        assert!(j.restore_one(0).is_err());
        assert_eq!(
            fs::read_to_string(work.path().join("n.md")).unwrap(),
            "later user edit"
        );
        j.restore_one(2).unwrap();
        j.restore_one(1).unwrap();
        assert!(!work.path().join("notes").exists());
    }
    #[test]
    fn move_restore_and_crash_reconcile_no_replay() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        fs::write(work.path().join("a.md"), "text").unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_move_or_rename",
            json!({"path":"a.md","destination":"b.md"}),
            &grant,
        );
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        j.execute_one(0).unwrap();
        j.records[0].state = Outcome::Prepared;
        j.records[0].post_identity = None;
        j.persist().unwrap();
        let mut restored = inventory(recovery.path()).remove(0).unwrap();
        assert_eq!(restored.records[0].state, Outcome::Applied);
        assert!(restored.execute_one(0).is_err());
        restored.restore_one(0).unwrap();
        assert_eq!(
            fs::read_to_string(work.path().join("a.md")).unwrap(),
            "text"
        );
        assert!(!work.path().join("b.md").exists());
        restored.retire().unwrap();
    }
    #[test]
    fn partial_failure_cancel_and_preparation_fail_closed() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"a.md","text":"a"}),
            &grant,
        );
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"b.md","text":"b"}),
            &grant,
        );
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        j.execute_one(0).unwrap();
        fs::write(work.path().join("b.md"), "external").unwrap();
        assert!(j.execute_one(1).is_err());
        assert_eq!(j.records[0].state, Outcome::Applied);
        assert_eq!(j.records[1].state, Outcome::Failed);
        assert_eq!(
            fs::read_to_string(work.path().join("b.md")).unwrap(),
            "external"
        );
        assert!(Journal::prepare(recovery.path(), &grant, &plan).is_err());
    }
    #[test]
    fn unchanged_prepared_is_unapplied_and_unknown_versions_are_retained() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"a.md","text":"a"}),
            &grant,
        );
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        j.reconcile().unwrap();
        assert_eq!(j.records[0].state, Outcome::Unapplied);
        assert!(!work.path().join("a.md").exists());
        j.version = 99;
        j.persist().unwrap();
        assert!(inventory(recovery.path())[0].is_err());
        assert!(j.path.exists());
    }
    #[test]
    fn linked_move_roundtrip_and_interrupted_restore() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        fs::write(work.path().join("a.md"), "[B](b.md)").unwrap();
        fs::write(work.path().join("b.md"), "[A](a.md#x)").unwrap();
        fs::create_dir(work.path().join("folder")).unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_move_or_rename",
            json!({"path":"a.md","destination":"folder/a.md"}),
            &grant,
        );
        assert_eq!(
            markion_ai::links::propose_updates(&mut plan, &grant, &BTreeMap::new(), 65536, 20)
                .unwrap(),
            1
        );
        assert!(!plan.operations[1].selected);
        plan.operations[1].selected = true;
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        j.execute_one(0).unwrap();
        j.execute_one(1).unwrap();
        assert_eq!(
            fs::read_to_string(work.path().join("folder/a.md")).unwrap(),
            "[B](../b.md)"
        );
        assert_eq!(
            fs::read_to_string(work.path().join("b.md")).unwrap(),
            "[A](folder/a.md#x)"
        );
        j.restore_one(1).unwrap();
        j.records[0].state = Outcome::Restoring;
        j.persist().unwrap();
        j.reconcile().unwrap();
        assert_eq!(j.records[0].state, Outcome::Applied);
        j.restore_one(0).unwrap();
        j.records[0].state = Outcome::Restoring;
        j.persist().unwrap();
        j.reconcile().unwrap();
        assert_eq!(j.records[0].state, Outcome::Restored);
        assert_eq!(
            fs::read_to_string(work.path().join("a.md")).unwrap(),
            "[B](b.md)"
        );
        j.retire().unwrap();
    }
    #[test]
    fn duplicate_review_destinations_budget_and_partial_move_fail_closed() {
        let work = tempfile::tempdir().unwrap();
        let recovery = tempfile::tempdir().unwrap();
        let grant = ReadGrant::new(work.path(), work.path()).unwrap();
        let mut plan = Plan::default();
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"a.md","text":"a"}),
            &grant,
        );
        propose(
            &mut plan,
            "propose_create_note",
            json!({"path":"b.md","text":"b"}),
            &grant,
        );
        if let Operation::CreateNote { path, .. } = &mut plan.operations[1].operation {
            *path = "A.md".into();
        }
        assert!(Journal::prepare(recovery.path(), &grant, &plan).is_err());
        assert!(!work.path().join("a.md").exists());
        plan.operations.pop();
        let cap = fs::File::create(recovery.path().join("unresolved.json")).unwrap();
        cap.set_len(JOURNAL_LIMIT).unwrap();
        assert_eq!(
            Journal::prepare(recovery.path(), &grant, &plan).err(),
            Some(AiError::Limit)
        );
        assert!(!work.path().join("a.md").exists());
        fs::remove_file(recovery.path().join("unresolved.json")).unwrap();
        fs::write(work.path().join("source.md"), "before").unwrap();
        plan = Plan::default();
        propose(
            &mut plan,
            "propose_move_or_rename",
            json!({"path":"source.md","destination":"dest.md"}),
            &grant,
        );
        let mut j = Journal::prepare(recovery.path(), &grant, &plan).unwrap();
        // A crash between destination creation and source removal leaves both paths; never call this unapplied.
        fs::write(work.path().join("dest.md"), "before").unwrap();
        j.reconcile().unwrap();
        assert_eq!(j.records[0].state, Outcome::Ambiguous);
        assert!(j.execute_one(0).is_err());
        assert!(j.retire().is_err());
    }
}
