use std::{
    collections::{HashMap, HashSet},
    io::{Cursor, Read},
    path::{Component, Path, PathBuf},
};

use quick_xml::{Reader, events::Event};
use zip::ZipArchive;

use crate::{ImportContext, ImportError, ImportLimits};

const OFFICE_DOCUMENT_REL: &str = "/officeDocument";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageStats {
    pub entries: usize,
    pub decompressed_bytes: usize,
    pub limits: ImportLimits,
}

pub(crate) struct DocxPackage {
    parts: HashMap<String, Vec<u8>>,
    pub main_document: String,
    pub stats: PackageStats,
}

impl DocxPackage {
    pub fn read(bytes: &[u8], context: &ImportContext) -> Result<Self, ImportError> {
        context.checkpoint()?;
        let mut archive = ZipArchive::new(Cursor::new(bytes))
            .map_err(|error| ImportError::InvalidPackage(error.to_string()))?;
        if archive.len() > context.limits.max_entries {
            return Err(ImportError::LimitExceeded("package entries"));
        }

        let mut names = HashSet::new();
        let mut parts = HashMap::new();
        let mut total = 0usize;
        for index in 0..archive.len() {
            context.checkpoint()?;
            let mut entry = archive
                .by_index(index)
                .map_err(|error| ImportError::InvalidPackage(error.to_string()))?;
            let name = normalize_part_name(entry.name()).ok_or_else(|| {
                ImportError::InvalidPackage(format!("unsafe package path: {}", entry.name()))
            })?;
            if name.is_empty() || entry.is_dir() {
                continue;
            }
            if !names.insert(name.clone()) {
                return Err(ImportError::InvalidPackage(format!(
                    "duplicate package part: {name}"
                )));
            }
            let declared = usize::try_from(entry.size())
                .map_err(|_| ImportError::LimitExceeded("entry bytes"))?;
            if is_xml_part(&name) && declared > context.limits.max_xml_part_bytes {
                return Err(ImportError::LimitExceeded("individual XML part bytes"));
            }
            if declared > context.limits.max_decompressed_bytes.saturating_sub(total) {
                return Err(ImportError::LimitExceeded("decompressed package bytes"));
            }
            let per_entry_limit = if is_xml_part(&name) {
                context.limits.max_xml_part_bytes
            } else {
                context.limits.max_decompressed_bytes.saturating_sub(total)
            };
            let mut data = Vec::with_capacity(declared.min(per_entry_limit));
            let actual = entry
                .by_ref()
                .take(per_entry_limit as u64 + 1)
                .read_to_end(&mut data)
                .map_err(|error| ImportError::InvalidPackage(error.to_string()))?;
            if actual > per_entry_limit {
                return Err(ImportError::LimitExceeded(if is_xml_part(&name) {
                    "individual XML part bytes"
                } else {
                    "decompressed package bytes"
                }));
            }
            total = total
                .checked_add(actual)
                .ok_or(ImportError::LimitExceeded("decompressed package bytes"))?;
            if total > context.limits.max_decompressed_bytes {
                return Err(ImportError::LimitExceeded("decompressed package bytes"));
            }
            parts.insert(name, data);
        }

        if parts.contains_key("EncryptionInfo") || parts.contains_key("EncryptedPackage") {
            return Err(ImportError::Encrypted);
        }
        let content_types = xml_text(&parts, "[Content_Types].xml")?;
        validate_xml("[Content_Types].xml", content_types, context)?;
        let lower_types = content_types.to_ascii_lowercase();
        if lower_types.contains("macroenabled") || lower_types.contains("vbaproject") {
            return Err(ImportError::MacroEnabled);
        }
        let content_type_doc =
            roxmltree::Document::parse(content_types).map_err(|error| ImportError::InvalidXml {
                part: "[Content_Types].xml".into(),
                message: error.to_string(),
            })?;

        let root_rels = xml_text(&parts, "_rels/.rels")?;
        validate_xml("_rels/.rels", root_rels, context)?;
        let rel_doc =
            roxmltree::Document::parse(root_rels).map_err(|error| ImportError::InvalidXml {
                part: "_rels/.rels".into(),
                message: error.to_string(),
            })?;
        let relationship = rel_doc.descendants().find(|node| {
            node.is_element()
                && node.tag_name().name() == "Relationship"
                && node
                    .attribute("Type")
                    .is_some_and(|kind| kind.ends_with(OFFICE_DOCUMENT_REL))
        });
        let target = relationship
            .and_then(|node| node.attribute("Target"))
            .ok_or_else(|| ImportError::MissingPart("office document relationship".into()))?;
        if relationship
            .and_then(|node| node.attribute("TargetMode"))
            .is_some_and(|mode| mode.eq_ignore_ascii_case("External"))
        {
            return Err(ImportError::InvalidRelationship(
                "main document cannot be external".into(),
            ));
        }
        let main_document = resolve_part("", target)?;
        if !parts.contains_key(&main_document) {
            return Err(ImportError::MissingPart(main_document));
        }
        let declares_main = content_type_doc.descendants().any(|node| {
            node.is_element()
                && node.tag_name().name() == "Override"
                && node
                    .attribute("PartName")
                    .and_then(normalize_part_name)
                    .is_some_and(|name| name == main_document)
                && node
                    .attribute("ContentType")
                    .is_some_and(|kind| kind.eq_ignore_ascii_case(
                        "application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"
                    ))
        });
        if !declares_main {
            return Err(ImportError::InvalidPackage(format!(
                "package does not declare {main_document} as a WordprocessingML main document"
            )));
        }

        let stats = PackageStats {
            entries: parts.len(),
            decompressed_bytes: total,
            limits: context.limits.clone(),
        };
        Ok(Self {
            parts,
            main_document,
            stats,
        })
    }

