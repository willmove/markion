use super::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InsertionTransferKind {
    SaveLocal,
    Upload,
}

struct InsertionTransferGroup {
    input: markion::ImageInput,
    kind: InsertionTransferKind,
    input_indexes: Vec<usize>,
}

enum InsertionTransferOutcome {
    Materialized(markion::image_transfer::MaterializedImage),
    Uploaded {
        image: markion::image_transfer::MaterializedImage,
        url: String,
    },
    Failed(String),
}

struct InsertionExecutionResult {
    groups: Vec<InsertionTransferOutcome>,
    staging_directory: PathBuf,
}

impl MarkionApp {
    pub(super) fn cancel_image_operations(
        &mut self,
        _: &CancelImageOperations,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let ids = self.image_cancellations.keys().copied().collect::<Vec<_>>();
        for id in ids {
            if let Some(cancellation) = self.image_cancellations.remove(&id) {
                cancellation.cancel();
            }
            self.image_operations.cancel(id);
        }
        self.status = t(self.language, Msg::StatusCanceled).into();
        cx.notify();
    }

    pub(super) fn discard_image_recovery(&mut self, manifest: PathBuf, cx: &mut Context<Self>) {
        let _ = markion::discard_image_recovery_record(
            &markion::default_image_recovery_dir(),
            &manifest,
        );
        self.refresh_image_recovery_entries();
        cx.notify();
    }

    pub(super) fn copy_image_recovery_url(&mut self, manifest: &Path, cx: &mut Context<Self>) {
        let url = self
            .image_recovery_entries
            .iter()
            .find(|entry| entry.manifest_path == manifest)
            .and_then(|entry| entry.record.as_ref().ok())
            .and_then(|record| record.items.iter().find_map(|item| item.output_url.clone()));
        if let Some(url) = url {
            cx.write_to_clipboard(ClipboardItem::new_string(url));
            self.status = t(self.language, Msg::StatusCopiedSelection).into();
            cx.notify();
        }
    }

    pub(super) fn reveal_image_recovery_file(&mut self, manifest: &Path, cx: &mut Context<Self>) {
        let path = self
            .image_recovery_entries
            .iter()
            .find(|entry| entry.manifest_path == manifest)
            .and_then(|entry| entry.record.as_ref().ok())
            .and_then(|record| {
                record
                    .items
                    .iter()
                    .find_map(|item| item.output_path.clone())
                    .or_else(|| record.owned_inputs.first().cloned())
            });
        if let Some(path) = path {
            let _ = reveal_in_system_file_manager(&path, true);
            cx.notify();
        }
    }

    fn refresh_image_recovery_entries(&mut self) {
        self.image_recovery_entries =
            markion::list_image_recovery_records(&markion::default_image_recovery_dir())
                .unwrap_or_default();
    }

    pub(super) fn start_inserted_fragment_image_policies(
        &mut self,
        range: Range<usize>,
        cx: &mut Context<Self>,
    ) {
        let scan = self.active_tab().document.image_occurrence_scan();
        let mut save = Vec::new();
        let mut upload = Vec::new();
        for occurrence in scan.occurrences.iter().filter(|occurrence| {
            occurrence.full_range.start >= range.start && occurrence.full_range.end <= range.end
        }) {
            let destination = occurrence.semantic_url.trim();
            let target =
                if destination.starts_with("http://") || destination.starts_with("https://") {
                    match self.image_preferences.remote_policy {
                        markion::RemoteImagePolicy::Keep => None,
                        markion::RemoteImagePolicy::Download => Some(&mut save),
                        markion::RemoteImagePolicy::Upload => Some(&mut upload),
                    }
                } else if destination.starts_with("data:") {
                    match self.image_preferences.clipboard_policy {
                        markion::ClipboardImagePolicy::Save => Some(&mut save),
                        markion::ClipboardImagePolicy::Upload => Some(&mut upload),
                    }
                } else {
                    match self.image_preferences.local_policy {
                        markion::LocalImagePolicy::Keep => None,
                        markion::LocalImagePolicy::Copy => Some(&mut save),
                        markion::LocalImagePolicy::Upload => Some(&mut upload),
                    }
                };
            if let Some(target) = target {
                target.push(occurrence.id);
            }
        }
        if !save.is_empty() {
            self.start_existing_image_operation(
                markion::ImagePlanSelection::Occurrences(save),
                markion::ImageOperationKind::SaveToResources,
                cx,
            );
        }
        if !upload.is_empty() {
            self.start_existing_image_operation(
                markion::ImagePlanSelection::Occurrences(upload),
                markion::ImageOperationKind::Upload,
                cx,
            );
        }
    }

