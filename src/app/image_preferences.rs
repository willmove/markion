use super::*;

impl MarkionApp {
    pub(super) fn set_local_image_policy(
        &mut self,
        value: markion::LocalImagePolicy,
        cx: &mut Context<Self>,
    ) {
        self.image_preferences.local_policy = value;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_clipboard_image_policy(
        &mut self,
        value: markion::ClipboardImagePolicy,
        cx: &mut Context<Self>,
    ) {
        self.image_preferences.clipboard_policy = value;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_remote_image_policy(
        &mut self,
        value: markion::RemoteImagePolicy,
        cx: &mut Context<Self>,
    ) {
        self.image_preferences.remote_policy = value;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_image_resource_directory(
        &mut self,
        value: impl Into<String>,
        cx: &mut Context<Self>,
    ) {
        self.image_preferences.directory = value.into();
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn set_image_uploader(
        &mut self,
        value: markion::ImageUploaderKind,
        cx: &mut Context<Self>,
    ) {
        self.image_preferences.uploader = value;
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn step_image_timeout(&mut self, delta: i64, cx: &mut Context<Self>) {
        self.image_preferences.timeout_secs = markion::normalize_image_transfer_timeout_secs(
            self.image_preferences.timeout_secs as i64 + delta,
        );
        self.persist_preferences();
        cx.notify();
    }

    pub(super) fn choose_image_provider_program(
        &mut self,
        core: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(image_t(self.language, ImageMsg::Program).into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let value = path.to_string_lossy().into_owned();
            let _ = this.update(cx, |app, cx| {
                if core {
                    app.image_preferences.picgo_core.executable = Some(value);
                } else {
                    app.image_preferences.command.executable = Some(value);
                }
                app.persist_preferences();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn choose_picgo_config(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(image_t(self.language, ImageMsg::ConfigFile).into()),
        });
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let value = path.to_string_lossy().into_owned();
            let _ = this.update(cx, |app, cx| {
                app.image_preferences.picgo_core.config_path = Some(value);
                app.persist_preferences();
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn test_image_upload(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let receiver = cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some(image_t(self.language, ImageMsg::ChooseFile).into()),
        });
        let preferences = self.image_preferences.clone();
        cx.spawn_in(window, async move |this, cx| {
            let Ok(Ok(Some(paths))) = receiver.await else {
                return;
            };
            let Some(path) = paths.into_iter().next() else {
                return;
            };
            let cancellation = markion::external_command::CommandCancellation::default();
            let staging = markion::default_image_recovery_dir().join("test-upload");
            let task = network::runtime_handle().spawn(async move {
                let mut materialized = markion::image_transfer::materialize_inputs_bounded(
                    vec![markion::ImageInput::Local(path)],
                    staging.clone(),
                    markion::image_transfer::ImageTransferLimits {
                        timeout: Duration::from_secs(preferences.timeout_secs),
                        ..Default::default()
                    },
                    cancellation.clone(),
                )
                .await;
                let image = materialized
                    .pop()
                    .expect("one test input")
                    .map_err(|error| error.to_string())?;
                markion::image_transfer::upload_materialized_image(
                    &preferences,
                    &image,
                    &staging,
                    &cancellation,
                )
                .await
                .map_err(|error| error.to_string())
            });
            let result = task.await;
            let _ = this.update(cx, |app, cx| {
                app.status = match result {
                    Ok(Ok(url)) => {
                        format!("{}: {url}", image_t(app.language, ImageMsg::TestUpload))
                    }
                    Ok(Err(error)) => {
                        format!("{}: {error}", image_t(app.language, ImageMsg::TestUpload))
                    }
                    Err(error) => {
                        format!("{}: {error}", image_t(app.language, ImageMsg::TestUpload))
                    }
                }
                .into();
                cx.notify();
            });
        })
        .detach();
    }
}