    pub fn bytes(&self, name: &str) -> Option<&[u8]> {
        self.parts.get(name).map(Vec::as_slice)
    }

    pub fn text(&self, name: &str) -> Result<Option<&str>, ImportError> {
        self.bytes(name)
            .map(|bytes| {
                std::str::from_utf8(bytes).map_err(|error| ImportError::InvalidXml {
                    part: name.into(),
                    message: error.to_string(),
                })
            })
            .transpose()
    }

    pub fn required_text(&self, name: &str) -> Result<&str, ImportError> {
        self.text(name)?
            .ok_or_else(|| ImportError::MissingPart(name.into()))
    }

    pub fn relationship_part(&self, source: &str) -> String {
        let source = Path::new(source);
        let parent = source.parent().unwrap_or(Path::new(""));
        let name = source.file_name().unwrap_or_default().to_string_lossy();
        normalize_part_name(
            &parent
                .join("_rels")
                .join(format!("{name}.rels"))
                .to_string_lossy(),
        )
        .unwrap_or_default()
    }

    pub fn resolve_from(&self, source: &str, target: &str) -> Result<String, ImportError> {
        let parent = Path::new(source)
            .parent()
            .unwrap_or(Path::new(""))
            .to_string_lossy();
        resolve_part(&parent, target)
    }

    pub fn xml(&self, name: &str, context: &ImportContext) -> Result<Option<&str>, ImportError> {
        let Some(text) = self.text(name)? else {
            return Ok(None);
        };
        validate_xml(name, text, context)?;
        Ok(Some(text))
    }
}

fn xml_text<'a>(parts: &'a HashMap<String, Vec<u8>>, name: &str) -> Result<&'a str, ImportError> {
    let bytes = parts
        .get(name)
        .ok_or_else(|| ImportError::MissingPart(name.into()))?;
    std::str::from_utf8(bytes).map_err(|error| ImportError::InvalidXml {
        part: name.into(),
        message: error.to_string(),
    })
}

pub(crate) fn validate_xml(
    name: &str,
    text: &str,
    context: &ImportContext,
) -> Result<(), ImportError> {
    if text.contains("<!DOCTYPE") || text.contains("<!ENTITY") {
        return Err(ImportError::InvalidXml {
            part: name.into(),
            message: "DTD/entity declarations are forbidden".into(),
        });
    }
    let mut reader = Reader::from_str(text);
    reader.config_mut().check_end_names = true;
    let mut depth = 0usize;
    loop {
        context.checkpoint()?;
        match reader.read_event() {
            Ok(Event::Start(_)) => {
                depth += 1;
                if depth > context.limits.max_xml_depth {
                    return Err(ImportError::LimitExceeded("XML nesting depth"));
                }
            }
            Ok(Event::End(_)) => depth = depth.saturating_sub(1),
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => {
                return Err(ImportError::InvalidXml {
                    part: name.into(),
                    message: error.to_string(),
                });
            }
        }
    }
    Ok(())
}

fn is_xml_part(name: &str) -> bool {
    name.ends_with(".xml") || name.ends_with(".rels") || name == "[Content_Types].xml"
}

pub(crate) fn normalize_part_name(name: &str) -> Option<String> {
    let normalized = name.replace('\\', "/");
    if normalized.contains('\0') {
        return None;
    }
    let path = Path::new(normalized.trim_start_matches('/'));
    let mut result = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => result.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    return None;
                }
            }
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    Some(result.to_string_lossy().replace('\\', "/"))
}

pub(crate) fn resolve_part(base: &str, target: &str) -> Result<String, ImportError> {
    if target.contains(':') || target.starts_with("//") {
        return Err(ImportError::InvalidRelationship(target.into()));
    }
    let joined = if target.starts_with('/') {
        target.trim_start_matches('/').to_string()
    } else if base.is_empty() {
        target.to_string()
    } else {
        format!("{base}/{target}")
    };
    normalize_part_name(&joined)
        .ok_or_else(|| ImportError::InvalidRelationship(format!("escaping target: {target}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_paths_are_contained() {
        assert_eq!(
            resolve_part("word", "media/image.png").unwrap(),
            "word/media/image.png"
        );
        assert_eq!(resolve_part("word", "../custom.xml").unwrap(), "custom.xml");
        assert!(resolve_part("", "../../secret").is_err());
        assert!(resolve_part("word", "file:///secret").is_err());
    }
}