    pub(super) fn upload_selected_image(
        &mut self,
        _: &UploadSelectedImage,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_image_operation_at_offset(
            self.cursor_offset(),
            markion::ImageOperationKind::Upload,
            cx,
        );
    }

    pub(super) fn save_selected_image_locally(
        &mut self,
        _: &SaveSelectedImageLocally,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_image_operation_at_offset(
            self.cursor_offset(),
            markion::ImageOperationKind::SaveToResources,
            cx,
        );
    }

    pub(super) fn upload_document_images(
        &mut self,
        _: &UploadDocumentImages,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_existing_image_operation(
            markion::ImagePlanSelection::CurrentDocument,
            markion::ImageOperationKind::Upload,
            cx,
        );
    }

    pub(super) fn save_document_images_locally(
        &mut self,
        _: &SaveDocumentImagesLocally,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_existing_image_operation(
            markion::ImagePlanSelection::CurrentDocument,
            markion::ImageOperationKind::SaveToResources,
            cx,
        );
    }

    pub(super) fn start_image_operation_at_offset(
        &mut self,
        offset: usize,
        kind: markion::ImageOperationKind,
        cx: &mut Context<Self>,
    ) {
        let scan = self.active_tab().document.image_occurrence_scan();
        let occurrence = scan.occurrences.iter().find(|occurrence| {
            occurrence.full_range.start <= offset && offset <= occurrence.full_range.end
        });
        let Some(occurrence) = occurrence else {
            self.status = image_status_tf(self.language, ImageStatusMsg::NoAtSelection, &[]).into();
            cx.notify();
            return;
        };
        self.start_existing_image_operation(
            markion::ImagePlanSelection::Single(occurrence.id),
            kind,
            cx,
        );
    }

    pub(super) fn start_existing_image_operation(
        &mut self,
        selection: markion::ImagePlanSelection,
        kind: markion::ImageOperationKind,
        cx: &mut Context<Self>,
    ) {
        let tab_index = self.active_tab;
        self.start_existing_image_operation_for_tab(tab_index, selection, kind, HashMap::new(), cx);
    }

    pub(super) fn start_image_replacement(
        &mut self,
        document: DocumentInstanceId,
        occurrence: ImageOccurrenceId,
        input: markion::ImageInput,
        cx: &mut Context<Self>,
    ) {
        let Some(tab_index) = self
            .tabs
            .iter()
            .position(|tab| tab.is_document() && tab.document.instance_id() == document)
        else {
            return;
        };
        let kind = match self.image_preferences.local_policy {
            markion::LocalImagePolicy::Keep => markion::ImageOperationKind::Insert,
            markion::LocalImagePolicy::Copy => markion::ImageOperationKind::SaveToResources,
            markion::LocalImagePolicy::Upload => markion::ImageOperationKind::Upload,
        };
        self.start_existing_image_operation_for_tab(
            tab_index,
            markion::ImagePlanSelection::Single(occurrence),
            kind,
            HashMap::from([(occurrence, input)]),
            cx,
        );
    }

