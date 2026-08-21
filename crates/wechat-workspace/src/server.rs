use std::{
    io,
    net::SocketAddr,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};

use axum::{
    Router,
    body::Body,
    extract::{Path as AxumPath, Request, State},
    http::{
        HeaderMap, HeaderValue, Response, StatusCode,
        header::{AUTHORIZATION, CACHE_CONTROL, CONTENT_TYPE},
    },
    middleware::{self, Next},
    routing::{get, post},
};
use serde::Serialize;
use thiserror::Error;
use tokio::{net::TcpListener, sync::oneshot, task::JoinHandle};

use crate::session::{SessionId, SessionStore};
use crate::{
    BundleError, Clock, OsTokenSource, PublishingSnapshot, SessionLimits, SystemClock, TokenSource,
    verify_bundle,
};

const CSP: &str = "default-src 'none'; script-src 'self'; style-src 'self' 'unsafe-inline'; font-src 'self'; img-src 'self' blob: data: http: https:; connect-src 'self'; frame-src 'none'; object-src 'none'; base-uri 'none'; form-action 'none'";

#[derive(Clone)]
pub struct WorkspaceConfig {
    asset_root: PathBuf,
    clock: Arc<dyn Clock>,
    token_source: Arc<dyn TokenSource>,
    limits: SessionLimits,
}

impl WorkspaceConfig {
    pub fn new(asset_root: impl Into<PathBuf>) -> Self {
        Self {
            asset_root: asset_root.into(),
            clock: Arc::new(SystemClock::default()),
            token_source: Arc::new(OsTokenSource),
            limits: SessionLimits::default(),
        }
    }

    pub fn with_clock(mut self, clock: Arc<dyn Clock>) -> Self {
        self.clock = clock;
        self
    }

    pub fn with_token_source(mut self, token_source: Arc<dyn TokenSource>) -> Self {
        self.token_source = token_source;
        self
    }

    pub fn with_session_limits(mut self, limits: SessionLimits) -> Self {
        self.limits = limits;
        self
    }
}

#[derive(Debug, Error)]
pub enum WorkspaceError {
    #[error(transparent)]
    Bundle(#[from] BundleError),
    #[error("the local publishing workspace listener could not start")]
    Bind(#[source] io::Error),
    #[error("a secure publishing capability could not be created")]
    Token(#[source] io::Error),
}

#[derive(Debug, Clone)]
pub struct LaunchSession {
    id: SessionId,
    url: String,
}

impl LaunchSession {
    pub fn url(&self) -> &str {
        &self.url
    }
}

#[derive(Clone)]
pub struct WorkspaceService {
    inner: Arc<ServiceInner>,
}

struct ServiceInner {
    asset_root: PathBuf,
    sessions: Arc<SessionStore>,
    running: Mutex<Option<RunningServer>>,
}

struct RunningServer {
    address: SocketAddr,
    shutdown: Option<oneshot::Sender<()>>,
    task: JoinHandle<()>,
}

#[derive(Clone)]
struct HttpState {
    asset_root: PathBuf,
    sessions: Arc<SessionStore>,
}

impl WorkspaceService {
    /// Validates configuration but does not bind a socket. Binding occurs on
    /// the first call to `create_session`.
    pub fn new(config: WorkspaceConfig) -> Result<Self, WorkspaceError> {
        verify_bundle(&config.asset_root)?;
        let asset_root = config.asset_root.canonicalize().map_err(BundleError::Io)?;
        Ok(Self {
            inner: Arc::new(ServiceInner {
                asset_root,
                sessions: Arc::new(SessionStore::new(
                    config.clock,
                    config.token_source,
                    config.limits,
                )),
                running: Mutex::new(None),
            }),
        })
    }

    pub async fn create_session(
        &self,
        snapshot: PublishingSnapshot,
    ) -> Result<LaunchSession, WorkspaceError> {
        let address = self.ensure_started().await?;
        let (id, claim) = self
            .inner
            .sessions
            .create(snapshot)
            .map_err(WorkspaceError::Token)?;
        Ok(LaunchSession {
            id,
            url: format!("http://{address}/#claim={claim}"),
        })
    }

    pub fn revoke(&self, launch: &LaunchSession) -> bool {
        self.inner.sessions.revoke(launch.id)
    }

    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.inner
            .running
            .lock()
            .expect("server state poisoned")
            .as_ref()
            .map(|server| server.address)
    }

    async fn ensure_started(&self) -> Result<SocketAddr, WorkspaceError> {
        if let Some(address) = self.local_addr() {
            return Ok(address);
        }
        let listener = TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, 0))
            .await
            .map_err(WorkspaceError::Bind)?;
        let address = listener.local_addr().map_err(WorkspaceError::Bind)?;

        let state = HttpState {
            asset_root: self.inner.asset_root.clone(),
            sessions: Arc::clone(&self.inner.sessions),
        };
        let router = Router::new()
            .route("/", get(index))
            .route("/static/{*path}", get(static_asset))
            .route("/api/claim", post(claim))
            .route("/api/document", get(document))
            .route("/api/heartbeat", post(heartbeat))
            .route("/api/resource/{id}", get(resource))
            .fallback(not_found)
            .layer(middleware::from_fn(security_headers))
            .with_state(state);
        let (shutdown, receiver) = oneshot::channel();
        let task = tokio::spawn(async move {
            tracing::debug!(%address, "local publishing workspace started");
            if let Err(error) = axum::serve(listener, router)
                .with_graceful_shutdown(async move {
                    let _ = receiver.await;
                })
                .await
            {
                tracing::warn!(%error, "local publishing workspace stopped with an error");
            } else {
                tracing::debug!("local publishing workspace stopped");
            }
        });

        let mut running = self.inner.running.lock().expect("server state poisoned");
        if let Some(existing) = running.as_ref() {
            let _ = shutdown.send(());
            task.abort();
            return Ok(existing.address);
        }
        *running = Some(RunningServer {
            address,
            shutdown: Some(shutdown),
            task,
        });
        Ok(address)
    }
}

