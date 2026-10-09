//! Source-mapped relative Markdown link analysis; never traverses outside a read grant.
use crate::{
    AiError,
    proposals::{Baseline, Operation, Plan, TextEdit},
    workspace::{BufferSnapshot, ReadGrant, identity, list, read},
};
use pulldown_cmark::{Event, LinkType, Options, Parser, Tag};
use std::{
    collections::BTreeMap,
    path::{Component, Path, PathBuf},
};
fn normalized(path: &Path) -> Option<PathBuf> {
    let mut out = PathBuf::new();
    for c in path.components() {
        match c {
            Component::Normal(v) => out.push(v),
            Component::ParentDir => {
                if !out.pop() {
                    return None;
                }
            }
            Component::CurDir => {}
            _ => return None,
        }
    }
    Some(out)
}
fn relative(from: &Path, to: &Path) -> String {
    let a = from.components().collect::<Vec<_>>();
    let b = to.components().collect::<Vec<_>>();
    let common = a.iter().zip(&b).take_while(|(x, y)| x == y).count();
    let mut parts = vec!["..".to_owned(); a.len() - common];
    parts.extend(
        b[common..]
            .iter()
            .map(|c| c.as_os_str().to_string_lossy().into_owned()),
    );
    parts.join("/")
}
pub fn link_edits(text: &str, source: &str, moved_from: &str, moved_to: &str) -> Vec<TextEdit> {
    let parser = Parser::new_ext(text, Options::all());
    let mut destinations = parser
        .reference_definitions()
        .iter()
        .map(|(_, d)| (d.dest.to_string(), d.span.clone(), true))
        .collect::<Vec<_>>();
    for (event, span) in parser.into_offset_iter() {
        if let Event::Start(
            Tag::Link {
                link_type: LinkType::Inline,
                dest_url,
                ..
            }
            | Tag::Image {
                link_type: LinkType::Inline,
                dest_url,
                ..
            },
        ) = event
        {
            destinations.push((dest_url.into_string(), span, false));
        }
    }
    let old = Path::new(moved_from);
    let new = Path::new(moved_to);
    let parent = Path::new(source).parent().unwrap_or(Path::new(""));
    let mut edits = Vec::new();
    for (dest, span, reference) in destinations {
        let split = dest.find(['#', '?']).unwrap_or(dest.len());
        let (url, suffix) = dest.split_at(split);
        if url.is_empty() || url.contains(':') || url.starts_with('/') || url.contains('\\') {
            continue;
        }
        let Ok(decoded) = percent_encoding::percent_decode_str(url).decode_utf8() else {
            continue;
        };
        let Some(resolved) = normalized(&parent.join(decoded.as_ref())) else {
            continue;
        };
        if resolved != old && source != moved_from {
            continue;
        }
        let target = if resolved == old {
            new
        } else {
            resolved.as_path()
        };
        let new_parent = if source == moved_from {
            new.parent().unwrap_or(Path::new(""))
        } else {
            parent
        };
        const ESCAPE: &percent_encoding::AsciiSet = &percent_encoding::CONTROLS
            .add(b' ')
            .add(b'"')
            .add(b'<')
            .add(b'>')
            .add(b'#')
            .add(b'?');
        let relative = relative(new_parent, target);
        let relative = percent_encoding::utf8_percent_encode(&relative, ESCAPE).to_string();
        if relative == url {
            continue;
        }
        let replacement = format!("{relative}{suffix}");
        let Some(raw) = text.get(span.clone()) else {
            continue;
        };
        let search_from = if reference {
            raw.find("]:").map(|i| i + 2).unwrap_or(0)
        } else {
            raw.find("](").map(|i| i + 2).unwrap_or(0)
        };
        let Some(index) = raw[search_from..].find(url).map(|i| i + search_from) else {
            continue;
        };
        if raw[index..].starts_with(url) {
            edits.push(TextEdit {
                start: span.start + index,
                end: span.start + index + url.len(),
                replacement: replacement
                    .strip_suffix(suffix)
                    .unwrap_or(&replacement)
                    .into(),
            });
        }
    }
    edits.sort_by_key(|e| e.start);
    edits.dedup_by_key(|e| e.start);
    edits
}
pub fn propose_updates(
    plan: &mut Plan,
    grant: &ReadGrant,
    buffers: &BTreeMap<PathBuf, BufferSnapshot>,
    budget: usize,
    max: usize,
) -> Result<usize, AiError> {
    let moves = plan
        .operations
        .iter()
        .filter_map(|p| match &p.operation {
            Operation::Move {
                path, destination, ..
            } => Some((p.id, path.clone(), destination.clone())),
            _ => None,
        })
        .collect::<Vec<_>>();
    let mut offset = 0;
    let mut used = 0;
    let mut added = 0;
    for (id, from, to) in &moves {
        if let Some(proposed) = plan.operations.iter_mut().find(|p| p.id == *id) {
            if let Operation::Move {
                baseline,
                link_edits,
                ..
            } = &mut proposed.operation
            {
                *link_edits = crate::links::link_edits(&baseline.disk, from, from, to);
            }
        }
    }
    loop {
        let listing = list(grant, offset, 100)?;
        for path in listing.files {
            if moves.iter().any(|(_, from, _)| *from == path) {
                continue;
            }
            let result = read(grant, &path, None, 1024 * 1024, buffers)?;
            used += result.text.len();
            if used > budget {
                return Err(AiError::Limit);
            }
            let mut edits = Vec::new();
            let mut dependencies = Vec::new();
            for (id, from, to) in &moves {
                let next = link_edits(&result.text, &path, from, to);
                if !next.is_empty() {
                    edits.extend(next);
                    dependencies.push(*id);
                }
            }
            if edits.is_empty() {
                continue;
            }
            let resolved = grant.resolve(&path, true, true)?;
            let disk = std::fs::read_to_string(&resolved).map_err(|_| AiError::Scope)?;
            let baseline = Baseline {
                identity: identity(&resolved)?,
                disk,
                text: result.text,
                buffer: buffers.get(&resolved).map(|b| (b.document, b.version)),
            };
            let id = plan.add(
                Operation::EditText {
                    path,
                    baseline,
                    edits,
                },
                max,
            )?;
            if let Some(p) = plan.operations.iter_mut().find(|p| p.id == id) {
                for dep in dependencies {
                    if !p.dependencies.contains(&dep) {
                        p.dependencies.push(dep);
                    }
                }
                p.selected = false;
            }
            added += 1;
        }
        match listing.next {
            Some(next) => offset = next,
            None => break,
        }
    }
    plan.unchecked_links = true;
    Ok(added)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inline_images_and_reference_definitions_keep_fragments_and_titles() {
        let text = "[note](../a.md#part \"title\")\n![note](../a.md)\n[n][ref]\n\n[ref]: ../a.md#anchor \"title\"\n\n```md\n[ignored](../a.md)\n```\n";
        let edits = link_edits(text, "notes/index.md", "a.md", "archive/a.md");
        assert_eq!(edits.len(), 3);
        let after = crate::proposals::apply_edits(text, &edits).unwrap();
        assert!(after.contains("../archive/a.md#part"));
        assert!(after.contains("../archive/a.md#anchor"));
        assert!(after.contains("[ignored](../a.md)"));
        assert!(link_edits(text, "notes/index.md", "outside/a.md", "b.md").is_empty());
    }
}