    fn start_existing_image_operation_for_tab(
        &mut self,
        tab_index: usize,
        selection: markion::ImagePlanSelection,
        kind: markion::ImageOperationKind,
        input_overrides: HashMap<ImageOccurrenceId, markion::ImageInput>,
        cx: &mut Context<Self>,
    ) {
        let document = &self.tabs[tab_index].document;
        let scan = document.image_occurrence_scan();
        let replacing = !input_overrides.is_empty();
        let mut plan = markion::plan_image_operation(&scan, selection, document.path(), |path| {
            replacing || path.is_file()
        });
        for group in &mut plan.input_groups {
            if let Some(input) = group
                .occurrences
                .iter()
                .find_map(|occurrence| input_overrides.get(occurrence))
            {
                group.input = input.clone();
            }
        }
        if plan.input_groups.is_empty() {
            self.status = image_status_tf(
                self.language,
                ImageStatusMsg::NoEligible,
                &[&plan.issues.len().to_string()],
            )
            .into();
            cx.notify();
            return;
        }
        if matches!(
            kind,
            markion::ImageOperationKind::SaveToResources
                | markion::ImageOperationKind::DownloadRemoteDocumentImages
                | markion::ImageOperationKind::OrganizeDocumentImages
        ) && document.path().is_none()
        {
            self.status = p0_t(self.language, P0Msg::SaveBeforeImage).into();
            cx.notify();
            return;
        }
        let preferences = self.image_preferences.clone();
        let config = markion::ImageJobConfig {
            document: document.instance_id(),
            document_path: document.path().map(Path::to_path_buf),
            document_version: document.version(),
            kind,
            preferences: preferences.clone(),
            targets: plan
                .selected_occurrences
                .iter()
                .map(markion::ImageOccurrence::target)
                .collect(),
            occurrences: plan.selected_occurrences,
        };
        let input_groups = plan.input_groups;
        let operation = self
            .image_operations
            .begin(document, config, input_groups.clone());
        let cancellation = markion::external_command::CommandCancellation::default();
        self.image_cancellations
            .insert(operation, cancellation.clone());
        let staging_directory = markion::default_image_recovery_dir()
            .join(format!(
                "document-{}-operation-{}",
                document.instance_id().get(),
                operation.0
            ))
            .join("inputs");
        let inputs = input_groups
            .iter()
            .map(|group| group.input.clone())
            .collect();
        let task_staging = staging_directory.clone();
        let task_preferences = preferences.clone();
        let upload = matches!(
            kind,
            markion::ImageOperationKind::Upload
                | markion::ImageOperationKind::UploadLocalDocumentImages
        );
        let task = network::runtime_handle().spawn(async move {
            let materialized = markion::image_transfer::materialize_inputs_bounded(
                inputs,
                task_staging.clone(),
                markion::image_transfer::ImageTransferLimits {
                    timeout: Duration::from_secs(task_preferences.timeout_secs),
                    ..Default::default()
                },
                cancellation.clone(),
            )
            .await;
            let mut groups = Vec::with_capacity(materialized.len());
            for result in materialized {
                let outcome = match result {
                    Ok(image) if upload => {
                        match markion::image_transfer::upload_materialized_image(
                            &task_preferences,
                            &image,
                            &task_staging,
                            &cancellation,
                        )
                        .await
                        {
                            Ok(url) => InsertionTransferOutcome::Uploaded { image, url },
                            Err(error) => InsertionTransferOutcome::Failed(error.to_string()),
                        }
                    }
                    Ok(image) => InsertionTransferOutcome::Materialized(image),
                    Err(error) => InsertionTransferOutcome::Failed(error.to_string()),
                };
                groups.push(outcome);
            }
            InsertionExecutionResult {
                groups,
                staging_directory: task_staging,
            }
        });
        self.status = image_status_tf(
            self.language,
            ImageStatusMsg::Processing,
            &[
                &input_groups
                    .iter()
                    .map(|group| group.occurrences.len())
                    .sum::<usize>()
                    .to_string(),
                &input_groups.len().to_string(),
            ],
        )
        .into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |app, cx| match result {
                Ok(result) => {
                    app.finish_existing_image_operation(operation, input_groups, result, cx)
                }
                Err(error) => {
                    app.image_cancellations.remove(&operation);
                    app.status = image_status_tf(
                        app.language,
                        ImageStatusMsg::OperationFailed,
                        &[&error.to_string()],
                    )
                    .into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn finish_existing_image_operation(
        &mut self,
        operation_id: ImageOperationId,
        input_groups: Vec<markion::ImageInputGroup>,
        result: InsertionExecutionResult,
        cx: &mut Context<Self>,
    ) {
        self.image_cancellations.remove(&operation_id);
        let Some(operation) = self.image_operations.operation(operation_id) else {
            return;
        };
        if operation.status == markion::ImageOperationStatus::Canceled {
            let document_path = operation.config.document_path.clone();
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                "operation was canceled",
            );
            self.status = t(self.language, Msg::StatusCanceled).into();
            cx.notify();
            return;
        }
        let document_id = operation.config.document;
        let kind = operation.config.kind;
        let resource_directory = operation.config.preferences.directory.clone();
        let generation = operation.generation;
        let Some(tab_index) = self
            .tabs
            .iter()
            .position(|tab| tab.is_document() && tab.document.instance_id() == document_id)
        else {
            self.persist_unapplied_insertion(operation_id, None, &result, "document was closed");
            return;
        };
        let document_path = self.tabs[tab_index].document.path().map(Path::to_path_buf);
        let needs_local = !matches!(
            kind,
            markion::ImageOperationKind::Upload
                | markion::ImageOperationKind::UploadLocalDocumentImages
        );
        let _admission = if needs_local {
            match document_path
                .as_deref()
                .map(|path| self.git_operations.try_write(path))
                .transpose()
            {
                Ok(admission) => admission,
                Err(_) => {
                    self.persist_unapplied_insertion(
                        operation_id,
                        document_path.as_deref(),
                        &result,
                        "workspace write is busy",
                    );
                    return;
                }
            }
        } else {
            None
        };
        let mut outputs = HashMap::new();
        let mut failures = Vec::new();
        for (group_index, (group, outcome)) in
            input_groups.iter().zip(result.groups.iter()).enumerate()
        {
            let resolved = match outcome {
                InsertionTransferOutcome::Uploaded { url, .. } => Ok(url.clone()),
                InsertionTransferOutcome::Materialized(_image)
                    if kind == markion::ImageOperationKind::Insert =>
                {
                    match &group.input {
                        markion::ImageInput::Local(path) => {
                            Ok(local_keep_url(document_path.as_deref(), path))
                        }
                        markion::ImageInput::Remote(url) => Ok(url.clone()),
                        _ => Err("this image input cannot be kept in place".into()),
                    }
                }
                InsertionTransferOutcome::Materialized(image) => match document_path.as_deref() {
                    Some(document_path) => {
                        markion::stage_image_file(document_path, &resource_directory, &image.path)
                            .and_then(|staged| {
                                markion::publish_staged_image(&staged, document_path)
                            })
                            .map(|published| published.relative_url)
                            .map_err(|error| error.to_string())
                    }
                    None => Err("saving images locally requires a saved document".into()),
                },
                InsertionTransferOutcome::Failed(error) => Err(error.clone()),
            };
            match resolved {
                Ok(url) => {
                    for occurrence in &group.occurrences {
                        outputs.insert(*occurrence, url.clone());
                    }
                    let _ = self.image_operations.record_group_result(
                        operation_id,
                        group_index,
                        generation,
                        markion::ImageItemResult {
                            occurrence: group.occurrences.first().copied(),
                            state: markion::ImageItemState::Unapplied,
                            source: "image reference".into(),
                            output_url: Some(url),
                            output_path: None,
                            message: None,
                        },
                    );
                }
                Err(error) => {
                    failures.push(error.clone());
                    let _ = self.image_operations.record_group_result(
                        operation_id,
                        group_index,
                        generation,
                        markion::ImageItemResult {
                            occurrence: group.occurrences.first().copied(),
                            state: markion::ImageItemState::Failed,
                            source: "image reference".into(),
                            output_url: None,
                            output_path: None,
                            message: Some(error),
                        },
                    );
                }
            }
        }
        let (applied, stale) = self.apply_image_operation_outputs(operation_id, &outputs, cx);
        if let Some(operation) = self.image_operations.operation(operation_id) {
            let generation = operation.generation;
            for (group_index, group) in input_groups.iter().enumerate() {
                if group
                    .occurrences
                    .iter()
                    .any(|occurrence| outputs.contains_key(occurrence))
                {
                    let known = self
                        .image_operations
                        .operation(operation_id)
                        .and_then(|operation| operation.known_output(group_index))
                        .cloned();
                    if let Some(mut known) = known {
                        known.state = markion::ImageItemState::Applied;
                        let _ = self.image_operations.record_group_result(
                            operation_id,
                            group_index,
                            generation,
                            known,
                        );
                    }
                }
            }
        }
        if failures.is_empty() && stale == 0 {
            let _ = fs::remove_dir_all(
                result
                    .staging_directory
                    .parent()
                    .unwrap_or(&result.staging_directory),
            );
        } else {
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                "some results were not applied",
            );
        }
        self.status = image_status_tf(
            self.language,
            ImageStatusMsg::Summary,
            &[
                &applied.to_string(),
                &failures.len().to_string(),
                &stale.to_string(),
            ],
        )
        .into();
        cx.notify();
    }

    pub(super) fn start_image_insertion(
        &mut self,
        batch: PendingImageBatch,
        cx: &mut Context<Self>,
    ) {
        let Some(tab_index) = self
            .tabs
            .iter()
            .position(|tab| tab.is_document() && tab.document.instance_id() == batch.document)
        else {
            return;
        };
        let document = &self.tabs[tab_index].document;
        let Some(expected_source) = document
            .text()
            .get(batch.selection.clone())
            .map(str::to_owned)
        else {
            return;
        };
        let preferences = self.image_preferences.clone();
        let mut keep_urls = vec![None; batch.inputs.len()];
        let mut transfer_groups: Vec<InsertionTransferGroup> = Vec::new();
        for (index, input) in batch.inputs.iter().enumerate() {
            let kind = match input.source_kind {
                PendingImageSourceKind::Clipboard => match preferences.clipboard_policy {
                    markion::ClipboardImagePolicy::Save => InsertionTransferKind::SaveLocal,
                    markion::ClipboardImagePolicy::Upload => InsertionTransferKind::Upload,
                },
                PendingImageSourceKind::Local => match preferences.local_policy {
                    markion::LocalImagePolicy::Keep => {
                        if let markion::ImageInput::Local(path) = &input.input {
                            keep_urls[index] = Some(local_keep_url(document.path(), path));
                        }
                        continue;
                    }
                    markion::LocalImagePolicy::Copy => InsertionTransferKind::SaveLocal,
                    markion::LocalImagePolicy::Upload => InsertionTransferKind::Upload,
                },
                PendingImageSourceKind::Remote => match preferences.remote_policy {
                    markion::RemoteImagePolicy::Keep => {
                        if let markion::ImageInput::Remote(url) = &input.input {
                            keep_urls[index] = Some(url.clone());
                        }
                        continue;
                    }
                    markion::RemoteImagePolicy::Download => InsertionTransferKind::SaveLocal,
                    markion::RemoteImagePolicy::Upload => InsertionTransferKind::Upload,
                },
            };
            if let Some(group) = transfer_groups
                .iter_mut()
                .find(|group| group.kind == kind && same_input(&group.input, &input.input))
            {
                group.input_indexes.push(index);
            } else {
                transfer_groups.push(InsertionTransferGroup {
                    input: input.input.clone(),
                    kind,
                    input_indexes: vec![index],
                });
            }
        }

        let target = markion::ImageOccurrenceTarget {
            id: ImageOccurrenceId(0),
            source_range: batch.selection.clone(),
            expected_source,
            semantic_url: String::new(),
        };
        let config = markion::ImageJobConfig {
            document: batch.document,
            document_path: document.path().map(Path::to_path_buf),
            document_version: document.version(),
            kind: markion::ImageOperationKind::Insert,
            preferences: preferences.clone(),
            targets: vec![target],
            occurrences: vec![],
        };
        let controller_groups = transfer_groups
            .iter()
            .map(|group| markion::ImageInputGroup {
                input: group.input.clone(),
                occurrences: group
                    .input_indexes
                    .iter()
                    .map(|index| ImageOccurrenceId(*index as u64 + 1))
                    .collect(),
            })
            .collect();
        let operation = self
            .image_operations
            .begin(document, config, controller_groups);
        let cancellation = markion::external_command::CommandCancellation::default();
        self.image_cancellations
            .insert(operation, cancellation.clone());
        let staging_directory = markion::default_image_recovery_dir()
            .join(format!(
                "document-{}-operation-{}",
                document.instance_id().get(),
                operation.0
            ))
            .join("inputs");
        let transfer_inputs = transfer_groups
            .iter()
            .map(|group| group.input.clone())
            .collect();
        let kinds: Vec<_> = transfer_groups.iter().map(|group| group.kind).collect();
        let task_preferences = preferences.clone();
        let task_staging = staging_directory.clone();
        let task = network::runtime_handle().spawn(async move {
            let materialized = markion::image_transfer::materialize_inputs_bounded(
                transfer_inputs,
                task_staging.clone(),
                markion::image_transfer::ImageTransferLimits {
                    timeout: Duration::from_secs(task_preferences.timeout_secs),
                    ..Default::default()
                },
                cancellation.clone(),
            )
            .await;
            let mut groups = Vec::with_capacity(materialized.len());
            for (result, kind) in materialized.into_iter().zip(kinds) {
                let outcome = match result {
                    Ok(image) if kind == InsertionTransferKind::Upload => {
                        match markion::image_transfer::upload_materialized_image(
                            &task_preferences,
                            &image,
                            &task_staging,
                            &cancellation,
                        )
                        .await
                        {
                            Ok(url) => InsertionTransferOutcome::Uploaded { image, url },
                            Err(error) => InsertionTransferOutcome::Failed(error.to_string()),
                        }
                    }
                    Ok(image) => InsertionTransferOutcome::Materialized(image),
                    Err(error) => InsertionTransferOutcome::Failed(error.to_string()),
                };
                groups.push(outcome);
            }
            InsertionExecutionResult {
                groups,
                staging_directory: task_staging,
            }
        });
        self.status = image_status_tf(self.language, ImageStatusMsg::GenericProcessing, &[]).into();
        cx.notify();
        cx.spawn(async move |this, cx| {
            let result = task.await;
            let _ = this.update(cx, |app, cx| match result {
                Ok(result) => app.finish_image_insertion(
                    operation,
                    batch,
                    transfer_groups,
                    keep_urls,
                    result,
                    cx,
                ),
                Err(error) => {
                    app.image_cancellations.remove(&operation);
                    app.status = image_status_tf(
                        app.language,
                        ImageStatusMsg::OperationFailed,
                        &[&error.to_string()],
                    )
                    .into();
                    cx.notify();
                }
            });
        })
        .detach();
    }

    fn finish_image_insertion(
        &mut self,
        operation_id: ImageOperationId,
        batch: PendingImageBatch,
        transfer_groups: Vec<InsertionTransferGroup>,
        keep_urls: Vec<Option<String>>,
        result: InsertionExecutionResult,
        cx: &mut Context<Self>,
    ) {
        self.image_cancellations.remove(&operation_id);
        if self
            .image_operations
            .operation(operation_id)
            .is_some_and(|operation| operation.status == markion::ImageOperationStatus::Canceled)
        {
            let document_path = self
                .image_operations
                .operation(operation_id)
                .and_then(|operation| operation.config.document_path.clone());
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                "operation was canceled",
            );
            self.status = t(self.language, Msg::StatusCanceled).into();
            cx.notify();
            return;
        }
        let Some(tab_index) = self
            .tabs
            .iter()
            .position(|tab| tab.is_document() && tab.document.instance_id() == batch.document)
        else {
            self.persist_unapplied_insertion(operation_id, None, &result, "document was closed");
            return;
        };
        self.image_operations
            .observe_document(&self.tabs[tab_index].document);
        let Some(operation) = self.image_operations.operation(operation_id) else {
            return;
        };
        let resource_directory = operation.config.preferences.directory.clone();
        let Some(anchor) = operation.anchors.first() else {
            return;
        };
        if !operation.auto_apply_allowed
            || !anchor.valid
            || self.tabs[tab_index]
                .document
                .text()
                .get(anchor.range.clone())
                != Some(anchor.expected_source.as_str())
        {
            let stale_document_path = self.tabs[tab_index].document.path().map(Path::to_path_buf);
            self.persist_unapplied_insertion(
                operation_id,
                stale_document_path.as_deref(),
                &result,
                "insertion target changed",
            );
            self.status = image_status_tf(self.language, ImageStatusMsg::TargetChanged, &[]).into();
            cx.notify();
            return;
        }
        let target_range = anchor.range.clone();
        let document_path = self.tabs[tab_index].document.path().map(Path::to_path_buf);
        let _admission = match document_path
            .as_deref()
            .map(|path| self.git_operations.try_write(path))
            .transpose()
        {
            Ok(admission) => admission,
            Err(error) => {
                self.persist_unapplied_insertion(
                    operation_id,
                    document_path.as_deref(),
                    &result,
                    "workspace write is busy",
                );
                self.status = if error == markion_git_sync::AdmissionError::ConflictOwned {
                    self.git_label(GitMsg::ConflictNeedsAttention).into()
                } else {
                    image_status_tf(self.language, ImageStatusMsg::WorkspaceBusy, &[]).into()
                };
                cx.notify();
                return;
            }
        };

        let mut urls = keep_urls;
        let mut failures = Vec::new();
        for (group_index, (group, outcome)) in
            transfer_groups.iter().zip(result.groups.iter()).enumerate()
        {
            let group_result = match outcome {
                InsertionTransferOutcome::Uploaded { url, .. } => Ok(url.clone()),
                InsertionTransferOutcome::Materialized(image) => match document_path.as_deref() {
                    Some(document_path) => {
                        markion::stage_image_file(document_path, &resource_directory, &image.path)
                            .and_then(|staged| {
                                markion::publish_staged_image(&staged, document_path)
                            })
                            .map(|published| published.relative_url)
                            .map_err(|error| error.to_string())
                    }
                    None => Err("saving images locally requires a saved document".to_owned()),
                },
                InsertionTransferOutcome::Failed(error) => Err(error.clone()),
            };
            let generation = self
                .image_operations
                .operation(operation_id)
                .map(|operation| operation.generation)
                .unwrap_or_default();
            match group_result {
                Ok(url) => {
                    for index in &group.input_indexes {
                        urls[*index] = Some(url.clone());
                    }
                    let _ = self.image_operations.record_group_result(
                        operation_id,
                        group_index,
                        generation,
                        markion::ImageItemResult {
                            occurrence: None,
                            state: markion::ImageItemState::Unapplied,
                            source: batch.inputs[group.input_indexes[0]].label.clone(),
                            output_url: Some(url),
                            output_path: None,
                            message: None,
                        },
                    );
                }
                Err(error) => {
                    failures.push(error.clone());
                    let _ = self.image_operations.record_group_result(
                        operation_id,
                        group_index,
                        generation,
                        markion::ImageItemResult {
                            occurrence: None,
                            state: markion::ImageItemState::Failed,
                            source: batch.inputs[group.input_indexes[0]].label.clone(),
                            output_url: None,
                            output_path: None,
                            message: Some(error),
                        },
                    );
                }
            }
        }
        let markdown: Vec<_> = batch
            .inputs
            .iter()
            .zip(urls.iter())
            .filter_map(|(input, url)| {
                url.as_ref().map(|url| {
                    serialize_inline_image(&input.label, url, input.title.as_deref(), None)
                })
            })
            .collect();
        if markdown.is_empty() {
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                "all image inputs failed",
            );
            self.status = failures.join("; ").into();
            cx.notify();
            return;
        }
        let replacement = markdown.join("\n");
        let tab = &mut self.tabs[tab_index];
        tab.finish_undo_capture();
        let snapshot = tab.snapshot();
        let mutation = tab.document.prepare_range_mutation(
            MutationOrigin::ImageOperation,
            target_range.clone(),
            &replacement,
        );
        if tab.document.apply_checked_mutation(mutation).is_err() {
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                "checked insertion was rejected",
            );
            return;
        }
        tab.commit_undo_snapshot(snapshot);
        tab.selected_range =
            target_range.start + replacement.len()..target_range.start + replacement.len();
        tab.selection_reversed = false;
        tab.marked_range = None;
        if let Some(operation) = self.image_operations.operation(operation_id) {
            let generation = operation.generation;
            for group_index in 0..transfer_groups.len() {
                let known = self
                    .image_operations
                    .operation(operation_id)
                    .and_then(|operation| operation.known_output(group_index))
                    .cloned();
                if let Some(mut known) = known {
                    known.state = markion::ImageItemState::Applied;
                    let _ = self.image_operations.record_group_result(
                        operation_id,
                        group_index,
                        generation,
                        known,
                    );
                }
            }
        }
        if failures.is_empty() {
            let _ = fs::remove_dir_all(
                result
                    .staging_directory
                    .parent()
                    .unwrap_or(&result.staging_directory),
            );
        } else {
            self.persist_unapplied_insertion(
                operation_id,
                document_path.as_deref(),
                &result,
                &failures.join("; "),
            );
        }
        self.status = if failures.is_empty() {
            t(self.language, Msg::StatusFmtImage).into()
        } else {
            p0_tf(
                self.language,
                P0Msg::ImagePartialFailure,
                &[&failures.join("; ")],
            )
            .into()
        };
        if tab_index == self.active_tab {
            self.after_document_changed(cx);
        } else {
            if let Some(path) = document_path.as_deref() {
                self.git_operations.note_edit(path);
            }
            cx.notify();
        }
    }

    fn persist_unapplied_insertion(
        &mut self,
        operation_id: ImageOperationId,
        document_path: Option<&Path>,
        result: &InsertionExecutionResult,
        message: &str,
    ) {
        let owned_inputs = result
            .groups
            .iter()
            .filter_map(|outcome| match outcome {
                InsertionTransferOutcome::Materialized(image)
                | InsertionTransferOutcome::Uploaded { image, .. } => Some(image.path.clone()),
                InsertionTransferOutcome::Failed(_) => None,
            })
            .collect();
        let items = result
            .groups
            .iter()
            .map(|outcome| markion::ImageItemResult {
                occurrence: None,
                state: markion::ImageItemState::Unapplied,
                source: "image input".into(),
                output_url: match outcome {
                    InsertionTransferOutcome::Uploaded { url, .. } => Some(url.clone()),
                    _ => None,
                },
                output_path: match outcome {
                    InsertionTransferOutcome::Materialized(image) => Some(image.path.clone()),
                    _ => None,
                },
                message: Some(message.to_owned()),
            })
            .collect();
        let record = markion::ImageRecoveryRecord {
            operation_id: self
                .image_operations
                .operation(operation_id)
                .map(|operation| {
                    format!(
                        "document-{}-operation-{}",
                        operation.config.document.get(),
                        operation_id.0
                    )
                })
                .unwrap_or_else(|| format!("operation-{}", operation_id.0)),
            document_path: document_path.map(Path::to_path_buf),
            created_unix_secs: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            items,
            owned_inputs,
        };
        let _ =
            markion::save_image_recovery_record(&markion::default_image_recovery_dir(), &record);
        self.refresh_image_recovery_entries();
    }

    /// Applies known image results to their originating tab without changing
    /// the active tab. The pure planner verifies anchors/current bytes and
    /// returns one checked mutation for all still-valid occurrences.
    pub(super) fn apply_image_operation_outputs(
        &mut self,
        operation_id: ImageOperationId,
        outputs: &HashMap<ImageOccurrenceId, String>,
        cx: &mut Context<Self>,
    ) -> (usize, usize) {
        let Some(document_id) = self
            .image_operations
            .operation(operation_id)
            .map(|operation| operation.config.document)
        else {
            return (0, outputs.len());
        };
        let Some(tab_index) = self
            .tabs
            .iter()
            .position(|tab| tab.is_document() && tab.document.instance_id() == document_id)
        else {
            self.image_operations.invalidate_document(document_id);
            return (0, outputs.len());
        };
        self.image_operations
            .observe_document(&self.tabs[tab_index].document);
        let plan = {
            let Some(operation) = self.image_operations.operation(operation_id) else {
                return (0, outputs.len());
            };
            build_image_application_plan(&self.tabs[tab_index].document, operation, outputs)
        };
        let applied_count = plan.applied.len();
        let stale_count = plan.stale.len();
        let Some(mutation) = plan.mutation else {
            return (applied_count, stale_count);
        };
        let document_path = self.tabs[tab_index].document.path().map(Path::to_path_buf);
        let _admission = match document_path
            .as_deref()
            .map(|path| self.git_operations.try_write(path))
            .transpose()
        {
            Ok(admission) => admission,
            Err(_) => return (0, outputs.len()),
        };

        let tab = &mut self.tabs[tab_index];
        tab.finish_undo_capture();
        let snapshot = tab.snapshot();
        if tab.document.apply_checked_mutation(mutation).is_err() {
            return (0, outputs.len());
        }
        tab.commit_undo_snapshot(snapshot);
        tab.selected_range = rebase_selection(
            tab.selected_range.clone(),
            &plan.edits,
            &plan.replacement_lengths,
        );
        tab.marked_range = None;

        if tab_index == self.active_tab {
            self.after_document_changed(cx);
        } else {
            if let Some(path) = document_path.as_deref() {
                self.git_operations.note_edit(path);
            }
            self.image_operations
                .observe_document(&self.tabs[tab_index].document);
            cx.notify();
        }
        (applied_count, stale_count)
    }
}

