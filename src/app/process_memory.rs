//! OS-level process memory counters for the memory report.
//!
//! Each counter is individually optional: platforms differ in what they
//! expose, and a failed query must never prevent a report from being produced.

/// Platform label printed in the footprint section (e.g. `"windows"`).
pub(super) fn process_footprint_platform() -> &'static str {
    #[cfg(windows)]
    {
        "windows"
    }
    #[cfg(target_os = "linux")]
    {
        "linux"
    }
    #[cfg(target_os = "macos")]
    {
        "macos"
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        "unknown"
    }
}

/// Process-level footprint measured by the host OS.
///
/// Counters that the platform cannot supply, or that fail to read, are `None`
/// and must be rendered as unavailable — never as zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub(super) struct ProcessFootprint {
    pub(super) resident_current: Option<usize>,
    pub(super) resident_peak: Option<usize>,
    pub(super) commit_current: Option<usize>,
    pub(super) commit_peak: Option<usize>,
}

impl ProcessFootprint {
    /// Read the current process footprint. Failures yield unavailable counters.
    pub(super) fn sample() -> Self {
        sample_process_footprint()
    }

    pub(super) fn format_log_section(&self, platform: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("--- process footprint ({platform}) ---\n"));
        out.push_str(&format_counter("resident_current", self.resident_current));
        out.push_str(&format_counter("resident_peak", self.resident_peak));
        out.push_str(&format_counter("commit_current", self.commit_current));
        out.push_str(&format_counter("commit_peak", self.commit_peak));
        out
    }
}

fn format_counter(name: &str, value: Option<usize>) -> String {
    match value {
        Some(bytes) => format!("  {name}={bytes}\n"),
        None => format!("  {name}=unavailable\n"),
    }
}

fn sample_process_footprint() -> ProcessFootprint {
    #[cfg(windows)]
    {
        sample_windows()
    }
    #[cfg(target_os = "linux")]
    {
        sample_linux()
    }
    #[cfg(target_os = "macos")]
    {
        sample_macos()
    }
    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        ProcessFootprint::default()
    }
}

#[cfg(windows)]
fn sample_windows() -> ProcessFootprint {
    use std::mem::MaybeUninit;
    use windows::Win32::System::ProcessStatus::{GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS};
    use windows::Win32::System::Threading::GetCurrentProcess;

    // SAFETY: GetCurrentProcess returns a pseudo-handle; GetProcessMemoryInfo
    // writes into a PROCESS_MEMORY_COUNTERS whose `cb` field is set first.
    unsafe {
        let mut counters = MaybeUninit::<PROCESS_MEMORY_COUNTERS>::zeroed();
        let counters_ptr = counters.as_mut_ptr();
        (*counters_ptr).cb = std::mem::size_of::<PROCESS_MEMORY_COUNTERS>() as u32;
        match GetProcessMemoryInfo(GetCurrentProcess(), counters_ptr, (*counters_ptr).cb) {
            Ok(()) => {
                let c = counters.assume_init();
                ProcessFootprint {
                    resident_current: Some(c.WorkingSetSize),
                    resident_peak: Some(c.PeakWorkingSetSize),
                    commit_current: Some(c.PagefileUsage),
                    commit_peak: Some(c.PeakPagefileUsage),
                }
            }
            Err(_) => ProcessFootprint::default(),
        }
    }
}

#[cfg(target_os = "linux")]
fn sample_linux() -> ProcessFootprint {
    // VmData is not private commit; leave commit counters unavailable rather
    // than reporting a misleading approximation. RssAnon would be closer but
    // is still not PagefileUsage-equivalent across kernels.
    let Ok(status) = std::fs::read_to_string("/proc/self/status") else {
        return ProcessFootprint::default();
    };
    ProcessFootprint {
        resident_current: parse_status_kib(&status, "VmRSS"),
        resident_peak: parse_status_kib(&status, "VmHWM"),
        commit_current: None,
        commit_peak: None,
    }
}

#[cfg(target_os = "linux")]
fn parse_status_kib(status: &str, key: &str) -> Option<usize> {
    let prefix = format!("{key}:");
    for line in status.lines() {
        let Some(rest) = line.strip_prefix(&prefix) else {
            continue;
        };
        let mut parts = rest.split_whitespace();
        let value = parts.next()?.parse::<usize>().ok()?;
        // Values are in kB.
        return Some(value.saturating_mul(1024));
    }
    None
}

