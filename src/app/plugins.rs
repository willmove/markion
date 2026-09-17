//! Host-owned official-plugin manager state and actions.
//!
//! Signed metadata and native workers never own GPUI elements. This module
//! keeps confirmation, progress, cancellation, failure mapping, and lifecycle
//! actions in the host while delegating package/store/process details to the
//! deep `plugin_platform` interfaces.

use super::*;
use markion::plugin_platform::{
    CoreFileHandler, CoreHandlerKind, FileHandlerCandidate, FileHandlerRegistrySnapshot,
    FileHandlerResolution, ManagedPluginEntry, ManagedPluginStatus, PluginInstallPlan,
    PluginManager, PluginStorePaths,
};
use markion_plugin_protocol::{Permission, ProtocolVersion, TargetSpec};
use semver::Version;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

const MAX_OFFICIAL_PLUGIN_ARCHIVE_BYTES: u64 = 6 * 1024 * 1024;
const MAX_PLUGIN_CATALOG_BYTES: usize = 1024 * 1024;
const MAX_PLUGIN_CATALOG_SIGNATURE_BYTES: usize = 16 * 1024;
const CATALOG_OPERATION_ID: &str = "host.catalog";
const PLUGIN_CATALOG_ENDPOINTS: [(&str, &str); 2] = [
    (
        "https://marknice.oss-cn-heyuan.aliyuncs.com/markion-releases/latest/plugin-catalog.json",
        "https://marknice.oss-cn-heyuan.aliyuncs.com/markion-releases/latest/plugin-catalog.json.minisig",
    ),
    (
        "https://github.com/willmove/markion/releases/latest/download/plugin-catalog.json",
        "https://github.com/willmove/markion/releases/latest/download/plugin-catalog.json.minisig",
    ),
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PluginOperationPhase {
    Downloading,
    Verifying,
    Activating,
    Completed,
    Refreshing,
    Failed(PluginOperationFailure),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PluginOperationFailure {
    Download,
    Verification,
    Activation,
    Unavailable,
    Catalog,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PluginOperationState {
    pub(super) plugin_id: String,
    pub(super) generation: u64,
    pub(super) phase: PluginOperationPhase,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) enum ResolvedFileHandler {
    Document,
    Image,
    Plugin(FileHandlerCandidate),
    Conflict,
}

pub(super) struct PluginUiState {
    initialized: bool,
    manager: Option<PluginManager>,
    registry: Option<Arc<FileHandlerRegistrySnapshot>>,
    pub(super) entries: Vec<ManagedPluginEntry>,
    pub(super) operation: Option<PluginOperationState>,
    pub(super) startup_failure: Option<PluginOperationFailure>,
    generation: u64,
    cancellation: Option<Arc<AtomicBool>>,
    retry_plan: Option<PluginInstallPlan>,
}

impl Default for PluginUiState {
    fn default() -> Self {
        Self {
            initialized: false,
            manager: None,
            registry: None,
            entries: Vec::new(),
            operation: None,
            startup_failure: None,
            generation: 0,
            cancellation: None,
            retry_plan: None,
        }
    }
}

impl PluginUiState {
    pub(super) fn ensure_initialized(&mut self, language: Language) {
        if self.initialized {
            self.refresh_entries(language);
            return;
        }
        self.initialized = true;
        if cfg!(test) {
            self.rebuild_registry();
            return;
        }
        let manager = markion::plugin_platform::bootstrap()
            .map_err(|_| ())
            .and_then(|bootstrap| {
                bootstrap
                    .plugin_manager(PluginStorePaths::default())
                    .map_err(|_| ())
            });
        match manager {
            Ok(manager) => {
                self.manager = Some(manager);
                self.refresh_entries(language);
                self.rebuild_registry();
            }
            Err(()) => {
                self.startup_failure = Some(PluginOperationFailure::Unavailable);
                self.rebuild_registry();
            }
        }
    }

    pub(super) fn refresh_entries(&mut self, language: Language) {
        let Some(manager) = self.manager.as_ref() else {
            return;
        };
        match manager.entries(language.plugin_locale()) {
            Ok(entries) => {
                self.entries = entries;
                self.startup_failure = None;
            }
            Err(_) => self.startup_failure = Some(PluginOperationFailure::Unavailable),
        }
    }

    pub(super) fn manager(&self) -> Option<PluginManager> {
        self.manager.clone()
    }

    pub(super) fn file_handler(&self, path: &Path) -> Option<FileHandlerResolution> {
        self.registry.as_ref()?.resolve_path(path).cloned()
    }

    pub(super) fn resolve_file_handler(
        &mut self,
        path: &Path,
        language: Language,
    ) -> Option<ResolvedFileHandler> {
        self.ensure_initialized(language);
        match self.file_handler(path)? {
            FileHandlerResolution::Handler(candidate) => match candidate.availability {
                markion::plugin_platform::FileHandlerAvailability::Core(
                    CoreHandlerKind::Markdown | CoreHandlerKind::Text,
                ) => Some(ResolvedFileHandler::Document),
                markion::plugin_platform::FileHandlerAvailability::Core(CoreHandlerKind::Image) => {
                    Some(ResolvedFileHandler::Image)
                }
                _ => Some(ResolvedFileHandler::Plugin(candidate)),
            },
            FileHandlerResolution::Conflict(_) => Some(ResolvedFileHandler::Conflict),
        }
    }

    pub(super) fn file_handler_extensions(&self) -> Arc<[String]> {
        self.registry
            .as_ref()
            .map(|registry| {
                registry
                    .handlers()
                    .iter()
                    .filter_map(|(extension, resolution)| {
                        let plugin_owned = match resolution {
                            FileHandlerResolution::Handler(candidate) => {
                                candidate.plugin_id.is_some()
                            }
                            FileHandlerResolution::Conflict(candidates) => candidates
                                .iter()
                                .any(|candidate| candidate.plugin_id.is_some()),
                        };
                        plugin_owned.then(|| extension.clone())
                    })
                    .collect::<Vec<_>>()
                    .into()
            })
            .unwrap_or_else(|| Arc::from([]))
    }

    pub(super) fn rebuild_registry(&mut self) {
        let revision = self
            .registry
            .as_ref()
            .map_or(1, |registry| registry.revision().wrapping_add(1).max(1));
        let registry = if let Some(manager) = self.manager.as_ref() {
            manager
                .file_handler_registry(revision, core_file_handlers())
                .map_err(|_| ())
        } else {
            markion::plugin_platform::bootstrap()
                .map_err(|_| ())
                .and_then(|bootstrap| {
                    FileHandlerRegistrySnapshot::build_uninstalled(
                        revision,
                        core_file_handlers(),
                        bootstrap.catalog(),
                        &Version::parse(env!("CARGO_PKG_VERSION")).expect("semver package version"),
                        ProtocolVersion::V1_0,
                        &TargetSpec::current(),
                    )
                    .map_err(|_| ())
                })
        };
        match registry {
            Ok(registry) => self.registry = Some(Arc::new(registry)),
            Err(_) => self.startup_failure = Some(PluginOperationFailure::Unavailable),
        }
    }

    fn replace_manager(&mut self, manager: PluginManager, language: Language) {
        self.manager = Some(manager);
        self.refresh_entries(language);
        self.rebuild_registry();
    }

    pub(super) fn begin(
        &mut self,
        plugin_id: String,
        phase: PluginOperationPhase,
    ) -> (u64, Arc<AtomicBool>) {
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.store(true, Ordering::Release);
        }
        self.generation = self.generation.wrapping_add(1).max(1);
        let cancellation = Arc::new(AtomicBool::new(false));
        self.cancellation = Some(Arc::clone(&cancellation));
        self.operation = Some(PluginOperationState {
            plugin_id,
            generation: self.generation,
            phase,
        });
        (self.generation, cancellation)
    }

    fn advance(&mut self, generation: u64, phase: PluginOperationPhase) -> bool {
        let Some(operation) = self.operation.as_mut() else {
            return false;
        };
        if operation.generation != generation {
            return false;
        }
        operation.phase = phase;
        if matches!(phase, PluginOperationPhase::Completed) {
            self.cancellation = None;
            self.retry_plan = None;
        }
        true
    }

    fn fail(
        &mut self,
        generation: u64,
        failure: PluginOperationFailure,
        retry_plan: Option<PluginInstallPlan>,
    ) -> bool {
        if !self.advance(generation, PluginOperationPhase::Failed(failure)) {
            return false;
        }
        self.cancellation = None;
        self.retry_plan = retry_plan;
        true
    }

    fn cancel_download(&mut self) -> bool {
        let downloading = self
            .operation
            .as_ref()
            .is_some_and(|operation| operation.phase == PluginOperationPhase::Downloading);
        if !downloading {
            return false;
        }
        if let Some(cancellation) = self.cancellation.take() {
            cancellation.store(true, Ordering::Release);
        }
        self.generation = self.generation.wrapping_add(1).max(1);
        self.operation = None;
        true
    }
}

fn core_file_handlers() -> Vec<CoreFileHandler> {
    let mut handlers = Vec::new();
    handlers.extend(markion::MARKDOWN_EXTENSIONS.iter().map(|extension| {
        CoreFileHandler::new(
            extension.to_string(),
            "document-markdown",
            CoreHandlerKind::Markdown,
        )
    }));
    handlers.extend(markion::TEXT_EXTENSIONS.iter().map(|extension| {
        CoreFileHandler::new(
            extension.to_string(),
            "document-text",
            CoreHandlerKind::Text,
        )
    }));
    handlers.extend(markion::IMAGE_EXTENSIONS.iter().map(|extension| {
        CoreFileHandler::new(
            extension.to_string(),
            "document-image",
            CoreHandlerKind::Image,
        )
    }));
    handlers
}

#[derive(Clone, Copy)]
enum PluginLifecycleAction {
    Enable,
    Disable,
    Rollback,
    Uninstall,
}

impl MarkionApp {
    pub(super) fn ensure_plugin_manager(&mut self, cx: &mut Context<Self>) {
        self.plugin_ui.ensure_initialized(self.language);
        cx.notify();
    }

    pub(super) fn prompt_plugin_install(
        &mut self,
        plugin_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(manager) = self.plugin_ui.manager() else {
            self.plugin_ui.startup_failure = Some(PluginOperationFailure::Unavailable);
            cx.notify();
            return;
        };
        let plan = match manager.plan_install(&plugin_id, self.language.plugin_locale()) {
            Ok(plan) => plan,
            Err(_) => {
                self.plugin_ui.startup_failure = Some(PluginOperationFailure::Unavailable);
                cx.notify();
                return;
            }
        };
        let capabilities = plugin_capabilities_label(self.language, &plan.capabilities);
        let permissions = plugin_permissions_label(self.language, &plan.permissions);
        let download = format_plugin_bytes(plan.artifact.length);
        let installed = format_plugin_bytes(plan.artifact.installed_size_bytes);
        let detail = plugin_tf(
            self.language,
            PluginMsg::ConfirmDetail,
            &[
                &plan.identity.description,
                &plan.publisher,
                &capabilities,
                &permissions,
                &download,
                &installed,
            ],
        );
        let title = plugin_t(
            self.language,
            if plan.is_update {
                PluginMsg::ConfirmUpdateTitle
            } else {
                PluginMsg::ConfirmInstallTitle
            },
        );
        let confirm = plugin_t(
            self.language,
            if plan.is_update {
                PluginMsg::ActionUpdate
            } else {
                PluginMsg::ActionInstall
            },
        );
        let answer = window.prompt(
            PromptLevel::Info,
            title,
            Some(&detail),
            &[
                PromptButton::ok(confirm),
                PromptButton::cancel(plugin_t(self.language, PluginMsg::ActionCancel)),
            ],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if matches!(answer.await, Ok(0)) {
                let _ = this.update(cx, |app, cx| app.start_plugin_install(plan, cx));
            }
        })
        .detach();
    }

    fn start_plugin_install(&mut self, plan: PluginInstallPlan, cx: &mut Context<Self>) {
        let Some(manager) = self.plugin_ui.manager() else {
            return;
        };
        let plugin_id = plan.plugin_id.clone();
        let reload_plugin_id = plugin_id.clone();
        let url = plan.artifact.url.clone();
        let expected_bytes = plan.artifact.length;
        let (generation, cancellation) = self
            .plugin_ui
            .begin(plugin_id, PluginOperationPhase::Downloading);
        self.plugin_ui.retry_plan = Some(plan.clone());
        cx.notify();

        cx.spawn(async move |this, cx| {
            let download_cancel = Arc::clone(&cancellation);
            let downloaded = cx
                .background_executor()
                .spawn(async move {
                    network::fetch_url_bytes_exact_cancellable(
                        &url,
                        expected_bytes,
                        MAX_OFFICIAL_PLUGIN_ARCHIVE_BYTES,
                        download_cancel,
                    )
                })
                .await;
            let archive = match downloaded {
                Ok(archive) => archive,
                Err(_) => {
                    if !cancellation.load(Ordering::Acquire) {
                        let retry = plan.clone();
                        let _ = this.update(cx, |app, cx| {
                            app.plugin_ui.fail(
                                generation,
                                PluginOperationFailure::Download,
                                Some(retry),
                            );
                            cx.notify();
                        });
                    }
                    return;
                }
            };
            let should_verify = this
                .update(cx, |app, cx| {
                    let current = app
                        .plugin_ui
                        .advance(generation, PluginOperationPhase::Verifying);
                    if current {
                        cx.notify();
                    }
                    current
                })
                .unwrap_or(false);
            if !should_verify {
                return;
            }
            let verify_manager = manager.clone();
            let verify_plan = plan.clone();
            let verified = cx
                .background_executor()
                .spawn(async move { verify_manager.verify_archive(&verify_plan, &archive) })
                .await;
            let package = match verified {
                Ok(package) => package,
                Err(_) => {
                    let retry = plan.clone();
                    let _ = this.update(cx, |app, cx| {
                        app.plugin_ui.fail(
                            generation,
                            PluginOperationFailure::Verification,
                            Some(retry),
                        );
                        cx.notify();
                    });
                    return;
                }
            };
            let should_activate = this
                .update(cx, |app, cx| {
                    let current = app
                        .plugin_ui
                        .advance(generation, PluginOperationPhase::Activating);
                    if current {
                        cx.notify();
                    }
                    current
                })
                .unwrap_or(false);
            if !should_activate {
                return;
            }
            let activate_manager = manager.clone();
            let activate_plan = plan.clone();
            let activated = cx
                .background_executor()
                .spawn(async move { activate_manager.activate_verified(&activate_plan, &package) })
                .await;
            let _ = this.update(cx, |app, cx| {
                match activated {
                    Ok(_) => {
                        if app
                            .plugin_ui
                            .advance(generation, PluginOperationPhase::Completed)
                        {
                            app.plugin_ui.refresh_entries(app.language);
                            app.plugin_ui.rebuild_registry();
                            app.reload_plugin_documents(&reload_plugin_id, None, cx);
                        }
                    }
                    Err(_) => {
                        app.plugin_ui.fail(
                            generation,
                            PluginOperationFailure::Activation,
                            Some(plan),
                        );
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn retry_plugin_operation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let plugin_id = self
            .plugin_ui
            .retry_plan
            .as_ref()
            .map(|plan| plan.plugin_id.clone())
            .or_else(|| {
                self.plugin_ui
                    .operation
                    .as_ref()
                    .map(|operation| operation.plugin_id.clone())
            });
        if let Some(plugin_id) = plugin_id {
            self.prompt_plugin_install(plugin_id, window, cx);
        }
    }

    pub(super) fn cancel_plugin_operation(&mut self, cx: &mut Context<Self>) {
        if self.plugin_ui.cancel_download() {
            cx.notify();
        }
    }

    pub(super) fn refresh_plugin_catalog(&mut self, cx: &mut Context<Self>) {
        self.plugin_ui.ensure_initialized(self.language);
        let (generation, _) = self.plugin_ui.begin(
            CATALOG_OPERATION_ID.to_owned(),
            PluginOperationPhase::Refreshing,
        );
        cx.notify();
        cx.spawn(async move |this, cx| {
            let refreshed = cx
                .background_executor()
                .spawn(async move { refresh_production_catalog() })
                .await;
            let _ = this.update(cx, |app, cx| {
                match refreshed {
                    Ok(manager) => {
                        if app
                            .plugin_ui
                            .advance(generation, PluginOperationPhase::Completed)
                        {
                            app.plugin_ui.replace_manager(manager, app.language);
                            app.schedule_file_tree_scan(None, cx);
                        }
                    }
                    Err(()) => {
                        app.plugin_ui
                            .fail(generation, PluginOperationFailure::Catalog, None);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }

    pub(super) fn set_plugin_enabled(
        &mut self,
        plugin_id: String,
        enabled: bool,
        cx: &mut Context<Self>,
    ) {
        self.run_plugin_lifecycle(
            plugin_id,
            if enabled {
                PluginLifecycleAction::Enable
            } else {
                PluginLifecycleAction::Disable
            },
            cx,
        );
    }

    pub(super) fn prompt_plugin_rollback(
        &mut self,
        plugin_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = self.plugin_name(&plugin_id);
        let detail = plugin_tf(self.language, PluginMsg::RollbackDetail, &[&name]);
        let answer = window.prompt(
            PromptLevel::Warning,
            plugin_t(self.language, PluginMsg::RollbackTitle),
            Some(&detail),
            &[
                PromptButton::ok(plugin_t(self.language, PluginMsg::ActionRollback)),
                PromptButton::cancel(plugin_t(self.language, PluginMsg::ActionCancel)),
            ],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if matches!(answer.await, Ok(0)) {
                let _ = this.update(cx, |app, cx| {
                    app.run_plugin_lifecycle(plugin_id, PluginLifecycleAction::Rollback, cx);
                });
            }
        })
        .detach();
    }

    pub(super) fn prompt_plugin_uninstall(
        &mut self,
        plugin_id: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let name = self.plugin_name(&plugin_id);
        let storage = self
            .plugin_ui
            .entries
            .iter()
            .find(|entry| entry.plugin_id == plugin_id)
            .map_or_else(
                || format_plugin_bytes(0),
                |entry| format_plugin_bytes(entry.usage.total_bytes()),
            );
        let detail = plugin_tf(
            self.language,
            PluginMsg::UninstallDetail,
            &[&name, &storage],
        );
        let answer = window.prompt(
            PromptLevel::Warning,
            plugin_t(self.language, PluginMsg::UninstallTitle),
            Some(&detail),
            &[
                PromptButton::ok(plugin_t(self.language, PluginMsg::ActionUninstall)),
                PromptButton::cancel(plugin_t(self.language, PluginMsg::ActionCancel)),
            ],
            cx,
        );
        cx.spawn(async move |this, cx| {
            if matches!(answer.await, Ok(0)) {
                let _ = this.update(cx, |app, cx| {
                    app.run_plugin_lifecycle(plugin_id, PluginLifecycleAction::Uninstall, cx);
                });
            }
        })
        .detach();
    }

    fn run_plugin_lifecycle(
        &mut self,
        plugin_id: String,
        action: PluginLifecycleAction,
        cx: &mut Context<Self>,
    ) {
        let Some(manager) = self.plugin_ui.manager() else {
            return;
        };
        let (generation, _) = self
            .plugin_ui
            .begin(plugin_id.clone(), PluginOperationPhase::Activating);
        cx.notify();
        cx.spawn(async move |this, cx| {
            let operation_plugin_id = plugin_id.clone();
            let result = cx
                .background_executor()
                .spawn(async move {
                    match action {
                        PluginLifecycleAction::Enable => {
                            manager.set_enabled(&operation_plugin_id, true)
                        }
                        PluginLifecycleAction::Disable => {
                            manager.set_enabled(&operation_plugin_id, false)
                        }
                        PluginLifecycleAction::Rollback => manager.rollback(&operation_plugin_id),
                        PluginLifecycleAction::Uninstall => {
                            manager.uninstall(&operation_plugin_id, true)
                        }
                    }
                })
                .await;
            let _ = this.update(cx, |app, cx| {
                if result.is_ok() {
                    if app
                        .plugin_ui
                        .advance(generation, PluginOperationPhase::Completed)
                    {
                        app.plugin_ui.refresh_entries(app.language);
                        app.plugin_ui.rebuild_registry();
                        let unavailable = match action {
                            PluginLifecycleAction::Enable | PluginLifecycleAction::Rollback => None,
                            PluginLifecycleAction::Disable => {
                                Some(PagedHostError::ProviderUnavailable)
                            }
                            PluginLifecycleAction::Uninstall => {
                                Some(PagedHostError::ProviderMissing)
                            }
                        };
                        app.reload_plugin_documents(&plugin_id, unavailable, cx);
                    }
                } else {
                    app.plugin_ui
                        .fail(generation, PluginOperationFailure::Activation, None);
                }
                cx.notify();
            });
        })
        .detach();
    }

    fn plugin_name(&self, plugin_id: &str) -> String {
        self.plugin_ui
            .entries
            .iter()
            .find(|entry| entry.plugin_id == plugin_id)
            .map(|entry| entry.identity.name.clone())
            .unwrap_or_else(|| plugin_id.to_owned())
    }
}

pub(super) fn plugin_status_label(language: Language, status: ManagedPluginStatus) -> &'static str {
    plugin_t(
        language,
        match status {
            ManagedPluginStatus::Available => PluginMsg::StatusAvailable,
            ManagedPluginStatus::Installed => PluginMsg::StatusInstalled,
            ManagedPluginStatus::Disabled => PluginMsg::StatusDisabled,
            ManagedPluginStatus::UpdateAvailable => PluginMsg::StatusUpdateAvailable,
            ManagedPluginStatus::Incompatible => PluginMsg::StatusIncompatible,
            ManagedPluginStatus::Quarantined => PluginMsg::StatusQuarantined,
        },
    )
}

pub(super) fn plugin_operation_label(
    language: Language,
    phase: PluginOperationPhase,
) -> &'static str {
    plugin_t(
        language,
        match phase {
            PluginOperationPhase::Downloading => PluginMsg::OperationDownloading,
            PluginOperationPhase::Verifying => PluginMsg::OperationVerifying,
            PluginOperationPhase::Activating => PluginMsg::OperationActivating,
            PluginOperationPhase::Completed => PluginMsg::OperationComplete,
            PluginOperationPhase::Refreshing => PluginMsg::OperationRefreshing,
            PluginOperationPhase::Failed(PluginOperationFailure::Download) => {
                PluginMsg::ErrorOffline
            }
            PluginOperationPhase::Failed(PluginOperationFailure::Verification) => {
                PluginMsg::ErrorVerification
            }
            PluginOperationPhase::Failed(PluginOperationFailure::Activation) => {
                PluginMsg::ErrorActivation
            }
            PluginOperationPhase::Failed(PluginOperationFailure::Unavailable) => {
                PluginMsg::ErrorUnavailable
            }
            PluginOperationPhase::Failed(PluginOperationFailure::Catalog) => {
                PluginMsg::ErrorCatalog
            }
        },
    )
}

pub(super) fn plugin_permissions_label(language: Language, permissions: &[Permission]) -> String {
    permissions
        .iter()
        .map(|permission| {
            plugin_t(
                language,
                match permission {
                    Permission::ReadSelectedFiles => PluginMsg::PermissionReadSelectedFiles,
                    Permission::Network => PluginMsg::PermissionNetwork,
                    Permission::CredentialBroker => PluginMsg::PermissionCredentialBroker,
                },
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn plugin_capabilities_label(language: Language, capabilities: &[String]) -> String {
    capabilities
        .iter()
        .map(|capability| match capability.as_str() {
            "paged-document/v1" => plugin_t(language, PluginMsg::CapabilityPagedDocument),
            _ => capability,
        })
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn format_plugin_bytes(bytes: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = 1024 * KIB;
    if bytes >= MIB {
        format!("{:.2} MiB", bytes as f64 / MIB as f64)
    } else if bytes >= KIB {
        format!("{:.1} KiB", bytes as f64 / KIB as f64)
    } else {
        format!("{bytes} B")
    }
}

pub(super) fn catalog_operation(
    operation: Option<&PluginOperationState>,
) -> Option<&PluginOperationState> {
    operation.filter(|operation| operation.plugin_id == CATALOG_OPERATION_ID)
}

fn refresh_production_catalog() -> Result<PluginManager, ()> {
    let bootstrap = markion::plugin_platform::bootstrap().map_err(|_| ())?;
    let paths = PluginStorePaths::default();
    let catalog_manager = bootstrap.catalog_manager(paths.clone());
    for (catalog_url, signature_url) in PLUGIN_CATALOG_ENDPOINTS {
        let Ok(catalog) = network::fetch_url_bytes_bounded(catalog_url, MAX_PLUGIN_CATALOG_BYTES)
        else {
            continue;
        };
        let Ok(signature) =
            network::fetch_url_bytes_bounded(signature_url, MAX_PLUGIN_CATALOG_SIGNATURE_BYTES)
        else {
            continue;
        };
        if catalog_manager.refresh(&catalog, &signature).is_ok() {
            return bootstrap.plugin_manager(paths).map_err(|_| ());
        }
    }
    Err(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn operation_generation_rejects_stale_completion_and_cancel_is_download_only() {
        let mut state = PluginUiState::default();
        let (first, first_cancel) =
            state.begin("dev.markion.pdf".into(), PluginOperationPhase::Downloading);
        let (second, _) = state.begin("dev.markion.pdf".into(), PluginOperationPhase::Downloading);
        assert!(first_cancel.load(Ordering::Acquire));
        assert!(!state.advance(first, PluginOperationPhase::Completed));
        assert_eq!(state.operation.as_ref().unwrap().generation, second);
        assert!(state.cancel_download());
        assert!(state.operation.is_none());

        let (_, _) = state.begin("dev.markion.pdf".into(), PluginOperationPhase::Verifying);
        assert!(!state.cancel_download());

        let (_, _) = state.begin(
            CATALOG_OPERATION_ID.into(),
            PluginOperationPhase::Refreshing,
        );
        assert!(catalog_operation(state.operation.as_ref()).is_some());
    }

    #[test]
    fn byte_labels_are_exact_enough_for_preinstall_disclosure() {
        assert_eq!(format_plugin_bytes(17), "17 B");
        assert_eq!(format_plugin_bytes(1536), "1.5 KiB");
        assert_eq!(format_plugin_bytes(4 * 1024 * 1024), "4.00 MiB");
    }
}