impl Drop for ServiceInner {
    fn drop(&mut self) {
        if let Ok(running) = self.running.get_mut()
            && let Some(mut server) = running.take()
        {
            if let Some(shutdown) = server.shutdown.take() {
                let _ = shutdown.send(());
            }
            // Signal graceful shutdown first. Abort is the final drop-time
            // guarantee that no detached listener can outlive its owner.
            server.task.abort();
        }
    }
}

async fn security_headers(request: Request, next: Next) -> Response<Body> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert("content-security-policy", HeaderValue::from_static(CSP));
    headers.insert("referrer-policy", HeaderValue::from_static("no-referrer"));
    headers.insert(
        "x-content-type-options",
        HeaderValue::from_static("nosniff"),
    );
    response
}

async fn index(State(state): State<HttpState>) -> Response<Body> {
    static_response(&state.asset_root.join("index.html"))
}

async fn static_asset(
    State(state): State<HttpState>,
    AxumPath(path): AxumPath<String>,
) -> Response<Body> {
    let Some(path) = safe_static_path(&path) else {
        return not_found().await;
    };
    static_response(&state.asset_root.join("static").join(path))
}

async fn claim(State(state): State<HttpState>, headers: HeaderMap) -> Response<Body> {
    let Some(claim) = bearer(&headers) else {
        return unauthorized();
    };
    match state.sessions.claim(claim) {
        Ok(Some(session_token)) => dynamic_json(StatusCode::OK, &ClaimResponse { session_token }),
        Ok(None) => unauthorized(),
        Err(error) => {
            tracing::warn!(%error, "publishing claim token generation failed");
            unauthorized()
        }
    }
}

async fn document(State(state): State<HttpState>, headers: HeaderMap) -> Response<Body> {
    let Some(snapshot) = authorize(&state, &headers) else {
        return unauthorized();
    };
    dynamic_json(StatusCode::OK, &snapshot.payload())
}

async fn heartbeat(State(state): State<HttpState>, headers: HeaderMap) -> Response<Body> {
    if authorize(&state, &headers).is_none() {
        return unauthorized();
    }
    dynamic_response(
        StatusCode::NO_CONTENT,
        "application/octet-stream",
        Vec::new(),
    )
}

