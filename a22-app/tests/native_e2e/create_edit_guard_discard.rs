//! Native create/edit, unsaved-guard, discard, and restart scenario.
//!
//! The bilingual scenario description is kept in `create_edit_guard_discard.md`.

use super::{
    native_test_lock, reopen_and_assert_persisted, RunningApp, TestResult, ACTION_TIMEOUT,
};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_create_edit_guard_discard_and_restart_persist() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.wait_for_selector("button[name='Add item']", ACTION_TIMEOUT)?;
    app.press("button[name='Add item']")?;
    app.wait_for_selector("button[name='Save']", ACTION_TIMEOUT)?;
    app.set_text("text_field[name='Value']", "E2E durable item")?;
    app.press("button[name='Save']")?;
    app.wait_for_selector("button[name='Edit']", ACTION_TIMEOUT)?;

    app.press("button[name='Edit']")?;
    app.wait_for_selector("text_field[name='Value']", ACTION_TIMEOUT)?;
    app.replace_text("text_field[name='Value']", "E2E discarded draft")?;
    app.press("button[name='Add item']")?;
    app.wait_for_selector("button[name='Go back']", ACTION_TIMEOUT)?;
    app.press("button[name='Go back']")?;
    app.wait_for_selector_to_disappear("button[name='Discard changes']", ACTION_TIMEOUT)?;
    app.wait_for_value(
        "text_field[name='Value']",
        "E2E discarded draft",
        ACTION_TIMEOUT,
    )?;

    app.press("button[name='Add item']")?;
    app.wait_for_selector("button[name='Discard changes']", ACTION_TIMEOUT)?;
    app.press("button[name='Discard changes']")?;
    app.wait_for_selector("button[name='Cancel']", ACTION_TIMEOUT)?;
    app.press("button[name='Cancel']")?;
    app.wait_for_selector("button[name='Edit']", ACTION_TIMEOUT)?;

    let profile_dir = app.profile_dir.clone();
    app.close_cleanly()?;
    reopen_and_assert_persisted(&profile_dir, "E2E durable item")
}
