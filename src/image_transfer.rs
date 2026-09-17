//! Bounded image materialization and uploader protocol adapters.

use std::{
    ffi::OsString,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::Duration,
};

use serde::Deserialize;
use tokio::io::AsyncWriteExt as _;

use crate::{
    CustomImageCommandPreferences, ImageInput, ImagePreferences, ImageUploaderKind,
    external_command::{
        CommandCancellation, ExternalCommandLimits, ExternalCommandRequest,
        ExternalCommandTermination, run_external_command,
    },
    model::PicGoCorePreferences,
};

pub const IMAGE_INPUT_MAX_BYTES: usize = 32 * 1024 * 1024;
pub const IMAGE_UPLOAD_RESULT_MAX_BYTES: usize = 1024 * 1024;
pub const IMAGE_MATERIALIZATION_CONCURRENCY: usize = 2;
const IMAGE_DOWNLOAD_REDIRECT_LIMIT: usize = 5;
const IMAGE_CONNECT_TIMEOUT: Duration = Duration::from_secs(15);
static NEXT_STAGED_IMAGE: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ImageTransferLimits {
    pub max_input_bytes: usize,
    pub timeout: Duration,
    pub max_result_bytes: usize,
}

impl Default for ImageTransferLimits {
    fn default() -> Self {
        Self {
            max_input_bytes: IMAGE_INPUT_MAX_BYTES,
            timeout: Duration::from_secs(60),
            max_result_bytes: IMAGE_UPLOAD_RESULT_MAX_BYTES,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaterializedImage {
    pub path: PathBuf,
    pub extension: String,
    pub byte_len: usize,
}

#[derive(Debug, thiserror::Error)]
pub enum ImageTransferError {
    #[error("image operation was canceled")]
    Canceled,
    #[error("image input exceeds the {limit} byte limit")]
    TooLarge { limit: usize },
    #[error("image input is empty")]
    Empty,
    #[error("unsupported or malformed image content")]
    UnsupportedContent,
    #[error("source image changed while it was being captured")]
    SourceChanged,
    #[error("invalid image data URI")]
    InvalidDataUri,
    #[error("HTTP request failed: {0}")]
    Http(String),
    #[error("image server returned HTTP {0}")]
    HttpStatus(u16),
    #[error("filesystem operation failed: {0}")]
    Io(#[from] io::Error),
    #[error("invalid uploader configuration: {0}")]
    InvalidConfiguration(String),
    #[error("uploader failed: {0}")]
    Provider(String),
    #[error("uploader result was malformed: {0}")]
    InvalidResult(String),
}

pub async fn materialize_inputs_bounded(
    inputs: Vec<ImageInput>,
    staging_directory: PathBuf,
    limits: ImageTransferLimits,
    cancellation: CommandCancellation,
) -> Vec<Result<MaterializedImage, ImageTransferError>> {
    let semaphore = std::sync::Arc::new(tokio::sync::Semaphore::new(
        IMAGE_MATERIALIZATION_CONCURRENCY,
    ));
    let mut pending = tokio::task::JoinSet::new();
    for (index, input) in inputs.into_iter().enumerate() {
        let semaphore = semaphore.clone();
        let staging_directory = staging_directory.clone();
        let cancellation = cancellation.clone();
        pending.spawn(async move {
            let permit = semaphore.acquire_owned().await;
            let result = match permit {
                Ok(_permit) => {
                    materialize_input(input, &staging_directory, limits, &cancellation).await
                }
                Err(_) => Err(ImageTransferError::Canceled),
            };
            (index, result)
        });
    }
    let mut results: Vec<Option<Result<MaterializedImage, ImageTransferError>>> =
        (0..pending.len()).map(|_| None).collect();
    while let Some(joined) = pending.join_next().await {
        if let Ok((index, result)) = joined {
            results[index] = Some(result);
        }
    }
    results
        .into_iter()
        .map(|result| result.unwrap_or(Err(ImageTransferError::Canceled)))
        .collect()
}

pub async fn materialize_input(
    input: ImageInput,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<MaterializedImage, ImageTransferError> {
    if cancellation.is_cancelled() {
        return Err(ImageTransferError::Canceled);
    }
    fs::create_dir_all(staging_directory)?;
    match input {
        ImageInput::Local(path) => {
            let staging_directory = staging_directory.to_path_buf();
            let cancellation = cancellation.clone();
            tokio::task::spawn_blocking(move || {
                materialize_local(&path, &staging_directory, limits, &cancellation)
            })
            .await
            .map_err(|error| ImageTransferError::Io(io::Error::other(error.to_string())))?
        }
        ImageInput::Bytes { stem, bytes, .. } => {
            materialize_bytes(&bytes, &stem, staging_directory, limits, cancellation)
        }
        ImageInput::DataUri(uri) => {
            if uri.len() > limits.max_input_bytes.saturating_mul(2) {
                return Err(ImageTransferError::TooLarge {
                    limit: limits.max_input_bytes,
                });
            }
            let processed =
                data_url::DataUrl::process(&uri).map_err(|_| ImageTransferError::InvalidDataUri)?;
            let (bytes, _) = processed
                .decode_to_vec()
                .map_err(|_| ImageTransferError::InvalidDataUri)?;
            materialize_bytes(
                &bytes,
                "embedded-image",
                staging_directory,
                limits,
                cancellation,
            )
        }
        ImageInput::Remote(url) => {
            materialize_remote(&url, staging_directory, limits, cancellation).await
        }
    }
}

fn materialize_local(
    source: &Path,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<MaterializedImage, ImageTransferError> {
    materialize_local_with_hook(source, staging_directory, limits, cancellation, |_| {})
}

fn materialize_local_with_hook(
    source: &Path,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
    mut after_chunk: impl FnMut(usize),
) -> Result<MaterializedImage, ImageTransferError> {
    fs::create_dir_all(staging_directory)?;
    let before = fs::metadata(source)?;
    if before.len() as usize > limits.max_input_bytes {
        return Err(ImageTransferError::TooLarge {
            limit: limits.max_input_bytes,
        });
    }
    let file = fs::File::open(source)?;
    let stem = source
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("image");
    let materialized = spool_reader_with_hook(
        file,
        stem,
        staging_directory,
        limits,
        cancellation,
        &mut after_chunk,
    )?;
    let after = fs::metadata(source)?;
    if before.len() != after.len() || before.modified().ok() != after.modified().ok() {
        let _ = fs::remove_file(&materialized.path);
        return Err(ImageTransferError::SourceChanged);
    }
    Ok(materialized)
}

fn materialize_bytes(
    bytes: &[u8],
    stem: &str,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<MaterializedImage, ImageTransferError> {
    spool_reader(bytes, stem, staging_directory, limits, cancellation)
}

fn spool_reader(
    mut reader: impl Read,
    stem: &str,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<MaterializedImage, ImageTransferError> {
    spool_reader_with_hook(
        &mut reader,
        stem,
        staging_directory,
        limits,
        cancellation,
        &mut |_| {},
    )
}

fn spool_reader_with_hook(
    mut reader: impl Read,
    stem: &str,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
    after_chunk: &mut impl FnMut(usize),
) -> Result<MaterializedImage, ImageTransferError> {
    let (part_path, id) = staging_part_path(staging_directory, stem);
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part_path)?;
    let mut header = Vec::with_capacity(4096);
    let mut total = 0usize;
    let mut buffer = [0_u8; 8192];
    loop {
        if cancellation.is_cancelled() {
            drop(output);
            let _ = fs::remove_file(&part_path);
            return Err(ImageTransferError::Canceled);
        }
        let count = reader.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        total = total.saturating_add(count);
        if total > limits.max_input_bytes {
            drop(output);
            let _ = fs::remove_file(&part_path);
            return Err(ImageTransferError::TooLarge {
                limit: limits.max_input_bytes,
            });
        }
        if header.len() < 4096 {
            let keep = (4096 - header.len()).min(count);
            header.extend_from_slice(&buffer[..keep]);
        }
        output.write_all(&buffer[..count])?;
        after_chunk(total);
    }
    if total == 0 {
        drop(output);
        let _ = fs::remove_file(&part_path);
        return Err(ImageTransferError::Empty);
    }
    let Some(extension) = detect_image_extension(&header) else {
        drop(output);
        let _ = fs::remove_file(&part_path);
        return Err(ImageTransferError::UnsupportedContent);
    };
    output.sync_all()?;
    drop(output);
    let final_path = staging_directory.join(format!("{}-{id}.{extension}", safe_stem(stem)));
    fs::rename(&part_path, &final_path)?;
    Ok(MaterializedImage {
        path: final_path,
        extension: extension.to_owned(),
        byte_len: total,
    })
}

async fn materialize_remote(
    url: &str,
    staging_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<MaterializedImage, ImageTransferError> {
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .connect_timeout(IMAGE_CONNECT_TIMEOUT)
        .timeout(limits.timeout)
        .redirect_policy(reqwest::redirect::Policy::limited(
            IMAGE_DOWNLOAD_REDIRECT_LIMIT,
        ))
        .user_agent(format!("Markion/{}", env!("CARGO_PKG_VERSION")))
        .build()
        .map_err(|error| ImageTransferError::Http(error.to_string()))?;
    let mut response = client
        .get(url)
        .send()
        .await
        .map_err(|error| ImageTransferError::Http(error.to_string()))?;
    if !response.status().is_success() {
        return Err(ImageTransferError::HttpStatus(response.status().as_u16()));
    }
    if response
        .content_length()
        .is_some_and(|length| length as usize > limits.max_input_bytes)
    {
        return Err(ImageTransferError::TooLarge {
            limit: limits.max_input_bytes,
        });
    }
    let (part_path, id) = staging_part_path(staging_directory, "downloaded-image");
    let mut output = tokio::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&part_path)
        .await?;
    let mut header = Vec::with_capacity(4096);
    let mut total = 0usize;
    loop {
        let chunk = tokio::select! {
            result = response.chunk() => result
                .map_err(|error| ImageTransferError::Http(error.to_string()))?,
            _ = tokio::time::sleep(Duration::from_millis(20)) => {
                if cancellation.is_cancelled() {
                    drop(output);
                    let _ = tokio::fs::remove_file(&part_path).await;
                    return Err(ImageTransferError::Canceled);
                }
                continue;
            }
        };
        let Some(chunk) = chunk else { break };
        if cancellation.is_cancelled() {
            drop(output);
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(ImageTransferError::Canceled);
        }
        total = total.saturating_add(chunk.len());
        if total > limits.max_input_bytes {
            drop(output);
            let _ = tokio::fs::remove_file(&part_path).await;
            return Err(ImageTransferError::TooLarge {
                limit: limits.max_input_bytes,
            });
        }
        if header.len() < 4096 {
            let keep = (4096 - header.len()).min(chunk.len());
            header.extend_from_slice(&chunk[..keep]);
        }
        output.write_all(&chunk).await?;
    }
    if total == 0 {
        drop(output);
        let _ = tokio::fs::remove_file(&part_path).await;
        return Err(ImageTransferError::Empty);
    }
    let Some(extension) = detect_image_extension(&header) else {
        drop(output);
        let _ = tokio::fs::remove_file(&part_path).await;
        return Err(ImageTransferError::UnsupportedContent);
    };
    output.sync_all().await?;
    drop(output);
    let final_path = staging_directory.join(format!("downloaded-image-{id}.{extension}"));
    tokio::fs::rename(&part_path, &final_path).await?;
    Ok(MaterializedImage {
        path: final_path,
        extension: extension.to_owned(),
        byte_len: total,
    })
}

fn staging_part_path(directory: &Path, stem: &str) -> (PathBuf, u64) {
    let id = NEXT_STAGED_IMAGE.fetch_add(1, Ordering::Relaxed);
    (
        directory.join(format!(
            ".{}-{}-{id}.part",
            safe_stem(stem),
            std::process::id()
        )),
        id,
    )
}

fn safe_stem(stem: &str) -> String {
    let stem: String = stem
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
                ch.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect();
    let stem = stem.trim_matches('-');
    if stem.is_empty() {
        "image".to_owned()
    } else {
        stem.to_owned()
    }
}

fn detect_image_extension(bytes: &[u8]) -> Option<&'static str> {
    if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some("png")
    } else if bytes.starts_with(b"\xff\xd8\xff") {
        Some("jpg")
    } else if bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a") {
        Some("gif")
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some("webp")
    } else if bytes.starts_with(b"BM") {
        Some("bmp")
    } else if bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*") {
        Some("tiff")
    } else {
        let text = String::from_utf8_lossy(bytes);
        let trimmed = text.trim_start_matches(|ch: char| ch.is_whitespace() || ch == '\u{feff}');
        (trimmed.starts_with("<svg") || (trimmed.starts_with("<?xml") && trimmed.contains("<svg")))
            .then_some("svg")
    }
}

pub fn validate_image_result_url(value: &str) -> Result<String, ImageTransferError> {
    let value = value.trim();
    if value.chars().any(char::is_control) {
        return Err(ImageTransferError::InvalidResult(
            "URL contains control characters".into(),
        ));
    }
    let url = reqwest::Url::parse(value)
        .map_err(|_| ImageTransferError::InvalidResult("expected one absolute URL".into()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.host_str().is_none()
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(ImageTransferError::InvalidResult(
            "expected an HTTP(S) URL without user information".into(),
        ));
    }
    Ok(value.to_owned())
}

pub fn redact_image_url(value: &str) -> String {
    let Ok(mut url) = reqwest::Url::parse(value) else {
        return "<invalid-url>".to_owned();
    };
    url.set_query(None);
    url.set_fragment(None);
    url.to_string()
}

#[derive(Deserialize)]
struct PicGoHttpResponse {
    success: bool,
    result: Vec<String>,
}

pub async fn upload_picgo_http(
    endpoint: &str,
    file: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<String, ImageTransferError> {
    let endpoint_url = reqwest::Url::parse(endpoint).map_err(|_| {
        ImageTransferError::InvalidConfiguration("PicGo endpoint is not a valid URL".into())
    })?;
    let loopback = endpoint_url.host_str().is_some_and(|host| {
        host.eq_ignore_ascii_case("localhost")
            || host
                .parse::<std::net::IpAddr>()
                .is_ok_and(|address| address.is_loopback())
    });
    if !matches!(endpoint_url.scheme(), "http" | "https")
        || !loopback
        || !endpoint_url.username().is_empty()
        || endpoint_url.password().is_some()
    {
        return Err(ImageTransferError::InvalidConfiguration(
            "PicGo HTTP endpoint must be a loopback HTTP(S) URL".into(),
        ));
    }
    if cancellation.is_cancelled() {
        return Err(ImageTransferError::Canceled);
    }
    let absolute_file = fs::canonicalize(file)?;
    let body = serde_json::to_vec(&serde_json::json!({
        "list": [absolute_file.to_string_lossy()]
    }))
    .map_err(|error| ImageTransferError::InvalidConfiguration(error.to_string()))?;
    let client = reqwest::Client::builder()
        .use_rustls_tls()
        .no_proxy()
        .connect_timeout(IMAGE_CONNECT_TIMEOUT)
        .timeout(limits.timeout)
        .redirect_policy(reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| ImageTransferError::Http(error.to_string()))?;
    let response = client
        .post(endpoint_url)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await
        .map_err(|error| ImageTransferError::Http(error.to_string()))?;
    if !response.status().is_success() {
        return Err(ImageTransferError::HttpStatus(response.status().as_u16()));
    }
    let bytes = read_bounded_response(response, limits.max_result_bytes, cancellation).await?;
    let response: PicGoHttpResponse = serde_json::from_slice(&bytes)
        .map_err(|_| ImageTransferError::InvalidResult("invalid PicGo JSON response".into()))?;
    if !response.success || response.result.len() != 1 {
        return Err(ImageTransferError::InvalidResult(
            "PicGo must report success with exactly one URL".into(),
        ));
    }
    validate_image_result_url(&response.result[0])
}

async fn read_bounded_response(
    mut response: reqwest::Response,
    limit: usize,
    cancellation: &CommandCancellation,
) -> Result<Vec<u8>, ImageTransferError> {
    let mut bytes = Vec::new();
    loop {
        let chunk = tokio::select! {
            result = response.chunk() => result
                .map_err(|error| ImageTransferError::Http(error.to_string()))?,
            _ = tokio::time::sleep(Duration::from_millis(20)) => {
                if cancellation.is_cancelled() {
                    return Err(ImageTransferError::Canceled);
                }
                continue;
            }
        };
        let Some(chunk) = chunk else { break };
        if cancellation.is_cancelled() {
            return Err(ImageTransferError::Canceled);
        }
        if bytes.len().saturating_add(chunk.len()) > limit {
            return Err(ImageTransferError::TooLarge { limit });
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

pub fn upload_with_custom_command(
    preferences: &CustomImageCommandPreferences,
    file: &Path,
    working_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<String, ImageTransferError> {
    let request = custom_command_request(preferences, file, working_directory)?;
    let output = run_external_command(
        &request,
        ExternalCommandLimits {
            timeout: limits.timeout,
            max_stdout_bytes: limits.max_result_bytes,
            max_stderr_bytes: limits.max_result_bytes,
        },
        cancellation,
    )
    .map_err(|error| ImageTransferError::Provider(error.to_string()))?;
    if output.termination == ExternalCommandTermination::Cancelled {
        return Err(ImageTransferError::Canceled);
    }
    if !output.success() {
        return Err(ImageTransferError::Provider(
            "custom uploader exited unsuccessfully; see stderr diagnostics".into(),
        ));
    }
    if output.stdout_truncated || output.stderr_truncated {
        return Err(ImageTransferError::InvalidResult(
            "custom uploader output exceeded the configured limit".into(),
        ));
    }
    parse_single_stdout_url(&output.stdout)
}

fn custom_command_request(
    preferences: &CustomImageCommandPreferences,
    file: &Path,
    working_directory: &Path,
) -> Result<ExternalCommandRequest, ImageTransferError> {
    let executable = preferences
        .executable
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ImageTransferError::InvalidConfiguration("custom uploader program is required".into())
        })?;
    let placeholders = preferences
        .args
        .iter()
        .filter(|argument| argument.as_str() == "{file}")
        .count();
    if placeholders != 1
        || preferences
            .args
            .iter()
            .any(|argument| argument.contains("{file}") && argument != "{file}")
    {
        return Err(ImageTransferError::InvalidConfiguration(
            "custom uploader arguments must contain {file} exactly once as a complete argument"
                .into(),
        ));
    }
    let arguments = preferences
        .args
        .iter()
        .map(|argument| {
            if argument == "{file}" {
                file.as_os_str().to_owned()
            } else {
                OsString::from(argument)
            }
        })
        .collect();
    Ok(ExternalCommandRequest {
        program: PathBuf::from(executable),
        arguments,
        working_directory: working_directory.to_path_buf(),
    })
}

fn parse_single_stdout_url(stdout: &[u8]) -> Result<String, ImageTransferError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| ImageTransferError::InvalidResult("stdout is not UTF-8".into()))?;
    let lines: Vec<_> = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .collect();
    if lines.len() != 1 || lines[0].trim() != text.trim() {
        return Err(ImageTransferError::InvalidResult(
            "stdout must contain exactly one non-empty URL line".into(),
        ));
    }
    validate_image_result_url(lines[0])
}

pub fn upload_with_picgo_core(
    preferences: &PicGoCorePreferences,
    file: &Path,
    working_directory: &Path,
    limits: ImageTransferLimits,
    cancellation: &CommandCancellation,
) -> Result<String, ImageTransferError> {
    let request = picgo_core_request(preferences, file, working_directory)?;
    let output = run_external_command(
        &request,
        ExternalCommandLimits {
            timeout: limits.timeout,
            max_stdout_bytes: limits.max_result_bytes,
            max_stderr_bytes: limits.max_result_bytes,
        },
        cancellation,
    )
    .map_err(|error| ImageTransferError::Provider(error.to_string()))?;
    if output.termination == ExternalCommandTermination::Cancelled {
        return Err(ImageTransferError::Canceled);
    }
    if !output.success() || output.stdout_truncated || output.stderr_truncated {
        return Err(ImageTransferError::Provider(
            "PicGo Core did not complete successfully; see stderr diagnostics".into(),
        ));
    }
    parse_picgo_core_output(&output.stdout)
}

fn picgo_core_request(
    preferences: &PicGoCorePreferences,
    file: &Path,
    working_directory: &Path,
) -> Result<ExternalCommandRequest, ImageTransferError> {
    let executable = preferences
        .executable
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            ImageTransferError::InvalidConfiguration("PicGo Core program is required".into())
        })?;
    let mut arguments: Vec<OsString> = preferences
        .launcher_args
        .iter()
        .map(OsString::from)
        .collect();
    arguments.push("upload".into());
    if let Some(config) = preferences
        .config_path
        .as_deref()
        .filter(|value| !value.trim().is_empty())
    {
        arguments.push("--config".into());
        arguments.push(config.into());
    }
    arguments.push(file.as_os_str().to_owned());
    Ok(ExternalCommandRequest {
        program: executable.into(),
        arguments,
        working_directory: working_directory.to_path_buf(),
    })
}

pub fn parse_picgo_core_output(stdout: &[u8]) -> Result<String, ImageTransferError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| ImageTransferError::InvalidResult("PicGo output is not UTF-8".into()))?;
    let stripped = strip_ansi(text);
    let mut after_marker = false;
    let mut urls = Vec::new();
    for line in stripped.lines() {
        let trimmed = line.trim();
        if let Some((_, suffix)) = trimmed.split_once("PicGo SUCCESS:") {
            after_marker = true;
            let suffix = suffix.trim().trim_start_matches(']').trim();
            if suffix.starts_with("http://") || suffix.starts_with("https://") {
                urls.push(suffix.to_owned());
            }
            continue;
        }
        if after_marker && (trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
            urls.push(trimmed.to_owned());
        }
    }
    if !after_marker || urls.len() != 1 {
        return Err(ImageTransferError::InvalidResult(
            "PicGo output must contain one URL after the success marker".into(),
        ));
    }
    validate_image_result_url(&urls[0])
}

pub async fn upload_materialized_image(
    preferences: &ImagePreferences,
    image: &MaterializedImage,
    working_directory: &Path,
    cancellation: &CommandCancellation,
) -> Result<String, ImageTransferError> {
    let limits = ImageTransferLimits {
        timeout: Duration::from_secs(preferences.timeout_secs),
        ..ImageTransferLimits::default()
    };
    match preferences.uploader {
        ImageUploaderKind::None => Err(ImageTransferError::InvalidConfiguration(
            "no image uploader is selected".into(),
        )),
        ImageUploaderKind::PicGoHttp => {
            upload_picgo_http(
                &preferences.picgo_http.endpoint,
                &image.path,
                limits,
                cancellation,
            )
            .await
        }
        ImageUploaderKind::PicGoCore => {
            let preferences = preferences.picgo_core.clone();
            let path = image.path.clone();
            let working_directory = working_directory.to_path_buf();
            let cancellation = cancellation.clone();
            tokio::task::spawn_blocking(move || {
                upload_with_picgo_core(
                    &preferences,
                    &path,
                    &working_directory,
                    limits,
                    &cancellation,
                )
            })
            .await
            .map_err(|error| ImageTransferError::Provider(error.to_string()))?
        }
        ImageUploaderKind::Command => {
            let preferences = preferences.command.clone();
            let path = image.path.clone();
            let working_directory = working_directory.to_path_buf();
            let cancellation = cancellation.clone();
            tokio::task::spawn_blocking(move || {
                upload_with_custom_command(
                    &preferences,
                    &path,
                    &working_directory,
                    limits,
                    &cancellation,
                )
            })
            .await
            .map_err(|error| ImageTransferError::Provider(error.to_string()))?
        }
    }
}

fn strip_ansi(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut output = String::with_capacity(value.len());
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index] == 0x1b && bytes.get(index + 1) == Some(&b'[') {
            index += 2;
            while index < bytes.len() {
                let byte = bytes[index];
                index += 1;
                if (0x40..=0x7e).contains(&byte) {
                    break;
                }
            }
        } else {
            let ch = value[index..].chars().next().expect("UTF-8 boundary");
            output.push(ch);
            index += ch.len_utf8();
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        net::TcpListener,
        sync::{
            Arc,
            atomic::{AtomicUsize, Ordering as AtomicOrdering},
        },
        thread,
    };

