//! Native unsaved-guard save-and-continue scenario.
//!
//! The bilingual scenario description is kept in `unsaved_guard_save.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_unsaved_guard_can_save_and_continue() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.create_item("E2E guard original")?;
    app.edit_current_item()?;
    app.set_text("text_field[name='Value']", "E2E guard saved")?;

    app.press("button[name='Add item']")?;
    app.wait_for_selector("button[name='Discard changes']", ACTION_TIMEOUT)?;
    app.press_last("button[name='Save']")?;
    app.wait_for_selector("button[name='Cancel']", ACTION_TIMEOUT)?;
    app.press("button[name='Cancel']")?;
    app.wait_for_selector("[name='E2E guard saved']", ACTION_TIMEOUT)?;

    let profile_dir = app.profile_dir.clone();
    app.close_cleanly()?;
    let mut reopened = RunningApp::launch_from_profile(profile_dir)?;
    reopened.wait_for_selector("[name='E2E guard saved']", ACTION_TIMEOUT)?;
    reopened.close_cleanly()
}
