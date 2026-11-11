//! Opt-in native input-callback to visible-caret paint observations.
//! Excludes OS input transport and GPU presentation; never logs document text.
use super::*;

#[derive(Clone, serde::Serialize)]
pub(super) struct PaintSample {
    kind: &'static str,
    bytes: usize,
    elapsed_us: u64,
}

static SAMPLES: std::sync::Mutex<VecDeque<PaintSample>> = std::sync::Mutex::new(VecDeque::new());

fn record(kind: &'static str, bytes: usize, elapsed_us: u64) {
    if let Ok(mut samples) = SAMPLES.lock() {
        if samples.len() >= 256 {
            samples.pop_front();
        }
        samples.push_back(PaintSample {
            kind,
            bytes,
            elapsed_us,
        });
    }
}

pub(super) fn take_samples() -> Vec<PaintSample> {
    SAMPLES
        .lock()
        .map(|mut samples| samples.drain(..).collect())
        .unwrap_or_default()
}

pub(super) fn enabled() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var_os("MARKION_PROFILE_VISUAL_INPUT").is_some())
}

impl MarkionApp {
    pub(super) fn begin_visual_open_profile(&mut self, start: Instant) {
        if enabled() {
            self.active_tab_mut().visual_open_profile = Some(start);
        }
    }

    pub(super) fn finish_visual_open_profile(&mut self) {
        let tab = self.active_tab_mut();
        if let Some(start) = tab.visual_open_profile.take() {
            record(
                "open",
                tab.document.text().len(),
                start.elapsed().as_micros() as u64,
            );
            tracing::info!(target: "markion::performance", bytes = tab.document.text().len(),
                elapsed_us = start.elapsed().as_micros() as u64, "document open to visual text paint");
        }
    }
    pub(super) fn begin_visual_input_profile(&mut self) {
        if !enabled() || !matches!(self.view_mode, ViewMode::VisualEdit) {
            return;
        }
        let tab = self.active_tab_mut();
        if tab.visual_input_profile.is_none() {
            tab.visual_input_profile = Some((tab.document.version(), Instant::now()));
        }
    }

    pub(super) fn finish_visual_input_profile(&mut self) {
        let tab = self.active_tab_mut();
        let Some((version, start)) = tab.visual_input_profile.take() else {
            return;
        };
        if version != tab.document.version() {
            return;
        }
        record(
            "input",
            tab.document.text().len(),
            start.elapsed().as_micros() as u64,
        );
        tracing::info!(target: "markion::performance", bytes = tab.document.text().len(),
            elapsed_us = start.elapsed().as_micros() as u64, "visual input callback to caret paint");
    }
}
