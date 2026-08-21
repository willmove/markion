//! Maintainer-only local browser harness for the pinned workspace.

use std::{process::ExitCode, sync::Arc};

use wechat_workspace::{
    PublishingResource, PublishingSnapshot, WorkspaceConfig, WorkspaceService,
    discover_workspace_assets,
};

#[tokio::main]
async fn main() -> ExitCode {
    let result = async {
        let assets = discover_workspace_assets()?;
        let packaged_assets = assets
            .parent()
            .expect("workspace lives below the packaged assets directory");
        let preview_image = packaged_assets.join("markion.png");
        let resource = PublishingResource::from_path(
            "note.assets/markion.png",
            packaged_assets,
            &preview_image,
        )
        .expect("packaged Markion icon is a supported browser-harness image");
        let service = WorkspaceService::new(WorkspaceConfig::new(assets))?;
        let launch = service
            .create_session(PublishingSnapshot {
                markdown: Arc::from(
                    "# MarkNice browser check\n\n- Theme and **rich text**\n- Math: $E=mc^2$\n\n| A | B |\n|---|---|\n| 1 | 2 |\n\n![managed](note.assets/markion.png)",
                ),
                display_name: "browser-check.md".into(),
                language: "en".into(),
                resources: vec![resource],
                unresolved_local_images: Vec::new(),
            })
            .await?;
        println!("WORKSPACE_URL={}", launch.url());
        println!(
            "SELF_TEST_URL=http://{}/static/self-test.html",
            service.local_addr().expect("service just started")
        );
        tokio::signal::ctrl_c()
            .await
            .map_err(wechat_workspace::WorkspaceError::Bind)?;
        Ok::<(), wechat_workspace::WorkspaceError>(())
    }
    .await;
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("preview workspace failed: {error}");
            ExitCode::FAILURE
        }
    }
}