    const TINY_PNG: &[u8] = b"\x89PNG\r\n\x1a\nfixture";

    #[test]
    fn content_detection_overrides_misleading_extension_and_bounds_input() {
        let dir = tempfile::tempdir().unwrap();
        let source = dir.path().join("misleading.jpg");
        fs::write(&source, TINY_PNG).unwrap();
        let staging = dir.path().join("stage");
        let image = materialize_local(
            &source,
            &staging,
            ImageTransferLimits::default(),
            &CommandCancellation::default(),
        )
        .unwrap();
        assert_eq!(image.extension, "png");
        assert_eq!(fs::read(&source).unwrap(), TINY_PNG);

        let error = materialize_bytes(
            TINY_PNG,
            "image",
            &staging,
            ImageTransferLimits {
                max_input_bytes: 4,
                ..ImageTransferLimits::default()
            },
            &CommandCancellation::default(),
        )
        .unwrap_err();
        assert!(matches!(error, ImageTransferError::TooLarge { .. }));

        let changing = dir.path().join("changing.png");
        fs::write(&changing, [TINY_PNG, b"before"].concat()).unwrap();
        let mut changed = false;
        let error = materialize_local_with_hook(
            &changing,
            &staging,
            ImageTransferLimits::default(),
            &CommandCancellation::default(),
            |_| {
                if !changed {
                    fs::write(&changing, [TINY_PNG, b"after-and-longer"].concat()).unwrap();
                    changed = true;
                }
            },
        )
        .unwrap_err();
        assert!(matches!(error, ImageTransferError::SourceChanged));
    }

