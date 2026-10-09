//! Typed immutable proposals and dependency validation. Construction has no filesystem effects.
use crate::{
    AiError,
    workspace::{
        BufferSnapshot, FileIdentity, ReadGrant, identity, protected, read, safe_name, supported,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub start: usize,
    pub end: usize,
    pub replacement: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Baseline {
    pub identity: FileIdentity,
    pub disk: String,
    pub text: String,
    pub buffer: Option<(u64, u64)>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Operation {
    CreateNote {
        path: String,
        text: String,
    },
    CreateFolder {
        path: String,
    },
    EditText {
        path: String,
        baseline: Baseline,
        edits: Vec<TextEdit>,
    },
    Move {
        path: String,
        destination: String,
        baseline: Baseline,
        #[serde(default)]
        link_edits: Vec<TextEdit>,
    },
}
impl Operation {
    pub fn path(&self) -> &str {
        match self {
            Self::CreateNote { path, .. }
            | Self::CreateFolder { path }
            | Self::EditText { path, .. }
            | Self::Move { path, .. } => path,
        }
    }
    pub fn destination(&self) -> Option<&str> {
        match self {
            Self::Move { destination, .. } => Some(destination),
            _ => None,
        }
    }
    pub fn before(&self) -> Option<&str> {
        match self {
            Self::EditText { baseline, .. } | Self::Move { baseline, .. } => Some(&baseline.text),
            _ => None,
        }
    }
    pub fn after(&self) -> Result<Option<String>, AiError> {
        match self {
            Self::CreateNote { text, .. } => Ok(Some(text.clone())),
            Self::EditText {
                baseline, edits, ..
            } => apply_edits(&baseline.text, edits).map(Some),
            Self::Move {
                baseline,
                link_edits,
                ..
            } => apply_edits(&baseline.disk, link_edits).map(Some),
            Self::CreateFolder { .. } => Ok(None),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Proposed {
    pub id: usize,
    pub selected: bool,
    pub dependencies: Vec<usize>,
    pub operation: Operation,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Plan {
    pub operations: Vec<Proposed>,
    pub unchecked_links: bool,
    pub revision: u64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ReadArgs {
    path: String,
    #[serde(default)]
    start: usize,
    #[serde(default)]
    end: Option<usize>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EditArgs {
    path: String,
    start: usize,
    end: usize,
    replacement: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CreateArgs {
    path: String,
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PathArgs {
    path: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MoveArgs {
    path: String,
    destination: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ListArgs {
    offset: usize,
    page: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchArgs {
    query: String,
    text: bool,
    offset: usize,
}
fn parse<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T, AiError> {
    serde_json::from_value(value).map_err(|_| AiError::Protocol)
}
pub fn apply_edits(text: &str, edits: &[TextEdit]) -> Result<String, AiError> {
    let mut edits = edits.to_vec();
    edits.sort_by_key(|e| (e.start, e.end));
    let mut last = None;
    for edit in &edits {
        if text.get(edit.start..edit.end).is_none()
            || last
                .is_some_and(|end| edit.start < end || edit.start == end && edit.start == edit.end)
        {
            return Err(AiError::Stale);
        }
        last = Some(edit.end);
    }
    let mut result = text.to_owned();
    for edit in edits.iter().rev() {
        result.replace_range(edit.start..edit.end, &edit.replacement);
    }
    Ok(result)
}
fn baseline(
    grant: &ReadGrant,
    path: &str,
    buffers: &BTreeMap<PathBuf, BufferSnapshot>,
) -> Result<Baseline, AiError> {
    let resolved = grant.resolve(path, true, true)?;
    let content = read(grant, path, None, 1024 * 1024, buffers)?;
    let disk = fs::read_to_string(&resolved).map_err(|_| AiError::Scope)?;
    Ok(Baseline {
        identity: identity(&resolved)?,
        disk,
        text: content.text,
        buffer: buffers.get(&resolved).map(|b| (b.document, b.version)),
    })
}
impl Plan {
    pub fn add(&mut self, operation: Operation, max: usize) -> Result<usize, AiError> {
        if self.operations.len() >= max {
            return Err(AiError::Limit);
        }
        if let Operation::EditText {
            path,
            baseline,
            edits,
        } = &operation
        {
            if let Some(existing) = self
                .operations
                .iter_mut()
                .find(|p| matches!(&p.operation,Operation::EditText{path:p,..}if p==path))
            {
                if let Operation::EditText {
                    baseline: b,
                    edits: e,
                    ..
                } = &mut existing.operation
                {
                    if b.identity != baseline.identity || b.text != baseline.text {
                        return Err(AiError::Stale);
                    }
                    let mut merged = e.clone();
                    merged.extend(edits.clone());
                    apply_edits(&b.text, &merged)?;
                    *e = merged;
                    self.revision += 1;
                    return Ok(existing.id);
                }
            }
        }
        if self.operations.iter().any(|p| {
            p.operation.path() == operation.path()
                || operation.destination().is_some_and(|d| {
                    p.operation.destination() == Some(d) || p.operation.path() == d
                })
        }) {
            return Err(AiError::Stale);
        }
        let mut dependencies = Vec::new();
        let dest = operation.destination().unwrap_or_else(|| operation.path());
        for p in &self.operations {
            if let Operation::CreateFolder { path } = &p.operation {
                if Path::new(dest)
                    .parent()
                    .is_some_and(|parent| parent.starts_with(path))
                {
                    dependencies.push(p.id);
                }
            }
        }
        let id = self.operations.len();
        self.operations.push(Proposed {
            id,
            selected: true,
            dependencies,
            operation,
        });
        self.revision += 1;
        Ok(id)
    }
    /// Rebuild dependencies and check edited review paths before a fresh approval.
    pub fn revalidate(&self, grant: &ReadGrant, max: usize) -> Result<Self, AiError> {
        grant.validate()?;
        let mut next = Self::default();
        let mut ids = BTreeMap::new();
        for p in &self.operations {
            match &p.operation {
                Operation::CreateFolder { path } => {
                    next.destination(grant, path, false)?;
                }
                Operation::CreateNote { path, .. } => {
                    next.destination(grant, path, true)?;
                }
                Operation::EditText { path, baseline, .. }
                | Operation::Move { path, baseline, .. } => {
                    let resolved = grant.resolve(path, true, true)?;
                    if identity(&resolved)? != baseline.identity
                        || fs::read_to_string(resolved).map_err(|_| AiError::Scope)?
                            != baseline.disk
                    {
                        return Err(AiError::Stale);
                    }
                    if let Some(d) = p.operation.destination() {
                        next.destination(grant, d, true)?;
                    }
                    p.operation.after()?;
                }
            }
            let id = next.add(p.operation.clone(), max)?;
            ids.insert(p.id, id);
            next.operations[id].selected = p.selected;
            for dep in &p.dependencies {
                let mapped = *ids.get(dep).ok_or(AiError::Stale)?;
                if !next.operations[id].dependencies.contains(&mapped) {
                    next.operations[id].dependencies.push(mapped);
                }
            }
        }
        next.unchecked_links = self.unchecked_links;
        next.revision = self.revision + 1;
        Ok(next)
    }
    pub fn validate_selection(&self) -> Result<(), AiError> {
        let selected = self
            .operations
            .iter()
            .filter(|p| p.selected)
            .map(|p| p.id)
            .collect::<BTreeSet<_>>();
        for p in self.operations.iter().filter(|p| p.selected) {
            if p.dependencies
                .iter()
                .any(|id| *id >= p.id || !selected.contains(id))
            {
                return Err(AiError::Stale);
            }
        }
        Ok(())
    }
    pub fn select(&mut self, id: usize, selected: bool) -> Result<(), AiError> {
        let target = self
            .operations
            .iter()
            .find(|p| p.id == id)
            .ok_or(AiError::Stale)?;
        if selected
            && target.dependencies.iter().any(|dependency| {
                self.operations
                    .iter()
                    .find(|p| p.id == *dependency)
                    .is_none_or(|p| !p.selected)
            })
            || !selected
                && self
                    .operations
                    .iter()
                    .any(|p| p.selected && p.dependencies.contains(&id))
        {
            return Err(AiError::Stale);
        }
        self.operations
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or(AiError::Stale)?
            .selected = selected;
        self.revision += 1;
        Ok(())
    }
    fn destination(&self, grant: &ReadGrant, path: &str, file: bool) -> Result<PathBuf, AiError> {
        match grant.resolve(path, false, file) {
            Ok(p) => Ok(p),
            Err(error) => {
                let candidate = Path::new(path);
                if protected(candidate)
                    || candidate
                        .components()
                        .any(|c| !matches!(c, std::path::Component::Normal(_)))
                    || !candidate
                        .components()
                        .all(|c| c.as_os_str().to_str().is_some_and(safe_name))
                    || file && !supported(candidate)
                {
                    return Err(error);
                }
                let parent = candidate.parent().ok_or(error)?;
                let parent_string = parent.to_string_lossy().replace('\\', "/");
                if !self.operations.iter().any(
                    |p| matches!(&p.operation,Operation::CreateFolder{path}if *path==parent_string),
                ) {
                    return Err(error);
                }
                grant.validate()?;
                Ok(grant.scope.join(candidate))
            }
        }
    }
    pub fn host_call(
        &mut self,
        name: &str,
        args: serde_json::Value,
        grant: &ReadGrant,
        buffers: &BTreeMap<PathBuf, BufferSnapshot>,
        budget: usize,
        max: usize,
    ) -> Result<serde_json::Value, AiError> {
        use serde_json::json;
        match name {
            "list_files" => {
                let a: ListArgs = parse(args)?;
                serde_json::to_value(crate::workspace::list(grant, a.offset, a.page)?)
                    .map_err(|_| AiError::Protocol)
            }
            "search_files" => {
                let a: SearchArgs = parse(args)?;
                let (hits, next) =
                    crate::workspace::search(grant, &a.query, a.text, a.offset, budget, buffers)?;
                Ok(json!({"hits":hits,"next":next}))
            }
            "read_text" => {
                let a: ReadArgs = parse(args)?;
                let range = match a.end {
                    Some(end) => Some(a.start..end),
                    None if a.start == 0 => None,
                    None => return Err(AiError::Scope),
                };
                serde_json::to_value(read(grant, &a.path, range, budget, buffers)?)
                    .map_err(|_| AiError::Protocol)
            }
            "propose_text_edit" => {
                let a: EditArgs = parse(args)?;
                let b = baseline(grant, &a.path, buffers)?;
                let edits = vec![TextEdit {
                    start: a.start,
                    end: a.end,
                    replacement: a.replacement,
                }];
                let after = apply_edits(&b.text, &edits)?;
                if after.len() > 1024 * 1024 {
                    return Err(AiError::Limit);
                }
                let id = self.add(
                    Operation::EditText {
                        path: a.path,
                        baseline: b,
                        edits,
                    },
                    max,
                )?;
                Ok(json!({"proposal":id,"state":"awaiting_user_review"}))
            }
            "propose_create_note" => {
                let a: CreateArgs = parse(args)?;
                self.destination(grant, &a.path, true)?;
                if a.text.len() > budget || a.text.contains('\0') {
                    return Err(AiError::Limit);
                }
                let id = self.add(
                    Operation::CreateNote {
                        path: a.path,
                        text: a.text,
                    },
                    max,
                )?;
                Ok(json!({"proposal":id,"state":"awaiting_user_review"}))
            }
            "propose_create_folder" => {
                let a: PathArgs = parse(args)?;
                self.destination(grant, &a.path, false)?;
                let id = self.add(Operation::CreateFolder { path: a.path }, max)?;
                Ok(json!({"proposal":id,"state":"awaiting_user_review"}))
            }
            "propose_move_or_rename" => {
                let a: MoveArgs = parse(args)?;
                let b = baseline(grant, &a.path, buffers)?;
                self.destination(grant, &a.destination, true)?;
                if b.buffer.is_some() && b.text != b.disk {
                    return Err(AiError::Stale);
                }
                let id = self.add(
                    Operation::Move {
                        path: a.path,
                        destination: a.destination,
                        baseline: b,
                        link_edits: Vec::new(),
                    },
                    max,
                )?;
                self.unchecked_links = true;
                Ok(
                    json!({"proposal":id,"state":"awaiting_user_review","links_outside_scope":"unchecked"}),
                )
            }
            _ => Err(AiError::Unsupported),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn proposals_are_pure_and_edits_merge_safely() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("n.md"), "abc😀def").unwrap();
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        let mut p = Plan::default();
        let b = BTreeMap::new();
        p.host_call(
            "propose_text_edit",
            serde_json::json!({"path":"n.md","start":0,"end":1,"replacement":"A"}),
            &grant,
            &b,
            4096,
            20,
        )
        .unwrap();
        p.host_call(
            "propose_text_edit",
            serde_json::json!({"path":"n.md","start":7,"end":10,"replacement":"DEF"}),
            &grant,
            &b,
            4096,
            20,
        )
        .unwrap();
        assert_eq!(p.operations.len(), 1);
        assert_eq!(
            p.operations[0].operation.after().unwrap().unwrap(),
            "Abc😀DEF"
        );
        assert_eq!(
            fs::read_to_string(dir.path().join("n.md")).unwrap(),
            "abc😀def"
        );
        assert!(
            p.host_call(
                "propose_text_edit",
                serde_json::json!({"path":"n.md","start":0,"end":3,"replacement":"X"}),
                &grant,
                &b,
                4096,
                20
            )
            .is_err()
        );
        assert!(
            p.host_call("shell", serde_json::json!({}), &grant, &b, 4096, 20)
                .is_err()
        );
        assert!(
            p.host_call(
                "read_text",
                serde_json::json!({"path":"n.md","start":0,"end":1,"command":"bad"}),
                &grant,
                &b,
                4096,
                20
            )
            .is_err()
        );
    }
    #[test]
    fn edited_destinations_need_fresh_dependencies_and_baselines() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("a.md"), "alpha").unwrap();
        let grant = ReadGrant::new(dir.path(), dir.path()).unwrap();
        let mut plan = Plan::default();
        let buffers = BTreeMap::new();
        let whole = plan
            .host_call(
                "read_text",
                json!({"path":"a.md"}),
                &grant,
                &buffers,
                4096,
                20,
            )
            .unwrap();
        assert_eq!(whole["text"], "alpha");
        assert_eq!(plan.operations.len(), 0);
        plan.host_call(
            "propose_create_folder",
            json!({"path":"notes"}),
            &grant,
            &buffers,
            4096,
            20,
        )
        .unwrap();
        plan.host_call(
            "propose_move_or_rename",
            json!({"path":"a.md","destination":"moved.md"}),
            &grant,
            &buffers,
            4096,
            20,
        )
        .unwrap();
        if let Operation::Move { destination, .. } = &mut plan.operations[1].operation {
            *destination = "notes/moved.md".into();
        }
        let refreshed = plan.revalidate(&grant, 20).unwrap();
        assert!(refreshed.revision > plan.revision);
        assert_eq!(refreshed.operations[1].dependencies, vec![0]);
        let mut unselected = refreshed.clone();
        unselected.operations[0].selected = false;
        assert!(unselected.validate_selection().is_err());
        fs::write(dir.path().join("a.md"), "newer user content").unwrap();
        assert!(matches!(
            refreshed.revalidate(&grant, 20),
            Err(AiError::Stale)
        ));
        assert!(!dir.path().join("notes").exists());
    }
    #[test]
    fn selected_dependencies_are_required() {
        let dir = tempfile::tempdir().unwrap();
        let g = ReadGrant::new(dir.path(), dir.path()).unwrap();
        let mut p = Plan::default();
        p.host_call(
            "propose_create_folder",
            serde_json::json!({"path":"notes"}),
            &g,
            &BTreeMap::new(),
            4096,
            20,
        )
        .unwrap();
        p.host_call(
            "propose_create_note",
            serde_json::json!({"path":"notes/new.md","text":"new"}),
            &g,
            &BTreeMap::new(),
            4096,
            20,
        )
        .unwrap();
        assert!(p.select(0, false).is_err());
        assert!(p.operations[0].selected);
        p.select(1, false).unwrap();
        p.select(0, false).unwrap();
        assert!(p.select(1, true).is_err());
        p.operations[1].selected = true;
        assert!(p.validate_selection().is_err());
        assert!(!dir.path().join("notes").exists());
    }
}
