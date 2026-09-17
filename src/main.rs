#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod ui;

fn main() {
    // Load and validate the tiny signed-catalog host seam before GPUI starts.
    // Failures remain contained: a bad bootstrap disables optional plugins and
    // must never prevent the core Markdown editor from opening.
    let _ = markion::plugin_platform::bootstrap_summary();
    app::run();
}