    #[test]
    fn custom_stdout_and_url_validation_are_strict_and_preserve_signed_queries() {
        assert_eq!(
            parse_single_stdout_url(b"https://cdn.test/a.png?token=secret&x=1\n").unwrap(),
            "https://cdn.test/a.png?token=secret&x=1"
        );
        for output in [
            b"".as_slice(),
            b"log\nhttps://cdn.test/a.png\n".as_slice(),
            b"![x](https://cdn.test/a.png)".as_slice(),
            b"file:///tmp/a.png".as_slice(),
        ] {
            assert!(parse_single_stdout_url(output).is_err());
        }
        assert_eq!(
            redact_image_url("https://cdn.test/a.png?token=secret#part"),
            "https://cdn.test/a.png"
        );

        let file = Path::new("C:/stage/图 片.png");
        let working = Path::new("C:/private-operation");
        let valid = CustomImageCommandPreferences {
            executable: Some("uploader.exe".into()),
            args: vec!["--quiet".into(), "{file}".into()],
        };
        let request = custom_command_request(&valid, file, working).unwrap();
        assert_eq!(request.arguments[1], file.as_os_str());
        for args in [
            vec!["--file={file}".into()],
            vec!["{file}".into(), "{file}".into()],
            vec!["--quiet".into()],
        ] {
            let invalid = CustomImageCommandPreferences {
                executable: Some("uploader.exe".into()),
                args,
            };
            assert!(custom_command_request(&invalid, file, working).is_err());
        }
    }

