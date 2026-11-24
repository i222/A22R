//! Native delete confirmation scenario.
//!
//! The bilingual scenario description is kept in `delete_confirmation.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_delete_requires_confirmation() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.create_item("E2E delete item")?;

    app.press("button[name='Delete']")?;
    app.wait_for_selector("button[name='Cancel']", ACTION_TIMEOUT)?;
    app.press("button[name='Cancel']")?;
    app.wait_for_selector_to_disappear("button[name='Cancel']", ACTION_TIMEOUT)?;
    app.wait_for_selector("[name='E2E delete item']", ACTION_TIMEOUT)?;

    app.press("button[name='Delete']")?;
    app.wait_for_selector("button[name='Cancel']", ACTION_TIMEOUT)?;
    app.press_last("button[name='Delete']")?;
    app.wait_for_selector_to_disappear("[name='E2E delete item']", ACTION_TIMEOUT)?;
    app.wait_for_selector_to_disappear("button[name='Delete']", ACTION_TIMEOUT)?;
    app.close_cleanly()
}
