//! Native tag autocomplete and persistence scenario.
//!
//! The bilingual scenario description is kept in `tag_autocomplete.md`.

use super::{native_test_lock, RunningApp, TestResult};

#[test]
#[ignore = "requires a native desktop session and platform accessibility permission"]
fn native_tag_autocomplete_accepts_default_and_persists() -> TestResult {
    let _lock = native_test_lock();
    let mut app = RunningApp::launch()?;
    app.press("button[name='Add item']")?;
    app.wait_for_selector("button[name='Save']", super::ACTION_TIMEOUT)?;
    app.set_text("text_field[name='Value']", "E2E tagged item")?;
    app.add_tag("poc")?;
    app.press("button[name='Save']")?;
    app.wait_for_selector("[name='E2E tagged item']", super::ACTION_TIMEOUT)?;
    app.wait_for_selector("[name='poc']", super::ACTION_TIMEOUT)?;

    let profile_dir = app.profile_dir.clone();
    app.close_cleanly()?;
    let mut reopened = RunningApp::launch_from_profile(profile_dir)?;
    reopened.wait_for_selector("[name='E2E tagged item']", super::ACTION_TIMEOUT)?;
    reopened.wait_for_selector("[name='poc']", super::ACTION_TIMEOUT)?;
    reopened.close_cleanly()
}
