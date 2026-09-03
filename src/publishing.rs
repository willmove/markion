//! Explicit, read-only handoff from Markion's document model to the local
//! browser publishing workspace.

use std::{path::Path, sync::Arc};

use wechat_workspace::{PublishingResource, PublishingSnapshot};

use crate::{
    MarkdownDocument,
    storage::resources::{document_scope_root, is_local_reference, resolve_local_reference},
};

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
        let scope_root = document_scope_root(document_path);
        let document_dir = document_path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        for authored_url in references {
            if !is_local_reference(&authored_url) {
                continue;
            }
            let candidate = resolve_local_reference(document_dir, &authored_url);
            match candidate.and_then(|candidate| {
                PublishingResource::from_path(&authored_url, &scope_root, &candidate).ok()
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document_asset_dir;
    use std::fs;

    #[test]
    fn saved_document_resolves_scope_images_without_touching_caches() {
        let temp = tempfile::tempdir().unwrap();
        let docs = temp.path().join("docs");
        let document_path = docs.join("My Note.md");
        let managed_dir = document_asset_dir(&document_path);
        fs::create_dir_all(managed_dir.parent().unwrap()).unwrap();
        fs::create_dir_all(docs.join("img/deep")).unwrap();
        fs::create_dir(&managed_dir).unwrap();
        fs::write(managed_dir.join("cover.png"), b"managed").unwrap();
        fs::write(docs.join("sibling.png"), b"sibling").unwrap();
        fs::write(docs.join("img/deep/nested.png"), b"nested").unwrap();
        fs::write(temp.path().join("banner.png"), b"banner").unwrap();
        let mut document = MarkdownDocument::from_text(
            "![managed](my-note.assets/cover.png)\n![sibling](sibling.png)\n![nested](img/deep/nested.png)\n![banner](../banner.png)\n![escape](../../escape.png)\n![absolute](/definitely/outside.png)\n![remote](https://example.com/a.png)",
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
        let resolved: Vec<_> = snapshot
            .resources
            .iter()
            .map(|resource| resource.authored_url())
            .collect();
        assert_eq!(
            resolved,
            [
                "../banner.png",
                "img/deep/nested.png",
                "my-note.assets/cover.png",
                "sibling.png"
            ]
        );
        assert_eq!(
            snapshot.unresolved_local_images,
            ["../../escape.png", "/definitely/outside.png"]
        );
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
