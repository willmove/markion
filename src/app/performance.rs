//! Opt-in native input-callback to visible-caret paint observations.
//! Excludes OS input transport and GPU presentation; never logs document text.
use super::*;

#[derive(Clone, serde::Serialize)]
pub(super) struct PaintSample {
    kind: &'static str,
    bytes: usize,
    elapsed_us: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    thread_cpu_us: Option<u64>,
}

static SAMPLES: std::sync::Mutex<VecDeque<PaintSample>> = std::sync::Mutex::new(VecDeque::new());

fn record(kind: &'static str, bytes: usize, elapsed_us: u64, thread_cpu_us: Option<u64>) {
    if let Ok(mut samples) = SAMPLES.lock() {
        if samples.len() >= 256 {
            samples.pop_front();
        }
        samples.push_back(PaintSample {
            kind,
            bytes,
            elapsed_us,
            thread_cpu_us,
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

/// Current UI-thread CPU, including kernel time; no privileged trace session.
fn thread_cpu_time() -> Option<Duration> {
    #[cfg(windows)]
    {
        use windows::Win32::{
            Foundation::FILETIME,
            System::Threading::{GetCurrentThread, GetThreadTimes},
        };
        let mut created = FILETIME::default();
        let mut exited = FILETIME::default();
        let mut kernel = FILETIME::default();
        let mut user = FILETIME::default();
        unsafe {
            GetThreadTimes(
                GetCurrentThread(),
                &mut created,
                &mut exited,
                &mut kernel,
                &mut user,
            )
            .ok()?;
        }
        let ticks =
            |time: FILETIME| (u64::from(time.dwHighDateTime) << 32) | u64::from(time.dwLowDateTime);
        Some(Duration::from_nanos(
            (ticks(kernel) + ticks(user)).saturating_mul(100),
        ))
    }
    #[cfg(not(windows))]
    None
}

fn elapsed_cpu_us(start: Option<Duration>) -> Option<u64> {
    Some(thread_cpu_time()?.saturating_sub(start?).as_micros() as u64)
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
                None,
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
            tab.visual_input_profile =
                Some((tab.document.version(), Instant::now(), thread_cpu_time()));
        }
    }

    pub(super) fn note_visual_input_stage(&self, stage: &'static str) {
        if self.active_tab().is_image() {
            return;
        }
        let tab = self.active_tab();
        if let Some((version, start, cpu_start)) = tab.visual_input_profile
            && version == tab.document.version()
        {
            tracing::info!(target: "markion::performance", stage,
                bytes = tab.document.text().len(), elapsed_us = start.elapsed().as_micros() as u64,
                thread_cpu_us = ?elapsed_cpu_us(cpu_start), "visual input checkpoint");
        }
    }

    pub(super) fn finish_visual_input_profile(&mut self) {
        let tab = self.active_tab_mut();
        let Some((version, start, cpu_start)) = tab.visual_input_profile.take() else {
            return;
        };
        if version != tab.document.version() {
            return;
        }
        let cpu_us = elapsed_cpu_us(cpu_start);
        record(
            "input",
            tab.document.text().len(),
            start.elapsed().as_micros() as u64,
            cpu_us,
        );
        tracing::info!(target: "markion::performance", bytes = tab.document.text().len(),
            elapsed_us = start.elapsed().as_micros() as u64, thread_cpu_us = ?cpu_us,
            "visual input callback to caret paint");
    }
}
