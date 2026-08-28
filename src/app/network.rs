use std::{any::type_name, collections::HashMap, sync::OnceLock, time::Duration};

use anyhow::{Context as _, Result, anyhow};
use gpui::{
    App,
    http_client::{AsyncBody, HttpClient, Inner, RedirectPolicy, Request, Response, Url, http},
};

static HTTP_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub(super) fn runtime_handle() -> tokio::runtime::Handle {
    HTTP_RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("failed to initialize Markion HTTP runtime")
        })
        .handle()
        .clone()
}

pub(super) struct MarkionHttpClient {
    client: reqwest::Client,
    handle: tokio::runtime::Handle,
    user_agent: http::HeaderValue,
}

impl MarkionHttpClient {
    pub(super) fn new() -> Result<Self> {
        let user_agent =
            http::HeaderValue::from_str(&format!("Markion/{}", env!("CARGO_PKG_VERSION")))
                .context("building Markion HTTP user agent")?;
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(15))
            .user_agent(user_agent.clone())
            .build()
            .context("building Markion HTTP client")?;
        let handle = runtime_handle();

        Ok(Self {
            client,
            handle,
            user_agent,
        })
    }
}

impl HttpClient for MarkionHttpClient {
    fn type_name(&self) -> &'static str {
        type_name::<Self>()
    }

    fn user_agent(&self) -> Option<&http::HeaderValue> {
        Some(&self.user_agent)
    }

    fn send(
        &self,
        request: Request<AsyncBody>,
    ) -> std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Response<AsyncBody>>> + Send + 'static>,
    > {
        let (parts, body) = request.into_parts();
        let body = match body.0 {
            Inner::Empty => reqwest::Body::default(),
            Inner::Bytes(bytes) => bytes.into_inner().into(),
            Inner::AsyncReader(_) => {
                return Box::pin(async {
                    Err(anyhow!("streaming HTTP request bodies are not supported"))
                });
            }
        };

        let mut request = self
            .client
            .request(parts.method, parts.uri.to_string())
            .headers(parts.headers)
            .body(body);
        if let Some(policy) = parts.extensions.get::<RedirectPolicy>() {
            request = request.redirect_policy(match policy {
                RedirectPolicy::NoFollow => reqwest::redirect::Policy::none(),
                RedirectPolicy::FollowLimit(limit) => {
                    reqwest::redirect::Policy::limited(*limit as usize)
                }
                RedirectPolicy::FollowAll => reqwest::redirect::Policy::limited(100),
            });
        }

        let handle = self.handle.clone();
        Box::pin(async move {
            handle
                .spawn(async move {
                    let response = request.send().await.context("sending HTTP request")?;
                    let status = response.status();
                    let version = response.version();
                    let headers = response.headers().clone();
                    let body = response
                        .bytes()
                        .await
                        .context("reading HTTP response body")?;
                    let mut response = Response::builder().status(status).version(version);
                    *response
                        .headers_mut()
                        .expect("new response builder must expose headers") = headers;
                    response
                        .body(AsyncBody::from(body.to_vec()))
                        .context("building HTTP response")
                })
                .await
                .context("joining HTTP runtime task")?
        })
    }

    fn proxy(&self) -> Option<&Url> {
        None
    }
}

pub(super) fn install_http_client(cx: &mut App) -> Result<()> {
    cx.set_http_client(std::sync::Arc::new(MarkionHttpClient::new()?));
    Ok(())
}

/// Overall (connect + body) timeout for single-URL fetches. Without it a
/// server that accepts the connection but stalls the body pins the calling
/// background-executor thread forever — the connect timeout alone never
/// fires once the TCP handshake succeeds.
const FETCH_URL_TIMEOUT: Duration = Duration::from_secs(60);