#[cfg(target_os = "macos")]
fn sample_macos() -> ProcessFootprint {
    use libc::{
        KERN_SUCCESS, MACH_TASK_BASIC_INFO, MACH_TASK_BASIC_INFO_COUNT, TASK_VM_INFO,
        TASK_VM_INFO_COUNT, kern_return_t, mach_msg_type_number_t, mach_task_basic_info, task_info,
        task_t, task_vm_info_data_t,
    };

    // SAFETY: mach_task_self() is a process-local port; task_info writes into
    // caller-owned structs whose count arguments match the flavour requested.
    unsafe {
        extern "C" {
            fn mach_task_self() -> task_t;
        }

        let mut basic: mach_task_basic_info = std::mem::zeroed();
        let mut basic_count: mach_msg_type_number_t = MACH_TASK_BASIC_INFO_COUNT;
        let basic_kr: kern_return_t = task_info(
            mach_task_self(),
            MACH_TASK_BASIC_INFO as u32,
            &mut basic as *mut _ as *mut _,
            &mut basic_count,
        );

        let mut vm: task_vm_info_data_t = std::mem::zeroed();
        let mut vm_count: mach_msg_type_number_t = TASK_VM_INFO_COUNT;
        let vm_kr: kern_return_t = task_info(
            mach_task_self(),
            TASK_VM_INFO as u32,
            &mut vm as *mut _ as *mut _,
            &mut vm_count,
        );

        let mut footprint = ProcessFootprint {
            resident_current: None,
            resident_peak: None,
            commit_current: None,
            commit_peak: None,
        };
        if basic_kr == KERN_SUCCESS {
            footprint.resident_current = Some(basic.resident_size as usize);
            footprint.resident_peak = Some(basic.resident_size_max as usize);
        }
        if vm_kr == KERN_SUCCESS {
            // phys_footprint is the closest macOS analogue of private commit.
            footprint.commit_current = Some(vm.phys_footprint as usize);
            if vm.resident_size > 0 {
                footprint.resident_current = Some(vm.resident_size as usize);
            }
            if vm.resident_size_peak > 0 {
                footprint.resident_peak = Some(vm.resident_size_peak as usize);
            }
        }
        footprint
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sample_returns_platform_label() {
        let platform = process_footprint_platform();
        assert!(
            matches!(platform, "windows" | "linux" | "macos" | "unknown"),
            "unexpected platform {platform}"
        );
    }

    #[test]
    fn sample_available_counters_are_nonzero_and_peaks_bound_currents() {
        let fp = ProcessFootprint::sample();
        for (name, current, peak) in [
            ("resident", fp.resident_current, fp.resident_peak),
            ("commit", fp.commit_current, fp.commit_peak),
        ] {
            if let Some(cur) = current {
                assert!(cur > 0, "{name}_current must be non-zero when available");
            }
            if let (Some(cur), Some(pk)) = (current, peak) {
                assert!(
                    pk >= cur,
                    "{name}_peak ({pk}) must be >= {name}_current ({cur})"
                );
            }
        }

        #[cfg(windows)]
        {
            assert!(fp.resident_current.is_some());
            assert!(fp.resident_peak.is_some());
            assert!(fp.commit_current.is_some());
            assert!(fp.commit_peak.is_some());
        }
        #[cfg(target_os = "linux")]
        {
            assert!(fp.resident_current.is_some());
            assert!(fp.resident_peak.is_some());
            assert!(fp.commit_current.is_none());
            assert!(fp.commit_peak.is_none());
        }
        #[cfg(target_os = "macos")]
        {
            assert!(fp.resident_current.is_some());
            assert!(fp.resident_peak.is_some());
            assert!(fp.commit_peak.is_none());
        }
    }

    #[test]
    fn unavailable_counters_render_as_unavailable_not_zero() {
        let fp = ProcessFootprint {
            resident_current: Some(1024),
            resident_peak: Some(2048),
            commit_current: None,
            commit_peak: None,
        };
        let section = fp.format_log_section("test");
        assert!(section.contains("resident_current=1024"));
        assert!(section.contains("commit_current=unavailable"));
        assert!(section.contains("commit_peak=unavailable"));
        assert!(!section.contains("commit_current=0\n"));
        assert!(!section.contains("commit_peak=0\n"));
    }

    #[test]
    fn successive_samples_never_lower_peaks() {
        let first = ProcessFootprint::sample();
        let second = ProcessFootprint::sample();
        if let (Some(a), Some(b)) = (first.resident_peak, second.resident_peak) {
            assert!(b >= a, "resident_peak must be monotonic ({a} -> {b})");
        }
        if let (Some(a), Some(b)) = (first.commit_peak, second.commit_peak) {
            assert!(b >= a, "commit_peak must be monotonic ({a} -> {b})");
        }
    }
}
