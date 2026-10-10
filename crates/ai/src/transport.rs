//! Dedicated async streaming transport. Never uses the host's buffered image HTTP adapter.
use crate::{
    AiError, Capabilities, Event, Profile, Protocol, Secret, Usage,
    protocol::{self, Request, ToolDefinition},
};
use serde_json::{Value, json};
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use tokio::{
    sync::{Notify, mpsc},
    time::timeout,
};

#[derive(Clone, Default)]
pub struct Cancellation {
    canceled: Arc<AtomicBool>,
    notify: Arc<Notify>,
}
impl Cancellation {
    pub fn cancel(&self) {
        self.canceled.store(true, Ordering::Release);
        self.notify.notify_waiters();
    }
    pub fn is_canceled(&self) -> bool {
        self.canceled.load(Ordering::Acquire)
    }
    pub async fn canceled(&self) {
        let notified = self.notify.notified();
        if self.is_canceled() {
            return;
        }
        notified.await;
    }
}

#[derive(Debug, Clone)]
pub struct ToolCall {
    pub id: String,
    pub name: String,
    pub arguments: Value,
}
#[derive(Debug, Default)]
pub struct Output {
    pub text: String,
    pub calls: Vec<ToolCall>,
    pub wire_output: Vec<Value>,
    pub usage: Usage,
    pub streaming: bool,
}

fn client(profile: &Profile) -> Result<reqwest::Client, AiError> {
    let url = profile.base_url()?;
    let mut builder = reqwest::Client::builder()
        .use_rustls_tls()
        .redirect_policy(reqwest::redirect::Policy::none())
        .connect_timeout(Duration::from_secs(15))
        .user_agent("Markion-AI");
    let loopback = match url.host() {
        Some(url::Host::Domain(v)) => v.eq_ignore_ascii_case("localhost"),
        Some(url::Host::Ipv4(v)) => v.is_loopback(),
        Some(url::Host::Ipv6(v)) => v.is_loopback(),
        None => false,
    };
    if loopback {
        builder = builder.no_proxy();
    }
    builder.build().map_err(|_| AiError::Unavailable)
}
fn authenticated(
    builder: reqwest::RequestBuilder,
    profile: &Profile,
    key: Option<&Secret>,
) -> Result<reqwest::RequestBuilder, AiError> {
    if profile.key_required() && key.is_none_or(|k| k.is_empty()) {
        return Err(AiError::MissingKey);
    }
    let protocol = Protocol::parse(&profile.protocol)?;
    let mut builder = builder;
    if protocol == Protocol::Anthropic {
        builder = builder.header("anthropic-version", "2023-06-01");
    }
    if let Some(key) = key.filter(|k| !k.is_empty()) {
        builder = if protocol == Protocol::Anthropic {
            builder.header("x-api-key", key.expose())
        } else {
            builder.bearer_auth(key.expose())
        };
    }
    Ok(builder)
}
fn status_error(status: reqwest::StatusCode) -> AiError {
    match status.as_u16() {
        401 | 403 => AiError::Authentication,
        404 => AiError::NotFound,
        429 => AiError::RateLimited,
        400 | 405 | 415 | 422 => AiError::Unsupported,
        300..=399 => AiError::InvalidProfile,
        _ => AiError::Unavailable,
    }
}

