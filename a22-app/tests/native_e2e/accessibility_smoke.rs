//! Native Accessibility smoke scenario.
//!
//! The bilingual scenario description is kept in `accessibility_smoke.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_accessibility_exposes_primary_controls() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    for selector in [
        "window[name='A22R — Item workspace']",
        "button[name='Add item']",
        "button[name='Import items']",
        "button[name='Export items']",
        "button[name='Open Inspector']",
        "button[name='Open Inspector component tree']",
        "button[name='Open UI Kit']",
        "text_field[name='Quick filter']",
        "text_field[name='Tag filter']",
    ] {
        app.wait_for_selector(selector, ACTION_TIMEOUT)?;
    }
    app.activate_window()?;
    app.press("button[name='Open Inspector']")?;
    app.wait_for_selector("window[name='Inspector']", ACTION_TIMEOUT)?;
    app.press("button[name='Open Inspector component tree']")?;
    app.wait_for_selector("window[name='Inspector component tree']", ACTION_TIMEOUT)?;
    app.close_cleanly()
}
