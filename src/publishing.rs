//! Explicit, read-only handoff from Markion's document model to the local
//! browser publishing workspace.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use percent_encoding::percent_decode_str;
use wechat_workspace::{PublishingResource, PublishingSnapshot};

use crate::{MarkdownDocument, document_asset_dir};

/// Builds an immutable publishing snapshot without populating, invalidating,
/// or replacing any of the document's per-version derived caches.
pub fn build_publishing_snapshot(
    document: &MarkdownDocument,
    language: impl Into<String>,
) -> PublishingSnapshot {
    let references = document.publishing_image_references();
    let mut resources = Vec::new();
    let mut unresolved_local_images = Vec::new();

    if let Some(document_path) = document.path() {
        let asset_root = document_asset_dir(document_path);
        let document_dir = document_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        for authored_url in references {
            if !is_local_reference(&authored_url) {
                continue;
            }
            let candidate = decoded_candidate(document_dir, &authored_url);
            match candidate.and_then(|candidate| {
                PublishingResource::from_path(&authored_url, &asset_root, &candidate).ok()
            }) {
                Some(resource) => resources.push(resource),
                None => unresolved_local_images.push(authored_url),
            }
        }
    } else {
        unresolved_local_images.extend(
            references
                .into_iter()
                .filter(|reference| is_local_reference(reference)),
        );
    }

    PublishingSnapshot {
        markdown: Arc::from(document.text()),
        display_name: document
            .path()
            .and_then(Path::file_name)
            .and_then(|name| name.to_str())
            .unwrap_or("Untitled")
            .to_owned(),
        language: language.into(),
        resources,
        unresolved_local_images,
    }
}

fn is_local_reference(reference: &str) -> bool {
    let lower = reference.trim_start().to_ascii_lowercase();
    !(lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("data:")
        || lower.starts_with("blob:")
        || lower.starts_with("//"))
}

fn decoded_candidate(document_dir: &Path, reference: &str) -> Option<PathBuf> {
    let path = reference.split(['?', '#']).next().unwrap_or_default();
    let decoded = percent_decode_str(path).decode_utf8().ok()?;
    let mut candidate = document_dir.to_path_buf();
    for component in decoded.replace('\\', "/").split('/') {
        candidate.push(component);
    }
    Some(candidate)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn saved_document_allows_only_managed_images_without_touching_caches() {
        let temp = tempfile::tempdir().unwrap();
        let document_path = temp.path().join("My Note.md");
        let managed_dir = document_asset_dir(&document_path);
        fs::create_dir(&managed_dir).unwrap();
        fs::write(managed_dir.join("cover.png"), b"image").unwrap();
        fs::write(temp.path().join("outside.png"), b"outside").unwrap();
        let mut document = MarkdownDocument::from_text(
            "![managed](my-note.assets/cover.png)\n![outside](outside.png)\n![remote](https://example.com/a.png)",
        );
        document.save_as(&document_path).unwrap();
        document.insert(document.text().len(), "\nDirty edit");
        let preview = document.preview_blocks_shared();
        let visual = document.visual_blocks_shared();
        let before_memory = document.memory_breakdown();
        let version = document.version();
        let dirty = document.is_dirty();

        let snapshot = build_publishing_snapshot(&document, "en");

        assert_eq!(snapshot.markdown.as_ref(), document.text());
        assert_eq!(snapshot.resources.len(), 1);
        assert_eq!(
            snapshot.resources[0].authored_url(),
            "my-note.assets/cover.png"
        );
        assert_eq!(snapshot.unresolved_local_images, ["outside.png"]);
        assert_eq!(document.version(), version);
        assert_eq!(document.is_dirty(), dirty);
        assert_eq!(document.memory_breakdown(), before_memory);
        assert!(Arc::ptr_eq(&preview, &document.preview_blocks_shared()));
        assert!(Arc::ptr_eq(&visual, &document.visual_blocks_shared()));
    }

    #[test]
    fn untitled_document_grants_no_filesystem_authority() {
        let document = MarkdownDocument::from_text(
            "![local](note.assets/a.png) ![remote](https://example.com/a.png)",
        );
        let snapshot = build_publishing_snapshot(&document, "zh-hans");
        assert!(snapshot.resources.is_empty());
        assert_eq!(snapshot.unresolved_local_images, ["note.assets/a.png"]);
        assert_eq!(snapshot.display_name, "Untitled");
    }

    #[test]
    fn empty_document_is_a_valid_immutable_snapshot() {
        let document = MarkdownDocument::new();
        let version = document.version();
        let dirty = document.is_dirty();

        let snapshot = build_publishing_snapshot(&document, "en");

        assert!(snapshot.markdown.is_empty());
        assert!(snapshot.resources.is_empty());
        assert!(snapshot.unresolved_local_images.is_empty());
        assert_eq!(document.version(), version);
        assert_eq!(document.is_dirty(), dirty);
    }
}
