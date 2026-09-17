//! Generic host-side scheduler for `paged-document/v1` providers.
//!
//! The service serializes document operations on one owner thread, bounds and
//! coalesces render requests, and exposes only owned host values to GPUI. It
//! has no PDF engine dependency; PDF is merely the first signed provider.

use markion::plugin_platform::{
    PagedDocument, PagedDocumentError, PagedDocumentLimits, PluginSession,
};
use markion_plugin_protocol::PluginErrorCode;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    path::PathBuf,
    sync::{
        Arc, Condvar, Mutex,
        mpsc::{self, Receiver},
    },
    thread::{self, JoinHandle},
};

pub(super) const MAX_RASTER_BYTES: usize = 32 * 1_048_576;
const MAX_COMMAND_QUEUE: usize = 32;
const RASTER_WIDTH_BUCKET_PX: u32 = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct PagedRequestId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct PagedDocumentId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) struct PagedGeneration(pub u64);

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct PagedPageGeometry {
    pub(super) width_points: f32,
    pub(super) height_points: f32,
}

impl PagedPageGeometry {
    pub(super) fn new(width_points: f32, height_points: f32) -> Result<Self, PagedHostError> {
        if !width_points.is_finite()
            || !height_points.is_finite()
            || width_points <= 0.0
            || height_points <= 0.0
        {
            return Err(PagedHostError::InvalidPageGeometry);
        }
        Ok(Self {
            width_points,
            height_points,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PagedRaster {
    pub(super) width_px: u32,
    pub(super) height_px: u32,
    pub(super) rgba: Vec<u8>,
}

impl PagedRaster {
    pub(super) fn new(
        width_px: u32,
        height_px: u32,
        rgba: Vec<u8>,
    ) -> Result<Self, PagedHostError> {
        let expected = raster_byte_len(width_px, height_px)?;
        if expected > MAX_RASTER_BYTES {
            return Err(PagedHostError::RasterTooLarge);
        }
        if rgba.len() != expected {
            return Err(PagedHostError::InvalidRaster);
        }
        Ok(Self {
            width_px,
            height_px,
            rgba,
        })
    }

    pub(super) fn byte_len(&self) -> usize {
        self.rgba.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(super) enum PagedRenderPriority {
    Adjacent,
    Visible,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PagedOpenRequest {
    pub(super) request_id: PagedRequestId,
    pub(super) path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(super) struct PagedRenderRequest {
    pub(super) document_id: PagedDocumentId,
    pub(super) generation: PagedGeneration,
    pub(super) page_index: u32,
    pub(super) target_width_px: u32,
    pub(super) priority: PagedRenderPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) enum PagedHostEvent {
    ServiceReady,
    Opened {
        request: PagedOpenRequest,
        document_id: PagedDocumentId,
        path: PathBuf,
        pages: Vec<PagedPageGeometry>,
    },
    OpenFailed {
        request: PagedOpenRequest,
        error: PagedHostError,
    },
    PageReady {
        request: PagedRenderRequest,
        raster: PagedRaster,
    },
    PageFailed {
        request: PagedRenderRequest,
        error: PagedHostError,
    },
    Closed(PagedDocumentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PagedHostError {
    ProviderMissing,
    ProviderUnavailable,
    FileUnavailable,
    SourceTooLarge,
    PageLimitExceeded,
    InvalidPageGeometry,
    InvalidPageNumber,
    RasterTooLarge,
    InvalidRaster,
    Encrypted,
    CorruptOrUnsupported,
    QueueFull,
    StaleRequest,
    ServiceStopped,
    Internal,
}

impl fmt::Display for PagedHostError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::ProviderMissing => "document provider is not installed",
            Self::ProviderUnavailable => "document provider is unavailable",
            Self::FileUnavailable => "document file is unavailable",
            Self::SourceTooLarge => "document exceeds the supported size",
            Self::PageLimitExceeded => "document has too many pages",
            Self::InvalidPageGeometry => "page geometry is invalid",
            Self::InvalidPageNumber => "page number is invalid",
            Self::RasterTooLarge => "page exceeds the raster limit",
            Self::InvalidRaster => "page raster is invalid",
            Self::Encrypted => "encrypted document is unavailable",
            Self::CorruptOrUnsupported => "document is corrupt or unsupported",
            Self::QueueFull => "document renderer queue is full",
            Self::StaleRequest => "document render request is stale",
            Self::ServiceStopped => "document provider has stopped",
            Self::Internal => "document rendering failed",
        })
    }
}

impl std::error::Error for PagedHostError {}

#[derive(Debug, Clone)]
enum PagedHostCommand {
    Open(PagedOpenRequest),
    Render(PagedRenderRequest),
    Close(PagedDocumentId),
    Shutdown,
}

pub(super) struct PagedDocumentService {
    queue: Arc<CommandQueue>,
    events: Receiver<PagedHostEvent>,
    worker: Option<JoinHandle<()>>,
}

impl PagedDocumentService {
    pub(super) fn start(session: PluginSession) -> Self {
        let queue = Arc::new(CommandQueue::new(MAX_COMMAND_QUEUE));
        let worker_queue = Arc::clone(&queue);
        let (event_sender, events) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("markion-paged-document-provider".to_owned())
            .spawn(move || {
                if event_sender.send(PagedHostEvent::ServiceReady).is_err() {
                    return;
                }
                let mut documents = HashMap::<PagedDocumentId, PagedDocument>::new();
                while let Some(command) = worker_queue.pop() {
                    let event = match command {
                        PagedHostCommand::Open(request) => {
                            match PagedDocument::open(
                                session.clone(),
                                &request.path,
                                PagedDocumentLimits::default(),
                            ) {
                                Ok(document) => {
                                    let document_id = PagedDocumentId(document.document_token());
                                    let pages = document
                                        .pages()
                                        .iter()
                                        .map(|page| {
                                            PagedPageGeometry::new(
                                                page.width_points as f32,
                                                page.height_points as f32,
                                            )
                                        })
                                        .collect::<Result<Vec<_>, _>>();
                                    match pages {
                                        Ok(pages) => {
                                            let path = document.path().to_path_buf();
                                            documents.insert(document_id, document);
                                            PagedHostEvent::Opened {
                                                request,
                                                document_id,
                                                path,
                                                pages,
                                            }
                                        }
                                        Err(error) => {
                                            let _ = document.close();
                                            PagedHostEvent::OpenFailed { request, error }
                                        }
                                    }
                                }
                                Err(error) => PagedHostEvent::OpenFailed {
                                    request,
                                    error: map_paged_error(error),
                                },
                            }
                        }
                        PagedHostCommand::Render(request) => {
                            let result = documents
                                .get(&request.document_id)
                                .ok_or(PagedHostError::StaleRequest)
                                .and_then(|document| {
                                    document
                                        .render(
                                            request.generation.0,
                                            request.page_index,
                                            request.target_width_px,
                                        )
                                        .map_err(map_paged_error)
                                });
                            if !worker_queue.is_current(request.document_id, request.generation) {
                                continue;
                            }
                            match result.and_then(|raster| {
                                PagedRaster::new(raster.width, raster.height, raster.rgba)
                            }) {
                                Ok(raster) => PagedHostEvent::PageReady { request, raster },
                                Err(error) => PagedHostEvent::PageFailed { request, error },
                            }
                        }
                        PagedHostCommand::Close(document_id) => {
                            if let Some(document) = documents.remove(&document_id) {
                                let _ = document.close();
                            }
                            PagedHostEvent::Closed(document_id)
                        }
                        PagedHostCommand::Shutdown => break,
                    };
                    if event_sender.send(event).is_err() {
                        break;
                    }
                }
                for (_, document) in documents {
                    let _ = document.close();
                }
            })
            .expect("spawn paged-document owner thread");
        Self {
            queue,
            events,
            worker: Some(worker),
        }
    }

    pub(super) fn open(&self, request: PagedOpenRequest) -> Result<(), PagedHostError> {
        self.queue.push(PagedHostCommand::Open(request))
    }

    pub(super) fn render(&self, request: PagedRenderRequest) -> Result<(), PagedHostError> {
        self.queue.push(PagedHostCommand::Render(request))
    }

    pub(super) fn close(&self, document_id: PagedDocumentId) -> Result<(), PagedHostError> {
        self.queue.push(PagedHostCommand::Close(document_id))
    }

    pub(super) fn try_recv(&self) -> Result<PagedHostEvent, mpsc::TryRecvError> {
        self.events.try_recv()
    }

    fn shutdown(&self) {
        let _ = self.queue.push(PagedHostCommand::Shutdown);
    }
}

impl Drop for PagedDocumentService {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

struct CommandQueue {
    capacity: usize,
    state: Mutex<QueueState>,
    wake: Condvar,
}

#[derive(Default)]
struct QueueState {
    commands: VecDeque<PagedHostCommand>,
    latest_generations: HashMap<PagedDocumentId, PagedGeneration>,
    closed_documents: HashSet<PagedDocumentId>,
    stopped: bool,
}

impl CommandQueue {
    fn new(capacity: usize) -> Self {
        Self {
            capacity,
            state: Mutex::new(QueueState::default()),
            wake: Condvar::new(),
        }
    }

    fn push(&self, command: PagedHostCommand) -> Result<(), PagedHostError> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| PagedHostError::ServiceStopped)?;
        if state.stopped {
            return Err(PagedHostError::ServiceStopped);
        }
        if matches!(command, PagedHostCommand::Shutdown) {
            state.stopped = true;
            state.commands.clear();
            self.wake.notify_all();
            return Ok(());
        }
        match &command {
            PagedHostCommand::Render(request) => {
                if state.closed_documents.contains(&request.document_id) {
                    return Err(PagedHostError::StaleRequest);
                }
                if state
                    .latest_generations
                    .get(&request.document_id)
                    .is_some_and(|generation| request.generation < *generation)
                {
                    return Err(PagedHostError::StaleRequest);
                }
                if state
                    .latest_generations
                    .get(&request.document_id)
                    .is_none_or(|generation| request.generation > *generation)
                {
                    state.commands.retain(|pending| {
                        !matches!(pending, PagedHostCommand::Render(old)
                            if old.document_id == request.document_id
                                && old.generation < request.generation)
                    });
                }
                state
                    .latest_generations
                    .insert(request.document_id, request.generation);
                if let Some(index) = state.commands.iter().position(|pending| {
                    matches!(pending, PagedHostCommand::Render(old)
                        if old.document_id == request.document_id
                            && old.generation == request.generation
                            && old.page_index == request.page_index
                            && old.target_width_px == request.target_width_px)
                }) {
                    state.commands.remove(index);
                }
            }
            PagedHostCommand::Close(document_id) => {
                state.closed_documents.insert(*document_id);
                state.latest_generations.remove(document_id);
                state.commands.retain(|pending| {
                    !matches!(pending, PagedHostCommand::Render(render)
                        if render.document_id == *document_id)
                });
            }
            PagedHostCommand::Open(_) => {}
            PagedHostCommand::Shutdown => unreachable!(),
        }
        if state.commands.len() >= self.capacity {
            let Some(index) = state
                .commands
                .iter()
                .rposition(|pending| matches!(pending, PagedHostCommand::Render(_)))
            else {
                return Err(PagedHostError::QueueFull);
            };
            state.commands.remove(index);
        }
        match &command {
            PagedHostCommand::Close(_) => state.commands.push_front(command),
            PagedHostCommand::Render(render) if render.priority == PagedRenderPriority::Visible => {
                let index = state
                    .commands
                    .iter()
                    .position(|pending| {
                        matches!(pending, PagedHostCommand::Render(other)
                            if other.priority == PagedRenderPriority::Adjacent)
                    })
                    .unwrap_or(state.commands.len());
                state.commands.insert(index, command);
            }
            _ => state.commands.push_back(command),
        }
        self.wake.notify_one();
        Ok(())
    }

    fn pop(&self) -> Option<PagedHostCommand> {
        let mut state = self.state.lock().ok()?;
        loop {
            if state.stopped {
                return None;
            }
            if let Some(command) = state.commands.pop_front() {
                return Some(command);
            }
            state = self.wake.wait(state).ok()?;
        }
    }

    fn is_current(&self, document_id: PagedDocumentId, generation: PagedGeneration) -> bool {
        let Ok(state) = self.state.lock() else {
            return false;
        };
        !state.stopped
            && !state.closed_documents.contains(&document_id)
            && state.latest_generations.get(&document_id) == Some(&generation)
    }
}

pub(super) fn target_width_bucket(target_width_px: u32) -> Result<u32, PagedHostError> {
    if target_width_px == 0 {
        return Err(PagedHostError::InvalidPageGeometry);
    }
    let bucket = (u64::from(target_width_px) + u64::from(RASTER_WIDTH_BUCKET_PX - 1))
        / u64::from(RASTER_WIDTH_BUCKET_PX)
        * u64::from(RASTER_WIDTH_BUCKET_PX);
    u32::try_from(bucket).map_err(|_| PagedHostError::RasterTooLarge)
}

fn raster_byte_len(width: u32, height: u32) -> Result<usize, PagedHostError> {
    let bytes = u64::from(width)
        .checked_mul(u64::from(height))
        .and_then(|pixels| pixels.checked_mul(4))
        .ok_or(PagedHostError::RasterTooLarge)?;
    usize::try_from(bytes).map_err(|_| PagedHostError::RasterTooLarge)
}

fn map_paged_error(error: PagedDocumentError) -> PagedHostError {
    match error {
        PagedDocumentError::SourceUnavailable => PagedHostError::FileUnavailable,
        PagedDocumentError::SourceTooLarge => PagedHostError::SourceTooLarge,
        PagedDocumentError::InvalidLimits | PagedDocumentError::InvalidRequest => {
            PagedHostError::InvalidPageNumber
        }
        PagedDocumentError::InvalidResponse => PagedHostError::InvalidRaster,
        PagedDocumentError::Supervisor(_) => PagedHostError::ProviderUnavailable,
        PagedDocumentError::Plugin(error) => match error.code {
            PluginErrorCode::SourceTooLarge => PagedHostError::SourceTooLarge,
            PluginErrorCode::TooManyPages => PagedHostError::PageLimitExceeded,
            PluginErrorCode::RasterTooLarge => PagedHostError::RasterTooLarge,
            PluginErrorCode::CorruptDocument => PagedHostError::CorruptOrUnsupported,
            PluginErrorCode::EncryptedDocument => PagedHostError::Encrypted,
            PluginErrorCode::RuntimeUnavailable => PagedHostError::ProviderUnavailable,
            PluginErrorCode::Cancelled => PagedHostError::StaleRequest,
            PluginErrorCode::InvalidRequest
            | PluginErrorCode::IncompatibleProtocol
            | PluginErrorCode::UnsupportedCapability
            | PluginErrorCode::Internal => PagedHostError::Internal,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn width_buckets_are_bounded_and_deterministic() {
        assert_eq!(target_width_bucket(1), Ok(64));
        assert_eq!(target_width_bucket(64), Ok(64));
        assert_eq!(target_width_bucket(65), Ok(128));
        assert_eq!(
            target_width_bucket(0),
            Err(PagedHostError::InvalidPageGeometry)
        );
    }

    #[test]
    fn queue_rejects_stale_generations_and_prioritizes_visible_pages() {
        let queue = CommandQueue::new(4);
        let id = PagedDocumentId(1);
        let request = |generation, page_index, priority| PagedRenderRequest {
            document_id: id,
            generation: PagedGeneration(generation),
            page_index,
            target_width_px: 640,
            priority,
        };
        queue
            .push(PagedHostCommand::Render(request(
                1,
                0,
                PagedRenderPriority::Adjacent,
            )))
            .unwrap();
        queue
            .push(PagedHostCommand::Render(request(
                2,
                2,
                PagedRenderPriority::Adjacent,
            )))
            .unwrap();
        queue
            .push(PagedHostCommand::Render(request(
                2,
                1,
                PagedRenderPriority::Visible,
            )))
            .unwrap();
        let PagedHostCommand::Render(first) = queue.pop().unwrap() else {
            panic!("expected render")
        };
        assert_eq!(first.page_index, 1);
        assert_eq!(
            queue.push(PagedHostCommand::Render(request(
                1,
                3,
                PagedRenderPriority::Visible,
            ))),
            Err(PagedHostError::StaleRequest)
        );
    }
}
