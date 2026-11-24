//! Native Quick filter and accessible-row scenario.
//!
//! The bilingual scenario description is kept in `search_filter.md`.

use super::{native_test_lock, RunningApp, TestResult, ACTION_TIMEOUT};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_search_filters_accessible_item_rows() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.wait_for_selector("button[name='Add item']", ACTION_TIMEOUT)?;
    app.create_item("E2E Alpha item")?;
    app.create_item("E2E Beta item")?;

    app.set_text("text_field[name='Quick filter']", "Alpha")?;
    app.wait_for_selector("[name='E2E Alpha item']", ACTION_TIMEOUT)?;
    app.wait_for_selector_to_disappear("[name='E2E Beta item']", ACTION_TIMEOUT)?;

    app.set_text("text_field[name='Quick filter']", "")?;
    app.wait_for_selector("[name='E2E Beta item']", ACTION_TIMEOUT)?;
    app.close_cleanly()
}