fn rebase_selection(
    selection: Range<usize>,
    edits: &[Range<usize>],
    replacement_lengths: &[usize],
) -> Range<usize> {
    rebase_offset(selection.start, edits, replacement_lengths)
        ..rebase_offset(selection.end, edits, replacement_lengths)
}

fn same_input(left: &markion::ImageInput, right: &markion::ImageInput) -> bool {
    match (left, right) {
        (markion::ImageInput::Local(left), markion::ImageInput::Local(right)) => left == right,
        (markion::ImageInput::Remote(left), markion::ImageInput::Remote(right))
        | (markion::ImageInput::DataUri(left), markion::ImageInput::DataUri(right)) => {
            left == right
        }
        (
            markion::ImageInput::Bytes {
                extension: left_extension,
                bytes: left_bytes,
                ..
            },
            markion::ImageInput::Bytes {
                extension: right_extension,
                bytes: right_bytes,
                ..
            },
        ) => left_extension.eq_ignore_ascii_case(right_extension) && left_bytes == right_bytes,
        _ => false,
    }
}

fn local_keep_url(document_path: Option<&Path>, image_path: &Path) -> String {
    let relative = document_path
        .and_then(Path::parent)
        .and_then(|directory| image_path.strip_prefix(directory).ok())
        .unwrap_or(image_path);
    relative.to_string_lossy().replace('\\', "/")
}

fn rebase_offset(offset: usize, edits: &[Range<usize>], replacement_lengths: &[usize]) -> usize {
    let mut delta = 0isize;
    for (range, replacement_len) in edits.iter().zip(replacement_lengths.iter().copied()) {
        if offset <= range.start {
            break;
        }
        if offset < range.end {
            return (range.start + replacement_len).saturating_add_signed(delta);
        }
        delta += replacement_len as isize - range.end.saturating_sub(range.start) as isize;
    }
    offset.saturating_add_signed(delta)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_rebase_preserves_positions_between_multiple_edits() {
        let edits = vec![2..4, 8..10];
        let lengths = vec![5, 1];
        assert_eq!(rebase_selection(6..12, &edits, &lengths), 9..14);
        assert_eq!(rebase_selection(3..3, &edits, &lengths), 7..7);
    }
}