async fn resource(
    State(state): State<HttpState>,
    AxumPath(id): AxumPath<String>,
    headers: HeaderMap,
) -> Response<Body> {
    let Some(snapshot) = authorize(&state, &headers) else {
        return unauthorized();
    };
    let Some(resource) = snapshot
        .resources
        .iter()
        .find(|resource| resource.id() == id)
    else {
        return not_found_dynamic();
    };
    match resource.read() {
        Ok(resource) => dynamic_response(StatusCode::OK, &resource.mime, resource.bytes),
        Err(_) => not_found_dynamic(),
    }
}

fn authorize(state: &HttpState, headers: &HeaderMap) -> Option<Arc<PublishingSnapshot>> {
    state.sessions.authorize(bearer(headers)?)
}

fn bearer(headers: &HeaderMap) -> Option<&str> {
    headers
        .get(AUTHORIZATION)?
        .to_str()
        .ok()?
        .strip_prefix("Bearer ")
        .filter(|token| !token.is_empty())
}

fn static_response(path: &Path) -> Response<Body> {
    let Ok(bytes) = std::fs::read(path) else {
        return plain_response(StatusCode::NOT_FOUND, "not found");
    };
    let mime = mime_guess::from_path(path)
        .first_raw()
        .unwrap_or("application/octet-stream");
    response(StatusCode::OK, mime, bytes, None)
}

fn dynamic_json<T: Serialize>(status: StatusCode, value: &T) -> Response<Body> {
    match serde_json::to_vec(value) {
        Ok(bytes) => dynamic_response(status, "application/json; charset=utf-8", bytes),
        Err(_) => dynamic_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "application/json; charset=utf-8",
            br#"{"error":"unavailable"}"#.to_vec(),
        ),
    }
}

fn dynamic_response(status: StatusCode, mime: &str, bytes: Vec<u8>) -> Response<Body> {
    response(status, mime, bytes, Some("no-store"))
}

fn unauthorized() -> Response<Body> {
    dynamic_response(
        StatusCode::UNAUTHORIZED,
        "application/json; charset=utf-8",
        br#"{"error":"unauthorized"}"#.to_vec(),
    )
}

fn not_found_dynamic() -> Response<Body> {
    dynamic_response(
        StatusCode::NOT_FOUND,
        "application/json; charset=utf-8",
        br#"{"error":"not_found"}"#.to_vec(),
    )
}

async fn not_found() -> Response<Body> {
    plain_response(StatusCode::NOT_FOUND, "not found")
}

fn plain_response(status: StatusCode, text: &str) -> Response<Body> {
    response(
        status,
        "text/plain; charset=utf-8",
        text.as_bytes().to_vec(),
        None,
    )
}

fn response(
    status: StatusCode,
    mime: &str,
    bytes: Vec<u8>,
    cache_control: Option<&'static str>,
) -> Response<Body> {
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = status;
    if let Ok(mime) = HeaderValue::from_str(mime) {
        response.headers_mut().insert(CONTENT_TYPE, mime);
    }
    if let Some(cache_control) = cache_control {
        response
            .headers_mut()
            .insert(CACHE_CONTROL, HeaderValue::from_static(cache_control));
    }
    response
}

fn safe_static_path(path: &str) -> Option<PathBuf> {
    let path = Path::new(path);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }
    Some(path.to_path_buf())
}

