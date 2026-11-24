//! Native multi-item CRUD and relationship lifecycle scenario.
//!
//! The bilingual scenario description is kept in
//! `multi_item_relationship_lifecycle.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_multi_item_relationship_lifecycle_survives_restart() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;

    app.create_item("E2E Alpha")?;
    app.create_item("E2E Beta")?;
    app.create_item("E2E Gamma")?;

    // Target a non-selected row so the test covers stable row-scoped actions,
    // not only the currently selected item's toolbar.
    app.edit_item("E2E Beta")?;
    app.replace_text("text_field[name='Value']", "E2E Beta edited")?;
    app.add_relationship("E2E Alpha")?;
    app.add_relationship("E2E Gamma")?;
    app.set_text_at("text_field[name='Select key']", 1, "blocks")?;
    app.set_text_last("text_field[name='Select key']", "related")?;
    app.press("button[name='Save']")?;
    app.wait_for_selector("[name='E2E Beta edited']", ACTION_TIMEOUT)?;

    // Change one relationship type and remove the other before committing the
    // edit. The target buttons are the relationship editor's semantic hooks.
    app.edit_item("E2E Beta edited")?;
    app.wait_for_selector("button[name='E2E Alpha']", ACTION_TIMEOUT)?;
    app.wait_for_selector("button[name='E2E Gamma']", ACTION_TIMEOUT)?;
    app.set_text_at("text_field[name='Select key']", 1, "depends_on")?;
    app.remove_last_relationship("E2E Gamma")?;
    app.press("button[name='Save']")?;
    app.wait_for_selector("[name='E2E Beta edited']", ACTION_TIMEOUT)?;

    // Alpha is still a committed target of Beta. Deleting it must remove the
    // outgoing link through the normal worker/database cascade.
    app.delete_item("E2E Alpha")?;
    app.wait_for_selector("[name='E2E Beta edited']", ACTION_TIMEOUT)?;
    app.edit_item("E2E Beta edited")?;
    app.wait_for_selector_to_disappear("button[name='E2E Alpha']", ACTION_TIMEOUT)?;
    app.wait_for_selector_to_disappear("button[name='E2E Gamma']", ACTION_TIMEOUT)?;
    app.press("button[name='Cancel']")?;

    let profile_dir = app.profile_dir.clone();
    app.close_cleanly()?;

    // Verify both the surviving edit and relationship cascade after a real
    // process restart, not only after an in-memory snapshot refresh.
    let mut reopened = RunningApp::launch_from_profile(profile_dir)?;
    reopened.wait_for_selector("[name='E2E Beta edited']", ACTION_TIMEOUT)?;
    reopened.wait_for_selector("[name='E2E Gamma']", ACTION_TIMEOUT)?;
    reopened.wait_for_selector_to_disappear("[name='E2E Alpha']", ACTION_TIMEOUT)?;
    reopened.edit_item("E2E Beta edited")?;
    reopened.wait_for_selector_to_disappear("button[name='E2E Alpha']", ACTION_TIMEOUT)?;
    reopened.wait_for_selector_to_disappear("button[name='E2E Gamma']", ACTION_TIMEOUT)?;
    reopened.press("button[name='Cancel']")?;
    reopened.close_cleanly()
}
