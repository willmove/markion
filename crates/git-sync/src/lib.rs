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
    AdmissionError, ExclusiveAdmission, GitOperationRegistry, ReadEpoch, WriteAdmission,
};
pub use auth::{
    AskpassBridge, AuthFailure, Authentication, CredentialHelper, CredentialPlatform,
    CredentialStorage, RemoteTransport, RemoteUrl, RemoteUrlError, RepositoryAuthor,
    credential_helper_is_noninteractive,
};
pub use background::{BackgroundFetchResult, BackgroundFetchScheduler, BackgroundNotification};
pub use conflict::{
    ConflictFile, ConflictManager, ConflictResolution, ConflictSession, ConflictSide,
    ConflictSource, RecoveryAssessment, RecoveryManager,
};
pub use engine::{GitSyncEngine, SyncEngineError, SyncOptions};
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
    NewFileClass, NewFileRule, PathDecision, PolicyError, PolicyStore, RepositoryPolicy,
    ReviewPlan, ScopeException, SyncPolicies, default_commit_message,
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
    inspect_note_attachments,
};
pub use setup::{
    CloneResult, ConnectReview, OnboardingError, OnboardingService, PublicationResult,
};
pub use status::{BranchStatus, StatusParseError, parse_porcelain_v2};
pub use system::{GitAvailability, GitVersion, detect_git};