#[derive(Serialize)]
struct ClaimResponse {
    session_token: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BundleFile, BundleManifest, ManualClock, ThirdPartyComponent};
    use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
    use sha2::{Digest, Sha256};
    use std::{collections::VecDeque, fs, sync::Mutex as StdMutex, time::Duration};

    #[derive(Debug)]
    struct ScriptedTokens(StdMutex<VecDeque<String>>);

    impl ScriptedTokens {
        fn new(count: usize) -> Self {
            let tokens = (0..count)
                .map(|index| URL_SAFE_NO_PAD.encode([index as u8; 32]))
                .collect();
            Self(StdMutex::new(tokens))
        }
    }

    impl TokenSource for ScriptedTokens {
        fn generate(&self) -> io::Result<String> {
            self.0
                .lock()
                .unwrap()
                .pop_front()
                .ok_or_else(|| io::Error::other("scripted token source exhausted"))
        }
    }

    fn bundle() -> tempfile::TempDir {
        let temp = tempfile::tempdir().unwrap();
        fs::create_dir(temp.path().join("static")).unwrap();
        fs::write(
            temp.path().join("index.html"),
            "<!doctype html><h1>workspace</h1>",
        )
        .unwrap();
        fs::write(temp.path().join("static/app.js"), "window.ready=true;").unwrap();
        fs::write(temp.path().join("LICENSE.txt"), "MIT").unwrap();
        let files = ["index.html", "static/app.js", "LICENSE.txt"]
            .into_iter()
            .map(|path| BundleFile {
                path: path.into(),
                sha256: format!(
                    "{:x}",
                    Sha256::digest(fs::read(temp.path().join(path)).unwrap())
                ),
            })
            .collect();
        let manifest = BundleManifest {
            import_format_version: 1,
            source_repository: "https://example.invalid/marknice".into(),
            source_commit: "c009c1ec7e7c92f89afa5a32edcb126b5296bda7".into(),
            third_party: vec![ThirdPartyComponent {
                name: "fixture".into(),
                version: "1".into(),
                license: "MIT".into(),
                license_file: "LICENSE.txt".into(),
            }],
            files,
        };
        fs::write(
            temp.path().join(crate::assets::BUNDLE_MANIFEST),
            serde_json::to_vec_pretty(&manifest).unwrap(),
        )
        .unwrap();
        temp
    }

    fn snapshot(markdown: &str) -> PublishingSnapshot {
        PublishingSnapshot {
            markdown: Arc::from(markdown),
            display_name: "note.md".into(),
            language: "en-US".into(),
            resources: Vec::new(),
            unresolved_local_images: Vec::new(),
        }
    }

    fn claim_from_url(url: &str) -> &str {
        url.split("#claim=").nth(1).unwrap()
    }

    async fn exchange(client: &reqwest::Client, base: &str, claim: &str) -> (StatusCode, String) {
        let response = client
            .post(format!("{base}/api/claim"))
            .bearer_auth(claim)
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        (status, body)
    }

    #[tokio::test]
    async fn starts_lazily_on_loopback_and_enforces_claim_and_headers() {
        let bundle = bundle();
        let service = WorkspaceService::new(
            WorkspaceConfig::new(bundle.path()).with_token_source(Arc::new(ScriptedTokens::new(8))),
        )
        .unwrap();
        assert!(service.local_addr().is_none());
        let launch = service.create_session(snapshot("# hello")).await.unwrap();
        let address = service.local_addr().unwrap();
        assert!(address.ip().is_loopback());
        assert_ne!(address.port(), 0);
        let base = format!("http://{address}");
        let client = reqwest::Client::builder().no_proxy().build().unwrap();

        let shell = client.get(&base).send().await.unwrap();
        assert_eq!(shell.status(), StatusCode::OK);
        assert_eq!(shell.headers()["x-content-type-options"], "nosniff");
        assert!(
            shell.headers()["content-security-policy"]
                .to_str()
                .unwrap()
                .contains("connect-src 'self'")
        );

        let claim = claim_from_url(launch.url());
        let (status, body) = exchange(&client, &base, claim).await;
        assert_eq!(status, StatusCode::OK);
        let token = serde_json::from_str::<serde_json::Value>(&body).unwrap()["session_token"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(
            exchange(&client, &base, claim).await.0,
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            exchange(&client, &base, "invalid").await.0,
            StatusCode::UNAUTHORIZED
        );

        let document = client
            .get(format!("{base}/api/document"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(document.status(), StatusCode::OK);
        assert_eq!(document.headers()[CACHE_CONTROL], "no-store");
        assert!(document.text().await.unwrap().contains("# hello"));
        assert_eq!(
            client
                .get(format!("{base}/api/document"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
        assert_eq!(
            client
                .get(format!("{base}/unknown"))
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::NOT_FOUND
        );
    }

    #[tokio::test]
    async fn sessions_are_isolated_expire_and_can_be_revoked() {
        let bundle = bundle();
        let clock = Arc::new(ManualClock::default());
        let service = WorkspaceService::new(
            WorkspaceConfig::new(bundle.path())
                .with_clock(clock.clone())
                .with_token_source(Arc::new(ScriptedTokens::new(12)))
                .with_session_limits(SessionLimits {
                    max_sessions: 8,
                    idle_timeout: Duration::from_secs(10),
                }),
        )
        .unwrap();
        let one = service.create_session(snapshot("one")).await.unwrap();
        let two = service.create_session(snapshot("two")).await.unwrap();
        let base = format!("http://{}", service.local_addr().unwrap());
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let (_, one_body) = exchange(&client, &base, claim_from_url(one.url())).await;
        let one_token =
            serde_json::from_str::<serde_json::Value>(&one_body).unwrap()["session_token"]
                .as_str()
                .unwrap()
                .to_owned();
        let (_, two_body) = exchange(&client, &base, claim_from_url(two.url())).await;
        let two_token =
            serde_json::from_str::<serde_json::Value>(&two_body).unwrap()["session_token"]
                .as_str()
                .unwrap()
                .to_owned();
        let one_document = client
            .get(format!("{base}/api/document"))
            .bearer_auth(&one_token)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let two_document = client
            .get(format!("{base}/api/document"))
            .bearer_auth(&two_token)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        assert!(one_document.contains("one"));
        assert!(two_document.contains("two"));

        clock.advance(Duration::from_secs(11));
        assert_eq!(
            client
                .get(format!("{base}/api/document"))
                .bearer_auth(&one_token)
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );

        let unused = service.create_session(snapshot("unused")).await.unwrap();
        assert!(service.revoke(&unused));
        assert_eq!(
            exchange(&client, &base, claim_from_url(unused.url()))
                .await
                .0,
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn protected_resource_is_mime_safe_no_store_and_session_scoped() {
        let bundle = bundle();
        let managed = tempfile::tempdir().unwrap();
        let image = managed.path().join("cover.svg");
        fs::write(&image, b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>").unwrap();
        let resource =
            crate::PublishingResource::from_path("note.assets/cover.svg", managed.path(), &image)
                .unwrap();
        let resource_id = resource.id().to_owned();
        let mut with_resource = snapshot("image");
        with_resource.resources.push(resource);
        let service = WorkspaceService::new(
            WorkspaceConfig::new(bundle.path()).with_token_source(Arc::new(ScriptedTokens::new(8))),
        )
        .unwrap();
        let launch = service.create_session(with_resource).await.unwrap();
        let base = format!("http://{}", service.local_addr().unwrap());
        let client = reqwest::Client::builder().no_proxy().build().unwrap();
        let (_, claim_body) = exchange(&client, &base, claim_from_url(launch.url())).await;
        let token =
            serde_json::from_str::<serde_json::Value>(&claim_body).unwrap()["session_token"]
                .as_str()
                .unwrap()
                .to_owned();

        let response = client
            .get(format!("{base}/api/resource/{resource_id}"))
            .bearer_auth(&token)
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[CONTENT_TYPE], "image/svg+xml");
        assert_eq!(response.headers()[CACHE_CONTROL], "no-store");
        assert_eq!(
            response.bytes().await.unwrap().as_ref(),
            b"<svg xmlns=\"http://www.w3.org/2000/svg\"></svg>"
        );
        assert_eq!(
            client
                .get(format!("{base}/api/resource/{resource_id}"))
                .bearer_auth("another-session")
                .send()
                .await
                .unwrap()
                .status(),
            StatusCode::UNAUTHORIZED
        );
    }

    #[tokio::test]
    async fn dropping_the_last_service_handle_stops_the_listener() {
        let bundle = bundle();
        let service = WorkspaceService::new(
            WorkspaceConfig::new(bundle.path()).with_token_source(Arc::new(ScriptedTokens::new(4))),
        )
        .unwrap();
        service.create_session(snapshot("shutdown")).await.unwrap();
        let address = service.local_addr().unwrap();
        assert_eq!(Arc::strong_count(&service.inner), 1);
        drop(service);

        let client = reqwest::Client::builder()
            .no_proxy()
            .timeout(Duration::from_millis(100))
            .build()
            .unwrap();
        let stopped = tokio::time::timeout(Duration::from_secs(2), async move {
            loop {
                if client
                    .get(format!("http://{address}/"))
                    .send()
                    .await
                    .is_err()
                {
                    break;
                }
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        })
        .await;
        assert!(
            stopped.is_ok(),
            "listener should stop after its owner is dropped"
        );
    }
}
