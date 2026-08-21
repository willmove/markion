use super::*;

pub(super) trait BrowserLauncher: Send + Sync {
    fn open(&self, url: &str) -> io::Result<()>;
}

pub(super) struct DefaultBrowserLauncher;

impl BrowserLauncher for DefaultBrowserLauncher {
    fn open(&self, url: &str) -> io::Result<()> {
        #[cfg(target_os = "windows")]
        let mut command = {
            let mut command = Command::new("rundll32.exe");
            command.args(["url.dll,FileProtocolHandler", url]);
            command
        };
        #[cfg(target_os = "macos")]
        let mut command = {
            let mut command = Command::new("open");
            command.arg(url);
            command
        };
        #[cfg(all(unix, not(target_os = "macos")))]
        let mut command = {
            let mut command = Command::new("xdg-open");
            command.arg(url);
            command
        };
        command.spawn().map(|_| ())
    }
}

impl MarkionApp {
    pub(super) fn publish_wechat(
        &mut self,
        _: &PublishWechat,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let service = match self.publishing_service.clone() {
            Some(service) => service,
            None => {
                let service = wechat_workspace::discover_workspace_assets()
                    .map_err(|error| error.to_string())
                    .and_then(|root| {
                        wechat_workspace::WorkspaceService::new(
                            wechat_workspace::WorkspaceConfig::new(root),
                        )
                        .map_err(|error| error.to_string())
                    });
                match service {
                    Ok(service) => {
                        self.publishing_service = Some(service.clone());
                        service
                    }
                    Err(error) => {
                        tracing::warn!(error = %error, "publishing workspace setup failed");
                        self.status = self.trf(Msg::StatusPublishSetupFailed, &[&error]);
                        self.active_menu = None;
                        cx.notify();
                        return;
                    }
                }
            }
        };

        let snapshot = build_publishing_snapshot(&self.active_tab().document, self.language.code());
        let task_service = service.clone();
        let launch_task = network::runtime_handle()
            .spawn(async move { task_service.create_session(snapshot).await });
        self.status = self.tr(Msg::StatusPublishingOpening).into();
        self.active_menu = None;
        cx.notify();

        cx.spawn(async move |this, cx| {
            let outcome = launch_task.await;
            let _ = this.update(cx, |app, cx| {
                match outcome {
                    Ok(Ok(launch)) => match app.browser_launcher.open(launch.url()) {
                        Ok(()) => {
                            app.status = app.tr(Msg::StatusPublishingOpened).into();
                        }
                        Err(error) => {
                            service.revoke(&launch);
                            tracing::warn!(error = %error, "default browser dispatch failed");
                            app.status =
                                app.trf(Msg::StatusPublishLaunchFailed, &[&error.to_string()]);
                        }
                    },
                    Ok(Err(error)) => {
                        tracing::warn!(error = %error, "publishing workspace startup failed");
                        app.status = app.trf(Msg::StatusPublishSetupFailed, &[&error.to_string()]);
                    }
                    Err(error) => {
                        tracing::warn!(error = %error, "publishing workspace task failed");
                        app.status = app.trf(Msg::StatusPublishSetupFailed, &[&error.to_string()]);
                    }
                }
                cx.notify();
            });
        })
        .detach();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[derive(Default)]
    struct FakeBrowserLauncher {
        urls: Mutex<Vec<String>>,
        failure: Option<io::ErrorKind>,
    }

    impl BrowserLauncher for FakeBrowserLauncher {
        fn open(&self, url: &str) -> io::Result<()> {
            if let Some(kind) = self.failure {
                return Err(io::Error::new(kind, "scripted browser failure"));
            }
            self.urls.lock().unwrap().push(url.to_owned());
            Ok(())
        }
    }

    #[test]
    fn browser_launcher_is_injectable_and_reports_dispatch_failure() {
        let success = FakeBrowserLauncher::default();
        success.open("http://127.0.0.1:1/#claim=secret").unwrap();
        assert_eq!(success.urls.lock().unwrap().len(), 1);

        let failure = FakeBrowserLauncher {
            failure: Some(io::ErrorKind::NotFound),
            ..Default::default()
        };
        assert_eq!(
            failure.open("http://127.0.0.1:1/").unwrap_err().kind(),
            io::ErrorKind::NotFound
        );
    }
}