    #[test]
    fn picgo_core_parser_handles_ansi_logs_and_rejects_ambiguous_results() {
        let output =
            b"info preparing\n\x1b[32m[PicGo SUCCESS:]\x1b[0m\nhttps://cdn.test/a.png?sig=x\n";
        assert_eq!(
            parse_picgo_core_output(output).unwrap(),
            "https://cdn.test/a.png?sig=x"
        );
        assert!(
            parse_picgo_core_output(
                b"[PicGo SUCCESS:]\nhttps://a.test/a.png\nhttps://b.test/b.png\n"
            )
            .is_err()
        );
        assert!(parse_picgo_core_output(b"https://cdn.test/a.png\n").is_err());

        let preferences = PicGoCorePreferences {
            executable: Some("node.exe".into()),
            launcher_args: vec!["C:/PicGo/bin/picgo".into()],
            config_path: Some("C:/PicGo/config.json".into()),
        };
        let file = Path::new("C:/stage/图 片.png");
        let request = picgo_core_request(&preferences, file, Path::new("C:/private")).unwrap();
        assert_eq!(request.program, Path::new("node.exe"));
        assert_eq!(
            request.arguments,
            vec![
                OsString::from("C:/PicGo/bin/picgo"),
                OsString::from("upload"),
                OsString::from("--config"),
                OsString::from("C:/PicGo/config.json"),
                file.as_os_str().to_owned(),
            ]
        );
        assert!(
            picgo_core_request(
                &PicGoCorePreferences::default(),
                file,
                Path::new("C:/private")
            )
            .is_err()
        );
    }

