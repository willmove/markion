use crate::AiError;
use serde::{Deserialize, Deserializer, Serialize};
use url::Url;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Protocol {
    Responses,
    Chat,
    Anthropic,
}
impl Protocol {
    pub fn parse(value: &str) -> Result<Self, AiError> {
        match value {
            "responses" => Ok(Self::Responses),
            "chat" => Ok(Self::Chat),
            "anthropic" => Ok(Self::Anthropic),
            _ => Err(AiError::InvalidProfile),
        }
    }
    pub fn route(self) -> &'static str {
        match self {
            Self::Responses => "responses",
            Self::Chat => "chat/completions",
            Self::Anthropic => "messages",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Limits {
    pub input_bytes: usize,
    pub output_bytes: usize,
    pub tool_bytes: usize,
    pub max_tools: usize,
    pub max_operations: usize,
    pub timeout_secs: u64,
    pub idle_secs: u64,
    pub output_tokens: u32,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: 64 * 1024,
            output_bytes: 128 * 1024,
            tool_bytes: 64 * 1024,
            max_tools: 12,
            max_operations: 20,
            timeout_secs: 300,
            idle_secs: 30,
            output_tokens: 4096,
        }
    }
}
impl Limits {
    pub fn validate(&self) -> Result<(), AiError> {
        if !(1024..=1024 * 1024).contains(&self.input_bytes)
            || !(1024..=1024 * 1024).contains(&self.output_bytes)
            || !(256..=256 * 1024).contains(&self.tool_bytes)
            || !(1..=64).contains(&self.max_tools)
            || !(1..=100).contains(&self.max_operations)
            || !(5..=600).contains(&self.timeout_secs)
            || !(1..=120).contains(&self.idle_secs)
            || self.idle_secs > self.timeout_secs
            || !(128..=32768).contains(&self.output_tokens)
        {
            Err(AiError::InvalidProfile)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub provider: String,
    pub protocol: String,
    pub endpoint: String,
    pub model: String,
    pub limits: Limits,
    /// Non-secret state preserving an unusable hand-edited profile across saves.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid_reason: Option<String>,
}
impl Default for Profile {
    fn default() -> Self {
        Self::preset("openai", "default")
    }
}
impl Profile {
    pub const PRESETS: [&'static str; 5] = ["openai", "anthropic", "deepseek", "local", "custom"];
    pub fn preset(provider: &str, id: &str) -> Self {
        let (name, protocol, endpoint) = match provider {
            "openai" => ("OpenAI", "responses", "https://api.openai.com/v1"),
            "anthropic" => ("Anthropic", "anthropic", "https://api.anthropic.com/v1"),
            "deepseek" => ("DeepSeek", "chat", "https://api.deepseek.com/v1"),
            "local" => ("Local / Ollama", "chat", "http://localhost:11434/v1"),
            _ => ("Custom", "chat", ""),
        };
        Self {
            id: id.into(),
            name: name.into(),
            provider: provider.into(),
            protocol: protocol.into(),
            endpoint: endpoint.into(),
            model: String::new(),
            limits: Limits::default(),
            invalid_reason: None,
        }
    }
    pub fn base_url(&self) -> Result<Url, AiError> {
        let mut url = Url::parse(self.endpoint.trim()).map_err(|_| AiError::InvalidProfile)?;
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(AiError::InvalidProfile);
        }
        let local = match url.host() {
            Some(url::Host::Domain(name)) => name.eq_ignore_ascii_case("localhost"),
            Some(url::Host::Ipv4(address)) => address.is_loopback(),
            Some(url::Host::Ipv6(address)) => address.is_loopback(),
            None => false,
        };
        if url.scheme() != "https" && !(url.scheme() == "http" && local) {
            return Err(AiError::InvalidProfile);
        }
        if url.host().is_none() {
            return Err(AiError::InvalidProfile);
        }
        let path = url.path().trim_end_matches('/').to_string();
        // Config stores a versioned API root, never an already-suffixed method.
        if path.ends_with("/responses")
            || path.ends_with("/chat/completions")
            || path.ends_with("/messages")
        {
            return Err(AiError::InvalidProfile);
        }
        url.set_path(&format!("{path}/"));
        Ok(url)
    }
    pub fn request_url(&self) -> Result<Url, AiError> {
        self.base_url()?
            .join(Protocol::parse(&self.protocol)?.route())
            .map_err(|_| AiError::InvalidProfile)
    }
    pub fn validate(&self) -> Result<(), AiError> {
        if self.invalid_reason.is_some()
            || !Self::PRESETS.contains(&self.provider.as_str())
            || self.id.is_empty()
            || self.id.len() > 80
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
            || self.model.trim().is_empty()
            || self.model.len() > 256
        {
            return Err(AiError::InvalidProfile);
        }
        self.limits.validate()?;
        self.request_url()?;
        Ok(())
    }
    pub fn key_required(&self) -> bool {
        matches!(self.provider.as_str(), "openai" | "anthropic" | "deepseek")
    }
    pub fn credential_reference(&self) -> Result<String, AiError> {
        let base = self.base_url()?;
        Ok(format!("dev.markion.app/ai/{}/{}", self.id, base.as_str()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AiPreferences {
    pub enabled: bool,
    pub selected_profile: String,
    pub save_history: bool,
    pub writing_guidance: String,
    pub profiles: Vec<Profile>,
}
impl Default for AiPreferences {
    fn default() -> Self {
        Self {
            enabled: false,
            selected_profile: "default".into(),
            save_history: false,
            writing_guidance: String::new(),
            profiles: vec![Profile::default()],
        }
    }
}
impl AiPreferences {
    pub fn selected(&self) -> Option<&Profile> {
        let mut matching = self
            .profiles
            .iter()
            .filter(|p| p.id == self.selected_profile);
        let profile = matching.next()?;
        if matching.next().is_some() {
            None
        } else {
            Some(profile)
        }
    }
    pub fn selected_mut(&mut self) -> Option<&mut Profile> {
        if self
            .profiles
            .iter()
            .filter(|p| p.id == self.selected_profile)
            .count()
            != 1
        {
            return None;
        }
        self.profiles
            .iter_mut()
            .find(|p| p.id == self.selected_profile)
    }
    pub fn request_profile(&self) -> Result<&Profile, AiError> {
        if !self.enabled {
            return Err(AiError::Disabled);
        }
        let p = self.selected().ok_or(AiError::InvalidProfile)?;
        p.validate()?;
        Ok(p)
    }
}
// Isolate malformed AI fields so unrelated editor preferences remain usable.
impl<'de> Deserialize<'de> for AiPreferences {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = serde_json::Value::deserialize(deserializer)?;
        let mut result = Self::default();
        result.enabled = raw
            .get("enabled")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        result.save_history = raw
            .get("save_history")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        if let Some(v) = raw.get("selected_profile").and_then(|v| v.as_str()) {
            result.selected_profile = v.into();
        }
        if let Some(v) = raw.get("writing_guidance").and_then(|v| v.as_str()) {
            result.writing_guidance = v.chars().take(8192).collect();
        }
        if let Some(profiles) = raw.get("profiles") {
            result.profiles = match profiles.as_array() {
                Some(values) => values
                    .iter()
                    .take(32)
                    .enumerate()
                    .map(|(i, value)| {
                        serde_json::from_value(value.clone()).unwrap_or_else(|_| {
                            let mut p = Profile::preset("custom", &format!("invalid-{i}"));
                            p.invalid_reason = Some("Invalid profile fields".into());
                            p
                        })
                    })
                    .collect(),
                None => Vec::new(),
            };
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn presets_normalize_once_and_local_needs_no_key() {
        for provider in ["openai", "anthropic", "deepseek", "local"] {
            let mut p = Profile::preset(provider, "test");
            p.model = "test-model".into();
            p.endpoint.push('/');
            p.validate().unwrap();
            assert!(!p.request_url().unwrap().as_str().contains("/v1/v1/"));
        }
        assert!(!Profile::preset("local", "test").key_required());
    }
    #[test]
    fn unsafe_endpoints_and_invalid_limits_fail_closed() {
        let mut p = Profile::preset("custom", "test");
        p.model = "model".into();
        for endpoint in [
            "http://example.com/v1",
            "https://key@example.com/v1",
            "https://example.com/v1?api_key=key",
            "file:///tmp",
            "https://example.com/v1/chat/completions",
        ] {
            p.endpoint = endpoint.into();
            assert!(p.validate().is_err(), "{endpoint}");
        }
        for endpoint in [
            "http://127.0.0.1:8080/v1",
            "http://[::1]:8080/v1",
            "https://example.com/custom-root",
        ] {
            p.endpoint = endpoint.into();
            p.validate().unwrap();
        }
        p.limits.max_tools = 0;
        assert!(p.validate().is_err());
    }
    #[test]
    fn malformed_settings_are_isolated_and_profiles_unavailable() {
        let prefs: AiPreferences = toml::from_str(
            "enabled = true\nselected_profile = 'broken'\n[[profiles]]\nid='broken'\nendpoint=37\n",
        )
        .unwrap();
        assert!(prefs.request_profile().is_err());
        let prefs: AiPreferences = toml::from_str("enabled = 'wrong'\nsave_history = 12").unwrap();
        assert!(!prefs.enabled);
        assert!(!prefs.save_history);
        assert_eq!(
            toml::from_str::<AiPreferences>(&toml::to_string(&prefs).unwrap()).unwrap(),
            prefs
        );
    }
    #[test]
    fn credential_identity_changes_with_endpoint_and_duplicate_ids_fail() {
        let mut p = Profile::preset("local", "test");
        let a = p.credential_reference().unwrap();
        p.endpoint = "http://localhost:8888/v1".into();
        assert_ne!(a, p.credential_reference().unwrap());
        let mut prefs = AiPreferences::default();
        prefs.profiles.push(Profile::default());
        assert!(prefs.selected().is_none());
    }
}