pub async fn execute(
    profile: &Profile,
    key: Option<&Secret>,
    request: &Request,
    stream: bool,
    cancellation: &Cancellation,
    events: Option<mpsc::Sender<Event>>,
) -> Result<Output, AiError> {
    if cancellation.is_canceled() {
        return Err(AiError::Canceled);
    }
    tokio::select! {
        _=cancellation.canceled()=>Err(AiError::Canceled),
        result=timeout(Duration::from_secs(profile.limits.timeout_secs),execute_inner(profile,key,request,stream,events))=>result.map_err(|_|AiError::Timeout)?,
    }
}
async fn execute_inner(
    profile: &Profile,
    key: Option<&Secret>,
    request: &Request,
    stream: bool,
    events: Option<mpsc::Sender<Event>>,
) -> Result<Output, AiError> {
    let body = protocol::request_body(profile, request, stream)?;
    let client = client(profile)?;
    let mut response = authenticated(
        client.post(profile.request_url()?).json(&body),
        profile,
        key,
    )?
    .send()
    .await
    .map_err(|e| {
        if e.is_timeout() {
            AiError::Timeout
        } else {
            AiError::Unavailable
        }
    })?;
    if !response.status().is_success() {
        return Err(status_error(response.status()));
    }
    let streaming = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|h| h.to_str().ok())
        .is_some_and(|s| {
            s.split(';')
                .next()
                .is_some_and(|v| v.trim().eq_ignore_ascii_case("text/event-stream"))
        });
    let mut decoder = protocol::Decoder::new(profile)?;
    let mut sse = crate::sse::Decoder::new(profile.limits.tool_bytes.max(1024));
    let mut complete_body = Vec::new();
    let mut output = Output {
        streaming,
        ..Default::default()
    };
    let mut received = 0usize;
    while let Some(chunk) = timeout(
        Duration::from_secs(profile.limits.idle_secs),
        response.chunk(),
    )
    .await
    .map_err(|_| AiError::Timeout)?
    .map_err(|_| AiError::Protocol)?
    {
        received = received.checked_add(chunk.len()).ok_or(AiError::Limit)?;
        if received > profile.limits.output_bytes.saturating_mul(8) {
            return Err(AiError::Limit);
        }
        if streaming {
            for frame in sse.push(&chunk)? {
                for event in decoder.push(&frame)? {
                    consume(event, &mut output, profile, &events).await?;
                }
            }
        } else {
            complete_body.extend_from_slice(&chunk);
        }
    }
    if streaming {
        sse.finish()?;
    } else {
        let value = serde_json::from_slice(&complete_body).map_err(|_| AiError::Protocol)?;
        for event in decoder.completed_response(value)? {
            consume(event, &mut output, profile, &events).await?;
        }
    }
    if !decoder.is_complete() {
        return Err(AiError::Protocol);
    }
    output.wire_output = decoder.output;
    Ok(output)
}
async fn consume(
    event: Event,
    output: &mut Output,
    profile: &Profile,
    sender: &Option<mpsc::Sender<Event>>,
) -> Result<(), AiError> {
    match &event {
        Event::Text(text) => output.text.push_str(text),
        Event::ToolCall {
            id,
            name,
            arguments,
        } => {
            if output.calls.len() >= profile.limits.max_tools {
                return Err(AiError::Limit);
            }
            output.calls.push(ToolCall {
                id: id.clone(),
                name: name.clone(),
                arguments: arguments.clone(),
            });
        }
        Event::Usage(usage) => {
            if usage.input_tokens.is_some() {
                output.usage.input_tokens = usage.input_tokens;
            }
            if usage.output_tokens.is_some() {
                output.usage.output_tokens = usage.output_tokens;
            }
        }
        Event::Complete => {}
    }
    if let Some(sender) = sender {
        sender.send(event).await.map_err(|_| AiError::Canceled)?;
    }
    Ok(())
}

pub async fn test_connection(
    profile: &Profile,
    key: Option<&Secret>,
    cancellation: &Cancellation,
) -> Result<Capabilities, AiError> {
    let mut request = Request {
        messages: vec![crate::Message {
            role: "user".into(),
            content: "Reply with OK.".into(),
        }],
        ..Default::default()
    };
    let text = match execute(profile, key, &request, true, cancellation, None).await {
        Err(AiError::Unsupported) => {
            execute(profile, key, &request, false, cancellation, None).await?
        }
        value => value?,
    };
    if text.text.is_empty() {
        return Err(AiError::Protocol);
    }
    request.messages[0].content =
        "Call connection_probe with the argument ok=true. This is only a connection test.".into();
    request.tools = vec![ToolDefinition {
        name: "connection_probe".into(),
        description: "Harmless synthetic connection check; does not read or write any file.".into(),
        schema: json!({"type":"object","properties":{"ok":{"type":"boolean"}},"required":["ok"],"additionalProperties":false}),
    }];
    let tools = match execute(profile, key, &request, text.streaming, cancellation, None).await {
        Ok(output) => output
            .calls
            .iter()
            .any(|c| c.name == "connection_probe" && c.arguments == json!({"ok":true})),
        Err(AiError::Unsupported | AiError::Protocol) => false,
        Err(error) => return Err(error),
    };
    Ok(Capabilities {
        text: true,
        streaming: text.streaming,
        tools,
    })
}