/// Fetch a remote URL's response body on the shared HTTP runtime.
///
/// Used by the Markdown preview-image cache so decode work can run off the UI
/// thread without depending on GPUI's asset loader.
pub(super) fn fetch_url_bytes(url: &str) -> Result<Vec<u8>> {
    runtime_handle().block_on(async {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(15))
            .timeout(FETCH_URL_TIMEOUT)
            .user_agent(format!("Markion/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("building preview-image HTTP client")?;
        let response = client
            .get(url)
            .send()
            .await
            .with_context(|| format!("requesting {url}"))?;
        if !response.status().is_success() {
            return Err(anyhow!(
                "HTTP {} fetching {url}",
                response.status().as_u16()
            ));
        }
        let bytes = response
            .bytes()
            .await
            .with_context(|| format!("reading body from {url}"))?;
        Ok(bytes.to_vec())
    })
}

/// Per-image download cap for the export prefetch.
const EXPORT_IMAGE_MAX_BYTES: usize = 32 * 1024 * 1024;
/// Per-request overall timeout for the export prefetch (connect + body),
/// bounding how long one slow server can delay an export.
const EXPORT_IMAGE_TIMEOUT: Duration = Duration::from_secs(30);

/// Concurrently GETs every URL on the shared HTTP runtime, returning the
/// successfully fetched bodies keyed by URL. A URL that errors, times out,
/// answers non-2xx, or exceeds the size cap is logged and omitted — the
/// corresponding image keeps the DOCX export text fallback instead of
/// failing the export. Intended to run on a background executor.
pub(super) fn fetch_url_bytes_all(urls: Vec<String>) -> HashMap<String, Vec<u8>> {
    if urls.is_empty() {
        return HashMap::new();
    }
    runtime_handle().block_on(async {
        let client = reqwest::Client::builder()
            .use_rustls_tls()
            .connect_timeout(Duration::from_secs(15))
            .user_agent(format!("Markion/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .context("building export-image HTTP client")
            .expect("export-image HTTP client builds with valid defaults");
        let mut pending = tokio::task::JoinSet::new();
        for url in urls {
            let client = client.clone();
            pending.spawn(async move {
                let fetch = async {
                    let response = client
                        .get(&url)
                        .send()
                        .await
                        .with_context(|| format!("requesting {url}"))?;
                    if !response.status().is_success() {
                        anyhow::bail!("HTTP {} fetching {url}", response.status().as_u16());
                    }
                    let bytes = response
                        .bytes()
                        .await
                        .with_context(|| format!("reading body from {url}"))?;
                    if bytes.len() > EXPORT_IMAGE_MAX_BYTES {
                        anyhow::bail!(
                            "image at {url} is {} bytes, over the {} byte export cap",
                            bytes.len(),
                            EXPORT_IMAGE_MAX_BYTES
                        );
                    }
                    Ok(bytes.to_vec())
                };
                (
                    url.clone(),
                    tokio::time::timeout(EXPORT_IMAGE_TIMEOUT, fetch).await,
                )
            });
        }
        let mut fetched = HashMap::new();
        while let Some(joined) = pending.join_next().await {
            let Ok((url, result)) = joined else {
                continue;
            };
            match result {
                Ok(Ok(bytes)) => {
                    fetched.insert(url, bytes);
                }
                Ok(Err(err)) => {
                    tracing::warn!(error = %err, "skipping remote export image");
                }
                Err(_) => {
                    tracing::warn!(
                        timeout = ?EXPORT_IMAGE_TIMEOUT,
                        "remote export image timed out"
                    );
                }
            }
        }
        fetched
    })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read as _, Write as _},
        net::{Shutdown, TcpListener, TcpStream},
        thread,
    };

    use gpui::http_client::HttpClient as _;

    use super::*;

    fn read_http_request(stream: &mut TcpStream) -> String {
        let mut request = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let read = stream.read(&mut chunk).unwrap();
            if read == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }
        String::from_utf8_lossy(&request).into_owned()
    }

    fn serve_http_response(listener: TcpListener, response: Vec<u8>) -> String {
        let (mut stream, _) = listener.accept().unwrap();
        let request = read_http_request(&mut stream);
        stream.write_all(&response).unwrap();
        stream.flush().unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        let mut trailing = Vec::new();
        stream.read_to_end(&mut trailing).unwrap();
        request
    }

    #[test]
    fn concrete_http_client_executes_loopback_request() {
        let redirect_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let redirect_address = redirect_listener.local_addr().unwrap();
        let image_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let image_address = image_listener.local_addr().unwrap();
        let redirect_response = format!(
            "HTTP/1.1 302 Found\r\nLocation: http://{image_address}/image.png\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
        )
        .into_bytes();
        let redirect_server =
            thread::spawn(move || serve_http_response(redirect_listener, redirect_response));
        let image_server = thread::spawn(move || {
            serve_http_response(
                image_listener,
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: image/png\r\nConnection: close\r\n\r\nok"
                    .to_vec(),
            )
        });

        let client = MarkionHttpClient::new().unwrap();
        let response = client
            .handle
            .block_on(client.get(
                &format!("http://{redirect_address}/redirect"),
                ().into(),
                true,
            ))
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);
        match response.into_body().0 {
            Inner::Bytes(bytes) => assert_eq!(bytes.into_inner().as_ref(), b"ok"),
            _ => panic!("expected a buffered response body"),
        }
        let redirect_request = redirect_server.join().unwrap();
        let image_request = image_server.join().unwrap();
        assert!(redirect_request.starts_with("GET /redirect "));
        assert!(image_request.starts_with("GET /image.png "));
        assert!(redirect_request.to_ascii_lowercase().contains(&format!(
            "user-agent: markion/{}",
            env!("CARGO_PKG_VERSION")
        )));
    }

    #[test]
    fn fetch_url_bytes_all_keeps_successes_and_skips_failures() {
        let ok_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let ok_address = ok_listener.local_addr().unwrap();
        let fail_listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let fail_address = fail_listener.local_addr().unwrap();
        let ok_server = thread::spawn(move || {
            serve_http_response(
                ok_listener,
                b"HTTP/1.1 200 OK\r\nContent-Length: 4\r\nConnection: close\r\n\r\nabcd".to_vec(),
            )
        });
        let fail_server = thread::spawn(move || {
            serve_http_response(
                fail_listener,
                b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .to_vec(),
            )
        });

        let fetched = fetch_url_bytes_all(vec![
            format!("http://{ok_address}/ok.png"),
            format!("http://{fail_address}/gone.png"),
        ]);

        // The 404 is skipped, not fatal; the success is keyed by its URL.
        assert_eq!(fetched.len(), 1);
        let (url, bytes) = fetched.iter().next().unwrap();
        assert!(url.ends_with("/ok.png"));
        assert_eq!(bytes, b"abcd");
        ok_server.join().unwrap();
        fail_server.join().unwrap();
    }

    #[test]
    fn fetch_url_bytes_all_returns_empty_for_no_urls() {
        assert!(fetch_url_bytes_all(Vec::new()).is_empty());
    }

    #[test]
    #[ignore = "requires MARKION_TEST_REMOTE_IMAGE_URL and external network access"]
    fn concrete_http_client_fetches_external_image() {
        let url = std::env::var("MARKION_TEST_REMOTE_IMAGE_URL")
            .expect("MARKION_TEST_REMOTE_IMAGE_URL must be set");
        let client = MarkionHttpClient::new().unwrap();
        let response = client
            .handle
            .block_on(client.get(&url, ().into(), true))
            .unwrap();

        assert!(response.status().is_success());
        assert!(
            response
                .headers()
                .get(http::header::CONTENT_TYPE)
                .and_then(|value| value.to_str().ok())
                .is_some_and(|value| value.starts_with("image/"))
        );
    }
}
