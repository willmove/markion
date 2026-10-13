//! GUI-free Git synchronization contracts for Markion.
//!
//! The application crate owns presentation, document entities, and GPUI
//! scheduling. This crate owns repository identity, synchronization plans,
//! process-independent state, and the system-Git adapter.

mod admission;
mod auth;
mod background;
mod conflict;
mod engine;
mod exclusion;
mod journal;
mod model;
mod persist;
mod policy;
mod process;
mod repository;
mod resources;
mod setup;
mod status;
mod system;

pub use admission::{
    AdmissionError, ConflictClaim, ExclusiveAdmission, GitOperationRegistry, ReadEpoch,
    WriteAdmission,
};
pub use auth::{
    AskpassBridge, AuthFailure, Authentication, CredentialHelper, CredentialPlatform,
    CredentialStorage, RemoteTransport, RemoteUrl, RemoteUrlError, RepositoryAuthor,
    credential_helper_is_noninteractive,
};
pub use background::{BackgroundFetchResult, BackgroundFetchScheduler, BackgroundNotification};
pub use conflict::{
    ConflictError, ConflictFile, ConflictManager, ConflictResolution, ConflictSession,
    ConflictSide, ConflictSource, ReconcileReport, RecoveryAssessment, RecoveryManager,
};
pub use engine::{GitSyncEngine, SyncEngineError, SyncOptions};
pub use exclusion::{add_device_exclusion, device_exclusions, remove_device_exclusion};
pub use journal::{
    ConflictDraft, JournalError, JournalStore, OperationCheckpoint, OperationJournal,
    OperationSummary, PreimageEntry, PreimageManifest,
};
pub use model::{
    BackupSyncAction, BackupSyncFacts, BackupSyncPresentation, BackupSyncState, ChangeKind,
    ConflictKind, ConnectionState, DocumentSyncState, FileChange, GitObjectId, HistoryRelation,
    OperationKind, OperationPhase, OperationProgress, RepositoryCapabilities, RepositoryIdentity,
    RepositorySnapshot, SyncOutcome, SyncPlan, SyncTarget, WorktreeState,
};
pub use policy::{
    HeldBack, HeldBackReason, NewFileClass, NewFileRule, PathDecision, PolicyError, PolicyStore,
    RepositoryPolicy, ReviewPlan, ScopeException, ScopeFacts, ScopePartition, SyncPolicies,
    default_commit_message, file_extension, is_os_artifact, tracked_path_is_notes_like,
    valid_extension,
};
pub use process::{
    CancellationToken, CommandLimits, GitCommand, GitCommandError, GitCommandOutput,
    GitCommandRunner, ProcessTermination,
};
pub use repository::{
    DiffContent, DiffRequest, GitRepository, HistoryEntry, RepositoryError, RepositoryState,
    TargetResolutionError, VersionComparison, content_fingerprint_bytes,
};
pub use resources::{
    AttachmentIssue, AttachmentIssueKind, AttachmentReport, attachment_reference_fingerprint,
    inspect_note_attachments, inspect_notes_attachments,
};
pub use setup::{
    CloneResult, ConnectReview, OnboardingError, OnboardingService, PublicationResult,
    repository_adoptable,
};
pub use status::{BranchStatus, StatusParseError, parse_porcelain_v2};
pub use system::{GitAvailability, GitVersion, detect_git};
