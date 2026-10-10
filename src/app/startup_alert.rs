//! Startup-failure report that does not depend on GPUI: when the renderer (or
//! anything else before the first window) fails, GPUI cannot draw, so the
//! message goes to stderr and, where available, to an OS-native dialog.

use std::{
    any::Any,
    env,
    io::{self, Write as _},
    path::Path,
    process::{Command, ExitStatus},
};

use markion::i18n::{Language, StartupFailureMsg, startup_failure_tf};

pub(super) const TROUBLESHOOTING_URL: &str =
    "https://github.com/willmove/markion/blob/main/docs/linux-gpu-troubleshooting.md";

/// Test hook: makes startup fail exactly like a missing Vulkan driver does.
pub(super) const SIMULATE_RENDERER_FAILURE_ENV: &str = "MARKION_SIMULATE_RENDERER_FAILURE";
pub(super) const SIMULATED_RENDERER_FAILURE: &str =
    "Unable to init GPU context: simulated renderer failure (MARKION_SIMULATE_RENDERER_FAILURE)";

/// Substrings of the panics GPUI raises when no renderer can be created:
/// the X11 client's `Failed to initialize X11 client` chain, the Wayland
/// client's `expect`, and Markion's own main-window `expect`.
const RENDERER_FAILURE_MARKERS: [&str; 3] = [
    "Unable to init GPU context",
    "NoSupportedDeviceFound",
    "failed to open the Markion main window",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum StartupFailureKind {
    Renderer,
    Other,
}

pub(super) fn classify_failure(message: &str) -> StartupFailureKind {
    if RENDERER_FAILURE_MARKERS
        .iter()
        .any(|marker| message.contains(marker))
    {
        StartupFailureKind::Renderer
    } else {
        StartupFailureKind::Other
    }
}

pub(super) fn panic_message(payload: &(dyn Any + Send)) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "unknown panic payload".to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct StartupAlert {
    pub title: String,
    pub body: String,
}

pub(super) fn compose_alert(
    language: Language,
    kind: StartupFailureKind,
    software_fallback_failed: bool,
    detail: &str,
    log_dir: Option<&Path>,
) -> StartupAlert {
    let tf = |message, args: &[&str]| startup_failure_tf(language, message, args);
    let mut body = match kind {
        StartupFailureKind::Renderer if cfg!(target_os = "linux") => {
            tf(StartupFailureMsg::RendererLinux, &[TROUBLESHOOTING_URL])
        }
        StartupFailureKind::Renderer => tf(StartupFailureMsg::RendererGeneric, &[]),
        StartupFailureKind::Other => tf(StartupFailureMsg::Generic, &[]),
    };
    if software_fallback_failed {
        body.push_str("\n\n");
        body.push_str(&tf(StartupFailureMsg::SoftwareFallbackFailed, &[]));
    }
    body.push_str("\n\n");
    body.push_str(&tf(StartupFailureMsg::Details, &[detail.trim()]));
    if let Some(log_dir) = log_dir {
        body.push('\n');
        body.push_str(&tf(
            StartupFailureMsg::LogDirectory,
            &[&log_dir.display().to_string()],
        ));
    }
    StartupAlert {
        title: tf(StartupFailureMsg::Title, &[]),
        body,
    }
}

/// Writes the alert to stderr, then shows it in a native dialog if possible.
pub(super) fn show(alert: &StartupAlert) {
    let _ = writeln!(io::stderr(), "\n{}\n\n{}\n", alert.title, alert.body);
    let display_available =
        env::var_os("DISPLAY").is_some() || env::var_os("WAYLAND_DISPLAY").is_some();
    if cfg!(any(target_os = "linux", target_os = "freebsd")) {
        let shown = show_dialog(alert, display_available, &mut |program, args| {
            Command::new(program).args(args).status()
        });
        if let Some(program) = shown {
            tracing::info!(program, "startup failure dialog shown");
        }
    }
}

type Spawner<'a> = dyn FnMut(&str, &[String]) -> io::Result<ExitStatus> + 'a;

fn dialog_commands(alert: &StartupAlert) -> [(&'static str, Vec<String>); 3] {
    let StartupAlert { title, body } = alert;
    [
        (
            "zenity",
            vec![
                "--error".into(),
                "--no-markup".into(),
                "--width=560".into(),
                format!("--title={title}"),
                format!("--text={body}"),
            ],
        ),
        (
            "kdialog",
            vec![
                "--title".into(),
                title.clone(),
                "--error".into(),
                body.clone(),
            ],
        ),
        (
            "xmessage",
            vec![
                "-center".into(),
                "-buttons".into(),
                "OK:0".into(),
                "-default".into(),
                "OK".into(),
                wrap_text(&format!("{title}\n\n{body}"), XMESSAGE_COLUMNS),
            ],
        ),
    ]
}

/// xmessage neither wraps nor scrolls horizontally, so long lines would be cut
/// off at the screen edge.
const XMESSAGE_COLUMNS: usize = 76;

/// Wraps every line at spaces to at most `columns` display columns, counting
/// wide (CJK) characters as two and breaking words that do not fit. Wrapped
/// lines keep the original indentation, plus two spaces for `- ` list items.
fn wrap_text(text: &str, columns: usize) -> String {
    fn char_width(ch: char) -> usize {
        if ('\u{1100}'..='\u{115F}').contains(&ch)
            || ('\u{2E80}'..='\u{A4CF}').contains(&ch)
            || ('\u{AC00}'..='\u{D7A3}').contains(&ch)
            || ('\u{F900}'..='\u{FAFF}').contains(&ch)
            || ('\u{FE30}'..='\u{FE4F}').contains(&ch)
            || ('\u{FF00}'..='\u{FF60}').contains(&ch)
            || ('\u{FFE0}'..='\u{FFE6}').contains(&ch)
        {
            2
        } else {
            1
        }
    }

    let mut wrapped = Vec::new();
    for line in text.lines() {
        let content = line.trim_start_matches(' ');
        let indent = &line[..line.len() - content.len()];
        let hanging = if content.starts_with("- ") {
            format!("{indent}  ")
        } else {
            indent.to_string()
        };
        let mut current = indent.to_string();
        let mut width = indent.len();
        let mut at_line_start = true;
        let mut break_line = |current: &mut String, width: &mut usize| {
            wrapped.push(std::mem::replace(current, hanging.clone()));
            *width = hanging.len();
        };
        for word in content.split(' ').filter(|word| !word.is_empty()) {
            let word_width: usize = word.chars().map(char_width).sum();
            if !at_line_start && width + 1 + word_width > columns {
                break_line(&mut current, &mut width);
                at_line_start = true;
            }
            if !at_line_start {
                current.push(' ');
                width += 1;
            }
            for ch in word.chars() {
                let ch_width = char_width(ch);
                if !at_line_start && width + ch_width > columns {
                    break_line(&mut current, &mut width);
                }
                current.push(ch);
                width += ch_width;
                at_line_start = false;
            }
        }
        wrapped.push(current);
    }
    wrapped.join("\n")
}

/// Tries each dialog tool in order and returns the one that ran. Exit status
/// 1 counts as shown: zenity/kdialog use it when the dialog is dismissed.
fn show_dialog(
    alert: &StartupAlert,
    display_available: bool,
    spawn: &mut Spawner<'_>,
) -> Option<&'static str> {
    if !display_available {
        return None;
    }
    dialog_commands(alert)
        .into_iter()
        .find_map(|(program, args)| match spawn(program, &args) {
            Ok(status) if matches!(status.code(), Some(0 | 1)) => Some(program),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn alert() -> StartupAlert {
        StartupAlert {
            title: "Markion cannot start".into(),
            body: "a <b>body</b> & more".into(),
        }
    }

    #[test]
    fn renderer_panics_are_classified_as_renderer_failures() {
        let x11 = "called `Result::unwrap()` on an `Err` value: Failed to initialize X11 client.\n\n\
                   Caused by:\n    0: Unable to init GPU context\n    1: NoSupportedDeviceFound";
        let wayland = "Unable to init GPU context: Platform(Init(ERROR_INITIALIZATION_FAILED))";
        let window = "failed to open the Markion main window: NoSupportedDeviceFound";
        for message in [x11, wayland, window, SIMULATED_RENDERER_FAILURE] {
            assert_eq!(classify_failure(message), StartupFailureKind::Renderer);
        }
        assert_eq!(
            classify_failure("index out of bounds: the len is 0 but the index is 3"),
            StartupFailureKind::Other
        );
    }

    #[test]
    fn panic_message_reads_str_and_string_payloads() {
        let payload: Box<dyn Any + Send> = Box::new("static");
        assert_eq!(panic_message(&*payload), "static");
        let payload: Box<dyn Any + Send> = Box::new(String::from("owned"));
        assert_eq!(panic_message(&*payload), "owned");
        let payload: Box<dyn Any + Send> = Box::new(7_u8);
        assert_eq!(panic_message(&*payload), "unknown panic payload");
    }

    #[test]
    fn renderer_alert_carries_guidance_detail_and_log_directory() {
        let alert = compose_alert(
            Language::En,
            StartupFailureKind::Renderer,
            true,
            " NoSupportedDeviceFound \n",
            Some(Path::new("/home/u/.cache/markion/logs")),
        );
        assert_eq!(alert.title, "Markion cannot start");
        assert!(alert.body.contains("no usable Vulkan driver was found"));
        assert!(alert.body.contains(TROUBLESHOOTING_URL));
        assert!(alert.body.contains("could not start either"));
        assert!(alert.body.contains("Details: NoSupportedDeviceFound\n"));
        assert!(alert.body.ends_with("Log files: /home/u/.cache/markion/logs"));
    }

    #[test]
    fn alert_follows_the_interface_language() {
        let alert = compose_alert(Language::ZhHans, StartupFailureKind::Other, false, "boom", None);
        assert_eq!(alert.title, "Markion 无法启动");
        assert_eq!(alert.body, "Markion 启动失败。\n\n详细信息：boom");
    }

    #[test]
    fn dialog_chain_tries_zenity_then_kdialog_then_xmessage() {
        let mut calls = Vec::new();
        let shown = show_dialog(&alert(), true, &mut |program, args| {
            calls.push((program.to_string(), args.to_vec()));
            Err(io::Error::from(io::ErrorKind::NotFound))
        });
        assert_eq!(shown, None);
        let programs: Vec<_> = calls.iter().map(|(program, _)| program.as_str()).collect();
        assert_eq!(programs, ["zenity", "kdialog", "xmessage"]);
        assert!(calls[0].1.contains(&"--no-markup".to_string()));
        assert!(calls[0].1.contains(&"--text=a <b>body</b> & more".to_string()));
        assert_eq!(
            calls[1].1,
            ["--title", "Markion cannot start", "--error", "a <b>body</b> & more"]
        );
        assert_eq!(
            calls[2].1.last().unwrap(),
            "Markion cannot start\n\na <b>body</b> & more"
        );
    }

    #[cfg(unix)]
    #[test]
    fn dialog_chain_stops_at_the_first_tool_that_ran() {
        use std::os::unix::process::ExitStatusExt as _;

        let mut calls = Vec::new();
        let shown = show_dialog(&alert(), true, &mut |program, _| {
            calls.push(program.to_string());
            match program {
                "zenity" => Ok(ExitStatus::from_raw(255 << 8)),
                _ => Ok(ExitStatus::from_raw(1 << 8)),
            }
        });
        assert_eq!(shown, Some("kdialog"));
        assert_eq!(calls, ["zenity", "kdialog"]);
    }

    #[test]
    fn xmessage_text_is_wrapped_to_fit_the_screen() {
        let long = format!("Title\n\n- {}\n    0: detail", "word ".repeat(40).trim_end());
        let wrapped = wrap_text(&long, 20);
        let lines: Vec<_> = wrapped.lines().collect();
        assert_eq!(lines[0], "Title");
        assert_eq!(lines[1], "");
        assert_eq!(lines[2], "- word word word");
        assert_eq!(lines[3], "  word word word");
        assert_eq!(*lines.last().unwrap(), "    0: detail");
        assert!(lines.iter().all(|line| line.chars().count() <= 20));
        assert_eq!(wrapped.split_whitespace().filter(|word| *word == "word").count(), 40);
    }

    #[test]
    fn wrapping_breaks_unspaced_cjk_text_by_display_width() {
        let wrapped = wrap_text(&"图".repeat(15), 10);
        assert_eq!(wrapped.lines().collect::<Vec<_>>(), ["图图图图图", "图图图图图", "图图图图图"]);
        assert_eq!(wrap_text("https://example.com/abcdefghij", 10).lines().count(), 3);
    }

    #[test]
    fn no_dialog_is_attempted_without_a_display() {
        let shown = show_dialog(&alert(), false, &mut |_, _| panic!("must not spawn"));
        assert_eq!(shown, None);
    }
}
