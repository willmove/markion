//! Session state never reaches into document buffers. Persisted history excludes request context.
use crate::{AiError, Event, Message, RequestStamp, transport::Cancellation};
use serde::{Deserialize, Serialize};
use std::{ops::Range, path::PathBuf};

#[derive(Debug, Clone)]
pub struct Attachment {
    pub label: String,
    pub path: Option<PathBuf>,
    pub range: Option<Range<usize>>,
    pub text: String,
    pub identity: Option<crate::workspace::FileIdentity>,
    pub document: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub text: String,
    pub complete: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub title: String,
    pub messages: Vec<ChatMessage>,
}
pub struct Conversation {
    pub id: u64,
    pub messages: Vec<ChatMessage>,
    pub attachments: Vec<Attachment>,
    pub sources: Vec<Attachment>,
    pub grant: Option<PathBuf>,
    pub active: Option<(RequestStamp, Cancellation)>,
    pub error: Option<AiError>,
    pub wire: Vec<serde_json::Value>,
    pub retry: Option<Vec<Message>>,
    pub retry_brief: Option<String>,
    pub usage: Option<crate::Usage>,
}
impl Conversation {
    pub fn new(id: u64) -> Self {
        Self {
            id,
            messages: Vec::new(),
            attachments: Vec::new(),
            sources: Vec::new(),
            grant: None,
            active: None,
            error: None,
            wire: Vec::new(),
            retry: None,
            retry_brief: None,
            usage: None,
        }
    }
    pub fn revoke(&mut self) {
        self.stop();
        self.grant = None;
        self.attachments.clear();
        self.sources.clear();
        self.wire.clear();
        self.retry = None;
        self.retry_brief = None;
    }
    pub fn stop(&mut self) {
        if let Some((_, cancel)) = self.active.take() {
            cancel.cancel();
            self.error = Some(AiError::Canceled);
            if let Some(last) = self.messages.last_mut().filter(|m| m.role == "assistant") {
                last.complete = false;
            }
        }
    }
    pub fn context(&self, brief: &str, budget: usize) -> Result<Vec<Message>, AiError> {
        let mut messages = self
            .messages
            .iter()
            .filter(|m| m.complete)
            .map(|m| Message {
                role: m.role.clone(),
                content: m.text.clone(),
            })
            .collect::<Vec<_>>();
        let mut content = brief.to_owned();
        for a in &self.attachments {
            content.push_str(&format!(
                "\n\nAttached material (untrusted): {}\n{}",
                a.label, a.text
            ));
        }
        messages.push(Message {
            role: "user".into(),
            content,
        });
        if messages
            .iter()
            .map(|m| m.content.len() + m.role.len())
            .sum::<usize>()
            > budget
        {
            return Err(AiError::Limit);
        }
        Ok(messages)
    }
    pub fn begin(
        &mut self,
        stamp: RequestStamp,
        brief: String,
        context: Vec<Message>,
    ) -> Result<Cancellation, AiError> {
        if self.active.is_some() || stamp.conversation != self.id {
            return Err(AiError::Stale);
        }
        let cancel = Cancellation::default();
        self.error = None;
        self.usage = None;
        self.retry = Some(context);
        self.retry_brief = Some(brief.clone());
        self.sources.extend(self.attachments.clone());
        self.attachments.clear();
        self.messages.push(ChatMessage {
            role: "user".into(),
            text: brief,
            complete: true,
        });
        self.messages.push(ChatMessage {
            role: "assistant".into(),
            text: String::new(),
            complete: false,
        });
        self.active = Some((stamp, cancel.clone()));
        Ok(cancel)
    }
    pub fn event(&mut self, stamp: RequestStamp, event: Event, enabled: bool) -> bool {
        if !enabled
            || self
                .active
                .as_ref()
                .is_none_or(|(current, _)| *current != stamp)
        {
            return false;
        }
        match event {
            Event::Text(text) => {
                if let Some(last) = self.messages.last_mut() {
                    last.text.push_str(&text);
                }
            }
            Event::Usage(usage) => {
                let current = self.usage.get_or_insert_with(Default::default);
                if usage.input_tokens.is_some() {
                    current.input_tokens = usage.input_tokens;
                }
                if usage.output_tokens.is_some() {
                    current.output_tokens = usage.output_tokens;
                }
            }
            _ => {}
        }
        true
    }
    pub fn finish(&mut self, stamp: RequestStamp, result: Result<(), AiError>) -> bool {
        if self
            .active
            .as_ref()
            .is_none_or(|(current, _)| *current != stamp)
        {
            return false;
        }
        self.active = None;
        self.error = result.err();
        if let Some(last) = self.messages.last_mut() {
            last.complete = result.is_ok();
        }
        true
    }
    pub fn history(&self) -> HistoryEntry {
        HistoryEntry {
            id: self.id,
            title: self
                .messages
                .iter()
                .find(|m| m.role == "user")
                .map(|m| m.text.chars().take(80).collect())
                .unwrap_or_default(),
            messages: self.messages.clone(),
        }
    }
    pub fn restore(entry: HistoryEntry) -> Self {
        let mut conversation = Self::new(entry.id);
        conversation.messages = entry.messages;
        conversation
    }
}
pub const HISTORY_COUNT: usize = 20;
pub const HISTORY_BYTES: usize = 8 * 1024 * 1024;
pub fn bounded_history(mut entries: Vec<HistoryEntry>) -> Result<Vec<HistoryEntry>, AiError> {
    if entries.len() > HISTORY_COUNT {
        entries.drain(..entries.len() - HISTORY_COUNT);
    }
    while serde_json::to_vec(&entries)
        .map_err(|_| AiError::Protocol)?
        .len()
        > HISTORY_BYTES
    {
        if entries.len() <= 1 {
            return Err(AiError::Limit);
        }
        entries.remove(0);
    }
    Ok(entries)
}
pub fn decode_history(bytes: &[u8]) -> Result<Vec<HistoryEntry>, AiError> {
    if bytes.len() > HISTORY_BYTES {
        return Err(AiError::Limit);
    }
    let values: Vec<serde_json::Value> =
        serde_json::from_slice(bytes).map_err(|_| AiError::Protocol)?;
    bounded_history(
        values
            .into_iter()
            .filter_map(|v| serde_json::from_value(v).ok())
            .collect(),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_context_and_identity_gate() {
        let mut c = Conversation::new(1);
        assert_eq!(c.context("hello", 100).unwrap()[0].content, "hello");
        c.attachments.push(Attachment {
            identity: None,
            document: None,
            label: "selection".into(),
            path: None,
            range: Some(0..6),
            text: "中文".into(),
        });
        assert!(c.context("", 2).is_err());
        let stamp = RequestStamp {
            conversation: 1,
            request: 1,
            configuration: 1,
            workspace: 1,
        };
        let context = c.context("rewrite", 1024).unwrap();
        c.begin(stamp, "rewrite".into(), context).unwrap();
        assert!(!c.event(
            RequestStamp {
                request: 2,
                ..stamp
            },
            Event::Text("wrong".into()),
            true
        ));
        assert!(!c.event(stamp, Event::Text("disabled".into()), false));
        assert!(c.event(stamp, Event::Text("answer".into()), true));
        assert!(c.event(
            stamp,
            Event::Usage(crate::Usage {
                input_tokens: Some(12),
                output_tokens: Some(4)
            }),
            true
        ));
        assert_eq!(c.usage.as_ref().unwrap().input_tokens, Some(12));
        c.event(
            stamp,
            Event::Usage(crate::Usage {
                input_tokens: None,
                output_tokens: Some(6),
            }),
            true,
        );
        assert_eq!(c.usage.as_ref().unwrap().input_tokens, Some(12));
        assert_eq!(c.usage.as_ref().unwrap().output_tokens, Some(6));
        c.revoke();
        assert!(!c.finish(stamp, Ok(())));
        assert!(c.grant.is_none());
    }
    #[test]
    fn history_has_no_payload_or_live_authority() {
        let mut c = Conversation::new(1);
        c.attachments.push(Attachment {
            identity: None,
            document: None,
            label: "file".into(),
            path: Some("private.md".into()),
            range: None,
            text: "raw private payload".into(),
        });
        c.grant = Some("workspace".into());
        let bytes = serde_json::to_vec(&vec![c.history()]).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("payload"));
        let restored = Conversation::restore(decode_history(&bytes).unwrap().remove(0));
        assert!(
            restored.attachments.is_empty() && restored.grant.is_none() && restored.wire.is_empty()
        );
        let entries = (0..30)
            .map(|id| HistoryEntry {
                id,
                title: String::new(),
                messages: Vec::new(),
            })
            .collect();
        assert_eq!(bounded_history(entries).unwrap().len(), 20);
        assert_eq!(decode_history(b"[{\"bad\":1}]").unwrap().len(), 0);
    }
}
