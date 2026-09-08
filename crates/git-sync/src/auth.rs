use std::path::{Path, PathBuf};

use crate::{
    CancellationToken, CommandLimits, GitCommand, GitCommandError, GitCommandRunner,
    ProcessTermination,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoteTransport {
    Https,
    Ssh,
    Local,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RemoteUrl {
    raw: String,
    pub transport: RemoteTransport,
    pub host: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, thiserror::Error)]
pub enum RemoteUrlError {
    #[error("remote URL is empty or contains control characters")]
    Invalid,
    #[error("HTTPS remote URLs must not contain embedded credentials")]
    EmbeddedCredentials,
    #[error("remote URL uses an unsupported transport")]
    UnsupportedTransport,
}

impl RemoteUrl {
    pub fn parse(raw: impl Into<String>) -> Result<Self, RemoteUrlError> {
        let raw = raw.into();
        if raw.trim() != raw || raw.is_empty() || raw.chars().any(char::is_control) {
            return Err(RemoteUrlError::Invalid);
        }
        if raw.starts_with('-') {
            return Err(RemoteUrlError::Invalid);
        }
        if let Some(authority) = raw.strip_prefix("https://") {
            let authority = authority.split('/').next().unwrap_or_default();
            if authority.is_empty() {
                return Err(RemoteUrlError::Invalid);
            }
            if authority.contains('@') {
                return Err(RemoteUrlError::EmbeddedCredentials);
            }
            let host = strip_port(authority).to_string();
            return Ok(Self {
                raw,
                transport: RemoteTransport::Https,
                host: Some(host),
            });
        }
        if let Some(authority) = raw.strip_prefix("ssh://") {
            let authority = authority.split('/').next().unwrap_or_default();
            let host = authority.rsplit('@').next().unwrap_or_default();
            if host.is_empty() {
                return Err(RemoteUrlError::Invalid);
            }
            let host = strip_port(host).to_string();
            return Ok(Self {
                raw,
                transport: RemoteTransport::Ssh,
                host: Some(host),
            });
        }
        if !raw.contains("://")
            && let Some((left, path)) = raw.split_once(':')
            && !left.is_empty()
            && !path.is_empty()
            && !left.contains(['/', '\\'])
        {
            let host = left.rsplit('@').next().unwrap_or_default();
            if host.is_empty() {
                return Err(RemoteUrlError::Invalid);
            }
            let host = host.to_string();
            return Ok(Self {
                raw,
                transport: RemoteTransport::Ssh,
                host: Some(host),
            });
        }
        if raw.contains("://") {
            return Err(RemoteUrlError::UnsupportedTransport);
        }
        Ok(Self {
            raw,
            transport: RemoteTransport::Local,
            host: None,
        })
    }

    pub fn as_str(&self) -> &str {
        &self.raw
    }
}

fn strip_port(authority: &str) -> &str {
    if authority.starts_with('[') {
        return authority
            .split_once(']')
            .map(|(host, _)| host)
            .unwrap_or(authority);
    }
    authority.split(':').next().unwrap_or(authority)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CredentialHelper {
    pub command: String,
    pub secure_storage: bool,
    pub supports_noninteractive_use: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CredentialPlatform {
    Windows,
    MacOs,
    Linux,
    Other,
}

impl CredentialPlatform {
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::MacOs
        } else if cfg!(target_os = "linux") {
            Self::Linux
        } else {
            Self::Other
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CredentialStorage {
    SessionOnly,
    SecureHelper(CredentialHelper),
    Unavailable,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AskpassBridge {
    /// App-owned helper executable. It obtains the value through the one-time
    /// local endpoint; the credential itself never enters Git's environment.
    pub executable: PathBuf,
    pub one_time_endpoint: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RepositoryAuthor {
    pub name: String,
    pub email: String,
}

#[derive(Clone, Debug)]
pub struct Authentication {
    runner: GitCommandRunner,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AuthFailure {
    AuthenticationNeeded,
    UnknownHostKey,
    ChangedHostKey,
    EncryptedKey,
    Cancelled,
    VerificationFailed,
    Other(String),
}

impl Authentication {
    pub fn new(runner: GitCommandRunner) -> Self {
        Self { runner }
    }

    pub fn credential_helpers(
        &self,
        repository: &Path,
    ) -> Result<Vec<CredentialHelper>, GitCommandError> {
        let request = GitCommand::new(repository)
            .args(["config", "--get-all", "credential.helper"])
            .read_only(true);
        let output = match self.runner.run(
            &request,
            CommandLimits::default(),
            &CancellationToken::new(),
        ) {
            Ok(output) => output,
            Err(GitCommandError::Failed { output }) if output.status_code == Some(1) => {
                return Ok(Vec::new());
            }
            Err(error) => return Err(error),
        };
        Ok(output
            .stdout_text()
            .lines()
            .map(str::trim)
            .filter(|helper| !helper.is_empty())
            .map(|helper| {
                let normalized = helper.to_ascii_lowercase();
                let secure = matches!(
                    normalized.as_str(),
                    "manager" | "manager-core" | "osxkeychain" | "libsecret" | "wincred"
                );
                CredentialHelper {
                    command: helper.to_string(),
                    secure_storage: secure && !normalized.contains("store"),
                    supports_noninteractive_use: credential_helper_is_noninteractive(
                        &normalized,
                        CredentialPlatform::current(),
                    ),
                }
            })
            .collect())
    }

    pub fn storage_choices(
        &self,
        repository: &Path,
    ) -> Result<Vec<CredentialStorage>, GitCommandError> {
        let mut choices = vec![CredentialStorage::SessionOnly];
        choices.extend(
            self.credential_helpers(repository)?
                .into_iter()
                .filter(|helper| helper.secure_storage)
                .map(CredentialStorage::SecureHelper),
        );
        if choices.len() == 1 {
            choices.push(CredentialStorage::Unavailable);
        }
        Ok(choices)
    }

    pub fn with_askpass(&self, request: GitCommand, bridge: &AskpassBridge) -> GitCommand {
        request
            .interactive_credentials(true)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_ASKPASS", bridge.executable.as_os_str())
            .env("SSH_ASKPASS", bridge.executable.as_os_str())
            .env("SSH_ASKPASS_REQUIRE", "force")
            .env("MARKION_ASKPASS_ENDPOINT", &bridge.one_time_endpoint)
    }

    pub fn approve_secure_credential(
        &self,
        repository: &Path,
        remote: &RemoteUrl,
        username: &str,
        secret: &str,
    ) -> Result<(), GitCommandError> {
        let _helper = self
            .credential_helpers(repository)?
            .into_iter()
            .find(|helper| helper.secure_storage)
            .ok_or_else(|| {
                GitCommandError::Output(std::io::Error::other("no secure credential helper"))
            })?;
        let host = remote.host.as_deref().unwrap_or_default();
        let input = format!(
            "protocol=https\nhost={host}\nusername={}\npassword={}\n\n",
            protocol_value(username),
            protocol_value(secret)
        );
        let request = GitCommand::new(repository)
            .args(["credential", "approve"])
            .standard_input(input.into_bytes());
        self.runner
            .run(
                &request,
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .map(|_| ())
    }

    pub fn configure_author(
        &self,
        repository: &Path,
        author: &RepositoryAuthor,
    ) -> Result<(), GitCommandError> {
        if !valid_config_value(&author.name) || !valid_config_value(&author.email) {
            return Err(GitCommandError::Output(std::io::Error::other(
                "invalid repository author",
            )));
        }
        for (key, value) in [("user.name", &author.name), ("user.email", &author.email)] {
            self.runner.run(
                &GitCommand::new(repository).args(["config", "--local", key, value]),
                CommandLimits::default(),
                &CancellationToken::new(),
            )?;
        }
        Ok(())
    }

    pub fn classify_failure(error: &GitCommandError) -> AuthFailure {
        let Some(output) = error.output() else {
            return AuthFailure::Other(error.to_string());
        };
        if output.termination == ProcessTermination::Cancelled {
            return AuthFailure::Cancelled;
        }
        let message = output.stderr_text().to_ascii_lowercase();
        if message.contains("remote host identification has changed") {
            return AuthFailure::ChangedHostKey;
        }
        if message.contains("authenticity of host") || message.contains("host key is not cached") {
            return AuthFailure::UnknownHostKey;
        }
        if message.contains("host key verification failed") {
            return AuthFailure::VerificationFailed;
        }
        if message.contains("incorrect passphrase") || message.contains("enter passphrase for key")
        {
            return AuthFailure::EncryptedKey;
        }
        if message.contains("authentication failed")
            || message.contains("permission denied")
            || message.contains("could not read username")
        {
            return AuthFailure::AuthenticationNeeded;
        }
        AuthFailure::Other(redact_sensitive_text(&output.stderr_text()))
    }
}

pub fn credential_helper_is_noninteractive(helper: &str, platform: CredentialPlatform) -> bool {
    let helper = helper.trim().to_ascii_lowercase();
    if helper.starts_with('!') || helper.contains(char::is_whitespace) {
        return false;
    }
    match helper.as_str() {
        // GCM honors GCM_INTERACTIVE=Never and credential.interactive=never,
        // which the background request boundary always supplies.
        "manager" | "manager-core" => true,
        // The in-memory cache helper returns a miss without opening UI.
        "cache" => true,
        "wincred" => platform == CredentialPlatform::Windows,
        "osxkeychain" => platform == CredentialPlatform::MacOs,
        // libsecret may unlock a desktop keyring interactively; fail closed.
        _ => false,
    }
}

fn valid_config_value(value: &str) -> bool {
    !value.trim().is_empty() && !value.chars().any(char::is_control)
}

fn protocol_value(value: &str) -> String {
    value
        .chars()
        .filter(|character| !matches!(character, '\r' | '\n' | '\0'))
        .collect()
}

fn redact_sensitive_text(message: &str) -> String {
    message
        .split_whitespace()
        .map(|word| {
            if word.starts_with("https://") && word.contains('@') {
                "https://[redacted]".to_string()
            } else {
                word.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GitCommandOutput;
    use std::time::Duration;

    #[test]
    fn remote_urls_reject_https_userinfo_and_classify_ssh() {
        assert_eq!(
            RemoteUrl::parse("https://token@example.com/notes.git"),
            Err(RemoteUrlError::EmbeddedCredentials)
        );
        assert_eq!(
            RemoteUrl::parse("git@example.com:notes.git")
                .unwrap()
                .transport,
            RemoteTransport::Ssh
        );
    }

    #[test]
    fn command_debug_redacts_bridge_values_and_standard_input() {
        let auth = Authentication::new(GitCommandRunner::new("git"));
        let command = auth.with_askpass(
            GitCommand::new(".").standard_input(b"password=secret".to_vec()),
            &AskpassBridge {
                executable: PathBuf::from("askpass"),
                one_time_endpoint: "secret-endpoint".into(),
            },
        );
        let debug = format!("{command:?}");
        assert!(!debug.contains("secret"));
        assert!(debug.contains("GIT_ASKPASS"));
    }

    #[test]
    fn author_configuration_is_repository_local() {
        let dir = tempfile::tempdir().unwrap();
        let runner = GitCommandRunner::new("git");
        runner
            .run(
                &GitCommand::new(dir.path()).args(["init", "-q"]),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        Authentication::new(runner.clone())
            .configure_author(
                dir.path(),
                &RepositoryAuthor {
                    name: "Markion User".into(),
                    email: "user@markion.invalid".into(),
                },
            )
            .unwrap();
        let name = runner
            .run(
                &GitCommand::new(dir.path())
                    .args(["config", "--local", "--get", "user.name"])
                    .read_only(true),
                CommandLimits::default(),
                &CancellationToken::new(),
            )
            .unwrap();
        assert_eq!(name.stdout_text().trim(), "Markion User");
    }

    #[test]
    fn ssh_security_failures_remain_distinct() {
        let failure = |message: &str| GitCommandError::Failed {
            output: GitCommandOutput {
                status_code: Some(128),
                stdout: Vec::new(),
                stderr: message.as_bytes().to_vec(),
                stdout_truncated: false,
                stderr_truncated: false,
                termination: ProcessTermination::Exited,
                elapsed: Duration::ZERO,
            },
        };
        assert_eq!(
            Authentication::classify_failure(&failure(
                "WARNING: REMOTE HOST IDENTIFICATION HAS CHANGED!"
            )),
            AuthFailure::ChangedHostKey
        );
        assert_eq!(
            Authentication::classify_failure(&failure(
                "The authenticity of host can't be established"
            )),
            AuthFailure::UnknownHostKey
        );
        assert_eq!(
            Authentication::classify_failure(&failure("Enter passphrase for key")),
            AuthFailure::EncryptedKey
        );
    }

    #[test]
    fn background_helper_support_is_platform_specific_and_fail_closed() {
        use CredentialPlatform::*;
        assert!(credential_helper_is_noninteractive("manager-core", Windows));
        assert!(credential_helper_is_noninteractive("manager", Linux));
        assert!(credential_helper_is_noninteractive("cache", Other));
        assert!(credential_helper_is_noninteractive("wincred", Windows));
        assert!(!credential_helper_is_noninteractive("wincred", MacOs));
        assert!(credential_helper_is_noninteractive("osxkeychain", MacOs));
        assert!(!credential_helper_is_noninteractive("osxkeychain", Linux));
        assert!(!credential_helper_is_noninteractive("libsecret", Linux));
        assert!(!credential_helper_is_noninteractive("store", Windows));
        assert!(!credential_helper_is_noninteractive(
            "!custom helper",
            Windows
        ));
    }
}
