use std::{any::type_name, sync::OnceLock, time::Duration};

use anyhow::{Context as _, Result, anyhow};
use gpui::{
    App,
    http_client::{AsyncBody, HttpClient, Inner, RedirectPolicy, Request, Response, Url, http},
};

static HTTP_RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

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
        let runtime = HTTP_RUNTIME.get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .expect("failed to initialize Markion HTTP runtime")
        });

        Ok(Self {
            client,
            handle: runtime.handle().clone(),
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

#[cfg(test)]
mod tests {
    use std::{
        io::{Read as _, Write as _},
        net::TcpListener,
        thread,
    };

    use gpui::http_client::HttpClient as _;

    use super::*;

    #[test]
    fn concrete_http_client_executes_loopback_request() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            let mut requests = Vec::new();
            for response in [
                b"HTTP/1.1 302 Found\r\nLocation: /image.png\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                    .as_slice(),
                b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nContent-Type: image/png\r\nConnection: close\r\n\r\nok"
                    .as_slice(),
            ] {
                let (mut stream, _) = listener.accept().unwrap();
                let mut request = [0_u8; 2048];
                let request_len = stream.read(&mut request).unwrap();
                requests.push(String::from_utf8_lossy(&request[..request_len]).into_owned());
                stream.write_all(response).unwrap();
            }
            requests
        });

        let client = MarkionHttpClient::new().unwrap();
        let response = client
            .handle
            .block_on(client.get(&format!("http://{address}/redirect"), ().into(), true))
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);
        match response.into_body().0 {
            Inner::Bytes(bytes) => assert_eq!(bytes.into_inner().as_ref(), b"ok"),
            _ => panic!("expected a buffered response body"),
        }
        let requests = server.join().unwrap();
        assert!(requests[0].starts_with("GET /redirect "));
        assert!(requests[1].starts_with("GET /image.png "));
        assert!(requests[0].to_ascii_lowercase().contains(&format!(
            "user-agent: markion/{}",
            env!("CARGO_PKG_VERSION")
        )));
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