    fn read_request(stream: &mut std::net::TcpStream) -> Vec<u8> {
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        let mut expected = None;
        loop {
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..count]);
            if expected.is_none()
                && let Some(header_end) = request.windows(4).position(|part| part == b"\r\n\r\n")
            {
                let header_end = header_end + 4;
                let headers = String::from_utf8_lossy(&request[..header_end]);
                let content_length = headers
                    .lines()
                    .find_map(|line| {
                        line.to_ascii_lowercase()
                            .strip_prefix("content-length:")
                            .map(str::trim)
                            .and_then(|value| value.parse::<usize>().ok())
                    })
                    .unwrap_or(0);
                expected = Some(header_end + content_length);
            }
            if expected.is_some_and(|expected| request.len() >= expected) {
                break;
            }
        }
        request
    }

    fn one_response_server(response: Vec<u8>) -> (String, thread::JoinHandle<Vec<u8>>) {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let thread = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let request = read_request(&mut stream);
            stream.write_all(&response).unwrap();
            stream.flush().unwrap();
            request
        });
        (format!("http://{address}/upload"), thread)
    }

    #[test]
    fn picgo_http_requires_loopback_strict_success_and_one_url() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tiny.png");
        fs::write(&file, TINY_PNG).unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let success_body = br#"{"success":true,"result":["https://cdn.test/a.png?sig=x"]}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            success_body.len(),
            String::from_utf8_lossy(success_body)
        )
        .into_bytes();
        let (endpoint, server) = one_response_server(response);
        let url = runtime
            .block_on(upload_picgo_http(
                &endpoint,
                &file,
                ImageTransferLimits::default(),
                &CommandCancellation::default(),
            ))
            .unwrap();
        assert_eq!(url, "https://cdn.test/a.png?sig=x");
        let request = String::from_utf8_lossy(&server.join().unwrap()).to_string();
        assert!(request.starts_with("POST /upload "));
        assert!(request.contains("\"list\":["));

        for body in [
            r#"{"success":false,"result":["https://cdn.test/a.png"]}"#,
            r#"{"success":true,"result":[]}"#,
            r#"{"success":true,"result":["https://a.test/a.png","https://b.test/b.png"]}"#,
            "not json",
        ] {
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            )
            .into_bytes();
            let (endpoint, server) = one_response_server(response);
            assert!(
                runtime
                    .block_on(upload_picgo_http(
                        &endpoint,
                        &file,
                        ImageTransferLimits::default(),
                        &CommandCancellation::default(),
                    ))
                    .is_err()
            );
            server.join().unwrap();
        }
        assert!(
            runtime
                .block_on(upload_picgo_http(
                    "https://example.com/upload",
                    &file,
                    ImageTransferLimits::default(),
                    &CommandCancellation::default(),
                ))
                .is_err()
        );
        let absent = TcpListener::bind("127.0.0.1:0").unwrap();
        let absent_endpoint = format!("http://{}/upload", absent.local_addr().unwrap());
        drop(absent);
        assert!(
            runtime
                .block_on(upload_picgo_http(
                    &absent_endpoint,
                    &file,
                    ImageTransferLimits {
                        timeout: Duration::from_millis(200),
                        ..ImageTransferLimits::default()
                    },
                    &CommandCancellation::default(),
                ))
                .is_err()
        );
    }

    #[test]
    fn picgo_http_refuses_redirects_and_observes_mid_response_cancellation() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("tiny.png");
        fs::write(&file, TINY_PNG).unwrap();
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let (endpoint, server) = one_response_server(
            b"HTTP/1.1 302 Found\r\nLocation: http://127.0.0.1:1/other\r\nContent-Length: 0\r\nConnection: close\r\n\r\n".to_vec(),
        );
        let error = runtime
            .block_on(upload_picgo_http(
                &endpoint,
                &file,
                ImageTransferLimits::default(),
                &CommandCancellation::default(),
            ))
            .unwrap_err();
        assert!(matches!(error, ImageTransferError::HttpStatus(302)));
        server.join().unwrap();

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = format!("http://{}/upload", listener.local_addr().unwrap());
        let slow_server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = read_request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 64\r\n\r\n{\"success\":true,")
                .unwrap();
            stream.flush().unwrap();
            thread::sleep(Duration::from_millis(300));
        });
        let cancellation = CommandCancellation::default();
        let cancel = cancellation.clone();
        let cancel_thread = thread::spawn(move || {
            thread::sleep(Duration::from_millis(50));
            cancel.cancel();
        });
        let error = runtime
            .block_on(upload_picgo_http(
                &endpoint,
                &file,
                ImageTransferLimits::default(),
                &cancellation,
            ))
            .unwrap_err();
        assert!(matches!(error, ImageTransferError::Canceled));
        cancel_thread.join().unwrap();
        slow_server.join().unwrap();
    }

    #[test]
    fn remote_materialization_times_out_and_batch_concurrency_is_bounded() {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        let dir = tempfile::tempdir().unwrap();
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/stalled.png", listener.local_addr().unwrap());
        let stalled = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let _ = read_request(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 12\r\n\r\n")
                .unwrap();
            stream.flush().unwrap();
            thread::sleep(Duration::from_millis(300));
        });
        let result = runtime.block_on(materialize_input(
            ImageInput::Remote(url),
            dir.path(),
            ImageTransferLimits {
                timeout: Duration::from_millis(80),
                ..ImageTransferLimits::default()
            },
            &CommandCancellation::default(),
        ));
        assert!(matches!(result, Err(ImageTransferError::Http(_))));
        stalled.join().unwrap();

        let active = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let mut urls = Vec::new();
        let mut servers = Vec::new();
        for _ in 0..3 {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            urls.push(format!(
                "http://{}/image.png",
                listener.local_addr().unwrap()
            ));
            let active = active.clone();
            let maximum = maximum.clone();
            servers.push(thread::spawn(move || {
                let (mut stream, _) = listener.accept().unwrap();
                let _ = read_request(&mut stream);
                let now = active.fetch_add(1, AtomicOrdering::SeqCst) + 1;
                maximum.fetch_max(now, AtomicOrdering::SeqCst);
                thread::sleep(Duration::from_millis(100));
                active.fetch_sub(1, AtomicOrdering::SeqCst);
                stream
                    .write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 15\r\nConnection: close\r\n\r\n\x89PNG\r\n\x1a\nfixture",
                    )
                    .unwrap();
            }));
        }
        let results = runtime.block_on(materialize_inputs_bounded(
            urls.into_iter().map(ImageInput::Remote).collect(),
            dir.path().join("batch"),
            ImageTransferLimits::default(),
            CommandCancellation::default(),
        ));
        assert!(results.iter().all(Result::is_ok));
        assert_eq!(maximum.load(AtomicOrdering::SeqCst), 2);
        for server in servers {
            server.join().unwrap();
        }
    }

    #[cfg(windows)]
    #[test]
    fn command_adapters_execute_fixture_programs_without_shell_interpolation() {
        let dir = tempfile::tempdir().unwrap();
        let image = dir.path().join("图 片 & test.png");
        fs::write(&image, TINY_PNG).unwrap();

        let custom_script = dir.path().join("custom uploader.ps1");
        fs::write(
            &custom_script,
            "param([string]$file) [Console]::OutputEncoding=[Text.UTF8Encoding]::new(); [Console]::Out.Write('https://cdn.test/custom.png?sig=kept')",
        )
        .unwrap();
        let custom = CustomImageCommandPreferences {
            executable: Some("pwsh.exe".into()),
            args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-File".into(),
                custom_script.to_string_lossy().into_owned(),
                "{file}".into(),
            ],
        };
        assert_eq!(
            upload_with_custom_command(
                &custom,
                &image,
                dir.path(),
                ImageTransferLimits::default(),
                &CommandCancellation::default(),
            )
            .unwrap(),
            "https://cdn.test/custom.png?sig=kept"
        );

        let args_file = dir.path().join("picgo-args.txt");
        let core_script = dir.path().join("picgo fixture.ps1");
        fs::write(
            &core_script,
            format!(
                "param([Parameter(ValueFromRemainingArguments=$true)][string[]]$rest) [IO.File]::WriteAllLines('{}', $rest); [Console]::OutputEncoding=[Text.UTF8Encoding]::new(); Write-Output '[PicGo SUCCESS:]'; Write-Output 'https://cdn.test/core.png'",
                args_file.to_string_lossy().replace('\'', "''")
            ),
        )
        .unwrap();
        let core = PicGoCorePreferences {
            executable: Some("pwsh.exe".into()),
            launcher_args: vec![
                "-NoLogo".into(),
                "-NoProfile".into(),
                "-NonInteractive".into(),
                "-File".into(),
                core_script.to_string_lossy().into_owned(),
            ],
            config_path: Some(
                dir.path()
                    .join("config.json")
                    .to_string_lossy()
                    .into_owned(),
            ),
        };
        assert_eq!(
            upload_with_picgo_core(
                &core,
                &image,
                dir.path(),
                ImageTransferLimits::default(),
                &CommandCancellation::default(),
            )
            .unwrap(),
            "https://cdn.test/core.png"
        );
        let args = fs::read_to_string(args_file).unwrap();
        assert!(args.lines().any(|line| line == "upload"));
        assert!(args.lines().any(|line| line == "--config"));
        assert!(args.contains("图 片 & test.png"));

        let missing = PicGoCorePreferences {
            executable: Some("markion-definitely-missing-picgo.exe".into()),
            ..PicGoCorePreferences::default()
        };
        assert!(matches!(
            upload_with_picgo_core(
                &missing,
                &image,
                dir.path(),
                ImageTransferLimits::default(),
                &CommandCancellation::default(),
            ),
            Err(ImageTransferError::Provider(_))
        ));
    }
}
