//! AI protocol and policy logic. Live buffers, credentials and file writes belong to the host.
pub mod agent;
pub mod config;
pub mod conversation;
pub mod links;
pub mod markdown;
pub mod proposals;
pub mod protocol;
pub mod sse;
pub mod transport;
pub mod workspace;
pub mod writing;
pub use config::{AiPreferences, Limits, Profile, Protocol};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RequestStamp {
    pub conversation: u64,
    pub request: u64,
    pub configuration: u64,
    pub workspace: u64,
}

impl RequestStamp {
    pub fn accepts(self, incoming: Self, enabled: bool) -> bool {
        enabled && self == incoming
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Capabilities {
    pub text: bool,
    pub streaming: bool,
    pub tools: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Usage {
    pub input_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Event {
    Text(String),
    ToolCall {
        id: String,
        name: String,
        arguments: serde_json::Value,
    },
    Usage(Usage),
    Complete,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AiError {
    #[error("AI is disabled")]
    Disabled,
    #[error("Invalid AI profile")]
    InvalidProfile,
    #[error("API key required")]
    MissingKey,
    #[error("Authentication failed")]
    Authentication,
    #[error("Model or endpoint not found")]
    NotFound,
    #[error("Service rate limit reached")]
    RateLimited,
    #[error("Server unavailable")]
    Unavailable,
    #[error("Request timed out")]
    Timeout,
    #[error("Request canceled")]
    Canceled,
    #[error("Unsupported request or model capability")]
    Unsupported,
    #[error("Invalid or incomplete provider response")]
    Protocol,
    #[error("AI context, output or tool budget exceeded")]
    Limit,
    #[error("Operation is outside the approved scope")]
    Scope,
    #[error("Source changed; refresh the proposal")]
    Stale,
}

/// Deliberately has no serialization implementation; diagnostics reveal no credential bytes.
#[derive(Clone)]
pub struct Secret(String);
impl Secret {
    pub fn new(value: String) -> Self {
        Self(value)
    }
    pub fn expose(&self) -> &str {
        &self.0
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
impl std::fmt::Debug for Secret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Secret([redacted])")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_and_disabled_events_are_rejected() {
        let active = RequestStamp {
            request: 7,
            ..Default::default()
        };
        assert!(active.accepts(active, true));
        assert!(!active.accepts(active, false));
        assert!(!active.accepts(
            RequestStamp {
                request: 6,
                ..active
            },
            true
        ));
        assert!(!active.accepts(
            RequestStamp {
                workspace: 1,
                ..active
            },
            true
        ));
        assert!(!active.accepts(
            RequestStamp {
                conversation: 1,
                ..active
            },
            true
        ));
    }
    #[test]
    fn secret_debug_is_content_free() {
        assert!(!format!("{:?}", Secret::new("private-key".into())).contains("private-key"));
    }
}
