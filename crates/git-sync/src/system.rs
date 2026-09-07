use std::{path::PathBuf, time::Duration};

use crate::{CancellationToken, CommandLimits, GitCommand, GitCommandRunner};

pub const MINIMUM_GIT_VERSION: GitVersion = GitVersion {
    major: 2,
    minor: 39,
    patch: 0,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct GitVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl GitVersion {
    pub fn parse(output: &str) -> Option<Self> {
        let version = output.split_whitespace().find(|part| {
            part.bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_digit())
        })?;
        let mut numbers = version.split('.').map(|part| {
            part.chars()
                .take_while(char::is_ascii_digit)
                .collect::<String>()
                .parse::<u32>()
                .ok()
        });
        Some(Self {
            major: numbers.next()??,
            minor: numbers.next()??,
            patch: numbers.next().flatten().unwrap_or(0),
        })
    }

    pub fn is_supported(self) -> bool {
        self >= MINIMUM_GIT_VERSION
    }
}

#[derive(Debug)]
pub enum GitAvailability {
    Available {
        runner: GitCommandRunner,
        version: GitVersion,
    },
    Missing {
        executable: PathBuf,
        detail: String,
    },
    Unsupported {
        executable: PathBuf,
        version: GitVersion,
        minimum: GitVersion,
    },
    Unrecognized {
        executable: PathBuf,
        output: String,
    },
}

pub fn detect_git(executable_override: Option<PathBuf>) -> GitAvailability {
    let executable = executable_override.unwrap_or_else(|| PathBuf::from("git"));
    let runner = GitCommandRunner::new(&executable);
    let current = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let request = GitCommand::new(current).arg("--version").read_only(true);
    let output = match runner.run(
        &request,
        CommandLimits {
            timeout: Duration::from_secs(5),
            max_stdout_bytes: 4096,
            max_stderr_bytes: 4096,
        },
        &CancellationToken::new(),
    ) {
        Ok(output) => output,
        Err(error) => {
            return GitAvailability::Missing {
                executable,
                detail: error.to_string(),
            };
        }
    };
    let text = output.stdout_text();
    let Some(version) = GitVersion::parse(&text) else {
        return GitAvailability::Unrecognized {
            executable,
            output: text,
        };
    };
    if !version.is_supported() {
        return GitAvailability::Unsupported {
            executable,
            version,
            minimum: MINIMUM_GIT_VERSION,
        };
    }
    GitAvailability::Available { runner, version }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_platform_version_suffixes() {
        assert_eq!(
            GitVersion::parse("git version 2.51.2.windows.1"),
            Some(GitVersion {
                major: 2,
                minor: 51,
                patch: 2
            })
        );
        assert_eq!(
            GitVersion::parse("git version 2.39"),
            Some(GitVersion {
                major: 2,
                minor: 39,
                patch: 0
            })
        );
    }

    #[test]
    fn missing_override_is_nonfatal_availability() {
        assert!(matches!(
            detect_git(Some(PathBuf::from("definitely-not-markion-git"))),
            GitAvailability::Missing { .. }
        ));
    }
}