pub async fn discover_models(
    profile: &Profile,
    key: Option<&Secret>,
    cancellation: &Cancellation,
) -> Result<Vec<String>, AiError> {
    if cancellation.is_canceled() {
        return Err(AiError::Canceled);
    }
    let url = profile
        .base_url()?
        .join("models")
        .map_err(|_| AiError::InvalidProfile)?;
    let work = async {
        let mut response = authenticated(client(profile)?.get(url), profile, key)?
            .send()
            .await
            .map_err(|_| AiError::Unavailable)?;
        if !response.status().is_success() {
            return Err(status_error(response.status()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = timeout(
            Duration::from_secs(profile.limits.idle_secs),
            response.chunk(),
        )
        .await
        .map_err(|_| AiError::Timeout)?
        .map_err(|_| AiError::Protocol)?
        {
            if bytes.len() + chunk.len() > 1024 * 1024 {
                return Err(AiError::Limit);
            }
            bytes.extend_from_slice(&chunk);
        }
        let value: Value = serde_json::from_slice(&bytes).map_err(|_| AiError::Protocol)?;
        let mut models: Vec<String> = value["data"]
            .as_array()
            .ok_or(AiError::Unsupported)?
            .iter()
            .take(500)
            .filter_map(|v| {
                v["id"]
                    .as_str()
                    .filter(|v| v.len() <= 256)
                    .map(str::to_owned)
            })
            .collect();
        models.sort();
        models.dedup();
        Ok(models)
    };
    tokio::select! {_=cancellation.canceled()=>Err(AiError::Canceled),result=timeout(Duration::from_secs(profile.limits.timeout_secs),work)=>result.map_err(|_|AiError::Timeout)?}
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use tokio::{
        io::{AsyncReadExt, AsyncWriteExt},
        net::TcpListener,
    };
    async fn mock(
        status: &str,
        content_type: &str,
        body: &str,
    ) -> (Profile, tokio::task::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let reply = format!(
            "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let handle = tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut received = Vec::new();
            let mut buffer = [0u8; 8192];
            loop {
                let n = socket.read(&mut buffer).await.unwrap();
                if n == 0 {
                    break;
                }
                received.extend_from_slice(&buffer[..n]);
                if let Some(split) = received.windows(4).position(|v| v == b"\r\n\r\n") {
                    let header = String::from_utf8_lossy(&received[..split]);
                    let length = header
                        .lines()
                        .find_map(|l| {
                            l.to_ascii_lowercase()
                                .strip_prefix("content-length:")
                                .and_then(|v| v.trim().parse::<usize>().ok())
                        })
                        .unwrap_or(0);
                    if received.len() >= split + 4 + length {
                        break;
                    }
                }
            }
            for chunk in reply.as_bytes().chunks(11) {
                socket.write_all(chunk).await.unwrap();
            }
            String::from_utf8(received).unwrap()
        });
        let mut profile = Profile::preset("local", "fixture");
        profile.endpoint = format!("http://{address}/v1");
        profile.model = "fixture".into();
        (profile, handle)
    }
    fn request() -> Request {
        Request {
            messages: vec![crate::Message {
                role: "user".into(),
                content: "Hi".into(),
            }],
            ..Default::default()
        }
    }
    #[tokio::test]
    async fn actual_fragmented_http_stream_is_incremental_and_secret_stays_in_header() {
        let body = "data: {\"choices\":[{\"delta\":{\"content\":\"中文\"},\"finish_reason\":null}]}\n\ndata: {\"choices\":[{\"delta\":{},\"finish_reason\":\"stop\"}]}\n\ndata: [DONE]\n\n";
        let (p, handle) = mock("200 OK", "text/event-stream", body).await;
        let (tx, mut rx) = mpsc::channel(8);
        let key = Secret::new("fixture-private-key".into());
        let result = execute(
            &p,
            Some(&key),
            &request(),
            true,
            &Cancellation::default(),
            Some(tx),
        )
        .await
        .unwrap();
        assert_eq!(result.text, "中文");
        assert!(matches!(rx.recv().await, Some(Event::Text(_))));
        let received = handle.await.unwrap();
        let (header, json) = received.split_once("\r\n\r\n").unwrap();
        assert!(header.contains("fixture-private-key"));
        assert!(!json.contains("fixture-private-key"));
    }
    #[tokio::test]
    async fn completed_response_and_error_codes() {
        let(p,handle)=mock("200 OK","application/json",r#"{"choices":[{"message":{"role":"assistant","content":"OK"},"finish_reason":"stop"}]}"#).await;
        assert_eq!(
            execute(&p, None, &request(), true, &Cancellation::default(), None)
                .await
                .unwrap()
                .text,
            "OK"
        );
        handle.await.unwrap();
        for (status, expected) in [
            ("401 Unauthorized", AiError::Authentication),
            ("404 Not Found", AiError::NotFound),
            ("429 Too Many Requests", AiError::RateLimited),
            ("302 Found", AiError::InvalidProfile),
        ] {
            let (p, handle) = mock(status, "text/plain", "do not expose this body").await;
            assert_eq!(
                execute(&p, None, &request(), true, &Cancellation::default(), None)
                    .await
                    .err(),
                Some(expected)
            );
            handle.await.unwrap();
        }
    }
    #[tokio::test]
    async fn cancellation_stops_stalled_body_and_idle_timeout() {
        async fn stalled() -> Profile {
            let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address = listener.local_addr().unwrap();
            tokio::spawn(async move {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut b = [0u8; 8192];
                socket.read(&mut b).await.unwrap();
                socket.write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked\r\n\r\n").await.unwrap();
                tokio::time::sleep(Duration::from_secs(3)).await;
            });
            let mut p = Profile::preset("local", "stall");
            p.endpoint = format!("http://{address}/v1");
            p.model = "fixture".into();
            p.limits.idle_secs = 1;
            p
        }
        let p = stalled().await;
        let cancel = Cancellation::default();
        let trigger = cancel.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(50)).await;
            trigger.cancel();
        });
        assert_eq!(
            execute(&p, None, &request(), true, &cancel, None)
                .await
                .err(),
            Some(AiError::Canceled)
        );
        assert_eq!(
            execute(
                &stalled().await,
                None,
                &request(),
                true,
                &Cancellation::default(),
                None
            )
            .await
            .err(),
            Some(AiError::Timeout)
        );
    }
    #[tokio::test]
    async fn incomplete_and_oversized_responses_are_rejected() {
        let (p, handle) = mock(
            "200 OK",
            "text/event-stream",
            "data: {\"choices\":[{\"delta\":{\"content\":\"partial\"}}]}\n\n",
        )
        .await;
        assert_eq!(
            execute(&p, None, &request(), true, &Cancellation::default(), None)
                .await
                .err(),
            Some(AiError::Protocol)
        );
        handle.await.unwrap();
        let body=json!({"choices":[{"message":{"role":"assistant","content":"x".repeat(2000)},"finish_reason":"stop"}]}).to_string();
        let (mut p, handle) = mock("200 OK", "application/json", &body).await;
        p.limits.output_bytes = 1024;
        assert_eq!(
            execute(&p, None, &request(), true, &Cancellation::default(), None)
                .await
                .err(),
            Some(AiError::Limit)
        );
        handle.await.unwrap();
    }
    #[test]
    fn canceled_before_work_does_not_depend_on_a_waiter() {
        let cancel = Cancellation::default();
        cancel.cancel();
        assert!(cancel.is_canceled());
    }
    pub(crate) async fn series(
        replies: Vec<(String, String)>,
    ) -> (Profile, tokio::task::JoinHandle<Vec<String>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let handle = tokio::spawn(async move {
            let mut requests = Vec::new();
            for (status, body) in replies {
                let (mut socket, _) = listener.accept().await.unwrap();
                let mut received = Vec::new();
                let mut b = [0u8; 8192];
                loop {
                    let n = socket.read(&mut b).await.unwrap();
                    if n == 0 {
                        break;
                    }
                    received.extend_from_slice(&b[..n]);
                    if let Some(split) = received.windows(4).position(|v| v == b"\r\n\r\n") {
                        let len = String::from_utf8_lossy(&received[..split])
                            .lines()
                            .find_map(|l| {
                                l.to_lowercase()
                                    .strip_prefix("content-length:")
                                    .and_then(|v| v.trim().parse::<usize>().ok())
                            })
                            .unwrap_or(0);
                        if received.len() >= split + 4 + len {
                            break;
                        }
                    }
                }
                let reply = format!(
                    "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                socket.write_all(reply.as_bytes()).await.unwrap();
                requests.push(String::from_utf8(received).unwrap());
            }
            requests
        });
        let mut p = Profile::preset("local", "fixture");
        p.endpoint = format!("http://{address}/v1");
        p.model = "fixture".into();
        (p, handle)
    }
    pub(crate) fn tool_response(name: &str, args: &str) -> String {
        json!({"choices":[{"message":{"role":"assistant","content":null,"tool_calls":[{"id":"call1","type":"function","function":{"name":name,"arguments":args}}]},"finish_reason":"tool_calls"}]}).to_string()
    }
    pub(crate) fn text_response(text: &str) -> String {
        json!({"choices":[{"message":{"role":"assistant","content":text},"finish_reason":"stop"}]})
            .to_string()
    }
    #[tokio::test]
    async fn synthetic_probes_discovery_and_text_only_fallback() {
        for tools in [true, false] {
            let probe = if tools {
                (
                    "200 OK".into(),
                    tool_response("connection_probe", r#"{"ok":true}"#),
                )
            } else {
                ("400 Bad Request".into(), "{}".into())
            };
            let (p, handle) = series(vec![("200 OK".into(), text_response("OK")), probe]).await;
            let caps = test_connection(&p, None, &Cancellation::default())
                .await
                .unwrap();
            assert!(caps.text);
            assert_eq!(caps.tools, tools);
            assert!(!caps.streaming);
            let reqs = handle.await.unwrap();
            assert!(reqs[0].contains("Reply with OK."));
            assert!(reqs[1].contains("connection_probe"));
            assert!(!reqs.iter().any(|r| r.contains("Attached material")));
        }
        let (mut p, handle) = mock(
            "200 OK",
            "application/json",
            r#"{"data":[{"id":"b"},{"id":"a"},{"id":"a"}]}"#,
        )
        .await;
        p.model.clear();
        assert_eq!(
            discover_models(&p, None, &Cancellation::default())
                .await
                .unwrap(),
            vec!["a", "b"]
        );
        assert!(handle.await.unwrap().starts_with("GET /v1/models"));
        for (status, error) in [
            ("404 Not Found", AiError::NotFound),
            ("401 Unauthorized", AiError::Authentication),
            ("429 Too Many Requests", AiError::RateLimited),
            ("503 Unavailable", AiError::Unavailable),
        ] {
            let (p, h) = mock(status, "application/json", "{}").await;
            assert_eq!(
                test_connection(&p, None, &Cancellation::default())
                    .await
                    .err(),
                Some(error)
            );
            h.await.unwrap();
        }
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);
        let mut p = Profile::preset("local", "gone");
        p.model = "fixture".into();
        p.endpoint = format!("http://{addr}/v1");
        assert_eq!(
            discover_models(&p, None, &Cancellation::default())
                .await
                .err(),
            Some(AiError::Unavailable)
        );
    }
}
