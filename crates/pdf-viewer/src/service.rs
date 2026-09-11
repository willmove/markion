use crate::backend::{NativePdfBackend, PdfRendererBackend};
use crate::types::{
    DocumentId, Generation, MAX_COMMAND_QUEUE, MAX_PAGE_COUNT, MAX_SOURCE_BYTES, OpenRequest,
    OpenedDocument, PageRaster, PdfErrorKind, RenderPriority, RenderRequest,
};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PdfCommand {
    Open(OpenRequest),
    Render(RenderRequest),
    Close(DocumentId),
    Shutdown,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PdfEvent {
    ServiceReady,
    ServiceFailed(PdfErrorKind),
    Opened(OpenedDocument),
    OpenFailed {
        request: OpenRequest,
        error: PdfErrorKind,
    },
    PageReady {
        request: RenderRequest,
        raster: PageRaster,
    },
    PageFailed {
        request: RenderRequest,
        error: PdfErrorKind,
    },
    Closed(DocumentId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitOutcome {
    Queued,
    Coalesced,
    EvictedOlderRender,
}

pub struct PdfService {
    queue: Arc<CommandQueue>,
    events: Receiver<PdfEvent>,
    worker: Option<JoinHandle<()>>,
}

impl PdfService {
    pub fn start_native(runtime_path: impl AsRef<Path>) -> Result<Self, PdfErrorKind> {
        let runtime_path = runtime_path.as_ref().to_path_buf();
        if !runtime_path.is_file() {
            return Err(PdfErrorKind::RuntimeMissing);
        }
        Ok(Self::start_with_factory(move || {
            NativePdfBackend::load(&runtime_path)
                .map(|backend| Box::new(backend) as Box<dyn PdfRendererBackend>)
        }))
    }

    pub fn start_with_backend<B>(backend: B) -> Self
    where
        B: PdfRendererBackend + Send + 'static,
    {
        Self::start_with_factory(move || Ok(Box::new(backend)))
    }

    fn start_with_factory<F>(factory: F) -> Self
    where
        F: FnOnce() -> Result<Box<dyn PdfRendererBackend>, PdfErrorKind> + Send + 'static,
    {
        let queue = Arc::new(CommandQueue::new(MAX_COMMAND_QUEUE));
        let worker_queue = Arc::clone(&queue);
        let (event_sender, events) = mpsc::channel();
        let worker = thread::Builder::new()
            .name("markion-pdf-renderer".to_string())
            .spawn(move || {
                let Ok(mut backend) = factory() else {
                    let _ = event_sender
                        .send(PdfEvent::ServiceFailed(PdfErrorKind::RuntimeUnavailable));
                    return;
                };
                if event_sender.send(PdfEvent::ServiceReady).is_err() {
                    return;
                }

                let mut next_document_id = 1_u64;
                while let Some(command) = worker_queue.pop() {
                    match command {
                        PdfCommand::Open(request) => {
                            let result =
                                open_document(backend.as_mut(), &request, &mut next_document_id);
                            let event = match result {
                                Ok(document) => PdfEvent::Opened(document),
                                Err(error) => PdfEvent::OpenFailed { request, error },
                            };
                            if event_sender.send(event).is_err() {
                                break;
                            }
                        }
                        PdfCommand::Render(request) => {
                            let result = backend.render(
                                request.document_id,
                                request.page_index,
                                request.target_width_px,
                            );
                            if !worker_queue.is_current(request.document_id, request.generation) {
                                continue;
                            }
                            let event = match result.and_then(|raster| {
                                raster.validate()?;
                                Ok(raster)
                            }) {
                                Ok(raster) => PdfEvent::PageReady { request, raster },
                                Err(error) => PdfEvent::PageFailed { request, error },
                            };
                            if event_sender.send(event).is_err() {
                                break;
                            }
                        }
                        PdfCommand::Close(document_id) => {
                            backend.close(document_id);
                            if event_sender.send(PdfEvent::Closed(document_id)).is_err() {
                                break;
                            }
                        }
                        PdfCommand::Shutdown => break,
                    }
                }
            })
            .expect("spawn the PDF renderer owner thread");

        Self {
            queue,
            events,
            worker: Some(worker),
        }
    }

    pub fn submit(&self, command: PdfCommand) -> Result<SubmitOutcome, PdfErrorKind> {
        self.queue.push(command)
    }

    pub fn open(&self, request: OpenRequest) -> Result<SubmitOutcome, PdfErrorKind> {
        self.submit(PdfCommand::Open(request))
    }

    pub fn render(&self, request: RenderRequest) -> Result<SubmitOutcome, PdfErrorKind> {
        self.submit(PdfCommand::Render(request))
    }

    pub fn close(&self, document_id: DocumentId) -> Result<SubmitOutcome, PdfErrorKind> {
        self.submit(PdfCommand::Close(document_id))
    }

    pub fn try_recv(&self) -> Result<PdfEvent, mpsc::TryRecvError> {
        self.events.try_recv()
    }

    pub fn recv_timeout(&self, timeout: Duration) -> Result<PdfEvent, RecvTimeoutError> {
        self.events.recv_timeout(timeout)
    }

    pub fn shutdown(&self) {
        let _ = self.submit(PdfCommand::Shutdown);
    }
}

impl Drop for PdfService {
    fn drop(&mut self) {
        self.shutdown();
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

fn open_document(
    backend: &mut dyn PdfRendererBackend,
    request: &OpenRequest,
    next_document_id: &mut u64,
) -> Result<OpenedDocument, PdfErrorKind> {
    let metadata = std::fs::metadata(&request.path).map_err(|_| PdfErrorKind::FileUnavailable)?;
    if !metadata.is_file() {
        return Err(PdfErrorKind::FileUnavailable);
    }
    if metadata.len() > MAX_SOURCE_BYTES {
        return Err(PdfErrorKind::SourceTooLarge);
    }
    let path = request
        .path
        .canonicalize()
        .map_err(|_| PdfErrorKind::FileUnavailable)?;
    let document_id = DocumentId(*next_document_id);
    *next_document_id = next_document_id
        .checked_add(1)
        .ok_or(PdfErrorKind::Internal)?;
    let pages = backend.open(document_id, &path)?;
    if pages.is_empty() {
        backend.close(document_id);
        return Err(PdfErrorKind::CorruptOrUnsupported);
    }
    if pages.len() > MAX_PAGE_COUNT as usize {
        backend.close(document_id);
        return Err(PdfErrorKind::PageLimitExceeded);
    }
    Ok(OpenedDocument {
        document_id,
        path,
        pages,
    })
}

struct CommandQueue {
    capacity: usize,
    state: Mutex<QueueState>,
    wake: Condvar,
}

#[derive(Default)]
struct QueueState {
    commands: VecDeque<PdfCommand>,
    latest_generations: HashMap<DocumentId, Generation>,
    closed_documents: HashSet<DocumentId>,
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

    fn push(&self, command: PdfCommand) -> Result<SubmitOutcome, PdfErrorKind> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| PdfErrorKind::ServiceStopped)?;
        if state.stopped {
            return Err(PdfErrorKind::ServiceStopped);
        }
        if matches!(command, PdfCommand::Shutdown) {
            state.stopped = true;
            state.commands.clear();
            self.wake.notify_all();
            return Ok(SubmitOutcome::Queued);
        }

        let mut outcome = SubmitOutcome::Queued;
        match &command {
            PdfCommand::Render(request) => {
                if state.closed_documents.contains(&request.document_id) {
                    return Err(PdfErrorKind::StaleRequest);
                }
                if let Some(generation) = state.latest_generations.get(&request.document_id)
                    && request.generation < *generation
                {
                    return Err(PdfErrorKind::StaleRequest);
                }
                if state
                    .latest_generations
                    .get(&request.document_id)
                    .is_none_or(|generation| request.generation > *generation)
                {
                    state.commands.retain(|pending| {
                        !matches!(pending, PdfCommand::Render(old) if old.document_id == request.document_id && old.generation < request.generation)
                    });
                }
                state
                    .latest_generations
                    .insert(request.document_id, request.generation);

                if let Some(index) = state.commands.iter().position(|pending| {
                    matches!(pending, PdfCommand::Render(old)
                        if old.document_id == request.document_id
                            && old.generation == request.generation
                            && old.page_index == request.page_index
                            && old.target_width_px == request.target_width_px)
                }) {
                    state.commands.remove(index);
                    outcome = SubmitOutcome::Coalesced;
                }
            }
            PdfCommand::Close(document_id) => {
                state.closed_documents.insert(*document_id);
                state.latest_generations.remove(document_id);
                state.commands.retain(|pending| {
                    !matches!(pending, PdfCommand::Render(render) if render.document_id == *document_id)
                });
            }
            PdfCommand::Open(_) => {}
            PdfCommand::Shutdown => unreachable!(),
        }

        if state.commands.len() >= self.capacity {
            let removable = state
                .commands
                .iter()
                .rposition(|pending| matches!(pending, PdfCommand::Render(_)));
            let Some(index) = removable else {
                return Err(PdfErrorKind::QueueFull);
            };
            state.commands.remove(index);
            if outcome == SubmitOutcome::Queued {
                outcome = SubmitOutcome::EvictedOlderRender;
            }
        }

        match &command {
            PdfCommand::Close(_) => state.commands.push_front(command),
            PdfCommand::Render(render) if render.priority == RenderPriority::Visible => {
                let index = state
                    .commands
                    .iter()
                    .position(|pending| {
                        matches!(pending, PdfCommand::Render(other) if other.priority == RenderPriority::Adjacent)
                    })
                    .unwrap_or(state.commands.len());
                state.commands.insert(index, command);
            }
            _ => state.commands.push_back(command),
        }
        self.wake.notify_one();
        Ok(outcome)
    }

    fn pop(&self) -> Option<PdfCommand> {
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

    fn is_current(&self, document_id: DocumentId, generation: Generation) -> bool {
        let Ok(state) = self.state.lock() else {
            return false;
        };
        !state.stopped
            && !state.closed_documents.contains(&document_id)
            && state.latest_generations.get(&document_id) == Some(&generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{PageGeometry, RequestId};
    use std::fs::File;
    use std::sync::atomic::{AtomicUsize, Ordering};

    struct FakeBackend {
        log: Arc<Mutex<Vec<String>>>,
        open_result: Result<Vec<PageGeometry>, PdfErrorKind>,
        active: Arc<AtomicUsize>,
        max_active: Arc<AtomicUsize>,
    }

    impl FakeBackend {
        fn healthy(log: Arc<Mutex<Vec<String>>>) -> Self {
            Self {
                log,
                open_result: Ok(vec![PageGeometry::new(612.0, 792.0).unwrap()]),
                active: Arc::new(AtomicUsize::new(0)),
                max_active: Arc::new(AtomicUsize::new(0)),
            }
        }
    }

    impl PdfRendererBackend for FakeBackend {
        fn open(
            &mut self,
            _document_id: DocumentId,
            _path: &Path,
        ) -> Result<Vec<PageGeometry>, PdfErrorKind> {
            self.log.lock().unwrap().push("open".into());
            self.open_result.clone()
        }

        fn render(
            &mut self,
            _document_id: DocumentId,
            page_index: u32,
            _target_width_px: u32,
        ) -> Result<PageRaster, PdfErrorKind> {
            self.log
                .lock()
                .unwrap()
                .push(format!("render:{page_index}"));
            let active = self.active.fetch_add(1, Ordering::SeqCst) + 1;
            self.max_active.fetch_max(active, Ordering::SeqCst);
            self.active.fetch_sub(1, Ordering::SeqCst);
            PageRaster::new(1, 1, vec![255; 4])
        }

        fn close(&mut self, document_id: DocumentId) {
            self.log
                .lock()
                .unwrap()
                .push(format!("close:{}", document_id.0));
        }
    }

    fn ready(service: &PdfService) {
        assert_eq!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::ServiceReady)
        );
    }

    fn render_request(
        document_id: DocumentId,
        generation: u64,
        page_index: u32,
        priority: RenderPriority,
    ) -> RenderRequest {
        RenderRequest {
            document_id,
            generation: Generation(generation),
            page_index,
            target_width_px: 640,
            priority,
        }
    }

    #[test]
    fn open_render_close_are_serialized_on_the_owner() {
        let temporary = tempfile::NamedTempFile::new().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend::healthy(Arc::clone(&log));
        let max_active = Arc::clone(&backend.max_active);
        let service = PdfService::start_with_backend(backend);
        ready(&service);
        service
            .open(OpenRequest {
                request_id: RequestId(7),
                path: temporary.path().to_path_buf(),
            })
            .unwrap();
        let PdfEvent::Opened(opened) = service.recv_timeout(Duration::from_secs(2)).unwrap() else {
            panic!("expected opened event")
        };
        service
            .render(render_request(
                opened.document_id,
                1,
                0,
                RenderPriority::Visible,
            ))
            .unwrap();
        assert!(matches!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::PageReady { .. })
        ));
        service.close(opened.document_id).unwrap();
        assert_eq!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::Closed(opened.document_id))
        );
        drop(service);
        assert_eq!(&*log.lock().unwrap(), &["open", "render:0", "close:1"]);
        assert_eq!(max_active.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn queue_coalesces_generations_and_prioritizes_visible_pages() {
        let queue = CommandQueue::new(4);
        let id = DocumentId(1);
        assert_eq!(
            queue.push(PdfCommand::Render(render_request(
                id,
                1,
                0,
                RenderPriority::Adjacent
            ))),
            Ok(SubmitOutcome::Queued)
        );
        assert_eq!(
            queue.push(PdfCommand::Render(render_request(
                id,
                1,
                0,
                RenderPriority::Visible
            ))),
            Ok(SubmitOutcome::Coalesced)
        );
        queue
            .push(PdfCommand::Render(render_request(
                id,
                2,
                2,
                RenderPriority::Adjacent,
            )))
            .unwrap();
        queue
            .push(PdfCommand::Render(render_request(
                id,
                2,
                1,
                RenderPriority::Visible,
            )))
            .unwrap();
        let PdfCommand::Render(first) = queue.pop().unwrap() else {
            panic!("expected render")
        };
        assert_eq!(first.page_index, 1);
        assert_eq!(first.generation, Generation(2));
        assert_eq!(
            queue.push(PdfCommand::Render(render_request(
                id,
                1,
                3,
                RenderPriority::Visible
            ))),
            Err(PdfErrorKind::StaleRequest)
        );
    }

    #[test]
    fn close_discards_a_render_that_finishes_late() {
        struct BlockingBackend {
            started: mpsc::SyncSender<()>,
            release: mpsc::Receiver<()>,
        }

        impl PdfRendererBackend for BlockingBackend {
            fn open(
                &mut self,
                _document_id: DocumentId,
                _path: &Path,
            ) -> Result<Vec<PageGeometry>, PdfErrorKind> {
                Ok(vec![PageGeometry::new(1.0, 1.0).unwrap()])
            }

            fn render(
                &mut self,
                _document_id: DocumentId,
                _page_index: u32,
                _target_width_px: u32,
            ) -> Result<PageRaster, PdfErrorKind> {
                self.started.send(()).unwrap();
                self.release.recv().unwrap();
                PageRaster::new(1, 1, vec![255; 4])
            }

            fn close(&mut self, _document_id: DocumentId) {}
        }

        let (started_sender, started_receiver) = mpsc::sync_channel(0);
        let (release_sender, release_receiver) = mpsc::sync_channel(0);
        let service = PdfService::start_with_backend(BlockingBackend {
            started: started_sender,
            release: release_receiver,
        });
        ready(&service);
        let document_id = DocumentId(42);
        service
            .render(render_request(document_id, 1, 0, RenderPriority::Visible))
            .unwrap();
        started_receiver
            .recv_timeout(Duration::from_secs(2))
            .unwrap();
        service.close(document_id).unwrap();
        release_sender.send(()).unwrap();
        assert_eq!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::Closed(document_id))
        );
        assert!(matches!(service.try_recv(), Err(mpsc::TryRecvError::Empty)));
    }

    #[test]
    fn source_and_page_limits_fail_before_rendering() {
        let temporary = tempfile::tempdir().unwrap();
        let huge = temporary.path().join("huge.pdf");
        File::create(&huge)
            .unwrap()
            .set_len(MAX_SOURCE_BYTES + 1)
            .unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let service = PdfService::start_with_backend(FakeBackend::healthy(Arc::clone(&log)));
        ready(&service);
        let request = OpenRequest {
            request_id: RequestId(1),
            path: huge,
        };
        service.open(request.clone()).unwrap();
        assert_eq!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::OpenFailed {
                request,
                error: PdfErrorKind::SourceTooLarge,
            })
        );
        assert!(log.lock().unwrap().is_empty());
        drop(service);

        let small = tempfile::NamedTempFile::new().unwrap();
        let log = Arc::new(Mutex::new(Vec::new()));
        let backend = FakeBackend {
            log: Arc::clone(&log),
            open_result: Ok(vec![
                PageGeometry::new(1.0, 1.0).unwrap();
                MAX_PAGE_COUNT as usize + 1
            ]),
            active: Arc::new(AtomicUsize::new(0)),
            max_active: Arc::new(AtomicUsize::new(0)),
        };
        let service = PdfService::start_with_backend(backend);
        ready(&service);
        let request = OpenRequest {
            request_id: RequestId(2),
            path: small.path().to_path_buf(),
        };
        service.open(request.clone()).unwrap();
        assert_eq!(
            service.recv_timeout(Duration::from_secs(2)),
            Ok(PdfEvent::OpenFailed {
                request,
                error: PdfErrorKind::PageLimitExceeded,
            })
        );
        drop(service);
        assert_eq!(&*log.lock().unwrap(), &["open", "close:1"]);
    }

    #[test]
    fn fake_backend_preserves_safe_error_categories() {
        for expected in [
            PdfErrorKind::Encrypted,
            PdfErrorKind::CorruptOrUnsupported,
            PdfErrorKind::PageUnavailable,
        ] {
            let file = tempfile::NamedTempFile::new().unwrap();
            let backend = FakeBackend {
                log: Arc::new(Mutex::new(Vec::new())),
                open_result: Err(expected),
                active: Arc::new(AtomicUsize::new(0)),
                max_active: Arc::new(AtomicUsize::new(0)),
            };
            let service = PdfService::start_with_backend(backend);
            ready(&service);
            let request = OpenRequest {
                request_id: RequestId(9),
                path: file.path().to_path_buf(),
            };
            service.open(request.clone()).unwrap();
            assert_eq!(
                service.recv_timeout(Duration::from_secs(2)),
                Ok(PdfEvent::OpenFailed {
                    request,
                    error: expected,
                })
            );
        }
    }

    #[test]
    fn shutdown_rejects_new_work_and_joins_cleanly() {
        let service =
            PdfService::start_with_backend(FakeBackend::healthy(Arc::new(Mutex::new(Vec::new()))));
        ready(&service);
        service.shutdown();
        assert_eq!(
            service.render(render_request(DocumentId(1), 1, 0, RenderPriority::Visible)),
            Err(PdfErrorKind::ServiceStopped)
        );
        drop(service);
    }
}
